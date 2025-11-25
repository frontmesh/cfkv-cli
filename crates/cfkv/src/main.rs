mod cli;
mod config;
mod formatter;

use cli::{Cli, Commands, BatchCommands, BlogCommands, ConfigCommands};
use cloudflare_kv::{ClientConfig, KvClient, PaginationParams};
use cfkv_blog::BlogPublisher;
use clap::Parser;
use formatter::{Formatter, OutputFormat};
use std::fs;
use std::path::Path;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.debug {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("cf_kv=debug")),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    let format = OutputFormat::from_str(&cli.format).unwrap_or(OutputFormat::Text);

    // Load configuration
    let config_path = if let Some(config) = cli.config {
        config
    } else {
        config::Config::default_path()?
    };

    let mut config = config::Config::load_or_create(&config_path).unwrap_or_default();

    // Merge CLI arguments with config
    if let Some(account_id) = cli.account_id {
        config.account_id = Some(account_id);
    }
    if let Some(namespace_id) = cli.namespace_id {
        config.namespace_id = Some(namespace_id);
    }
    if let Some(api_token) = cli.api_token {
        config.api_token = Some(api_token);
    }

    match cli.command {
        Commands::Config { command } => handle_config_command(command, &config, &config_path, format).await?,
        _ => {
            // Validate configuration for other commands
            let account_id = config
                .account_id
                .ok_or("Account ID not configured. Set with: cf-kv config set-account <ID>")?;
            let namespace_id = config
                .namespace_id
                .ok_or("Namespace ID not configured. Set with: cf-kv config set-namespace <ID>")?;
            let api_token = config
                .api_token
                .ok_or("API token not configured. Set with: cf-kv config set-token <TOKEN>")?;

            let client_config = ClientConfig::new(&account_id, &namespace_id, cloudflare_kv::AuthCredentials::token(api_token));
            let client = KvClient::new(client_config);

            match cli.command {
                Commands::Get { key, pretty } => {
                    handle_get(&client, &key, format, pretty).await?
                }
                Commands::Put {
                    key,
                    value,
                    file,
                    ttl,
                    metadata,
                } => {
                    handle_put(&client, &key, value, file, ttl, metadata, format).await?
                }
                Commands::Delete { key } => handle_delete(&client, &key, format).await?,
                Commands::List {
                    limit,
                    cursor,
                    metadata,
                } => {
                    handle_list(&client, limit, cursor, metadata, format).await?
                }
                Commands::Batch { command } => {
                    handle_batch(&client, command, format).await?
                }
                Commands::Namespace { command: _ } => {
                    println!("{}", Formatter::format_text(
                        "Namespace management coming soon",
                        format
                    ));
                }
                Commands::Interactive => {
                    println!("{}", Formatter::format_text(
                        "Interactive mode coming soon",
                        format
                    ));
                }
                Commands::Blog { command } => {
                    handle_blog(&client, command, format).await?
                }
                Commands::Config { .. } => unreachable!(),
            }
        }
    }

    Ok(())
}

async fn handle_get(
    client: &KvClient,
    key: &str,
    format: OutputFormat,
    pretty: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match client.get(key).await {
        Ok(Some(kv_pair)) => {
            let output = match format {
                OutputFormat::Json => {
                    if pretty {
                        format!("{{\n  \"key\": \"{}\",\n  \"value\": \"{}\"\n}}", kv_pair.key, kv_pair.value)
                    } else {
                        format!("{{\"key\":\"{}\",\"value\":\"{}\"}}", kv_pair.key, kv_pair.value)
                    }
                }
                OutputFormat::Yaml => {
                    format!("key: {}\nvalue: {}", kv_pair.key, kv_pair.value)
                }
                OutputFormat::Text => kv_pair.value,
            };
            println!("{}", output);
        }
        Ok(None) => {
            eprintln!("{}", Formatter::format_error(&format!("Key not found: {}", key), format));
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("{}", Formatter::format_error(&e.to_string(), format));
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_put(
    client: &KvClient,
    key: &str,
    value: Option<String>,
    file: Option<std::path::PathBuf>,
    ttl: Option<u64>,
    metadata: Option<String>,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let value_bytes = if let Some(file_path) = file {
        fs::read(&file_path)?
    } else if let Some(val) = value {
        val.into_bytes()
    } else {
        eprintln!("{}", Formatter::format_error("Either --value or --file must be provided", format));
        std::process::exit(1);
    };

    let result = if ttl.is_some() || metadata.is_some() {
        let meta = metadata.and_then(|m| serde_json::from_str(&m).ok());
        client.put_with_options(key, &value_bytes, ttl, meta).await
    } else {
        client.put(key, &value_bytes).await
    };

    match result {
        Ok(()) => println!("{}", Formatter::format_success(&format!("Successfully put key: {}", key), format)),
        Err(e) => {
            eprintln!("{}", Formatter::format_error(&e.to_string(), format));
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_delete(
    client: &KvClient,
    key: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match client.delete(key).await {
        Ok(()) => println!("{}", Formatter::format_success(&format!("Successfully deleted key: {}", key), format)),
        Err(e) => {
            eprintln!("{}", Formatter::format_error(&e.to_string(), format));
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_list(
    client: &KvClient,
    limit: u32,
    cursor: Option<String>,
    _metadata: bool,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = PaginationParams::new()
        .with_limit(limit)
        .with_cursor(cursor.unwrap_or_default());

    match client.list(Some(params)).await {
        Ok(response) => {
            let keys: Vec<String> = response.keys.into_iter().map(|k| k.name).collect();

            let output = match format {
                OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
                    "keys": keys,
                    "list_complete": response.list_complete,
                    "cursor": response.cursor
                }))?,
                OutputFormat::Yaml => serde_yaml::to_string(&serde_json::json!({
                    "keys": keys,
                    "list_complete": response.list_complete,
                    "cursor": response.cursor
                }))?,
                OutputFormat::Text => {
                    let mut output = String::new();
                    for key in keys {
                        output.push_str(&format!("{}\n", key));
                    }
                    output
                }
            };

            println!("{}", output);
        }
        Err(e) => {
            eprintln!("{}", Formatter::format_error(&e.to_string(), format));
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_batch(
    client: &KvClient,
    command: BatchCommands,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        BatchCommands::Delete { keys } => {
            let key_refs: Vec<&str> = keys.iter().map(|k| k.as_str()).collect();
            match client.batch_delete(key_refs).await {
                Ok(()) => println!("{}", Formatter::format_success("Batch delete successful", format)),
                Err(e) => {
                    eprintln!("{}", Formatter::format_error(&e.to_string(), format));
                    std::process::exit(1);
                }
            }
        }
        BatchCommands::Import { file } => {
            let _content = fs::read_to_string(&file)?;
            // TODO: Parse JSON/YAML and import
            println!("{}", Formatter::format_text("Batch import coming soon", format));
        }
        BatchCommands::Export { output: _ } => {
            // TODO: Export keys to file
            println!("{}", Formatter::format_text("Batch export coming soon", format));
        }
    }

    Ok(())
}

async fn handle_config_command(
    command: ConfigCommands,
    config: &config::Config,
    config_path: &Path,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ConfigCommands::SetToken { token } => {
            let mut new_config = config.clone();
            new_config.api_token = Some(token);
            new_config.save(config_path)?;
            println!("{}", Formatter::format_success("API token saved", format));
        }
        ConfigCommands::SetAccount { account_id } => {
            let mut new_config = config.clone();
            new_config.account_id = Some(account_id);
            new_config.save(config_path)?;
            println!("{}", Formatter::format_success("Account ID saved", format));
        }
        ConfigCommands::SetNamespace { namespace_id } => {
            let mut new_config = config.clone();
            new_config.namespace_id = Some(namespace_id);
            new_config.save(config_path)?;
            println!("{}", Formatter::format_success("Namespace ID saved", format));
        }
        ConfigCommands::Show => {
            let output = match format {
                OutputFormat::Json => serde_json::to_string_pretty(config)?,
                OutputFormat::Yaml => serde_yaml::to_string(config)?,
                OutputFormat::Text => {
                    format!(
                        "Account ID: {}\nNamespace ID: {}\nAPI Token: {}",
                        config.account_id.as_deref().unwrap_or("Not set"),
                        config.namespace_id.as_deref().unwrap_or("Not set"),
                        if config.api_token.is_some() { "***" } else { "Not set" }
                    )
                }
            };
            println!("{}", output);
        }
        ConfigCommands::Reset => {
            let new_config = config::Config::default();
            new_config.save(config_path)?;
            println!("{}", Formatter::format_success("Configuration reset", format));
        }
    }

    Ok(())
}

async fn handle_blog(
    client: &KvClient,
    command: BlogCommands,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let publisher = BlogPublisher::new(client);

    match command {
        BlogCommands::Publish { file } => {
            publisher.publish_from_file(&file).await?;
            let title = file.file_name().and_then(|n| n.to_str()).unwrap_or("blog post");
            println!(
                "{}",
                Formatter::format_success(
                    &format!("Successfully published: {}", title),
                    format
                )
            );
        }
        BlogCommands::List => {
            let posts = publisher.list_posts().await?;

            if posts.is_empty() {
                println!("{}", Formatter::format_text("No blog posts found", format));
                return Ok(());
            }

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&posts)?);
                }
                OutputFormat::Yaml => {
                    println!("{}", serde_yaml::to_string(&posts)?);
                }
                OutputFormat::Text => {
                    println!("Found {} blog posts:\n", posts.len());
                    for post in posts {
                        println!("• {}", post.title);
                        println!("  Slug: {}", post.slug);
                        println!("  Date: {}", post.date);
                        println!("  Author: {}", post.author);
                        println!("  Tags: {}\n", post.tags.join(", "));
                    }
                }
            }
        }
        BlogCommands::Delete { slug } => {
            publisher.delete_post(&slug).await?;
            println!(
                "{}",
                Formatter::format_success(&format!("Successfully deleted: {}", slug), format)
            );
        }
    }

    Ok(())
}

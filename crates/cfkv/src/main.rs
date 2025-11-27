mod cli;
mod config;
mod formatter;

use cfkv_blog::BlogPublisher;
use clap::Parser;
use cli::{BatchCommands, BlogCommands, Cli, Commands, ConfigCommands, StorageCommands};
use cloudflare_kv::{ClientConfig, KvClient, PaginationParams};
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
        Commands::Config { command } => {
            handle_config_command(command, &config, &config_path, format).await?
        }
        Commands::Storage { command } => {
            // For storage commands, ensure migration is done and config is saved if needed
            let needs_migration = config.storages.is_empty()
                && (config.account_id.is_some()
                    || config.namespace_id.is_some()
                    || config.api_token.is_some());

            if needs_migration {
                config.migrate_legacy_format();
                config.save(&config_path)?;
            }

            handle_storage_command(command, &mut config, &config_path, format).await?
        }
        _ => {
            // Validate configuration for other commands
            // Try to get active storage, fallback to legacy format if available
            let (account_id, namespace_id, api_token) = if let Some(storage) =
                config.get_active_storage()
            {
                (
                    storage.account_id.clone(),
                    storage.namespace_id.clone(),
                    storage.api_token.clone(),
                )
            } else if let (Some(acc), Some(ns), Some(token)) =
                (&config.account_id, &config.namespace_id, &config.api_token)
            {
                (acc.clone(), ns.clone(), token.clone())
            } else {
                return Err("No storage configured. Add one with: cfkv storage add <name> --account-id <ID> --namespace-id <ID> --api-token <TOKEN>".into());
            };

            let client_config = ClientConfig::new(
                &account_id,
                &namespace_id,
                cloudflare_kv::AuthCredentials::token(api_token),
            );
            let client = KvClient::new(client_config);

            match cli.command {
                Commands::Get { key, pretty } => handle_get(&client, &key, format, pretty).await?,
                Commands::Put {
                    key,
                    value,
                    file,
                    ttl,
                    metadata,
                } => handle_put(&client, &key, value, file, ttl, metadata, format).await?,
                Commands::Delete { key } => handle_delete(&client, &key, format).await?,
                Commands::List {
                    limit,
                    cursor,
                    metadata,
                } => handle_list(&client, limit, cursor, metadata, format).await?,
                Commands::Batch { command } => handle_batch(&client, command, format).await?,
                Commands::Namespace { command: _ } => {
                    println!(
                        "{}",
                        Formatter::format_text("Namespace management coming soon", format)
                    );
                }
                Commands::Interactive => {
                    println!(
                        "{}",
                        Formatter::format_text("Interactive mode coming soon", format)
                    );
                }
                Commands::Blog { command } => handle_blog(&client, command, format).await?,
                Commands::Config { .. } => unreachable!(),
                Commands::Storage { .. } => unreachable!(),
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
                        format!(
                            "{{\n  \"key\": \"{}\",\n  \"value\": \"{}\"\n}}",
                            kv_pair.key, kv_pair.value
                        )
                    } else {
                        format!(
                            "{{\"key\":\"{}\",\"value\":\"{}\"}}",
                            kv_pair.key, kv_pair.value
                        )
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
            eprintln!(
                "{}",
                Formatter::format_error(&format!("Key not found: {}", key), format)
            );
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
        eprintln!(
            "{}",
            Formatter::format_error("Either --value or --file must be provided", format)
        );
        std::process::exit(1);
    };

    let result = if ttl.is_some() || metadata.is_some() {
        let meta = metadata.and_then(|m| serde_json::from_str(&m).ok());
        client.put_with_options(key, &value_bytes, ttl, meta).await
    } else {
        client.put(key, &value_bytes).await
    };

    match result {
        Ok(()) => println!(
            "{}",
            Formatter::format_success(&format!("Successfully put key: {}", key), format)
        ),
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
        Ok(()) => println!(
            "{}",
            Formatter::format_success(&format!("Successfully deleted key: {}", key), format)
        ),
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
            let key_refs: Vec<&str> = keys.iter().map(|k: &String| k.as_str()).collect();
            match client.batch_delete(key_refs).await {
                Ok(()) => println!(
                    "{}",
                    Formatter::format_success("Batch delete successful", format)
                ),
                Err(e) => {
                    eprintln!("{}", Formatter::format_error(&e.to_string(), format));
                    std::process::exit(1);
                }
            }
        }
        BatchCommands::Import { file } => {
            let _content = fs::read_to_string(&file)?;
            // TODO: Parse JSON/YAML and import
            println!(
                "{}",
                Formatter::format_text("Batch import coming soon", format)
            );
        }
        BatchCommands::Export { output: _ } => {
            // TODO: Export keys to file
            println!(
                "{}",
                Formatter::format_text("Batch export coming soon", format)
            );
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
            println!(
                "{}",
                Formatter::format_success("Namespace ID saved", format)
            );
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
                        if config.api_token.is_some() {
                            "***"
                        } else {
                            "Not set"
                        }
                    )
                }
            };
            println!("{}", output);
        }
        ConfigCommands::Reset => {
            let new_config = config::Config::default();
            new_config.save(config_path)?;
            println!(
                "{}",
                Formatter::format_success("Configuration reset", format)
            );
        }
    }

    Ok(())
}

async fn handle_storage_command(
    command: StorageCommands,
    config: &mut config::Config,
    config_path: &Path,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        StorageCommands::Add {
            name,
            account_id,
            namespace_id,
            api_token,
        } => {
            config.add_storage(name.clone(), account_id, namespace_id, api_token);
            config.save(config_path)?;
            println!(
                "{}",
                Formatter::format_success(&format!("Storage '{}' added", name), format)
            );
        }
        StorageCommands::List => {
            let storages = config.list_storages();
            if storages.is_empty() {
                println!(
                    "{}",
                    Formatter::format_text("No storages configured", format)
                );
                return Ok(());
            }

            match format {
                OutputFormat::Json => {
                    let storage_list: Vec<serde_json::Value> = storages
                        .iter()
                        .map(|name| {
                            let storage = config.get_storage(name).unwrap();
                            let is_active = config.active_storage.as_deref() == Some(name);
                            serde_json::json!({
                                "name": storage.name,
                                "account_id": storage.account_id,
                                "namespace_id": storage.namespace_id,
                                "active": is_active,
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&storage_list)?);
                }
                OutputFormat::Yaml => {
                    let storage_list: Vec<serde_json::Value> = storages
                        .iter()
                        .map(|name| {
                            let storage = config.get_storage(name).unwrap();
                            let is_active = config.active_storage.as_deref() == Some(name);
                            serde_json::json!({
                                "name": storage.name,
                                "account_id": storage.account_id,
                                "namespace_id": storage.namespace_id,
                                "active": is_active,
                            })
                        })
                        .collect();
                    println!("{}", serde_yaml::to_string(&storage_list)?);
                }
                OutputFormat::Text => {
                    println!("Available storages:\n");
                    for name in storages {
                        let storage = config.get_storage(name).unwrap();
                        let is_active = config.active_storage.as_deref() == Some(name);
                        let marker = if is_active { "* " } else { "  " };
                        println!(
                            "{}{}  (account: {}, namespace: {})",
                            marker, name, storage.account_id, storage.namespace_id
                        );
                    }
                }
            }
        }
        StorageCommands::Current => match config.get_active_storage() {
            Some(storage) => {
                let output = match format {
                    OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
                        "name": storage.name,
                        "account_id": storage.account_id,
                        "namespace_id": storage.namespace_id,
                    }))?,
                    OutputFormat::Yaml => serde_yaml::to_string(&serde_json::json!({
                        "name": storage.name,
                        "account_id": storage.account_id,
                        "namespace_id": storage.namespace_id,
                    }))?,
                    OutputFormat::Text => {
                        format!(
                            "Current storage: {}\nAccount ID: {}\nNamespace ID: {}",
                            storage.name, storage.account_id, storage.namespace_id
                        )
                    }
                };
                println!("{}", output);
            }
            None => {
                eprintln!(
                    "{}",
                    Formatter::format_error("No active storage configured", format)
                );
                std::process::exit(1);
            }
        },
        StorageCommands::Switch { name } => {
            config.set_active_storage(name.clone())?;
            config.save(config_path)?;
            println!(
                "{}",
                Formatter::format_success(&format!("Switched to storage '{}'", name), format)
            );
        }
        StorageCommands::Remove { name } => {
            config.remove_storage(&name)?;
            config.save(config_path)?;
            println!(
                "{}",
                Formatter::format_success(&format!("Storage '{}' removed", name), format)
            );
        }
        StorageCommands::Rename { old_name, new_name } => {
            config.rename_storage(&old_name, new_name.clone())?;
            config.save(config_path)?;
            println!(
                "{}",
                Formatter::format_success(
                    &format!("Storage renamed from '{}' to '{}'", old_name, new_name),
                    format
                )
            );
        }
        StorageCommands::Show { name } => {
            let storage = if let Some(storage_name) = name {
                config.get_storage(&storage_name).ok_or_else(|| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Storage '{}' not found", &storage_name),
                    )) as Box<dyn std::error::Error>
                })?
            } else {
                config.get_active_storage().ok_or_else(|| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "No active storage configured",
                    )) as Box<dyn std::error::Error>
                })?
            };

            let output = match format {
                OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
                    "name": storage.name,
                    "account_id": storage.account_id,
                    "namespace_id": storage.namespace_id,
                }))?,
                OutputFormat::Yaml => serde_yaml::to_string(&serde_json::json!({
                    "name": storage.name,
                    "account_id": storage.account_id,
                    "namespace_id": storage.namespace_id,
                }))?,
                OutputFormat::Text => {
                    format!(
                        "Storage: {}\nAccount ID: {}\nNamespace ID: {}",
                        storage.name, storage.account_id, storage.namespace_id
                    )
                }
            };
            println!("{}", output);
        }
        StorageCommands::Export { file } => {
            let json = config.export_to_json()?;

            if let Some(output_path) = file {
                fs::write(&output_path, &json)?;
                println!(
                    "{}",
                    Formatter::format_success(
                        &format!("Storages exported to '{}'", output_path.display()),
                        format
                    )
                );
            } else {
                println!("{}", json);
            }
        }
        StorageCommands::Import { file } => {
            let json = fs::read_to_string(&file)?;
            config.import_from_json(&json)?;
            config.save(config_path)?;
            println!(
                "{}",
                Formatter::format_success(
                    &format!("Storages imported from '{}'", file.display()),
                    format
                )
            );
        }
        StorageCommands::LoadEnv => {
            config.merge_from_env()?;
            config.save(config_path)?;
            let env_storages = config::Config::load_from_env()?;
            if env_storages.is_empty() {
                println!(
                    "{}",
                    Formatter::format_text("No storages found in environment variables", format)
                );
            } else {
                let count = env_storages.len();
                println!(
                    "{}",
                    Formatter::format_success(
                        &format!("Loaded {} storage(ies) from environment variables", count),
                        format
                    )
                );
                for (name, _) in env_storages {
                    println!("  - {}", name);
                }
            }
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
            let title = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("blog post");
            println!(
                "{}",
                Formatter::format_success(&format!("Successfully published: {}", title), format)
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

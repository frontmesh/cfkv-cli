use serde_json::json;

#[derive(Clone, Copy, Debug)]
pub enum OutputFormat {
    Json,
    Yaml,
    Text,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(OutputFormat::Json),
            "yaml" | "yml" => Some(OutputFormat::Yaml),
            "text" => Some(OutputFormat::Text),
            _ => None,
        }
    }
}

pub struct Formatter;

impl Formatter {
    /// Format a text value based on the output format
    fn format_json(value: serde_json::Value) -> String {
        serde_json::to_string(&value).unwrap_or_else(|_| String::new())
    }

    /// Format a value for JSON and YAML output
    fn format_structured(value: serde_json::Value, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_json(value),
            OutputFormat::Yaml => serde_yaml::to_string(&value).unwrap_or_else(|_| String::new()),
            OutputFormat::Text => String::new(),
        }
    }

    pub fn format_text(text: &str, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_structured(json!({ "value": text }), format),
            OutputFormat::Yaml => Self::format_structured(json!({ "value": text }), format),
            OutputFormat::Text => text.to_string(),
        }
    }

    pub fn format_success(message: &str, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_json(json!({ "success": true, "message": message })),
            OutputFormat::Yaml => {
                Self::format_structured(json!({ "success": true, "message": message }), format)
            }
            OutputFormat::Text => message.to_string(),
        }
    }

    pub fn format_error(error: &str, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_json(json!({ "error": error, "success": false })),
            OutputFormat::Yaml => {
                Self::format_structured(json!({ "error": error, "success": false }), format)
            }
            OutputFormat::Text => format!("Error: {}", error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert!(matches!(
            OutputFormat::from_str("json"),
            Some(OutputFormat::Json)
        ));
        assert!(matches!(
            OutputFormat::from_str("yaml"),
            Some(OutputFormat::Yaml)
        ));
        assert!(matches!(
            OutputFormat::from_str("yml"),
            Some(OutputFormat::Yaml)
        ));
        assert!(matches!(
            OutputFormat::from_str("text"),
            Some(OutputFormat::Text)
        ));
        assert!(OutputFormat::from_str("invalid").is_none());
    }

    #[test]
    fn test_output_format_case_insensitive() {
        assert!(matches!(
            OutputFormat::from_str("JSON"),
            Some(OutputFormat::Json)
        ));
        assert!(matches!(
            OutputFormat::from_str("YAML"),
            Some(OutputFormat::Yaml)
        ));
        assert!(matches!(
            OutputFormat::from_str("TEXT"),
            Some(OutputFormat::Text)
        ));
    }

    #[test]
    fn test_format_text() {
        assert_eq!(
            Formatter::format_text("hello world", OutputFormat::Text),
            "hello world"
        );
        assert!(Formatter::format_text("test", OutputFormat::Json).contains("value"));
        assert!(Formatter::format_text("test", OutputFormat::Yaml).contains("value"));
    }

    #[test]
    fn test_format_success() {
        let msg = "Operation successful";
        assert_eq!(Formatter::format_success(msg, OutputFormat::Text), msg);
        assert!(Formatter::format_success(msg, OutputFormat::Json).contains("success"));
    }

    #[test]
    fn test_format_error() {
        let err = "Something went wrong";
        assert!(Formatter::format_error(err, OutputFormat::Text).contains("Error"));
        assert!(Formatter::format_error(err, OutputFormat::Json).contains("error"));
    }

    #[test]
    fn test_format_special_characters() {
        let text = "Hello \"World\" with 'quotes' and \\ backslash";
        assert_eq!(Formatter::format_text(text, OutputFormat::Text), text);
    }
}

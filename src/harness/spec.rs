use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::{Result, TagisanError};

/// Target language or source type for the synthesized harness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Python,
    Rust,
    JavaScript,
    TypeScript,
    OpenApi,
    Auto,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Python => write!(f, "python"),
            Self::Rust => write!(f, "rust"),
            Self::JavaScript => write!(f, "javascript"),
            Self::TypeScript => write!(f, "typescript"),
            Self::OpenApi => write!(f, "openapi"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

impl FromStr for SourceType {
    type Err = TagisanError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "python" | "py" => Ok(Self::Python),
            "rust" | "rs" => Ok(Self::Rust),
            "javascript" | "js" | "mjs" | "cjs" => Ok(Self::JavaScript),
            "typescript" | "ts" | "mts" | "cts" => Ok(Self::TypeScript),
            "openapi" | "swagger" | "yaml" | "yml" => Ok(Self::OpenApi),
            "auto" | "" => Ok(Self::Auto),
            other => Err(TagisanError::Execution(format!(
                "Unsupported source language: '{}'. Supported: python, rust, javascript, typescript, openapi, auto",
                other
            ))),
        }
    }
}

/// Output formats supported by the synthesized CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Json,
    Text,
    Table,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Text => write!(f, "text"),
            Self::Table => write!(f, "table"),
        }
    }
}

/// Argument type descriptor for synthesized CLI flags and positional arguments
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "details", rename_all = "lowercase")]
pub enum ArgumentType {
    String,
    Integer,
    Float,
    Boolean,
    List(Box<ArgumentType>),
    Dict,
    FilePath,
    Choice(Vec<String>),
}

impl ArgumentType {
    /// Maps a type name string (from Python/TS/Rust/OpenAPI) to an ArgumentType
    pub fn from_type_str(type_str: &str) -> Self {
        let cleaned = type_str.trim().to_lowercase();
        if cleaned.is_empty() {
            return Self::String;
        }

        // Check for Optional[T] or Option<T>
        let unwrap_optional = if cleaned.starts_with("optional[") && cleaned.ends_with(']') {
            &cleaned[9..cleaned.len() - 1]
        } else if cleaned.starts_with("option<") && cleaned.ends_with('>') {
            &cleaned[7..cleaned.len() - 1]
        } else {
            &cleaned[..]
        };

        let target = unwrap_optional.trim();

        // Check for List[T], list[T], Vec<T>, Array<T>, T[]
        if (target.starts_with("list[") && target.ends_with(']'))
            || (target.starts_with("vec<") && target.ends_with('>'))
            || (target.starts_with("array<") && target.ends_with('>'))
        {
            let inner_start = target.find('[').or_else(|| target.find('<')).unwrap_or(0) + 1;
            let inner_str = &target[inner_start..target.len() - 1];
            return Self::List(Box::new(Self::from_type_str(inner_str)));
        }
        if target.ends_with("[]") {
            let inner_str = &target[..target.len() - 2];
            return Self::List(Box::new(Self::from_type_str(inner_str)));
        }

        match target {
            "int" | "integer" | "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16"
            | "u32" | "u64" | "usize" => Self::Integer,
            "float" | "double" | "number" | "f32" | "f64" => Self::Float,
            "bool" | "boolean" => Self::Boolean,
            "path" | "filepath" | "pathbuf" | "file" => Self::FilePath,
            "dict" | "dictionary" | "json" | "object" | "hashmap" | "map" | "record" => {
                Self::Dict
            }
            "list" | "vec" | "array" => Self::List(Box::new(Self::String)),
            _ => Self::String,
        }
    }

    /// Returns the Python type hint string
    pub fn python_type_hint(&self) -> String {
        match self {
            Self::String => "str".to_string(),
            Self::Integer => "int".to_string(),
            Self::Float => "float".to_string(),
            Self::Boolean => "bool".to_string(),
            Self::List(inner) => format!("List[{}]", inner.python_type_hint()),
            Self::Dict => "Dict[str, Any]".to_string(),
            Self::FilePath => "str".to_string(),
            Self::Choice(_) => "str".to_string(),
        }
    }

    /// Returns the argparse type specification
    pub fn argparse_type_code(&self) -> String {
        match self {
            Self::String | Self::FilePath | Self::Choice(_) => "type=str".to_string(),
            Self::Integer => "type=int".to_string(),
            Self::Float => "type=float".to_string(),
            Self::Boolean => "action='store_true'".to_string(),
            Self::List(inner) => match **inner {
                Self::Integer => "type=int, nargs='+'".to_string(),
                Self::Float => "type=float, nargs='+'".to_string(),
                _ => "type=str, nargs='+'".to_string(),
            },
            Self::Dict => "type=json.loads".to_string(),
        }
    }

    /// Returns a realistic sample value for testing and examples
    pub fn sample_value(&self) -> String {
        match self {
            Self::String => "\"example\"".to_string(),
            Self::Integer => "42".to_string(),
            Self::Float => "3.14".to_string(),
            Self::Boolean => "true".to_string(),
            Self::List(inner) => format!("[{}]", inner.sample_value()),
            Self::Dict => "{\"key\": \"value\"}".to_string(),
            Self::FilePath => "\"/tmp/sample.txt\"".to_string(),
            Self::Choice(choices) => choices
                .first()
                .cloned()
                .unwrap_or_else(|| "option1".to_string()),
        }
    }

    /// Returns CLI parameter invocation example (e.g. `--num 42`)
    pub fn sample_cli_arg(&self, flag: &str) -> String {
        match self {
            Self::Boolean => flag.to_string(),
            Self::Integer => format!("{} 42", flag),
            Self::Float => format!("{} 3.14", flag),
            Self::String => format!("{} \"test-value\"", flag),
            Self::FilePath => format!("{} ./sample.txt", flag),
            Self::List(_) => format!("{} item1 item2", flag),
            Self::Dict => format!("{} '{{\"key\": \"value\"}}'", flag),
            Self::Choice(c) => format!(
                "{} {}",
                flag,
                c.first().map(|s| s.as_str()).unwrap_or("choice1")
            ),
        }
    }
}

/// Specification for a single command line argument
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgumentSpec {
    pub name: String,
    pub arg_type: ArgumentType,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub short: Option<char>,
    pub positional: bool,
}

impl ArgumentSpec {
    pub fn new(name: impl Into<String>, arg_type: ArgumentType) -> Self {
        let name_str = name.into();
        Self {
            name: name_str,
            arg_type,
            description: String::new(),
            required: false,
            default_value: None,
            short: None,
            positional: false,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_default(mut self, default_val: impl Into<String>) -> Self {
        self.default_value = Some(default_val.into());
        self
    }

    pub fn with_short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    pub fn with_positional(mut self, positional: bool) -> Self {
        self.positional = positional;
        self
    }

    /// Returns the normalized CLI flag name (e.g. `user_id` -> `--user-id`)
    pub fn cli_flag(&self) -> String {
        let norm = self.name.replace('_', "-").to_lowercase();
        format!("--{}", norm)
    }

    /// Returns the Python variable/attribute name (e.g. `user-id` -> `user_id`)
    pub fn python_var_name(&self) -> String {
        self.name.replace('-', "_").to_lowercase()
    }
}

/// Specification for a single subcommand in the synthesized harness
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandSpec {
    pub name: String,
    pub description: String,
    pub function_name: String,
    pub class_name: Option<String>,
    pub arguments: Vec<ArgumentSpec>,
    pub return_type: Option<String>,
    pub examples: Vec<String>,
}

impl CommandSpec {
    pub fn new(name: impl Into<String>, function_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            function_name: function_name.into(),
            class_name: None,
            arguments: Vec::new(),
            return_type: None,
            examples: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_class_name(mut self, class_name: impl Into<String>) -> Self {
        self.class_name = Some(class_name.into());
        self
    }

    pub fn with_argument(mut self, arg: ArgumentSpec) -> Self {
        self.arguments.push(arg);
        self
    }

    pub fn with_arguments(mut self, args: Vec<ArgumentSpec>) -> Self {
        self.arguments = args;
        self
    }

    pub fn with_return_type(mut self, ret: impl Into<String>) -> Self {
        self.return_type = Some(ret.into());
        self
    }

    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }

    /// Formats a complete example invocation of this subcommand
    pub fn sample_invocation(&self, harness_bin: &str) -> String {
        let mut parts = vec![harness_bin.to_string(), self.name.clone()];
        for arg in &self.arguments {
            if arg.positional {
                parts.push(arg.arg_type.sample_value());
            } else if arg.required {
                parts.push(arg.arg_type.sample_cli_arg(&arg.cli_flag()));
            }
        }
        parts.push("--json".to_string());
        parts.join(" ")
    }
}

/// Specification for the entire synthesized CLI harness
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessSpec {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source_path: PathBuf,
    pub source_type: SourceType,
    pub commands: Vec<CommandSpec>,
    pub global_options: Vec<ArgumentSpec>,
    pub metadata: HashMap<String, String>,
}

impl HarnessSpec {
    pub fn new(
        name: impl Into<String>,
        source_path: impl Into<PathBuf>,
        source_type: SourceType,
    ) -> Self {
        let name_str = name.into();
        let mut global_options = Vec::new();

        // Standard global options for agent-native CLIs
        global_options.push(
            ArgumentSpec::new("json", ArgumentType::Boolean)
                .with_description("Emit structured machine-readable JSON output to stdout")
                .with_default("false"),
        );
        global_options.push(
            ArgumentSpec::new("verbose", ArgumentType::Boolean)
                .with_short('v')
                .with_description("Enable verbose diagnostic output to stderr")
                .with_default("false"),
        );
        global_options.push(
            ArgumentSpec::new("quiet", ArgumentType::Boolean)
                .with_short('q')
                .with_description("Suppress non-essential progress output")
                .with_default("false"),
        );

        Self {
            name: name_str,
            version: "0.1.0".to_string(),
            description: String::new(),
            source_path: source_path.into(),
            source_type,
            commands: Vec::new(),
            global_options,
            metadata: HashMap::new(),
        }
    }

    pub fn with_version(mut self, ver: impl Into<String>) -> Self {
        self.version = ver.into();
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_command(mut self, cmd: CommandSpec) -> Self {
        self.commands.push(cmd);
        self
    }

    pub fn with_commands(mut self, cmds: Vec<CommandSpec>) -> Self {
        self.commands = cmds;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn find_command(&self, name: &str) -> Option<&CommandSpec> {
        let norm = name.trim().to_lowercase().replace('_', "-");
        self.commands
            .iter()
            .find(|c| c.name.to_lowercase().replace('_', "-") == norm)
    }

    /// Validates the harness specification for integrity and consistency
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(TagisanError::Execution(
                "Harness specification missing 'name'".to_string(),
            ));
        }

        if self.commands.is_empty() {
            return Err(TagisanError::Execution(format!(
                "Harness specification '{}' has zero commands defined",
                self.name
            )));
        }

        let mut seen_commands = std::collections::HashSet::new();
        for cmd in &self.commands {
            let cmd_norm = cmd.name.trim().to_lowercase().replace('_', "-");
            if cmd_norm.is_empty() {
                return Err(TagisanError::Execution(
                    "Command name cannot be empty".to_string(),
                ));
            }
            if !seen_commands.insert(cmd_norm.clone()) {
                return Err(TagisanError::Execution(format!(
                    "Duplicate command name in harness '{}': {}",
                    self.name, cmd.name
                )));
            }
        }

        Ok(())
    }
}

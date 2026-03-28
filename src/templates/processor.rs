//! Advanced template processor.
//!
//! Implements intelligent template processing including variable substitution,
//! validation, and text transformations.

use crate::error::CliError;
use crate::templates::config::*;
use regex::Regex;

use std::collections::HashMap;
use std::fmt;

type TemplateFn = Box<dyn Fn(&[String]) -> Result<String, CliError>>;

/// Template processor
pub struct TemplateProcessor {
    /// Global variables
    global_variables: HashMap<String, String>,
    /// Custom registered functions
    custom_functions: HashMap<String, TemplateFn>,
    /// Compiled regex cache
    regex_cache: HashMap<String, Regex>,
}

/// Processing context
#[derive(Debug, Clone)]
pub struct ProcessingContext {
    /// Context variables
    pub variables: HashMap<String, String>,
    /// Developer profile
    pub developer_profile: DeveloperProfile,
    /// Template-specific configuration
    pub template_config: Option<TemplateSpecificConfig>,
    /// Processing mode
    pub mode: ProcessingMode,
}

/// Processing modes
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingMode {
    /// Normal mode
    Normal,
    /// Debug mode (preserves debug comments)
    Debug,
    /// Production mode (optimized output)
    Production,
    /// Interactive mode
    Interactive,
}

/// Processing result
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Processed content
    pub content: String,
    /// Variables that were substituted
    pub used_variables: Vec<String>,
    /// Functions that were called
    pub used_functions: Vec<String>,
    /// Warnings generated during processing
    pub warnings: Vec<String>,
    /// Processing statistics
    pub stats: ProcessingStats,
}

/// Processing statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessingStats {
    /// Number of substitutions performed
    pub substitutions: u32,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Original content size
    pub original_size: usize,
    /// Final content size
    pub final_size: usize,
}

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateProcessor {
    /// Creates a new processor
    pub fn new() -> Self {
        let mut processor = Self {
            global_variables: HashMap::new(),
            custom_functions: HashMap::new(),
            regex_cache: HashMap::new(),
        };
        
        processor.register_builtin_functions();
        processor
    }

    /// Registers built-in functions
    fn register_builtin_functions(&mut self) {
        // Convert to CamelCase
        self.register_function("camelCase", Box::new(|args: &[String]| -> Result<String, CliError> {
            if args.is_empty() {
                return Err(CliError::ValidationError("camelCase requires at least one argument".to_string()));
            }
            Ok(to_camel_case(&args[0]))
        }));

        // Convert to snake_case
        self.register_function("snakeCase", Box::new(|args: &[String]| -> Result<String, CliError> {
            if args.is_empty() {
                return Err(CliError::ValidationError("snakeCase requires at least one argument".to_string()));
            }
            Ok(to_snake_case(&args[0]))
        }));

        // Convert to kebab-case
        self.register_function("kebabCase", Box::new(|args: &[String]| -> Result<String, CliError> {
            if args.is_empty() {
                return Err(CliError::ValidationError("kebabCase requires at least one argument".to_string()));
            }
            Ok(to_kebab_case(&args[0]))
        }));

        // Convert to uppercase
        self.register_function("upper", Box::new(|args: &[String]| -> Result<String, CliError> {
            if args.is_empty() {
                return Err(CliError::ValidationError("upper requires at least one argument".to_string()));
            }
            Ok(args[0].to_uppercase())
        }));

        // Convert to lowercase
        self.register_function("lower", Box::new(|args: &[String]| -> Result<String, CliError> {
            if args.is_empty() {
                return Err(CliError::ValidationError("lower requires at least one argument".to_string()));
            }
            Ok(args[0].to_lowercase())
        }));

        // Current date/time
        self.register_function("now", Box::new(|args: &[String]| -> Result<String, CliError> {
            let format = args.first().map(|s| s.as_str()).unwrap_or("%Y-%m-%d %H:%M:%S");
            let now = chrono::Local::now();
            Ok(now.format(format).to_string())
        }));

        // Generate UUID
        self.register_function("uuid", Box::new(|_args: &[String]| -> Result<String, CliError> {
            Ok(uuid::Uuid::new_v4().to_string())
        }));
    }

    /// Registers a custom function
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[String]) -> Result<String, CliError> + 'static,
    {
        self.custom_functions.insert(name.to_string(), Box::new(func));
    }

    /// Sets a global variable
    pub fn set_global_variable(&mut self, name: &str, value: &str) {
        self.global_variables.insert(name.to_string(), value.to_string());
    }

    /// Processes a template with the given context
    pub fn process(&mut self, content: &str, context: &ProcessingContext) -> Result<ProcessingResult, CliError> {
        let start_time = std::time::Instant::now();
        let original_size = content.len();
        
        let mut result = ProcessingResult {
            content: content.to_string(),
            used_variables: Vec::new(),
            used_functions: Vec::new(),
            warnings: Vec::new(),
            stats: ProcessingStats {
                original_size,
                ..Default::default()
            },
        };

        // 1. Process simple variables
        let current_content = result.content.clone();
        let content_after_vars = self.process_variables(&current_content, context, &mut result)?;
        result.content = content_after_vars;

        // 2. Process functions
        let current_content = result.content.clone();
        let content_after_funcs = self.process_functions(&current_content, context, &mut result)?;
        result.content = content_after_funcs;

        // 3. Process conditionals
        let current_content = result.content.clone();
        let content_after_conditionals = self.process_conditionals(&current_content, context, &mut result)?;
        result.content = content_after_conditionals;

        // 4. Process loops
        let current_content = result.content.clone();
        let content_after_loops = self.process_loops(&current_content, context, &mut result)?;
        result.content = content_after_loops;

        // 5. Final cleanup
        if context.mode == ProcessingMode::Production {
            result.content = self.cleanup_production(&result.content)?;
        }

        // Update statistics
        result.stats.processing_time_ms = start_time.elapsed().as_millis() as u64;
        result.stats.final_size = result.content.len();

        Ok(result)
    }

    /// Processes simple variable substitutions
    fn process_variables(
        &mut self,
        content: &str,
        context: &ProcessingContext,
        result: &mut ProcessingResult,
    ) -> Result<String, CliError> {
        let var_regex_pattern = r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}";
        let var_regex = self.get_or_compile_regex(var_regex_pattern)?.clone();
        let mut processed = content.to_string();
        let mut used_vars = Vec::new();
        let mut warnings = Vec::new();
        let mut substitutions = 0;
        
        for cap in var_regex.captures_iter(content) {
            let full_match = &cap[0];
            let var_name = &cap[1];
            
            if let Some(value) = self.get_variable_value(var_name, context) {
                processed = processed.replace(full_match, &value);
                used_vars.push(var_name.to_string());
                substitutions += 1;
            } else {
                warnings.push(format!("Variable '{}' not found", var_name));
            }
        }
        
        result.used_variables.extend(used_vars);
        result.warnings.extend(warnings);
        result.stats.substitutions += substitutions;
        Ok(processed)
    }

    /// Processes function calls
    fn process_functions(
        &mut self,
        content: &str,
        _context: &ProcessingContext,
        result: &mut ProcessingResult,
    ) -> Result<String, CliError> {
        let func_regex_pattern = r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*\}\}";
        let func_regex = self.get_or_compile_regex(func_regex_pattern)?.clone();
        let mut processed = content.to_string();
        let mut used_funcs = Vec::new();
        let mut warnings = Vec::new();
        let mut substitutions = 0;
        
        // Collect all matches first to avoid borrowing issues
        let matches: Vec<_> = func_regex.captures_iter(content).collect();
        
        for cap in matches {
            let full_match = &cap[0];
            let func_name = &cap[1];
            let args_str = &cap[2];
            
            if let Some(func) = self.custom_functions.get(func_name) {
                let args: Vec<String> = if args_str.trim().is_empty() {
                    Vec::new()
                } else {
                    args_str.split(',').map(|s| s.trim().to_string()).collect()
                };
                
                match func(&args) {
                    Ok(value) => {
                        processed = processed.replace(full_match, &value);
                        used_funcs.push(func_name.to_string());
                        substitutions += 1;
                    }
                    Err(e) => {
                        warnings.push(format!("Function '{}' error: {}", func_name, e));
                    }
                }
            } else {
                warnings.push(format!("Function '{}' not found", func_name));
            }
        }
        
        result.used_functions.extend(used_funcs);
        result.warnings.extend(warnings);
        result.stats.substitutions += substitutions;
        Ok(processed)
    }

    /// Processes conditional blocks
    fn process_conditionals(
        &mut self,
        content: &str,
        context: &ProcessingContext,
        result: &mut ProcessingResult,
    ) -> Result<String, CliError> {
        let if_regex_pattern = r"\{%\s*if\s+([^%]+)\s*%\}([\s\S]*?)\{%\s*endif\s*%\}";
        let if_regex = self.get_or_compile_regex(if_regex_pattern)?.clone();
        let mut processed = content.to_string();
        let mut substitutions = 0;
        
        for cap in if_regex.captures_iter(content) {
            let full_match = &cap[0];
            let condition = &cap[1];
            let if_content = &cap[2];
            
            if self.evaluate_condition(condition, context)? {
                processed = processed.replace(full_match, if_content);
            } else {
                processed = processed.replace(full_match, "");
            }
            substitutions += 1;
        }
        
        result.stats.substitutions += substitutions;
        Ok(processed)
    }

    /// Processes loop blocks
    fn process_loops(
        &mut self,
        content: &str,
        context: &ProcessingContext,
        result: &mut ProcessingResult,
    ) -> Result<String, CliError> {
        let for_regex_pattern = r"\{%\s*for\s+(\w+)\s+in\s+(\w+)\s*%\}([\s\S]*?)\{%\s*endfor\s*%\}";
        let for_regex = self.get_or_compile_regex(for_regex_pattern)?.clone();
        let mut processed = content.to_string();
        let mut substitutions = 0;
        
        for cap in for_regex.captures_iter(content) {
            let full_match = &cap[0];
            let item_var = &cap[1];
            let array_var = &cap[2];
            let loop_content = &cap[3];
            
            if let Some(array_value) = self.get_variable_value(array_var, context) {
                let items: Vec<&str> = array_value.split(',').collect();
                let mut loop_result = String::new();
                
                for item in items {
                    let item_content = loop_content.replace(&format!("{{{{{}}}}}", item_var), item.trim());
                    loop_result.push_str(&item_content);
                }
                
                processed = processed.replace(full_match, &loop_result);
                substitutions += 1;
            }
        }
        
        result.stats.substitutions += substitutions;
        Ok(processed)
    }

    /// Cleans up output for production mode
    fn cleanup_production(&self, content: &str) -> Result<String, CliError> {
        let mut cleaned = content.to_string();
        
        // Remove debug comments
        let debug_regex = Regex::new(r"//\s*DEBUG:.*\n")?;
        cleaned = debug_regex.replace_all(&cleaned, "").to_string();
        
        // Remove excessive blank lines
        let empty_lines_regex = Regex::new(r"\n\s*\n\s*\n")?;
        cleaned = empty_lines_regex.replace_all(&cleaned, "\n\n").to_string();
        
        Ok(cleaned)
    }

    /// Gets the value of a variable
    fn get_variable_value(&self, name: &str, context: &ProcessingContext) -> Option<String> {
        // 1. Context variables
        if let Some(value) = context.variables.get(name) {
            return Some(value.clone());
        }

        // 2. Developer profile variables
        match name {
            "author" => Some(context.developer_profile.name.clone()),
            "email" => Some(context.developer_profile.email.clone()),
            "organization" => context.developer_profile.organization.clone(),
            "namespace" => context.developer_profile.default_namespace.clone(),
            _ => None,
        }.or_else(|| {
            // 3. Global variables
            self.global_variables.get(name).cloned()
        })
    }

    /// Evaluates a condition
    fn evaluate_condition(&self, condition: &str, context: &ProcessingContext) -> Result<bool, CliError> {
        let condition = condition.trim();

        // Simple condition: variable exists and is truthy
        if let Some(value) = self.get_variable_value(condition, context) {
            return Ok(!value.is_empty() && value != "false" && value != "0");
        }
        
        // Equality comparison
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let left = self.get_variable_value(parts[0].trim(), context).unwrap_or_default();
                let right = parts[1].trim().trim_matches('"').trim_matches('\'');
                return Ok(left == right);
            }
        }
        
        Ok(false)
    }

    /// Gets or compiles a regex (cached)
    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex, CliError> {
        if !self.regex_cache.contains_key(pattern) {
            let regex = Regex::new(pattern)
                .map_err(|e| CliError::ValidationError(format!("Invalid regex: {}", e)))?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }
        Ok(self.regex_cache.get(pattern).unwrap())
    }
}

/// Converts a string to CamelCase
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in s.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                result.push(c.to_uppercase().next().unwrap());
                capitalize_next = false;
            } else {
                result.push(c.to_lowercase().next().unwrap());
            }
        } else {
            capitalize_next = true;
        }
    }
    
    result
}

/// Converts a string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        if c.is_alphanumeric() {
            result.push(c.to_lowercase().next().unwrap());
        } else if c == ' ' || c == '-' {
            result.push('_');
        }
    }
    
    result
}

/// Converts a string to kebab-case
fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

impl Default for ProcessingContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            developer_profile: DeveloperProfile::default(),
            template_config: None,
            mode: ProcessingMode::Normal,
        }
    }
}

impl fmt::Display for ProcessingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingMode::Normal => write!(f, "normal"),
            ProcessingMode::Debug => write!(f, "debug"),
            ProcessingMode::Production => write!(f, "production"),
            ProcessingMode::Interactive => write!(f, "interactive"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with(vars: &[(&str, &str)]) -> ProcessingContext {
        let mut ctx = ProcessingContext::default();
        for (k, v) in vars {
            ctx.variables.insert(k.to_string(), v.to_string());
        }
        ctx
    }

    #[test]
    fn test_process_simple_variable_substitution() {
        let mut p = TemplateProcessor::new();
        let ctx = ctx_with(&[("mod", "User")]);
        let result = p.process("unit {{mod}}Service;", &ctx).unwrap();
        assert_eq!(result.content, "unit UserService;");
        assert_eq!(result.stats.substitutions, 1);
        assert!(result.used_variables.contains(&"mod".to_string()));
    }

    #[test]
    fn test_process_unknown_variable_generates_warning() {
        let mut p = TemplateProcessor::new();
        let ctx = ProcessingContext::default();
        let result = p.process("{{unknown}}", &ctx).unwrap();
        assert!(result.content.contains("{{unknown}}"));
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_process_function_upper() {
        let mut p = TemplateProcessor::new();
        let ctx = ProcessingContext::default();
        let result = p.process("{{upper(hello)}}", &ctx).unwrap();
        assert_eq!(result.content, "HELLO");
    }

    #[test]
    fn test_process_function_lower() {
        let mut p = TemplateProcessor::new();
        let ctx = ProcessingContext::default();
        let result = p.process("{{lower(WORLD)}}", &ctx).unwrap();
        assert_eq!(result.content, "world");
    }

    #[test]
    fn test_process_function_camel_case() {
        let mut p = TemplateProcessor::new();
        let ctx = ProcessingContext::default();
        let result = p.process("{{camelCase(my module)}}", &ctx).unwrap();
        assert_eq!(result.content, "MyModule");
    }

    #[test]
    fn test_process_function_snake_case() {
        let mut p = TemplateProcessor::new();
        let ctx = ProcessingContext::default();
        let result = p.process("{{snakeCase(MyService)}}", &ctx).unwrap();
        assert_eq!(result.content, "my_service");
    }

    #[test]
    fn test_process_conditional_true() {
        let mut p = TemplateProcessor::new();
        let ctx = ctx_with(&[("feature", "enabled")]);
        let result = p.process("{%if feature%}FEATURE{%endif%}", &ctx).unwrap();
        assert_eq!(result.content, "FEATURE");
    }

    #[test]
    fn test_process_conditional_false() {
        let mut p = TemplateProcessor::new();
        let ctx = ProcessingContext::default();
        let result = p.process("{%if missing%}NEVER{%endif%}", &ctx).unwrap();
        assert_eq!(result.content, "");
    }

    #[test]
    fn test_global_variable_used_in_template() {
        let mut p = TemplateProcessor::new();
        p.set_global_variable("app", "Nidus");
        let ctx = ProcessingContext::default();
        let result = p.process("{{app}}", &ctx).unwrap();
        assert_eq!(result.content, "Nidus");
    }

    #[test]
    fn test_production_mode_removes_debug_comments() {
        let mut p = TemplateProcessor::new();
        let mut ctx = ProcessingContext::default();
        ctx.mode = ProcessingMode::Production;
        let result = p.process("code\n// DEBUG: remove this\nmore", &ctx).unwrap();
        assert!(!result.content.contains("DEBUG"));
        assert!(result.content.contains("code"));
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("my module"), "MyModule");
        assert_eq!(to_camel_case("user-service"), "UserService");
        assert_eq!(to_camel_case("hello"), "Hello");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("MyService"), "my_service");
        assert_eq!(to_snake_case("UserController"), "user_controller");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("MyService"), "my-service");
    }
}
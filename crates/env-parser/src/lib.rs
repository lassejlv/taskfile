use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Clone)]
pub struct EnvConfig {
    pub files: Vec<String>,
}

#[derive(Debug)]
pub struct EnvParser {
    config: Option<EnvConfig>,
}

impl EnvParser {
    pub fn new() -> Self {
        Self { config: None }
    }

    pub fn with_config(config: EnvConfig) -> Self {
        Self {
            config: Some(config),
        }
    }

    pub fn load_env_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(env_config) = &self.config {
            for file_path in &env_config.files {
                if std::path::Path::new(file_path).exists() {
                    match self.load_env_file(file_path) {
                        Ok(count) => {
                            println!("Loaded {} environment variables from: {}", count, file_path)
                        }
                        Err(e) => eprintln!("Warning: Failed to load {}: {}", file_path, e),
                    }
                }
            }
        }
        Ok(())
    }

    fn load_env_file(&self, file_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE format
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                // Remove quotes if present
                let value = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    &value[1..value.len() - 1]
                } else {
                    value
                };

                unsafe {
                    env::set_var(key, value);
                }
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn substitute_env_vars(&self, command: &str) -> String {
        let mut result = command.to_string();

        // Find all $variable_name patterns
        let mut start = 0;
        while let Some(dollar_pos) = result[start..].find('$') {
            let dollar_pos = start + dollar_pos;
            let var_start = dollar_pos + 1;

            // Find the end of the variable name (alphanumeric + underscore)
            let var_end = result[var_start..]
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|pos| var_start + pos)
                .unwrap_or(result.len());

            if var_end > var_start {
                let var_name = &result[var_start..var_end];

                if let Ok(env_value) = env::var(var_name) {
                    result.replace_range(dollar_pos..var_end, &env_value);
                    start = dollar_pos + env_value.len();
                } else {
                    eprintln!("Warning: Environment variable '{}' not found", var_name);
                    start = var_end;
                }
            } else {
                start = dollar_pos + 1;
            }
        }

        result
    }

    pub fn get_env_var(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }

    pub fn set_env_var(&self, key: &str, value: &str) {
        unsafe {
            env::set_var(key, value);
        }
    }

    pub fn list_env_vars(&self) -> HashMap<String, String> {
        env::vars().collect()
    }
}

impl Default for EnvParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_substitute_env_vars() {
        let parser = EnvParser::new();

        // Set a test environment variable
        parser.set_env_var("TEST_VAR", "test_value");

        let result = parser.substitute_env_vars("Hello $TEST_VAR world");
        assert_eq!(result, "Hello test_value world");
    }

    #[test]
    fn test_substitute_missing_var() {
        let parser = EnvParser::new();

        let result = parser.substitute_env_vars("Hello $MISSING_VAR world");
        assert_eq!(result, "Hello $MISSING_VAR world");
    }

    #[test]
    fn test_load_env_file() {
        let parser = EnvParser::new();

        // Create a temporary env file
        let env_content = "TEST_KEY=test_value\n# This is a comment\nANOTHER_KEY=another_value\n";
        let mut file = fs::File::create("test.env").unwrap();
        file.write_all(env_content.as_bytes()).unwrap();

        let result = parser.load_env_file("test.env");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        assert_eq!(
            parser.get_env_var("TEST_KEY"),
            Some("test_value".to_string())
        );
        assert_eq!(
            parser.get_env_var("ANOTHER_KEY"),
            Some("another_value".to_string())
        );

        // Clean up
        fs::remove_file("test.env").unwrap();
    }

    #[test]
    fn test_env_config() {
        let config = EnvConfig {
            files: vec![".env".to_string(), ".env.local".to_string()],
        };

        let parser = EnvParser::with_config(config);
        assert!(parser.config.is_some());
    }
}

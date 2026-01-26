use std::io::{self, Write};

use colored::*;
use toml::Value;
use crate::core::plugin::{ConfigField, ConfigFieldType};
use crate::error::{Result, WorkOsError};

pub struct Terminal {}

impl Terminal {
    pub fn prompt(message: &str) -> Result<String> {
        print!("{}", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_string())
    }

    pub fn prompt_secret(message: &str) -> Result<String> {
        print!("{}", message);
        io::stdout().flush()?;

        rpassword::read_password()
            .map_err(|e| WorkOsError::Config(format!("Failed to read secret: {}", e)))
    }

    pub fn prompt_for_field(field: &ConfigField, current: Option<&Value>) -> Result<Option<Value>> {
        let current_hint = match current {
            Some(v) if field.is_secret() => Some("[currently set]".to_string()),
            Some(v) => Some(format!("[current: {}]", ConfigFieldType::format_value(v))),
            None => field.default.map(|d| format!("[default: {}]", d)),
        };

        let prompt_text = match &current_hint {
            Some(hint) => format!("  {} {}: ", field.label, hint),
            None => format!("  {}: ", field.label),
        };

        println!("  {}", field.help.dimmed());

        let input = match field.field_type {
            ConfigFieldType::Secret => Terminal::prompt_secret(&prompt_text)?,
            _ => Terminal::prompt(&prompt_text)?,
        };

        if input.is_empty() {
            if current.is_some() {
                return Ok(None);
            }

            if let Some(default) = field.default {
                return Ok(Some(ConfigFieldType::parse_value(
                    default,
                    &field.field_type,
                )));
            }
            if field.required {
                return Err(WorkOsError::Config(format!("{} is required", field.label)));
            }
            return Ok(None);
        }

        Ok(Some(ConfigFieldType::parse_value(
            &input,
            &field.field_type,
        )))
    }
}

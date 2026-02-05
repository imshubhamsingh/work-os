use std::io::{self, Write};

use crate::core::plugin::{ConfigField, ConfigFieldType, NestedFieldSchema};
use crate::error::{Result, WorkOsError};
use colored::*;
use toml::Value;

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
        if let ConfigFieldType::NestedArray(schema) = &field.field_type {
            return Terminal::prompt_nested_array(field, schema, current);
        }

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

    fn prompt_nested_array(
        field: &ConfigField,
        schema: &[NestedFieldSchema],
        current: Option<&Value>,
    ) -> Result<Option<Value>> {
        println!("\n{} {}", "🔍".bold(), field.label.bold());
        println!("  {}", field.help.dimmed());

        if let Some(current_val) = current {
            if let Value::Array(arr) = current_val {
                println!("  {} {} item(s)", "[current:".dimmed(), arr.len());
            }
        }

        if !Terminal::prompt_yes_no(&format!("\n  Configure {}? (y/n): ", field.label))? {
            return Ok(None);
        }

        let mut items = Vec::new();

        loop {
            println!("\n  {} {}", "→".cyan(), "Adding new item".bold());

            let mut item_map = toml::map::Map::new();
            let mut skip_item = false;

            for nested_field in schema {
                let value_result = Terminal::prompt_nested_field(nested_field)?;

                match value_result {
                    Some(value) => {
                        item_map.insert(nested_field.name.to_string(), value);
                    }
                    None => {
                        if nested_field.required {
                            println!(
                                "  {} {} is required, skipping item...",
                                "✖".red(),
                                nested_field.label
                            );
                            skip_item = true;
                            break;
                        }
                    }
                }
            }

            if !skip_item {
                items.push(Value::Table(item_map));
                println!("  {} Item added!", "✓".green());
            }

            if !Terminal::prompt_yes_no("\n  Add another item? (y/n): ")? {
                break;
            }
        }

        if items.is_empty() {
            Ok(None)
        } else {
            println!("\n  {} {} item(s) configured!", "✓".green(), items.len());
            Ok(Some(Value::Array(items)))
        }
    }

    fn prompt_nested_field(field: &NestedFieldSchema) -> Result<Option<Value>> {
        let default_hint = field.default.map(|d| format!(" [default: {}]", d));
        let prompt_text = format!(
            "    {}{}: ",
            field.label,
            default_hint.as_deref().unwrap_or("")
        );

        println!("    {}", field.help.dimmed());

        let input = match &field.field_type {
            ConfigFieldType::String => Terminal::prompt(&prompt_text)?,
            _ => String::new(),
        };

        if input.is_empty() {
            if let Some(default) = field.default {
                return Ok(Some(Value::String(default.to_string())));
            }
            return Ok(None);
        }

        Ok(Some(Value::String(input)))
    }

    pub fn prompt_yes_no(message: &str) -> Result<bool> {
        let response = Terminal::prompt(message)?;
        Ok(matches!(response.to_lowercase().as_str(), "y" | "yes"))
    }
}

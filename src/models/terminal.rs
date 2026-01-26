use std::io::{self, Write};

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
}

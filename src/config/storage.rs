use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Context, Result};

const DEFAULT_DIR: &str = ".config/gitbuddy";
const CONFIG_FILE_NAME: &str = "config.toml";

/// get config dir path
fn get_config_dir() -> Option<PathBuf> {
    match dirs::home_dir() {
        Some(mut home) => {
            home.push(DEFAULT_DIR);
            Some(home)
        }
        None => None,
    }
}

/// save config file to local config dir
pub(crate) fn save_config(content: &str) -> Result<()> {
    let dir = get_config_dir();

    if dir.is_none() {
        return Err(anyhow::anyhow!("get config dir failed"));
    }

    let mut path_buf = dir.unwrap();

    if !path_buf.is_absolute() {
        let current_dir = std::env::current_dir()?;
        path_buf = current_dir.join(path_buf);
    }

    if !path_buf.exists() {
        match std::fs::create_dir_all(&path_buf) {
            Ok(()) => println!("Directory created successfully"),
            Err(e) => println!("Error creating directory: {}", e),
        }
    }

    let config_file_name = path_buf.join(CONFIG_FILE_NAME);
    match std::fs::write(config_file_name, content) {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("Error writing file: {}", e);
            Err(anyhow!("Error writing file: {}", e))
        }
    }
}

/// read config file from local config dir
pub(crate) fn read_config() -> Result<String> {
    let dir = get_config_dir().context("Error getting config dir")?;

    let config_file_name = dir.join(CONFIG_FILE_NAME);

    let content = fs::read_to_string(&config_file_name).context("Error reading config file")?;

    Ok(content)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_config_dir() {
        let dir = get_config_dir();
        assert!(dir.is_some());

        println!("config dir: {:?}", dir)
    }

    #[test]
    fn test_save_config() {
        let content = r#"
[model.DeepSeek]
model = "gpt-3.5-turbo"
api_key = "sk-12345678"
        "#;

        let result = save_config(content);
        assert!(result.is_ok());
    }
}
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

const DEFAULT_DIR: &str = ".config/gitbuddy";
const CONFIG_FILE_NAME: &str = "config.toml";

/// get config dir path
fn get_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|mut home| {
        home.push(DEFAULT_DIR);
        home
    })
}

/// save config file to local config dir
pub(crate) fn save_config(content: &str) -> Result<()> {
    let path_buf = get_config_dir().ok_or_else(|| anyhow!("Failed to get home directory"))?;
    save_config_to(content, &path_buf)
}

/// read config file from local config dir
pub(crate) fn read_config() -> Option<String> {
    let dir = get_config_dir()?;
    read_config_from(&dir)
}

/// Save config content to a custom directory.
pub(crate) fn save_config_to(content: &str, dir: &Path) -> Result<()> {
    let mut path_buf = dir.to_path_buf();

    if !path_buf.is_absolute() {
        let current_dir = std::env::current_dir()?;
        path_buf = current_dir.join(path_buf);
    }

    if !path_buf.exists() {
        std::fs::create_dir_all(&path_buf).map_err(|e| anyhow!("Failed to create config directory: {}", e))?;
    }

    let config_file_name = path_buf.join(CONFIG_FILE_NAME);
    std::fs::write(&config_file_name, content)
        .map_err(|e| anyhow!("Failed to write config file '{}': {}", config_file_name.display(), e))
}

/// Read config content from a custom directory.
pub(crate) fn read_config_from(dir: &Path) -> Option<String> {
    let config_file_name = dir.join(CONFIG_FILE_NAME);
    fs::read_to_string(config_file_name).ok()
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
    fn test_save_and_read_config() {
        let temp_dir = std::env::temp_dir().join("gitbuddy-test-").join(uuid());
        let content = r#"
[model.DeepSeek]
model = "gpt-3.5-turbo"
api_key = "sk-12345678"
        "#;

        // save
        let result = save_config_to(content, &temp_dir);
        assert!(result.is_ok());

        // read back
        let read = read_config_from(&temp_dir);
        assert!(read.is_some());
        assert!(read.unwrap().contains("sk-12345678"));

        // cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    fn uuid() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        format!("{}", dur.as_millis())
    }
}

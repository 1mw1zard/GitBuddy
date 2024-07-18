use std::path::PathBuf;

/// TODO 
const DEFAULT_DIR: &str = ".gitbuddy";

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



// 定义配置文件信息
struct Config {}

#[cfg(test)]
mod test {

}
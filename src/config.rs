use std::env;

const STD_PORT: u16 = 3000;

pub struct Config {
    pub dir: String,
    pub port: u16,
}

impl Config {
    pub fn load() -> Self {
        let dir = env::var("RITZ_DIR").unwrap_or("./".to_string());
        let port = env::var("RITZ_PORT")
            .unwrap_or(STD_PORT.to_string())
            .parse::<u16>()
            .unwrap();
        Config { dir, port }
    }
}

use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub app_host: String,
    pub app_port: u16,
    pub database_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let app_host = env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let app_port = env::var("APP_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .expect("APP_PORT must be a valid u16");

        let database_url = env::var("DATABASE_URL").ok().and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        Self {
            app_host,
            app_port,
            database_url,
        }
    }
}

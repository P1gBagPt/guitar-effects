use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use reqwest::blocking::Client;
use std::time::Duration;

pub struct TabImporter {
    client: Client,
}

impl Default for TabImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TabImporter {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) GuitarWorkstation/1.0")
            .build()
            .unwrap_or_default();

        Self { client }
    }

    pub fn fetch_tab_base64(&self, url: &str) -> Result<(String, String), String> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err("URL must start with http:// or https://".to_string());
        }

        let resp = self
            .client
            .get(url)
            .send()
            .map_err(|e| format!("Failed to request URL: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(format!("HTTP error {}: {}", status.as_u16(), status));
        }

        let filename = url
            .split('/')
            .last()
            .unwrap_or("song.gp")
            .split('?')
            .next()
            .unwrap_or("song.gp")
            .to_string();

        let bytes = resp
            .bytes()
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        let base64_data = BASE64.encode(&bytes);
        Ok((filename, base64_data))
    }
}

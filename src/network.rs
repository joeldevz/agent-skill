use anyhow::{Result, Context, bail};
use reqwest::blocking::Client;
use std::time::Duration;

use crate::security::{validate_url, validate_skill_content};

const USER_AGENT: &str = concat!("skillctl/", env!("CARGO_PKG_VERSION"));
const TIMEOUT_SECS: u64 = 30;
const MAX_REDIRECTS: usize = 5;

pub struct SecureHttpClient {
    client: Client,
}

impl SecureHttpClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Download content from a URL with security validations
    pub fn download(&self, url: &str) -> Result<String> {
        // Validate URL before making request
        let validated_url = validate_url(url)?;

        // Make the request
        let response = self.client
            .get(validated_url.as_str())
            .send()
            .context("Failed to send HTTP request")?;

        // Check status code
        if !response.status().is_success() {
            bail!("HTTP request failed with status: {}", response.status());
        }

        // Check content type (should be text)
        if let Some(content_type) = response.headers().get("content-type") {
            let content_type_str = content_type.to_str().unwrap_or("");
            if !content_type_str.contains("text") && 
               !content_type_str.contains("markdown") &&
               !content_type_str.contains("plain") {
                bail!("Unexpected content type: {}. Expected text/markdown.", content_type_str);
            }
        }

        // Check content length (prevent DoS)
        if let Some(content_length) = response.content_length() {
            if content_length > 1_000_000 {  // 1MB limit
                bail!("Content too large: {} bytes (max 1MB)", content_length);
            }
        }

        // Download content
        let content = response.text()
            .context("Failed to read response body")?;

        // Validate content
        validate_skill_content(&content)?;

        Ok(content)
    }

    /// Try multiple paths to find a skill file
    pub fn find_skill(&self, repo_url: &str, skill_name: &str, custom_path: Option<String>) -> Result<(String, String)> {
        // Transform GitHub URL to raw URL
        let raw_base = repo_url
            .replace("github.com", "raw.githubusercontent.com")
            .trim_end_matches('/')
            .to_string();

        // Determine paths to try (in order of priority)
        let paths_to_try: Vec<String> = if let Some(custom) = custom_path {
            // If custom path provided, only try that
            vec![custom]
        } else {
            // Try common skill locations in order
            vec![
                // Standard structure (vercel-labs/skills)
                format!("skills/{}/SKILL.md", skill_name),
                // Plugin structures (wshobson/agents and similar)
                format!("plugins/javascript-typescript/skills/{}/SKILL.md", skill_name),
                format!("plugins/typescript/skills/{}/SKILL.md", skill_name),
                format!("plugins/javascript/skills/{}/SKILL.md", skill_name),
                // Other common patterns
                format!(".agent/skills/{}/SKILL.md", skill_name),
                format!(".cursor/skills/{}/SKILL.md", skill_name),
                format!(".windsurf/skills/{}/SKILL.md", skill_name),
            ]
        };

        // Try each path until one works
        let mut last_error = String::new();

        for path_in_repo in paths_to_try {
            let target_url = format!("{}/main/{}", raw_base, path_in_repo);
            
            match self.download(&target_url) {
                Ok(content) => {
                    return Ok((content, path_in_repo));
                }
                Err(e) => {
                    last_error = format!("{} ({})", target_url, e);
                }
            }
        }

        bail!("Could not find skill '{}' in repository. Last error: {}", skill_name, last_error)
    }
}

impl Default for SecureHttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SecureHttpClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_invalid_url() {
        let client = SecureHttpClient::new().unwrap();
        let result = client.download("https://192.168.1.1/test");
        assert!(result.is_err());
    }
}

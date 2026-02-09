use anyhow::{Result, bail, Context};
use std::path::{Path, PathBuf};
use url::Url;

/// Validates that a skill name is safe (no path traversal)
pub fn validate_skill_name(name: &str) -> Result<()> {
    // Check for empty or whitespace-only names
    if name.trim().is_empty() {
        bail!("Skill name cannot be empty");
    }

    // Check for path traversal attempts
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        bail!("Skill name contains invalid characters (path traversal attempt detected)");
    }

    // Check for hidden files
    if name.starts_with('.') {
        bail!("Skill name cannot start with a dot");
    }

    // Check for reserved names
    let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", 
                    "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", 
                    "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    if reserved.contains(&name.to_uppercase().as_str()) {
        bail!("Skill name is a reserved system name");
    }

    // Check length (reasonable limit)
    if name.len() > 100 {
        bail!("Skill name is too long (max 100 characters)");
    }

    // Only allow alphanumeric, hyphens, and underscores
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        bail!("Skill name can only contain letters, numbers, hyphens, and underscores");
    }

    Ok(())
}

/// Validates that a URL is safe (prevents SSRF)
pub fn validate_url(url: &str) -> Result<Url> {
    let parsed = Url::parse(url)
        .context("Invalid URL format")?;

    // Only allow HTTPS (or HTTP for localhost in dev)
    match parsed.scheme() {
        "https" => {},
        "http" => {
            // Only allow HTTP for localhost/127.0.0.1 (development)
            if let Some(host) = parsed.host_str() {
                if host != "localhost" && host != "127.0.0.1" {
                    bail!("Only HTTPS URLs are allowed (HTTP only permitted for localhost)");
                }
            }
        },
        _ => bail!("Only HTTP(S) URLs are allowed"),
    }

    // Block private IP ranges (SSRF prevention)
    if let Some(host) = parsed.host_str() {
        // Block localhost variations (except in dev mode with HTTP)
        if parsed.scheme() == "https" {
            if host == "localhost" || host == "127.0.0.1" || host.starts_with("127.") {
                bail!("Localhost URLs are not allowed with HTTPS");
            }
        }

        // Block private IP ranges
        if is_private_ip(host) {
            bail!("Private IP addresses are not allowed (SSRF protection)");
        }

        // Block metadata services
        let blocked_hosts = [
            "169.254.169.254",  // AWS metadata
            "metadata.google.internal",  // GCP metadata
            "169.254.169.253",  // Azure metadata (old)
            "metadata.azure.com",  // Azure metadata (new)
        ];
        if blocked_hosts.contains(&host) {
            bail!("Access to cloud metadata services is blocked");
        }
    }

    // Only allow GitHub and GitLab for now (whitelist approach)
    if let Some(host) = parsed.host_str() {
        let allowed_hosts = [
            "github.com",
            "raw.githubusercontent.com",
            "gitlab.com",
            "localhost",
            "127.0.0.1",
        ];
        
        if !allowed_hosts.iter().any(|&allowed| host == allowed || host.ends_with(&format!(".{}", allowed))) {
            bail!("Only GitHub and GitLab URLs are allowed");
        }
    }

    Ok(parsed)
}

/// Checks if a host string represents a private IP address
fn is_private_ip(host: &str) -> bool {
    // Try to parse as IP address
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                // Check private ranges
                let octets = ipv4.octets();
                matches!(
                    octets,
                    [10, _, _, _] |           // 10.0.0.0/8
                    [172, 16..=31, _, _] |    // 172.16.0.0/12
                    [192, 168, _, _] |        // 192.168.0.0/16
                    [127, _, _, _]            // 127.0.0.0/8 (loopback)
                )
            },
            std::net::IpAddr::V6(ipv6) => {
                // Check for loopback and private ranges
                ipv6.is_loopback() || 
                ipv6.segments()[0] & 0xfe00 == 0xfc00 || // fc00::/7
                ipv6.segments()[0] & 0xffc0 == 0xfe80    // fe80::/10
            }
        }
    } else {
        false
    }
}

/// Validates that a path is within the allowed directory (prevents path traversal)
pub fn validate_path_in_store(base_dir: &Path, target_path: &Path) -> Result<PathBuf> {
    // Canonicalize both paths
    let base = base_dir.canonicalize()
        .context("Failed to resolve base directory")?;
    
    let target = if target_path.is_absolute() {
        target_path.canonicalize()
            .context("Failed to resolve target path")?
    } else {
        base.join(target_path).canonicalize()
            .unwrap_or_else(|_| base.join(target_path))
    };

    // Ensure target is within base
    if !target.starts_with(&base) {
        bail!("Path traversal detected: target path is outside the allowed directory");
    }

    Ok(target)
}

/// Validates SKILL.md content for malicious patterns
pub fn validate_skill_content(content: &str) -> Result<()> {
    // Check for reasonable size (prevent DoS)
    if content.len() > 1_000_000 {  // 1MB limit
        bail!("Skill content is too large (max 1MB)");
    }

    // Check for null bytes (binary content)
    if content.contains('\0') {
        bail!("Skill content contains null bytes (binary content not allowed)");
    }

    // Basic YAML frontmatter validation
    if content.starts_with("---") {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() >= 3 {
            let frontmatter = parts[1];
            
            // Check for suspicious YAML patterns
            let suspicious_patterns = [
                "!!python",  // Python object deserialization
                "!!ruby",    // Ruby object deserialization
                "!!java",    // Java object deserialization
                "!include",  // File inclusion
                "!tag",      // Custom tags
            ];
            
            for pattern in &suspicious_patterns {
                if frontmatter.contains(pattern) {
                    bail!("Skill content contains suspicious YAML pattern: {}", pattern);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_skill_name() {
        // Valid names
        assert!(validate_skill_name("my-skill").is_ok());
        assert!(validate_skill_name("skill_123").is_ok());
        assert!(validate_skill_name("TypeScript-Advanced").is_ok());

        // Invalid names
        assert!(validate_skill_name("../etc/passwd").is_err());
        assert!(validate_skill_name("skill/name").is_err());
        assert!(validate_skill_name("skill\\name").is_err());
        assert!(validate_skill_name(".hidden").is_err());
        assert!(validate_skill_name("").is_err());
        assert!(validate_skill_name("CON").is_err());
    }

    #[test]
    fn test_validate_url() {
        // Valid URLs
        assert!(validate_url("https://github.com/user/repo").is_ok());
        assert!(validate_url("https://raw.githubusercontent.com/user/repo/main/file").is_ok());

        // Invalid URLs
        assert!(validate_url("http://github.com/user/repo").is_err());
        assert!(validate_url("https://169.254.169.254/latest/meta-data/").is_err());
        assert!(validate_url("https://localhost/test").is_err());
        assert!(validate_url("https://192.168.1.1/test").is_err());
        assert!(validate_url("ftp://github.com/user/repo").is_err());
    }

    #[test]
    fn test_is_private_ip() {
        assert!(is_private_ip("10.0.0.1"));
        assert!(is_private_ip("172.16.0.1"));
        assert!(is_private_ip("192.168.1.1"));
        assert!(is_private_ip("127.0.0.1"));
        assert!(!is_private_ip("8.8.8.8"));
        assert!(!is_private_ip("github.com"));
    }
}

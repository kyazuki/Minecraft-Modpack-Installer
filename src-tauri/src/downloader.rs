use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{bail, Context, Result};
use reqwest::blocking::{Client, Response};
use sha1::{Digest, Sha1};

const DOWNLOAD_TIMEOUT_SECS: u64 = 10;

#[derive(Clone)]
pub struct DownloadManager {
    client: Client,
}

#[derive(Debug, Clone, Copy)]
pub struct DownloadProgress {
    pub received_bytes: u64,
    pub total_bytes: Option<u64>,
}

impl DownloadManager {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client })
    }

    pub fn download_to_dir<F>(
        &self,
        url: &str,
        temp_dir: &Path,
        mut progress_callback: Option<F>,
    ) -> Result<DownloadOutcome>
    where
        F: FnMut(DownloadProgress) -> Result<()>,
    {
        let mut response = self
            .client
            .get(url)
            .send()
            .with_context(|| format!("Failed to download from {url}"))?;
        ensure_success(&mut response, url)?;
        let file_name = extract_file_name(&response)?;
        let destination = temp_dir.join(&file_name);
        fs::create_dir_all(temp_dir)
            .with_context(|| format!("Failed to create directory {}", temp_dir.display()))?;
        let mut file = File::create(&destination).with_context(|| {
            format!(
                "Failed to create destination file {}",
                destination.display()
            )
        })?;
        let total_bytes = response.content_length();
        let mut received_bytes = 0u64;
        let mut hasher = Sha1::new();
        let mut buffer = [0u8; 8192];
        loop {
            let read = response.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])?;
            hasher.update(&buffer[..read]);
            received_bytes += read as u64;
            if let Some(callback) = progress_callback.as_mut() {
                callback(DownloadProgress {
                    received_bytes,
                    total_bytes,
                })?;
            }
        }
        file.flush()?;
        let hash_bytes = hasher.finalize();
        let hash = hex::encode(hash_bytes);

        Ok(DownloadOutcome {
            path: destination.to_path_buf(),
            hash,
        })
    }
}

fn extract_file_name(response: &Response) -> Result<String> {
    // Try Content-Disposition header first
    if let Some(content_disposition) = response.headers().get("content-disposition") {
        if let Ok(header_value) = content_disposition.to_str() {
            // Parse: attachment; filename="example.jar"
            for part in header_value.split(';') {
                let part = part.trim();
                if let Some(file_name) = part.strip_prefix("filename=") {
                    let file_name = file_name.trim_matches('"').trim();
                    if !file_name.is_empty() {
                        return Ok(file_name.to_string());
                    }
                }
            }
        }
    }
    let url = response.url();
    // Fallback: extract from final URL path (after redirects)
    if let Some(segments) = url.path_segments() {
        if let Some(last_segment) = segments.last() {
            if !last_segment.is_empty() {
                // Remove query parameters if present
                let file_name = last_segment.split('?').next().unwrap_or(last_segment);
                if !file_name.is_empty() {
                    return Ok(file_name.to_string());
                }
            }
        }
    }
    bail!("Could not determine file name from final URL '{url}' or response headers")
}

fn ensure_success(response: &mut Response, url: &str) -> Result<()> {
    if response.status().is_success() {
        return Ok(());
    }
    let status = response.status();
    let mut body_snippet = String::new();
    if response.read_to_string(&mut body_snippet).is_err() || body_snippet.is_empty() {
        body_snippet = "<failed to read body>".to_string();
    } else if body_snippet.len() > 512 {
        body_snippet.truncate(512);
    }
    bail!("Request to {url} failed with status {status}. Body snippet: {body_snippet}");
}

#[derive(Debug)]
pub struct DownloadOutcome {
    pub path: PathBuf,
    pub hash: String,
}

pub fn move_file(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create destination directory: {}",
                parent.display()
            )
        })?;
    }
    if destination.exists() {
        log::warn!(
            "Destination file ({}) already exists and will be overwritten.",
            destination.display()
        );
    }
    fs::rename(source, destination)?;

    Ok(())
}

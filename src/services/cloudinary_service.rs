use actix_multipart::Multipart;
use actix_web::Error;
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use dotenv::dotenv;
use futures_util::StreamExt;
use hex;
use mime; // ✅ needed for MIME type checks
use reqwest::{
    multipart::{self, Part},
    Client,
};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{collections::HashMap, env, io::Write};
use tempfile::NamedTempFile;
use tokio::io::AsyncReadExt; // ✅ needed for hex encoding

const MAX_SIZE: usize = 5 * 1024 * 1024; // 5MB for images

#[derive(Serialize, Deserialize, Debug)]
pub struct CloudinaryResponse {
    pub public_id: String,
    pub secure_url: String,
}

enum ParamValue {
    Str(String),
    Int(i64),
}

pub struct CloudinaryService;

impl CloudinaryService {
    fn env_loader(key: &str) -> String {
        dotenv().ok();
        env::var(key).unwrap_or_else(|_| panic!("Missing env key {}", key))
    }

    fn generate_signature(params: HashMap<&str, ParamValue>, api_secret: &str) -> String {
        let mut sorted_keys: Vec<&&str> = params.keys().collect();
        sorted_keys.sort();

        let mut sorted_params = String::new();
        for key in sorted_keys {
            if !sorted_params.is_empty() {
                sorted_params.push('&');
            }
            let value = match &params[key] {
                ParamValue::Str(s) => s.clone(),
                ParamValue::Int(i) => i.to_string(),
            };
            sorted_params.push_str(&format!("{}={}", key, value));
        }

        let string_to_sign = format!("{}{}", sorted_params, api_secret);

        let mut hasher = Sha1::new();
        hasher.update(string_to_sign.as_bytes());
        hex::encode(hasher.finalize()) // ✅ now works
    }

    /// Save file temporarily (for multipart uploads)
    pub async fn save_file(mut payload: Multipart) -> Result<NamedTempFile, Error> {
        let mut total_size = 0;
        let mut temp_file = NamedTempFile::new()?;

        while let Some(field) = payload.next().await {
            let mut field = field?;

            let content_type = field.content_type();

            // Ensure it's an image
            if let Some(content_type) = content_type {
                if content_type.type_() != mime::IMAGE {
                    return Err(actix_web::error::ErrorBadRequest(
                        "Only image files allowed",
                    ));
                }
            } else {
                return Err(actix_web::error::ErrorBadRequest("Missing content type"));
            }

            while let Some(chunk) = field.next().await {
                let data = chunk?;
                total_size += data.len();
                if total_size > MAX_SIZE {
                    return Err(actix_web::error::ErrorBadRequest("File size exceeded"));
                }
                temp_file.write_all(&data)?;
            }
        }
        Ok(temp_file)
    }

    /// Upload image to Cloudinary (accepts file path as `&str`)
    /// Upload image to Cloudinary (accepts file path, base64, or URL)
    pub async fn upload_to_cloudinary(
        file_path_or_base64_or_url: &str,
    ) -> Result<CloudinaryResponse, String> {
        let client = Client::new();
        let cloud_name = CloudinaryService::env_loader("CLOUDINARY_CLOUD_NAME");
        let api_secret = CloudinaryService::env_loader("CLOUDINARY_API_SECRET");
        let api_key = CloudinaryService::env_loader("CLOUDINARY_API_KEY");
        let timestamp = chrono::Utc::now().timestamp();

        let (public_id, buffer): (String, Vec<u8>) =
            if file_path_or_base64_or_url.starts_with("data:") {
                // ---------- Base64 ----------
                let parts: Vec<&str> = file_path_or_base64_or_url.split(',').collect();
                if parts.len() != 2 {
                    return Err("Invalid base64 data".to_string());
                }

                let mime_type_part = parts[0];
                let ext = if mime_type_part.contains("jpeg") {
                    "jpg"
                } else if mime_type_part.contains("png") {
                    "png"
                } else if mime_type_part.contains("gif") {
                    "gif"
                } else {
                    "bin"
                };

                let public_id = format!("upload_{}", timestamp);
                let decoded = STANDARD
                    .decode(parts[1])
                    .map_err(|_| "Failed to decode base64 image".to_string())?;

                (format!("{}.{}", public_id, ext), decoded)
            } else if file_path_or_base64_or_url.starts_with("http://")
                || file_path_or_base64_or_url.starts_with("https://")
            {
                // ---------- URL ----------
                let res = client
                    .get(file_path_or_base64_or_url)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to fetch image from URL: {}", e))?;

                if !res.status().is_success() {
                    return Err(format!(
                        "Failed to download image (status {}): {}",
                        res.status(),
                        res.text().await.unwrap_or_default()
                    ));
                }

                let bytes = res
                    .bytes()
                    .await
                    .map_err(|e| format!("Failed to read image bytes: {}", e))?;

                let public_id = format!("url_upload_{}", timestamp);
                (public_id, bytes.to_vec())
            } else {
                // ---------- Local file ----------
                let path = std::path::Path::new(file_path_or_base64_or_url);
                let public_id = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("file")
                    .to_string();

                let mut file = tokio::fs::File::open(path)
                    .await
                    .map_err(|e| format!("Failed to open file ({}): {}", public_id, e))?;

                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .await
                    .map_err(|e| format!("Failed to read file ({}): {}", public_id, e))?;

                (public_id, buffer)
            };

        // ✅ Validate file size
        if buffer.len() > MAX_SIZE {
            return Err("File size exceeded 5MB".to_string());
        }

        // ---------- Cloudinary Upload ----------
        let mut params = HashMap::new();
        params.insert("public_id", ParamValue::Str(public_id.clone()));
        params.insert("timestamp", ParamValue::Int(timestamp));

        let signature = CloudinaryService::generate_signature(params, &api_secret);

        let part = Part::bytes(buffer).file_name(public_id.clone());

        let form = multipart::Form::new()
            .text("public_id", public_id.clone())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature)
            .text("api_key", api_key)
            .part("file", part);

        let res = client
            .post(format!(
                "https://api.cloudinary.com/v1_1/{}/image/upload",
                cloud_name
            ))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Cloudinary: {}", e))?;

        let status = res.status();
        let result = res
            .text()
            .await
            .map_err(|e| format!("Failed to read Cloudinary response: {}", e))?;

        if !status.is_success() {
            return Err(format!(
                "Cloudinary upload failed (status {}): {}",
                status, result
            ));
        }

        let cloudinary_response: CloudinaryResponse = serde_json::from_str(&result)
            .map_err(|e| format!("Failed to parse Cloudinary response: {}", e))?;

        Ok(cloudinary_response)
    }

    /// Delete image from Cloudinary by public_id
    pub async fn delete_from_cloudinary(public_id: &str) -> Result<(), String> {
        let client = Client::new();
        let cloud_name = CloudinaryService::env_loader("CLOUDINARY_CLOUD_NAME");
        let api_secret = CloudinaryService::env_loader("CLOUDINARY_API_SECRET");
        let api_key = CloudinaryService::env_loader("CLOUDINARY_API_KEY");
        let timestamp = chrono::Utc::now().timestamp();

        let mut params = HashMap::new();
        params.insert("public_id", ParamValue::Str(public_id.to_string()));
        params.insert("timestamp", ParamValue::Int(timestamp));

        let signature = CloudinaryService::generate_signature(params, &api_secret);

        let form = multipart::Form::new()
            .text("public_id", public_id.to_string())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature)
            .text("api_key", api_key);

        let res = client
            .post(format!(
                "https://api.cloudinary.com/v1_1/{}/image/destroy",
                cloud_name
            ))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to send delete request to Cloudinary: {}", e))?;

        let status = res.status();
        let result = res
            .text()
            .await
            .map_err(|e| format!("Failed to read delete response from Cloudinary: {}", e))?;

        if !status.is_success() {
            return Err(format!(
                "Cloudinary delete failed (status {}): {}",
                status, result
            ));
        }

        Ok(())
    }
}

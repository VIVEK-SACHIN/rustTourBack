//! User profile photo upload — Natours `uploadUserPhoto` + `resizeUserPhoto` (multer + sharp).

use std::path::Path;

use chrono::Utc;
use image::imageops::FilterType;
use image::ImageFormat;
use mongodb::bson::oid::ObjectId;

use crate::utils::error::AppError;

const MAX_BYTES: usize = 5 * 1024 * 1024;

/// Resize to 500×500 JPEG and write under `upload_dir` (Natours `public/img/users` on the API server).
pub fn save_user_photo(
    upload_dir: &Path,
    user_id: &ObjectId,
    bytes: &[u8],
    content_type: Option<&str>,
) -> Result<String, AppError> {
    if bytes.is_empty() {
        return Err(AppError::bad_request("Please upload an image file."));
    }
    if bytes.len() > MAX_BYTES {
        return Err(AppError::bad_request(
            "Image is too large. Maximum size is 5 MB.",
        ));
    }

    let is_image = content_type
        .map(|ct| ct.starts_with("image/"))
        .unwrap_or(true);
    if !is_image {
        return Err(AppError::bad_request(
            "Not an image! Please upload only images.",
        ));
    }

    std::fs::create_dir_all(upload_dir).map_err(|e| {
        AppError::internal(format!("Could not create upload directory: {e}"))
    })?;

    let img = image::load_from_memory(bytes)
        .map_err(|_| AppError::bad_request("Not an image! Please upload only images."))?;

    let resized = img.resize_to_fill(500, 500, FilterType::Lanczos3);
    let filename = format!("user-{}-{}.jpeg", user_id.to_hex(), Utc::now().timestamp_millis());
    let path = upload_dir.join(&filename);

    resized
        .save_with_format(&path, ImageFormat::Jpeg)
        .map_err(|e| AppError::internal(format!("Could not save profile photo: {e}")))?;

    Ok(filename)
}

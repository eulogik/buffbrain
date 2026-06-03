use crate::db::detect_type;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageBuffer, Rgba};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;

const POLL_INTERVAL: Duration = Duration::from_millis(400);
const THUMBNAIL_SIZE: u32 = 64;

pub struct ClipboardWatcher<R: Runtime> {
    app: AppHandle<R>,
    last_text: String,
    last_image_hash: u64,
    last_change: Instant,
}

impl<R: Runtime> ClipboardWatcher<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        let last_text = app.clipboard().read_text().unwrap_or_default();
        Self {
            app,
            last_text,
            last_image_hash: 0,
            last_change: Instant::now(),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::time::sleep(POLL_INTERVAL).await;
            if self.last_change.elapsed() < Duration::from_millis(200) {
                continue;
            }
            if let Err(e) = self.poll_once().await {
                eprintln!("[clipboard] poll error: {e}");
            }
        }
    }

    async fn poll_once(&mut self) -> Result<()> {
        let clipboard = self.app.clipboard();
        if let Ok(image) = clipboard.read_image() {
            let rgba = image.rgba();
            let hash = fnv1a_hash(rgba);
            if hash != self.last_image_hash {
                self.last_image_hash = hash;
                self.last_text.clear();
                let width = image.width();
                let height = image.height();
                let thumbnail = make_thumbnail(rgba, width, height);
                let _ = self.app.emit("clipboard-image", &thumbnail);
            }
            return Ok(());
        }
        if let Ok(text) = clipboard.read_text() {
            if text.is_empty() {
                return Ok(());
            }
            if text == self.last_text {
                return Ok(());
            }
            self.last_text = text.clone();
            let kind = detect_type(&text);
            let _ = self.app.emit("clipboard-text", &serde_json::json!({
                "content": text,
                "type": kind,
            }));
        }
        Ok(())
    }
}

fn make_thumbnail(rgba: &[u8], width: u32, height: u32) -> String {
    if width == 0 || height == 0 || rgba.len() < (width * height * 4) as usize {
        return String::new();
    }
    let img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width, height, rgba.to_vec()).unwrap();
    let dyn_img = DynamicImage::ImageRgba8(img);
    let (w, h) = dyn_img.dimensions();
    let _ = (w, h);
    let thumb = dyn_img.resize_exact(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Triangle);
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    if thumb.write_to(&mut cursor, image::ImageFormat::Png).is_err() {
        return String::new();
    }
    format!("data:image/png;base64,{}", general_purpose::STANDARD.encode(&buf))
}

fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}


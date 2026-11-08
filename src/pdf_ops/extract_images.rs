use lopdf::{Document, Object};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn extract_images_from_pdf(input_path: &Path, output_dir: &Path) -> Result<usize, String> {
    let doc = Document::load(input_path)
        .map_err(|e| format!("Fehler beim Laden von '{}': {e}", input_path.display()))?;

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("bild");

    let mut saved_count = 0;
    let mut image_index = 1;

    for (id, obj) in &doc.objects {
        if let Ok(stream) = obj.as_stream() {
            let is_image = stream
                .dict
                .get(b"Subtype")
                .and_then(|s| s.as_name())
                .map(|name| name == b"Image")
                .unwrap_or(false);

            if !is_image {
                continue;
            }

            let filter = stream.dict.get(b"Filter");
            let is_jpeg = match filter {
                Ok(Object::Name(name)) => name == b"DCTDecode",
                Ok(Object::Array(arr)) => arr.iter().any(|item| item.as_name().unwrap_or(b"") == b"DCTDecode"),
                _ => false,
            };

            if is_jpeg {
                let out_file = output_dir.join(format!("{stem}_img_{image_index:03}_obj{}_{}.jpg", id.0, id.1));
                if let Ok(mut f) = File::create(&out_file) {
                    if f.write_all(&stream.content).is_ok() {
                        saved_count += 1;
                        image_index += 1;
                        continue;
                    }
                }
            }

            // Flate or uncompressed image
            let width = stream.dict.get(b"Width").and_then(|w| w.as_i64()).unwrap_or(0) as u32;
            let height = stream.dict.get(b"Height").and_then(|h| h.as_i64()).unwrap_or(0) as u32;
            let bpc = stream.dict.get(b"BitsPerComponent").and_then(|b| b.as_i64()).unwrap_or(8) as u32;

            if width == 0 || height == 0 {
                continue;
            }

            let color_space: &[u8] = match stream.dict.get(b"ColorSpace") {
                Ok(Object::Name(n)) => n.as_slice(),
                Ok(Object::Array(a)) => {
                    a.first().and_then(|item| item.as_name().ok()).unwrap_or(b"DeviceRGB")
                }
                _ => b"DeviceRGB",
            };

            // Decompress stream content if needed
            let decompressed = match stream.decompressed_content() {
                Ok(data) => data,
                Err(_) => stream.content.clone(),
            };

            let out_file = output_dir.join(format!("{stem}_img_{image_index:03}_obj{}_{}.png", id.0, id.1));

            if color_space == b"DeviceRGB" && bpc == 8 && decompressed.len() >= (width * height * 3) as usize {
                if image::save_buffer(
                    &out_file,
                    &decompressed[..(width * height * 3) as usize],
                    width,
                    height,
                    image::ExtendedColorType::Rgb8,
                ).is_ok() {
                    saved_count += 1;
                    image_index += 1;
                }
            } else if (color_space == b"DeviceGray" || color_space == b"CalGray") && bpc == 8 && decompressed.len() >= (width * height) as usize {
                if image::save_buffer(
                    &out_file,
                    &decompressed[..(width * height) as usize],
                    width,
                    height,
                    image::ExtendedColorType::L8,
                ).is_ok() {
                    saved_count += 1;
                    image_index += 1;
                }
            }
        }
    }

    Ok(saved_count)
}

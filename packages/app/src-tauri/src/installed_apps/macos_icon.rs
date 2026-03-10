//! macOS icon extraction using NSWorkspace API
//! This module provides functionality to extract system-associated icons for any file type.

use base64::{engine::general_purpose, Engine as _};
use objc2::msg_send;
use objc2::AllocAnyThread;
use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSGraphicsContext, NSWorkspace};
use objc2_foundation::{NSDictionary, NSString};
use tracing::{error, info, warn};

/// Extract the system-associated icon for any file using NSWorkspace.iconForFile
pub fn extract_icon_from_file(file_path: &str) -> Option<String> {
    info!("[ICON] Extracting icon for file: {}", file_path);

    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let path_string = NSString::from_str(file_path);
        let icon = workspace.iconForFile(&path_string);

        // Set a reasonable size for the icon (64x64 is good for UI usage)
        let size = objc2_foundation::NSSize::new(64.0, 64.0);
        icon.setSize(size);

        // Get the best representation from the icon
        let representations = icon.representations();
        if representations.len() == 0 {
            warn!("[ICON] No representations found for: {}", file_path);
            return None;
        }

        // Create a new bitmap context to render the icon
        let bitmap_rep = NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut(),
            64,
            64,
            8,
            4,
            true,
            false,
            &objc2_app_kit::NSDeviceRGBColorSpace,
            0,
            0,
        );

        let bitmap_rep = match bitmap_rep {
            Some(rep) => rep,
            None => {
                error!(
                    "[ICON] Failed to create bitmap representation for: {}",
                    file_path
                );
                return None;
            }
        };

        // Create a graphics context from the bitmap
        let context = NSGraphicsContext::graphicsContextWithBitmapImageRep(&bitmap_rep);
        let context = match context {
            Some(ctx) => ctx,
            None => {
                error!(
                    "[ICON] Failed to create graphics context for: {}",
                    file_path
                );
                return None;
            }
        };

        // Save current context and set ours
        let previous_context = NSGraphicsContext::currentContext();
        NSGraphicsContext::setCurrentContext(Some(&context));

        // Draw the icon into our bitmap
        let rect = objc2_foundation::NSRect::new(objc2_foundation::NSPoint::new(0.0, 0.0), size);
        let from_rect =
            objc2_foundation::NSRect::new(objc2_foundation::NSPoint::new(0.0, 0.0), icon.size());
        icon.drawInRect_fromRect_operation_fraction(
            rect,
            from_rect,
            objc2_app_kit::NSCompositingOperation::SourceOver,
            1.0,
        );

        // Restore previous context
        if let Some(prev) = previous_context {
            NSGraphicsContext::setCurrentContext(Some(&prev));
        } else {
            NSGraphicsContext::setCurrentContext(None);
        }

        // Convert to PNG data
        let properties: objc2::rc::Retained<
            NSDictionary<objc2_foundation::NSString, objc2::runtime::AnyObject>,
        > = NSDictionary::new();
        let png_data =
            bitmap_rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &properties);

        let png_data = match png_data {
            Some(data) => data,
            None => {
                error!("[ICON] Failed to create PNG data for: {}", file_path);
                return None;
            }
        };

        // Get bytes from NSData using msg_send!
        let length: usize = msg_send![&*png_data, length];
        if length == 0 {
            error!("[ICON] PNG data has zero length for: {}", file_path);
            return None;
        }

        let bytes_ptr: *const u8 = msg_send![&*png_data, bytes];
        let bytes = std::slice::from_raw_parts(bytes_ptr, length);
        let base64_icon = general_purpose::STANDARD.encode(bytes);

        info!("[ICON] Successfully extracted icon for: {}", file_path);
        Some(base64_icon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_icon_from_existing_file() {
        // Test with a file that should always exist on macOS
        let result = extract_icon_from_file("/Applications/Safari.app");
        assert!(result.is_some(), "Should extract icon from Safari.app");

        // Verify it's valid base64
        let base64_str = result.unwrap();
        let decoded = general_purpose::STANDARD.decode(&base64_str);
        assert!(decoded.is_ok(), "Icon should be valid base64");

        // Verify it starts with PNG magic bytes
        let bytes = decoded.unwrap();
        assert!(bytes.len() > 8, "PNG should have header");
        assert_eq!(
            &bytes[0..4],
            &[0x89, 0x50, 0x4E, 0x47],
            "Should be PNG format"
        );
    }

    #[test]
    fn test_extract_icon_from_non_existing_file() {
        // Even non-existing files should return a generic icon on macOS
        let result = extract_icon_from_file("/nonexistent/path/file.txt");
        // NSWorkspace.iconForFile returns a generic icon for non-existing files
        assert!(
            result.is_some(),
            "Should return generic icon for non-existing file"
        );
    }
}

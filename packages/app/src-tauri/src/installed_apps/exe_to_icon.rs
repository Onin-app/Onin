use base64::Engine;
use image::{DynamicImage, ImageBuffer, RgbaImage};
use std::mem;
use windows::{
    core::{HSTRING, PCWSTR},
    Win32::{
        Foundation::HWND,
        Graphics::Gdi::{
            CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject,
            BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP,
        },
        UI::{
            Shell::{ExtractIconW, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON},
            WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO},
        },
    },
};

// #[tracing::instrument]
pub fn extract_icon_from_exe(exe_path: &str) -> Option<String> {
    unsafe {
        // 将 Rust &str (UTF-8) 转换为 Windows API 的 UTF-16 字符串 (PCWSTR)
        // 使用 HSTRING 更安全和方便
        let h_path = HSTRING::from(exe_path); // <--- 新增
        let c_path = PCWSTR(h_path.as_ptr()); // <--- 新增

        // 从 exe 文件中提取图标 (索引 0 表示第一个图标)
        let hicon = ExtractIconW(None, c_path, 0); // <--- ExtractIconA 改为 ExtractIconW
        if hicon.is_invalid() {
            let err = windows::core::Error::from_win32(); // 获取详细的错误信息
            tracing::debug!(
                "ExtractIconW failed for {}, falling back to file icon: {:?}",
                exe_path,
                err
            );
            return extract_icon_from_file(exe_path);
        }

        // 获取图标信息
        let mut icon_info = ICONINFO::default();
        if GetIconInfo(hicon, &mut icon_info).is_err() {
            let err = windows::core::Error::from_win32();
            tracing::warn!("GetIconInfo failed for {}: {:?}", exe_path, err);
            let _ = DestroyIcon(hicon);
            return extract_icon_from_file(exe_path);
        }

        // 提取位图数据
        // 这个函数体不需要修改，因为它接收的是 HBITMAP，不涉及字符串路径
        let bitmap_data = extract_bitmap_data(icon_info.hbmColor)?;

        // 清理资源
        if !icon_info.hbmMask.is_invalid() {
            let _ = DeleteObject(icon_info.hbmMask);
        }
        if !icon_info.hbmColor.is_invalid() {
            let _ = DeleteObject(icon_info.hbmColor);
        }
        let _ = DestroyIcon(hicon);

        // 将位图数据转换为 PNG 格式的 base64
        bitmap_to_base64(bitmap_data)
    }
}

/// Extract icon for any file type using SHGetFileInfo API.
/// This retrieves the system-associated icon based on file extension.
pub fn extract_icon_from_file(file_path: &str) -> Option<String> {
    unsafe {
        let h_path = HSTRING::from(file_path);
        let c_path = PCWSTR(h_path.as_ptr());

        let mut shfi: SHFILEINFOW = std::mem::zeroed();
        let shfi_size = std::mem::size_of::<SHFILEINFOW>() as u32;

        // Use SHGetFileInfoW to get the icon associated with the file type
        let result = SHGetFileInfoW(
            c_path,
            windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
            Some(&mut shfi),
            shfi_size,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        if result == 0 {
            tracing::debug!("SHGetFileInfoW failed for {}", file_path);
            return None;
        }

        let hicon = shfi.hIcon;
        if hicon.is_invalid() {
            tracing::debug!("SHGetFileInfoW returned invalid icon for {}", file_path);
            return None;
        }

        // Get icon info
        let mut icon_info = ICONINFO::default();
        if GetIconInfo(hicon, &mut icon_info).is_err() {
            let err = windows::core::Error::from_win32();
            tracing::warn!("GetIconInfo failed for {}: {:?}", file_path, err);
            let _ = DestroyIcon(hicon);
            return None;
        }

        // Extract bitmap data
        let bitmap_data = extract_bitmap_data(icon_info.hbmColor);

        // Cleanup resources
        if !icon_info.hbmMask.is_invalid() {
            let _ = DeleteObject(icon_info.hbmMask);
        }
        if !icon_info.hbmColor.is_invalid() {
            let _ = DeleteObject(icon_info.hbmColor);
        }
        let _ = DestroyIcon(hicon);

        bitmap_data.and_then(bitmap_to_base64)
    }
}

unsafe fn extract_bitmap_data(hbitmap: HBITMAP) -> Option<BitmapData> {
    if hbitmap.is_invalid() {
        tracing::warn!("extract_bitmap_data received invalid HBITMAP");
        return None;
    }

    let hdc = GetDC(HWND::default());
    if hdc.is_invalid() {
        let err = windows::core::Error::from_win32();
        tracing::warn!("GetDC failed: {:?}", err);
        return None;
    }

    let hdc_mem = CreateCompatibleDC(hdc);
    if hdc_mem.is_invalid() {
        let err = windows::core::Error::from_win32();
        tracing::warn!("CreateCompatibleDC failed: {:?}", err);
        let _ = ReleaseDC(HWND::default(), hdc);
        return None;
    }

    let old_bitmap = SelectObject(hdc_mem, hbitmap);
    if old_bitmap.is_invalid() {
        let err = windows::core::Error::from_win32();
        tracing::warn!("SelectObject failed: {:?}", err);
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(HWND::default(), hdc);
        return None;
    }

    // 创建位图信息结构 - 第一次调用 GetDIBits，biBitCount 应该为 0
    let mut bmp_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: 0,
            biHeight: 0,
            biPlanes: 1,
            biBitCount: 0, // <--- 🚨 关键修复：这里改为 0，让系统填充实际的位深信息
            biCompression: BI_RGB.0, // BI_RGB.0 或者 0 都可以，因为系统会填充实际的
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [Default::default(); 1],
    };

    // 获取位图尺寸信息 (第一次 GetDIBits 调用，获取头部信息)
    if GetDIBits(
        hdc_mem,
        hbitmap,
        0,
        0,    // num_scan_lines 为 0，表示只获取信息
        None, // lpBits 为 None，表示只获取信息
        &mut bmp_info as *mut _,
        DIB_RGB_COLORS,
    ) == 0
    {
        let err = windows::core::Error::from_win32();
        tracing::warn!("GetDIBits (info) failed: {:?}", err);
        let _ = SelectObject(hdc_mem, old_bitmap);
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(HWND::default(), hdc);
        return None;
    }

    // 第一次调用成功后，bmp_info.bmiHeader 现在包含了 HBITMAP 的实际信息。
    let width = bmp_info.bmiHeader.biWidth as u32;
    let height = bmp_info.bmiHeader.biHeight.abs() as u32;

    // IMPORTANT: 在第二次调用 GetDIBits 之前，我们可以将 bmiHeader.biBitCount 设置为 32
    // 强制它返回 32-bit BGRA 数据，这样与 image crate 的 RGBA 格式转换更加稳定。
    // 同时确保 biHeight 是负值，以获取 Top-Down DIB (对于 image crate 友好)
    bmp_info.bmiHeader.biHeight = -(height as i32);
    bmp_info.bmiHeader.biBitCount = 32; // <--- 🚨 第二次设置 biBitCount 为 32，请求 32位数据
    bmp_info.bmiHeader.biCompression = BI_RGB.0; // 确保是无压缩 RGB 格式
                                                 // biSizeImage 可以在这里重新计算，或者通常设为 0 让系统计算
                                                 // 因为我们强制为 32 位，所以 size = width * height * 4
    let size = (width * height * 4) as usize;
    bmp_info.bmiHeader.biSizeImage = size as u32;

    let mut buffer = vec![0u8; size];

    // 获取实际的位图数据 (第二次 GetDIBits 调用，获取像素数据)
    if GetDIBits(
        hdc_mem,
        hbitmap,
        0,      // 从第0行开始
        height, // 读取 height 行
        Some(buffer.as_mut_ptr() as *mut _),
        &mut bmp_info as *mut _,
        DIB_RGB_COLORS,
    ) == 0
    {
        let err = windows::core::Error::from_win32();
        tracing::warn!("GetDIBits (data) failed: {:?}", err);
        let _ = SelectObject(hdc_mem, old_bitmap);
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(HWND::default(), hdc);
        return None;
    }

    let _ = SelectObject(hdc_mem, old_bitmap);
    let _ = DeleteDC(hdc_mem);
    let _ = ReleaseDC(HWND::default(), hdc);

    Some(BitmapData {
        width,
        height,
        data: buffer,
    })
}

struct BitmapData {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

fn bitmap_to_base64(bitmap_data: BitmapData) -> Option<String> {
    // Windows 位图数据是 BGRA 格式，需要转换为 RGBA
    let mut rgba_data = Vec::with_capacity(bitmap_data.data.len());
    let mut has_visible_pixels = false;

    // BGRA -> RGBA，同时检测是否有可见像素
    for chunk in bitmap_data.data.chunks_exact(4) {
        let alpha = chunk[3];
        rgba_data.push(chunk[2]); // R (from B position)
        rgba_data.push(chunk[1]); // G
        rgba_data.push(chunk[0]); // B (from R position)
        rgba_data.push(alpha); // A

        // 如果有任何不完全透明的像素，标记为有可见内容
        if alpha > 0 {
            has_visible_pixels = true;
        }
    }

    // 如果图标完全透明，返回 None
    if !has_visible_pixels {
        tracing::warn!("Icon is fully transparent, skipping");
        return None;
    }

    // 创建 RGBA 图像
    let img: RgbaImage = ImageBuffer::from_raw(bitmap_data.width, bitmap_data.height, rgba_data)?;

    let dynamic_img = DynamicImage::ImageRgba8(img);

    // 将图像编码为 PNG 格式
    let mut png_data = Vec::new();
    {
        use std::io::Cursor;
        let mut cursor = Cursor::new(&mut png_data);
        dynamic_img
            .write_to(&mut cursor, image::ImageFormat::Png)
            .ok()?;
    }

    // 转换为 base64 字符串
    let base64_string = base64::engine::general_purpose::STANDARD.encode(&png_data);
    Some(base64_string)
}

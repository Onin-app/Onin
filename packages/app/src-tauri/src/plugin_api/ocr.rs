use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OcrWord {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OcrLine {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub words: Vec<OcrWord>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OcrResult {
    pub text: String,
    pub lines: Vec<OcrLine>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OcrOptions {
    pub language: Option<String>,
}

fn get_image_bytes(image_str: &str) -> Result<Vec<u8>, String> {
    if image_str.starts_with("data:") || image_str.contains(";base64,") {
        let parts: Vec<&str> = image_str.splitn(2, ";base64,").collect();
        let base64_data = if parts.len() == 2 { parts[1] } else { parts[0] };
        let cleaned = base64_data.trim();

        use base64::Engine;
        base64::prelude::BASE64_STANDARD
            .decode(cleaned)
            .map_err(|e| format!("Failed to decode base64: {}", e))
    } else {
        std::fs::read(image_str)
            .map_err(|e| format!("Failed to read image file '{}': {}", image_str, e))
    }
}

#[tauri::command]
pub async fn plugin_ocr_recognize(
    image: String,
    options: Option<OcrOptions>,
) -> Result<OcrResult, String> {
    #[cfg(target_os = "windows")]
    {
        run_windows_ocr(image, options).await
    }

    #[cfg(target_os = "macos")]
    {
        run_macos_ocr(image, options).await
    }

    #[cfg(target_os = "linux")]
    {
        run_linux_ocr(image, options).await
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = (image, options);
        Err("OCR is currently not supported on this platform.".to_string())
    }
}

// ==========================================
// Windows Platform Implementation
// ==========================================
#[cfg(target_os = "windows")]
async fn run_windows_ocr(image: String, options: Option<OcrOptions>) -> Result<OcrResult, String> {
    let bytes = get_image_bytes(&image)?;

    tokio::task::spawn_blocking(move || {
        let _com_guard = ComGuard::new();

        let result = (|| -> windows::core::Result<OcrResult> {
            use windows::core::HSTRING;
            use windows::Graphics::Imaging::BitmapDecoder;
            use windows::Media::Ocr::OcrEngine;
            use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};

            // 1. 写入内存数据流
            let stream = InMemoryRandomAccessStream::new()?;
            let writer = DataWriter::CreateDataWriter(&stream)?;
            writer.WriteBytes(&bytes)?;
            writer.StoreAsync()?.get()?;
            writer.FlushAsync()?.get()?;
            stream.Seek(0)?;

            // 2. 解码 SoftwareBitmap
            let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
            let software_bitmap = decoder.GetSoftwareBitmapAsync()?.get()?;

            // 3. 构建 OcrEngine
            let engine = if let Some(opts) = &options {
                if let Some(ref lang_code) = opts.language {
                    let lang = windows::Globalization::Language::CreateLanguage(&HSTRING::from(
                        lang_code,
                    ))?;
                    if OcrEngine::IsLanguageSupported(&lang)? {
                        OcrEngine::TryCreateFromLanguage(&lang)?
                    } else {
                        OcrEngine::TryCreateFromUserProfileLanguages()?
                    }
                } else {
                    OcrEngine::TryCreateFromUserProfileLanguages()?
                }
            } else {
                OcrEngine::TryCreateFromUserProfileLanguages()?
            };

            // 4. 执行识别
            let ocr_result = engine.RecognizeAsync(&software_bitmap)?.get()?;

            // 5. 拼装结果
            let text = ocr_result.Text()?.to_string();
            let mut lines = Vec::new();

            for line in ocr_result.Lines()? {
                let line_text = line.Text()?.to_string();
                let mut words = Vec::new();

                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;

                for word in line.Words()? {
                    let word_text = word.Text()?.to_string();
                    let rect = word.BoundingRect()?;

                    words.push(OcrWord {
                        text: word_text,
                        x: rect.X,
                        y: rect.Y,
                        width: rect.Width,
                        height: rect.Height,
                    });

                    if rect.X < min_x {
                        min_x = rect.X;
                    }
                    if rect.Y < min_y {
                        min_y = rect.Y;
                    }
                    let word_max_x = rect.X + rect.Width;
                    let word_max_y = rect.Y + rect.Height;
                    if word_max_x > max_x {
                        max_x = word_max_x;
                    }
                    if word_max_y > max_y {
                        max_y = word_max_y;
                    }
                }

                let (line_x, line_y, line_width, line_height) = if words.is_empty() {
                    (0.0, 0.0, 0.0, 0.0)
                } else {
                    (min_x, min_y, max_x - min_x, max_y - min_y)
                };

                lines.push(OcrLine {
                    text: line_text,
                    x: line_x,
                    y: line_y,
                    width: line_width,
                    height: line_height,
                    words,
                });
            }

            Ok(OcrResult { text, lines })
        })();

        result.map_err(|e| format!("Windows OCR Error: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// ==========================================
// macOS Platform Implementation
// ==========================================
#[cfg(target_os = "macos")]
extern "C" {
    fn dlopen(filename: *const std::ffi::c_char, flag: std::ffi::c_int) -> *mut std::ffi::c_void;
}

#[cfg(target_os = "macos")]
fn load_vision_framework() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        unsafe {
            dlopen(
                "/System/Library/Frameworks/Vision.framework/Vision\0".as_ptr()
                    as *const std::ffi::c_char,
                1, // RTLD_LAZY
            );
        }
    });
}

#[cfg(target_os = "macos")]
async fn run_macos_ocr(image: String, options: Option<OcrOptions>) -> Result<OcrResult, String> {
    let bytes = get_image_bytes(&image)?;

    // 快速读取图片物理宽度与高度
    let (width, height) = {
        let cursor = std::io::Cursor::new(&bytes);
        if let Ok(reader) = image::ImageReader::new(cursor).with_guessed_format() {
            if let Ok(dims) = reader.into_dimensions() {
                (dims.0 as f64, dims.1 as f64)
            } else {
                (1.0, 1.0)
            }
        } else {
            (1.0, 1.0)
        }
    };

    tokio::task::spawn_blocking(move || {
        load_vision_framework();

        use objc2::{msg_send, ClassType};
        use objc2::rc::Retained;
        use objc2_foundation::{NSData, NSArray, NSDictionary, NSString, NSRange};
        use core_graphics::geometry::CGRect;

        let result = (|| -> Result<OcrResult, String> {
            let cls_request = objc2::runtime::AnyClass::get("VNRecognizeTextRequest")
                .ok_or_else(|| "Failed to find class VNRecognizeTextRequest. Make sure Vision.framework is loaded.".to_string())?;
            let cls_handler = objc2::runtime::AnyClass::get("VNImageRequestHandler")
                .ok_or_else(|| "Failed to find class VNImageRequestHandler. Make sure Vision.framework is loaded.".to_string())?;

            // 1. 创建 VNRecognizeTextRequest
            let request: Retained<objc2::runtime::AnyObject> = unsafe {
                let obj: *mut objc2::runtime::AnyObject = msg_send![cls_request, alloc];
                let obj: *mut objc2::runtime::AnyObject = msg_send![obj, init];
                Retained::from_raw(obj).ok_or("Failed to initialize VNRecognizeTextRequest".to_string())?
            };

            // 识别参数设置
            let () = unsafe { msg_send![&request, setRecognitionLevel: 0isize] }; // 0: Accurate
            let () = unsafe { msg_send![&request, setUsesLanguageCorrection: true] };

            if let Some(opts) = &options {
                if let Some(ref lang_code) = opts.language {
                    let lang_nsstr = NSString::from_str(lang_code);
                    let langs_array = NSArray::from_retained_slice(&[lang_nsstr]);
                    let () = unsafe { msg_send![&request, setRecognitionLanguages: &*langs_array] };
                }
            }

            // 2. 创建 VNImageRequestHandler
            let ns_data = NSData::from_slice(&bytes);
            let options_dict = NSDictionary::<objc2::runtime::AnyObject, objc2::runtime::AnyObject>::new();

            let handler: Retained<objc2::runtime::AnyObject> = unsafe {
                let obj: *mut objc2::runtime::AnyObject = msg_send![cls_handler, alloc];
                let obj: *mut objc2::runtime::AnyObject = msg_send![obj, initWithData: &*ns_data options: &*options_dict];
                Retained::from_raw(obj).ok_or("Failed to initialize VNImageRequestHandler".to_string())?
            };

            // 3. 运行 OCR 请求
            let requests_array = NSArray::from_retained_slice(&[request.clone()]);
            let mut error: *mut objc2::runtime::AnyObject = std::ptr::null_mut();

            let success: bool = unsafe {
                msg_send![&handler, performRequests: &*requests_array error: &mut error]
            };

            if !success {
                return Err("macOS Vision OCR request perform failed.".to_string());
            }

            // 4. 解析结果
            let results: Option<Retained<NSArray<objc2::runtime::AnyObject>>> = unsafe {
                msg_send![&request, results]
            };
            let results = results.ok_or_else(|| "No results returned from Vision OCR".to_string())?;
            let count: usize = unsafe { msg_send![&results, count] };

            let mut lines = Vec::new();
            let mut full_text_parts = Vec::new();

            for i in 0..count {
                let observation: Retained<objc2::runtime::AnyObject> = unsafe {
                    let obj: *mut objc2::runtime::AnyObject = msg_send![&results, objectAtIndex: i];
                    Retained::retain(obj).ok_or("Failed to retain observation")?
                };

                let candidates: Retained<NSArray<objc2::runtime::AnyObject>> = unsafe {
                    msg_send![&observation, topCandidates: 1usize]
                };
                if unsafe { msg_send![&candidates, count] } == 0 {
                    continue;
                }

                let recognized_text: Retained<objc2::runtime::AnyObject> = unsafe {
                    let obj: *mut objc2::runtime::AnyObject = msg_send![&candidates, objectAtIndex: 0];
                    Retained::retain(obj).ok_or("Failed to retain recognized_text")?
                };

                let text_nsstr: Retained<NSString> = unsafe { msg_send![&recognized_text, string] };
                let line_text = text_nsstr.to_string();
                full_text_parts.push(line_text.clone());

                // 获取 boundingBox
                let bbox: CGRect = unsafe { msg_send![&observation, boundingBox] };

                // macOS Vision 归一化且原点在左下角的 CGRect 转换到常规的左上角像素坐标
                let pixel_x = bbox.origin.x * width;
                let pixel_y = (1.0 - (bbox.origin.y + bbox.size.height)) * height;
                let pixel_width = bbox.size.width * width;
                let pixel_height = bbox.size.height * height;

                // 拆分单字与英文单词
                let mut words = Vec::new();
                let utf16_chars: Vec<u16> = line_text.encode_utf16().collect();
                let mut current_word_start: Option<usize> = None;

                let mut idx = 0;
                while idx < utf16_chars.len() {
                    let c = utf16_chars[idx];
                    let is_whitespace = c == 32 || c == 9 || c == 10 || c == 13;

                    // 中日韩字符判定区间
                    let is_cjk = (c >= 0x4E00 && c <= 0x9FFF)
                        || (c >= 0x3000 && c <= 0x303F)
                        || (c >= 0x3040 && c <= 0x30FF)
                        || (c >= 0x1100 && c <= 0x11FF)
                        || (c >= 0xAC00 && c <= 0xD7AF);

                    if !is_whitespace {
                        if is_cjk {
                            // CJK 字符单独提取
                            let word_range = NSRange { location: idx, length: 1 };
                            if let Some(word_word) = get_word_at_range(&recognized_text, word_range, &utf16_chars, width, height) {
                                words.push(word_word);
                            }
                        } else {
                            if current_word_start.is_none() {
                                current_word_start = Some(idx);
                            }
                        }
                    } else {
                        if let Some(start) = current_word_start {
                            let len = idx - start;
                            let word_range = NSRange { location: start, length: len };
                            if let Some(word_word) = get_word_at_range(&recognized_text, word_range, &utf16_chars, width, height) {
                                words.push(word_word);
                            }
                            current_word_start = None;
                        }
                    }

                    idx += 1;
                }

                if let Some(start) = current_word_start {
                    let len = utf16_chars.len() - start;
                    let word_range = NSRange { location: start, length: len };
                    if let Some(word_word) = get_word_at_range(&recognized_text, word_range, &utf16_chars, width, height) {
                        words.push(word_word);
                    }
                }

                lines.push(OcrLine {
                    text: line_text,
                    x: pixel_x as f32,
                    y: pixel_y as f32,
                    width: pixel_width as f32,
                    height: pixel_height as f32,
                    words,
                });
            }

            Ok(OcrResult {
                text: full_text_parts.join("\n"),
                lines,
            })
        })();

        result
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[cfg(target_os = "macos")]
fn get_word_at_range(
    recognized_text: &objc2::runtime::AnyObject,
    range: objc2_foundation::NSRange,
    utf16_source: &[u16],
    img_w: f64,
    img_h: f64,
) -> Option<OcrWord> {
    use core_graphics::geometry::CGRect;
    use objc2::msg_send;
    use objc2::rc::Retained;

    let mut error: *mut objc2::runtime::AnyObject = std::ptr::null_mut();
    let rect_obs: Option<Retained<objc2::runtime::AnyObject>> =
        unsafe { msg_send![recognized_text, boundingBoxForRange: range error: &mut error] };

    if let Some(obs) = rect_obs {
        let bbox: CGRect = unsafe { msg_send![&obs, boundingBox] };

        let word_text =
            String::from_utf16(&utf16_source[range.location..(range.location + range.length)])
                .ok()?;

        let pixel_x = bbox.origin.x * img_w;
        let pixel_y = (1.0 - (bbox.origin.y + bbox.size.height)) * img_h;
        let pixel_width = bbox.size.width * img_w;
        let pixel_height = bbox.size.height * img_h;

        Some(OcrWord {
            text: word_text,
            x: pixel_x as f32,
            y: pixel_y as f32,
            width: pixel_width as f32,
            height: pixel_height as f32,
        })
    } else {
        None
    }
}

// ==========================================
// Windows COM RAII Guard
// ==========================================
#[cfg(target_os = "windows")]
struct ComGuard;

#[cfg(target_os = "windows")]
impl ComGuard {
    fn new() -> Self {
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
        }
        ComGuard
    }
}

#[cfg(target_os = "windows")]
impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            windows::Win32::System::Com::CoUninitialize();
        }
    }
}

// ==========================================
// Linux Platform Implementation (Tesseract Bridge)
// ==========================================
#[cfg(target_os = "linux")]
fn map_bcp47_to_tesseract(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "zh" | "zh-cn" | "zh-hans" => "chi_sim".to_string(),
        "zh-tw" | "zh-hk" | "zh-hant" => "chi_tra".to_string(),
        "en" | "en-us" | "en-gb" => "eng".to_string(),
        "ja" | "jp" => "jpn".to_string(),
        "ko" | "kr" => "kor".to_string(),
        "fr" => "fra".to_string(),
        "de" => "deu".to_string(),
        "ru" => "rus".to_string(),
        "es" => "spa".to_string(),
        "it" => "ita".to_string(),
        _ => lang.to_string(),
    }
}

#[cfg(target_os = "linux")]
fn parse_tesseract_tsv(tsv_content: &str) -> Result<OcrResult, String> {
    let mut lines: Vec<OcrLine> = Vec::new();
    let mut full_text_parts: Vec<String> = Vec::new();

    let tsv_lines = tsv_content.lines();
    for tsv_line in tsv_lines {
        let parts: Vec<&str> = tsv_line.split('\t').collect();
        if parts.is_empty() || parts[0] == "level" {
            continue; // Skip header
        }

        let level_str = parts[0].trim();
        let level = match level_str.parse::<u32>() {
            Ok(v) => v,
            Err(_) => continue,
        };

        if level == 4 || level == 5 {
            if parts.len() < 10 {
                continue;
            }
            let left = parts[6].trim().parse::<f32>().unwrap_or(0.0);
            let top = parts[7].trim().parse::<f32>().unwrap_or(0.0);
            let width = parts[8].trim().parse::<f32>().unwrap_or(0.0);
            let height = parts[9].trim().parse::<f32>().unwrap_or(0.0);

            let text = if parts.len() > 11 {
                parts[11].trim().to_string()
            } else {
                "".to_string()
            };

            if level == 4 {
                let line_text = text.clone();
                if !line_text.is_empty() {
                    full_text_parts.push(line_text.clone());
                }
                lines.push(OcrLine {
                    text: line_text,
                    x: left,
                    y: top,
                    width,
                    height,
                    words: Vec::new(),
                });
            } else if level == 5 {
                if let Some(current_line) = lines.last_mut() {
                    current_line.words.push(OcrWord {
                        text,
                        x: left,
                        y: top,
                        width,
                        height,
                    });
                }
            }
        }
    }

    Ok(OcrResult {
        text: full_text_parts.join("\n"),
        lines,
    })
}

#[cfg(target_os = "linux")]
async fn run_linux_ocr(image: String, options: Option<OcrOptions>) -> Result<OcrResult, String> {
    let bytes = get_image_bytes(&image)?;

    tokio::task::spawn_blocking(move || {
        use std::io::Write;
        use std::process::Command;

        // 1. Write image to a temporary file
        let mut temp_file = tempfile::Builder::new()
            .suffix(".png")
            .tempfile()
            .map_err(|e| format!("Failed to create temp file: {}", e))?;

        temp_file
            .write_all(&bytes)
            .map_err(|e| format!("Failed to write temp image: {}", e))?;

        let temp_path = temp_file.path();

        // 2. Set up Tesseract command
        let mut cmd = Command::new("tesseract");
        cmd.arg(temp_path).arg("stdout");

        if let Some(opts) = &options {
            if let Some(ref lang_code) = opts.language {
                let mapped_lang = map_bcp47_to_tesseract(lang_code);
                cmd.arg("-l").arg(mapped_lang);
            }
        }

        cmd.arg("tsv");

        // 3. Execute Tesseract
        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "tesseract command not found. Please install tesseract-ocr (e.g., 'sudo apt install tesseract-ocr' and language packs like 'tesseract-ocr-chi-sim').".to_string()
            } else {
                format!("Failed to execute tesseract: {}", e)
            }
        })?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Tesseract failed with exit code {:?}: {}",
                output.status.code(),
                err_msg
            ));
        }

        let stdout_str = String::from_utf8(output.stdout)
            .map_err(|e| format!("Tesseract output is not valid UTF-8: {}", e))?;

        // 4. Parse TSV output
        let ocr_result = parse_tesseract_tsv(&stdout_str)?;

        Ok(ocr_result)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

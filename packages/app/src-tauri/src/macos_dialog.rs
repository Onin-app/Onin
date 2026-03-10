//! macOS 专用的对话框功能
//!
//! 提供自定义的文件/文件夹选择对话框，支持同时选择文件和文件夹。
//! 这是因为 Tauri 的官方对话框插件不支持这个功能。

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSModalResponseOK, NSOpenPanel};
use objc2_foundation::NSURL;

/// 打开一个可以同时选择文件和文件夹的对话框
///
/// 返回用户选择的路径列表，如果用户取消则返回空列表
pub fn open_file_and_folder_dialog() -> Vec<String> {
    // 获取主线程标记，NSOpenPanel 必须在主线程上运行
    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    unsafe {
        let panel = NSOpenPanel::openPanel(mtm);

        // 关键设置：同时允许选择文件和文件夹
        panel.setCanChooseFiles(true);
        panel.setCanChooseDirectories(true);
        panel.setAllowsMultipleSelection(true);
        panel.setCanCreateDirectories(true);

        // 使用 runModal 同步运行对话框
        let response = panel.runModal();

        if response == NSModalResponseOK {
            let urls: Retained<objc2_foundation::NSArray<NSURL>> = panel.URLs();
            let count = urls.count();

            let mut paths = Vec::with_capacity(count);
            for i in 0..count {
                if let Some(url) = urls.objectAtIndex(i).path() {
                    paths.push(url.to_string());
                }
            }
            paths
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    // 测试需要在 GUI 环境中运行，这里只做编译检查
    #[test]
    fn test_module_compiles() {
        // 确保模块可以编译
        assert!(true);
    }
}

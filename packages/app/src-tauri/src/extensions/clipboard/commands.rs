use crate::extensions::clipboard::monitor::{ClipboardHistory, ClipboardItem};
use clipboard_rs::{Clipboard, ClipboardContext};
use tauri::{command, State};

#[command]
pub fn get_clipboard_history(state: State<'_, ClipboardHistory>) -> Vec<ClipboardItem> {
    state.get_all()
}

#[command]
pub fn set_clipboard_item(text: String) -> Result<(), String> {
    let ctx = ClipboardContext::new().map_err(|e| e.to_string())?;
    ctx.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}

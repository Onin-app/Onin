use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatSessionMeta {
    pub id: String,
    pub title: String,
    pub provider_name: String,
    pub model_name: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub provider_name: String,
    pub model_name: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub messages: Vec<SessionMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionMessage {
    pub role: String, // "user" | "assistant"
    pub content: String,
}

pub struct HistoryManager {
    base_dir: PathBuf,
}

impl HistoryManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let base_dir = app_data_dir.join("ai_history");
        // 自动创建目录
        if !base_dir.exists() {
            let _ = fs::create_dir_all(&base_dir);
        }
        Self { base_dir }
    }

    fn index_path(&self) -> PathBuf {
        self.base_dir.join("sessions_index.json")
    }

    fn session_path(&self, id: &str) -> PathBuf {
        self.base_dir.join(format!("session_{}.json", id))
    }

    /// 获取所有会话的索引列表（按更新时间降序）
    pub fn load_index(&self) -> Result<Vec<ChatSessionMeta>, String> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read history index: {}", e))?;

        let mut index: Vec<ChatSessionMeta> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse history index: {}", e))?;

        // 按更新时间降序排序（最新修改的排最前）
        index.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(index)
    }

    /// 保存索引
    fn save_index(&self, index: &[ChatSessionMeta]) -> Result<(), String> {
        let path = self.index_path();
        let content = serde_json::to_string_pretty(index)
            .map_err(|e| format!("Failed to serialize index: {}", e))?;
        fs::write(&path, content).map_err(|e| format!("Failed to write history index: {}", e))?;
        Ok(())
    }

    /// 获取单个完整会话
    pub fn get_session(&self, id: &str) -> Result<ChatSession, String> {
        let path = self.session_path(id);
        if !path.exists() {
            return Err(format!("Session with ID {} not found", id));
        }

        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read session file: {}", e))?;

        let session: ChatSession = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse session JSON: {}", e))?;

        Ok(session)
    }

    /// 保存或更新会话，并自动更新对应的索引
    pub fn save_session(&self, session: ChatSession) -> Result<(), String> {
        // 1. 保存独立的会话文件
        let path = self.session_path(&session.id);
        let content = serde_json::to_string_pretty(&session)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;
        fs::write(&path, content).map_err(|e| format!("Failed to write session file: {}", e))?;

        // 2. 更新索引文件
        let mut index = self.load_index()?;

        let meta = ChatSessionMeta {
            id: session.id.clone(),
            title: session.title.clone(),
            provider_name: session.provider_name.clone(),
            model_name: session.model_name.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
        };

        // 查找是否已存在对应索引，存在则更新，不存在则添加
        if let Some(pos) = index.iter().position(|item| item.id == session.id) {
            index[pos] = meta;
        } else {
            index.push(meta);
        }

        self.save_index(&index)?;
        Ok(())
    }

    /// 删除特定会话并更新索引
    pub fn delete_session(&self, id: &str) -> Result<(), String> {
        // 1. 删除会话文件
        let path = self.session_path(id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to delete session file: {}", e))?;
        }

        // 2. 从索引中移除并保存
        let mut index = self.load_index()?;
        if let Some(pos) = index.iter().position(|item| item.id == id) {
            index.remove(pos);
            self.save_index(&index)?;
        }

        Ok(())
    }

    /// 清空所有历史会话
    pub fn clear_all_sessions(&self) -> Result<(), String> {
        let index = self.load_index()?;
        for item in index {
            let path = self.session_path(&item.id);
            if path.exists() {
                let _ = fs::remove_file(&path);
            }
        }

        // 删除索引文件本身
        let path = self.index_path();
        if path.exists() {
            let _ = fs::remove_file(&path);
        }

        Ok(())
    }
}

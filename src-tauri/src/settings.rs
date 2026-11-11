//! 设置持久化：读写应用配置目录下的 settings.json。

use std::fs;
use std::path::PathBuf;

use crate::models::Settings;

fn settings_path() -> Result<PathBuf, String> {
    let dir = dirs::config_dir()
        .ok_or_else(|| "无法定位配置目录".to_string())?
        .join("mirrordl-desktop");
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败：{e}"))?;
    Ok(dir.join("settings.json"))
}

/// 读取设置；文件不存在或损坏时返回默认设置
pub fn load_settings() -> Settings {
    let Ok(path) = settings_path() else {
        return Settings::default();
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return Settings::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// 保存设置到磁盘
pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let path = settings_path()?;
    let text = serde_json::to_string_pretty(settings).map_err(|e| format!("序列化失败：{e}"))?;
    fs::write(&path, text).map_err(|e| format!("写入设置失败：{e}"))?;
    Ok(())
}

/// 配置文件所在目录（供展示）
pub fn config_dir_display() -> String {
    settings_path()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_string_lossy().to_string()))
        .unwrap_or_default()
}

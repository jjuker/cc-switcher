//! 公共工具函数

use anyhow::{Context, Result};
use std::path::PathBuf;

/// 数据目录路径: ~/.cc-switcher/
pub fn data_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("无法获取用户主目录")?;
    Ok(home.join(".cc-switcher"))
}

/// 确保数据目录存在
pub fn ensure_data_dir() -> Result<PathBuf> {
    let dir = data_dir()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir).context(format!("无法创建目录: {}", dir.display()))?;
    }
    Ok(dir)
}

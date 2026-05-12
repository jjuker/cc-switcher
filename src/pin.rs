//! 项目级绑定模块

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 项目绑定配置
#[derive(Debug, Serialize, Deserialize)]
pub struct PinConfig {
    /// 绑定的配置名称
    pub config: String,
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 项目配置文件名
const PIN_FILE: &str = ".cc-switcher.json";
/// 向上查找的最大深度
const MAX_SEARCH_DEPTH: usize = 20;

/// 从当前目录向上查找项目绑定配置
pub fn find_pin_config() -> Result<Option<(PathBuf, PinConfig)>> {
    let cwd = std::env::current_dir().context("无法获取当前目录")?;

    for (depth, dir) in cwd.ancestors().enumerate() {
        if depth > MAX_SEARCH_DEPTH {
            break;
        }
        let pin_file = dir.join(PIN_FILE);
        if pin_file.exists() {
            let content = std::fs::read_to_string(&pin_file)
                .context(format!("无法读取: {}", pin_file.display()))?;
            let config: PinConfig = serde_json::from_str(&content)
                .context(format!("无法解析: {}", pin_file.display()))?;
            return Ok(Some((pin_file, config)));
        }
    }

    Ok(None)
}

/// 绑定当前目录到指定配置
pub fn pin_current_dir(config_name: &str, description: Option<String>) -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("无法获取当前目录")?;
    let pin_file = cwd.join(PIN_FILE);

    // 检查文件是否存在且非 PinConfig 格式
    if pin_file.exists() {
        let content = std::fs::read_to_string(&pin_file)
            .context(format!("无法读取: {}", pin_file.display()))?;

        // 尝试解析为 PinConfig
        if serde_json::from_str::<PinConfig>(&content).is_err() {
            println!("⚠️  文件 {} 不是有效的绑定配置，将被覆盖", pin_file.display());
        }
    }

    let config = PinConfig {
        config: config_name.to_string(),
        description,
    };

    let content = serde_json::to_string_pretty(&config)
        .context("无法序列化绑定配置")?;

    std::fs::write(&pin_file, content)
        .context(format!("无法写入: {}", pin_file.display()))?;

    Ok(pin_file)
}

/// 解除当前目录的绑定
pub fn unpin_current_dir() -> Result<bool> {
    let cwd = std::env::current_dir().context("无法获取当前目录")?;
    let pin_file = cwd.join(PIN_FILE);

    if pin_file.exists() {
        std::fs::remove_file(&pin_file)
            .context(format!("无法删除: {}", pin_file.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}
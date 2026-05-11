//! 配置管理模块

mod store;

use anyhow::{Context, Result};
use std::path::PathBuf;

pub use store::ConfigStore;

/// 配置实体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// 配置名称
    pub name: String,
    /// 配置文件路径
    pub path: PathBuf,
    /// 描述
    pub description: Option<String>,
    /// 是否当前激活
    #[serde(default)]
    pub is_current: bool,
}

impl Config {
    /// 创建新配置
    pub fn new(name: String, path: PathBuf, description: Option<String>) -> Self {
        Self {
            name,
            path,
            description,
            is_current: false,
        }
    }
}

/// Claude Code settings.json 路径
pub fn settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("无法获取用户主目录")?;
    Ok(home.join(".claude").join("settings.json"))
}

/// 配置管理器
pub struct ConfigManager {
    store: ConfigStore,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            store: ConfigStore::load()?,
        })
    }

    /// 添加配置
    pub fn add(&mut self, name: String, path: PathBuf, description: Option<String>) -> Result<()> {
        // 相对路径转换为绝对路径
        let path = if path.is_relative() {
            std::env::current_dir()
                .context("无法获取当前目录")?
                .join(path)
        } else {
            path
        };

        // 验证路径存在
        if !path.exists() {
            return Err(anyhow::anyhow!("配置文件不存在: {}", path.display()));
        }

        let config = Config::new(name.clone(), path, description);
        self.store.add(config)?;
        println!("✅ 已添加配置: {}", name);
        Ok(())
    }

    /// 列出所有配置
    pub fn list(&self) -> Result<()> {
        let configs = self.store.list();

        if configs.is_empty() {
            println!("暂无配置，使用 `cc-switcher config add <name> <path>` 添加");
            return Ok(());
        }

        println!("配置列表:");
        for config in configs {
            let current = if config.is_current { "✓" } else { " " };
            let desc = config.description.as_deref().unwrap_or("");
            println!(
                "  {} {} → {} {}",
                current,
                config.name,
                config.path.display(),
                desc
            );
        }
        Ok(())
    }

    /// 切换配置
    pub fn switch(&mut self, name: &str) -> Result<()> {
        // 获取目标配置
        let config = self.store.get(name)?;

        // 读取目标配置内容
        let content = std::fs::read_to_string(&config.path)
            .context(format!("无法读取配置文件: {}", config.path.display()))?;

        // 写入 settings.json
        let settings = settings_path()?;
        std::fs::write(&settings, content)
            .context(format!("无法写入 settings.json: {}", settings.display()))?;

        // 更新当前配置标记
        self.store.set_current(name)?;

        println!("✅ 已切换到配置: {}", name);
        Ok(())
    }

    /// 删除配置
    pub fn remove(&mut self, name: &str) -> Result<()> {
        self.store.remove(name)?;
        println!("✅ 已删除配置: {}", name);
        Ok(())
    }

    /// 显示当前配置
    pub fn current(&self) -> Result<()> {
        let current = self.store.current();

        match current {
            Some(config) => {
                println!("当前配置: {} → {}", config.name, config.path.display());
                if let Some(desc) = &config.description {
                    println!("描述: {}", desc);
                }
            }
            None => {
                println!("暂无激活配置");
            }
        }
        Ok(())
    }
}
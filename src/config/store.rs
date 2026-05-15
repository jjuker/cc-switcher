//! 配置存储（JSON）

use anyhow::{Context, Result};
use std::path::PathBuf;

use super::Config;
use crate::utils::ensure_data_dir;

/// 配置存储
pub struct ConfigStore {
    /// 配置列表
    configs: Vec<Config>,
    /// 存储路径
    path: PathBuf,
}

impl ConfigStore {
    /// 加载配置
    pub fn load() -> Result<Self> {
        let dir = ensure_data_dir()?;
        let path = dir.join("configs.json");

        if !path.exists() {
            return Ok(Self {
                configs: Vec::new(),
                path,
            });
        }

        let content = std::fs::read_to_string(&path)
            .context(format!("无法读取配置文件: {}", path.display()))?;

        let configs: Vec<Config> = serde_json::from_str(&content).context("无法解析配置文件")?;

        Ok(Self { configs, path })
    }

    /// 保存配置
    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.configs).context("无法序列化配置")?;

        std::fs::write(&self.path, content)
            .context(format!("无法写入配置文件: {}", self.path.display()))?;

        Ok(())
    }

    /// 添加配置
    pub fn add(&mut self, config: Config) -> Result<()> {
        // 检查名称是否已存在
        if self.configs.iter().any(|c| c.name == config.name) {
            return Err(anyhow::anyhow!("配置名称已存在: {}", config.name));
        }

        self.configs.push(config);
        self.save()?;
        Ok(())
    }

    /// 获取配置
    pub fn get(&self, name: &str) -> Result<&Config> {
        self.configs
            .iter()
            .find(|c| c.name == name)
            .context(format!("配置不存在: {}", name))
    }

    /// 列出所有配置
    pub fn list(&self) -> &[Config] {
        &self.configs
    }

    /// 设置全局默认配置
    pub fn set_default(&mut self, name: &str) -> Result<()> {
        if !self.configs.iter().any(|c| c.name == name) {
            return Err(anyhow::anyhow!("配置不存在: {}", name));
        }
        for config in &mut self.configs {
            config.is_default = config.name == name;
        }
        self.save()
    }

    /// 删除配置
    pub fn remove(&mut self, name: &str) -> Result<RemoveInfo> {
        let index = self
            .configs
            .iter()
            .position(|c| c.name == name)
            .context(format!("配置不存在: {}", name))?;

        let config = self.configs.remove(index);
        self.save()?;

        Ok(RemoveInfo {
            was_default: config.is_default,
            path: config.path,
        })
    }

    /// 获取默认配置
    pub fn get_default(&self) -> Option<&Config> {
        self.configs.iter().find(|c| c.is_default)
    }
}

/// 删除结果信息
pub struct RemoveInfo {
    pub was_default: bool,
    pub path: PathBuf,
}

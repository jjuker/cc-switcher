//! 模型定价表，从 ~/.cc-switcher/pricing.json 加载

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::utils::ensure_data_dir;

/// 编译时嵌入默认定价配置
const DEFAULT_PRICING: &str = include_str!("../../pricing.default.json");

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PricingInfo {
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_read_per_million: f64,
    pub cache_creation_per_million: f64,
}

/// 模型定价
pub struct ModelPricing {
    pricing: HashMap<String, PricingInfo>,
}

impl ModelPricing {
    fn file_path() -> Result<PathBuf> {
        let dir = ensure_data_dir()?;
        Ok(dir.join("pricing.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::file_path()?;

        if !path.exists() {
            std::fs::write(&path, DEFAULT_PRICING)
                .context(format!("无法初始化定价配置: {}", path.display()))?;
        }

        let content = std::fs::read_to_string(&path)
            .context(format!("无法读取定价文件: {}", path.display()))?;
        let pricing: HashMap<String, PricingInfo> = serde_json::from_str(&content)
            .context("无法解析定价文件")?;
        Ok(Self { pricing })
    }

    pub fn get(&self, model: &str) -> Option<&PricingInfo> {
        if let Some(info) = self.pricing.get(model) {
            return Some(info);
        }

        let normalized = model
            .split('-')
            .take_while(|s| !s.starts_with('2'))
            .collect::<Vec<_>>()
            .join("-");

        self.pricing.get(&normalized)
    }

    pub fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_creation_tokens: u64,
    ) -> f64 {
        let info = match self.get(model) {
            Some(i) => i,
            None => return 0.0,
        };

        (input_tokens as f64 / 1_000_000.0) * info.input_per_million
            + (output_tokens as f64 / 1_000_000.0) * info.output_per_million
            + (cache_read_tokens as f64 / 1_000_000.0) * info.cache_read_per_million
            + (cache_creation_tokens as f64 / 1_000_000.0) * info.cache_creation_per_million
    }
}

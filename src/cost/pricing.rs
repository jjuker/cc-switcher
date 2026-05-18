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

impl PricingInfo {
    /// 根据定价信息计算费用
    pub fn calculate_cost(&self, record: &super::CostRecord) -> f64 {
        (record.input_tokens as f64 / 1_000_000.0) * self.input_per_million
            + (record.output_tokens as f64 / 1_000_000.0) * self.output_per_million
            + (record.cache_read_tokens as f64 / 1_000_000.0) * self.cache_read_per_million
            + (record.cache_creation_tokens as f64 / 1_000_000.0) * self.cache_creation_per_million
    }
}

/// pricing.json 文件结构（支持别名映射）
#[derive(serde::Deserialize)]
struct PricingFile {
    /// 模型定价（键为模型名，值为定价信息）
    #[serde(flatten)]
    pricing: HashMap<String, PricingInfo>,
    /// 模型别名映射（长名 → 短名），优先于启发式匹配
    #[serde(default)]
    aliases: HashMap<String, String>,
}

/// 模型定价
pub struct ModelPricing {
    pricing: HashMap<String, PricingInfo>,
    /// 显式别名表
    aliases: HashMap<String, String>,
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
        let file: PricingFile =
            serde_json::from_str(&content).context("无法解析定价文件")?;
        Ok(Self {
            pricing: file.pricing,
            aliases: file.aliases,
        })
    }

    /// 查找模型定价（优先级：直接匹配 > 别名表 > 启发式去日期后缀）
    pub fn get(&self, model: &str) -> Option<&PricingInfo> {
        // 1. 直接匹配
        if let Some(info) = self.pricing.get(model) {
            return Some(info);
        }

        // 2. 别名查找
        if let Some(alias) = self.aliases.get(model) {
            if let Some(info) = self.pricing.get(alias) {
                return Some(info);
            }
        }

        // 3. 启发式：去掉日期后缀（如 "-20250514"）
        if let Some(normalized) = normalize_model_name(model) {
            if normalized != model {
                if let Some(info) = self.pricing.get(&normalized) {
                    return Some(info);
                }
            }
        }

        None
    }
}

/// 判断 segment 是否为可能的 YYYYMMDD 日期后缀
fn is_date_segment(s: &str) -> bool {
    if s.len() != 8 || !s.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    // 年份千年位：2000-2999 范围内
    let first = s.as_bytes()[0];
    first == b'1' || first == b'2'
}

/// 基于命名约定的启发式归一化：只去掉末尾的日期段
/// 例:
///   "deepseek-v4-pro-20250514" → Some("deepseek-v4-pro")
///   "claude-3-opus-20240229-v2" → None（v2 保留，最后一段非日期）
///   全部为日期段 → None（如 "20250514"）
fn normalize_model_name(model: &str) -> Option<String> {
    let segments: Vec<&str> = model.split('-').collect();
    // 从末尾开始去掉 8 位纯数字 segment，遇到非日期段即停
    let end = segments.len()
        - segments
            .iter()
            .rev()
            .take_while(|s| is_date_segment(s))
            .count();

    if end == 0 || end == segments.len() {
        return None;
    }

    Some(segments[..end].join("-"))
}

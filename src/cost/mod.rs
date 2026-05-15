//! 成本追踪模块 — 直接从 JSONL 会话日志实时解析

mod collector;
mod display;
mod pricing;
mod session;

use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};

pub use pricing::ModelPricing;

/// 原始成本记录（从 session 解析）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CostRecord {
    pub date: NaiveDate,
    pub model: String,
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
}

/// 聚合统计（含预计算成本）
#[derive(Debug, Clone, serde::Serialize)]
pub struct AggregatedStats {
    pub date: NaiveDate,
    pub model: String,
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cost: f64,
}

/// 成本管理器
pub struct CostManager {
    pricing: ModelPricing,
}

impl CostManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            pricing: ModelPricing::load()?,
        })
    }

    pub fn today(&self) -> Result<()> {
        let today = Local::now().date_naive();
        let stats = collector::collect_stats(&self.pricing)?
            .into_iter()
            .filter(|s| s.date == today)
            .collect::<Vec<_>>();
        display::show_stats(&stats, today, today)?;
        Ok(())
    }

    pub fn month(&self) -> Result<()> {
        let now = Local::now();
        let start = now.with_day(1).unwrap().date_naive();
        let end = now.date_naive();
        let stats = collector::collect_stats(&self.pricing)?
            .into_iter()
            .filter(|s| s.date >= start && s.date <= end)
            .collect::<Vec<_>>();
        display::show_stats(&stats, start, end)?;
        Ok(())
    }

    pub fn report(&self, format: &str) -> Result<()> {
        let mut stats = collector::collect_stats(&self.pricing)?;
        stats.sort_by(|a, b| b.date.cmp(&a.date));
        display::report(&stats, format)?;
        Ok(())
    }
}

//! 成本追踪模块 — 直接从 JSONL 会话日志实时解析

mod collector;
mod display;
mod pricing;
mod session;

use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};
use std::collections::HashSet;

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

    /// 收集数据
    fn collect(&self) -> Result<collector::CollectResult> {
        collector::collect_stats(&self.pricing)
    }

    pub fn today(&self) -> Result<()> {
        let today = Local::now().date_naive();
        let result = self.collect()?;
        let stats: Vec<_> = result
            .stats
            .into_iter()
            .filter(|s| s.date == today)
            .collect();

        // 仅显示今日数据中出现的未知模型
        let models_today: HashSet<&str> =
            stats.iter().map(|s| s.model.as_str()).collect();
        let unknown: Vec<&str> = result
            .unknown_models
            .iter()
            .filter(|m| models_today.contains(m.as_str()))
            .map(|s| s.as_str())
            .collect();
        if !unknown.is_empty() {
            eprintln!(
                "⚠️  以下模型未在定价表中找到，费用按 $0.00 计算: {}",
                unknown.join(", ")
            );
        }

        display::show_stats(&stats, today, today)?;
        Ok(())
    }

    pub fn month(&self) -> Result<()> {
        let now = Local::now();
        let start = now.with_day(1).unwrap().date_naive();
        let end = now.date_naive();
        let result = self.collect()?;
        let stats: Vec<_> = result
            .stats
            .into_iter()
            .filter(|s| s.date >= start && s.date <= end)
            .collect();

        // 仅显示本月数据中出现的未知模型
        let models_this_month: HashSet<&str> =
            stats.iter().map(|s| s.model.as_str()).collect();
        let unknown: Vec<&str> = result
            .unknown_models
            .iter()
            .filter(|m| models_this_month.contains(m.as_str()))
            .map(|s| s.as_str())
            .collect();
        if !unknown.is_empty() {
            eprintln!(
                "⚠️  以下模型未在定价表中找到，费用按 $0.00 计算: {}",
                unknown.join(", ")
            );
        }

        display::show_stats(&stats, start, end)?;
        Ok(())
    }

    pub fn report(&self, format: &str) -> Result<()> {
        let result = self.collect()?;
        result.print_warnings();
        let mut stats = result.stats;
        stats.sort_by(|a, b| b.date.cmp(&a.date));
        display::report(&stats, format)?;
        Ok(())
    }
}

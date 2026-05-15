//! 成本追踪模块 — 直接从 JSONL 会话日志实时解析

mod pricing;
mod session;

use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};
use std::collections::HashMap;

pub use pricing::ModelPricing;

/// 成本记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CostRecord {
    /// 日期
    pub date: NaiveDate,
    /// 模型
    pub model: String,
    /// 请求次数
    pub requests: u64,
    /// 输入 tokens
    pub input_tokens: u64,
    /// 输出 tokens
    pub output_tokens: u64,
    /// 缓存读取 tokens
    pub cache_read_tokens: u64,
    /// 缓存创建 tokens
    pub cache_creation_tokens: u64,
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

    /// 扫描会话日志目录，解析所有 JSONL 文件并按 (日期, 模型) 聚合
    fn collect_records(&self) -> Result<Vec<CostRecord>> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        let projects_dir = home.join(".claude").join("projects");

        if !projects_dir.exists() {
            return Ok(vec![]);
        }

        let mut aggregated: HashMap<(NaiveDate, String), CostRecord> = HashMap::new();

        for entry in std::fs::read_dir(&projects_dir)? {
            let project_dir = entry?.path();
            if !project_dir.is_dir() {
                continue;
            }
            for session_file in std::fs::read_dir(&project_dir)? {
                let path = session_file?.path();
                if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                    if let Ok(records) = session::parse_session(&path) {
                        for record in records {
                            let key = (record.date, record.model.clone());
                            let entry = aggregated.entry(key).or_insert_with(|| CostRecord {
                                date: record.date,
                                model: record.model,
                                requests: 0,
                                input_tokens: 0,
                                output_tokens: 0,
                                cache_read_tokens: 0,
                                cache_creation_tokens: 0,
                            });
                            entry.requests += record.requests;
                            entry.input_tokens += record.input_tokens;
                            entry.output_tokens += record.output_tokens;
                            entry.cache_read_tokens += record.cache_read_tokens;
                            entry.cache_creation_tokens += record.cache_creation_tokens;
                        }
                    }
                }
            }
        }

        Ok(aggregated.into_values().collect())
    }

    /// 今日统计
    pub fn today(&self) -> Result<()> {
        let today = Local::now().date_naive();
        let records = self.collect_records()?;
        let filtered: Vec<CostRecord> = records
            .into_iter()
            .filter(|r| r.date == today)
            .collect();
        self.show_stats(&filtered, today, today)?;
        Ok(())
    }

    /// 本月统计
    pub fn month(&self) -> Result<()> {
        let now = Local::now();
        let start = now.with_day(1).unwrap().date_naive();
        let end = now.date_naive();
        let records = self.collect_records()?;
        let filtered: Vec<CostRecord> = records
            .into_iter()
            .filter(|r| r.date >= start && r.date <= end)
            .collect();
        self.show_stats(&filtered, start, end)?;
        Ok(())
    }

    /// 显示统计（按模型分组）
    fn show_stats(&self, records: &[CostRecord], start: NaiveDate, end: NaiveDate) -> Result<()> {
        if records.is_empty() {
            println!("暂无数据");
            return Ok(());
        }

        // 按 model 分组
        let mut by_model: HashMap<String, Vec<&CostRecord>> = HashMap::new();
        for r in records {
            by_model.entry(r.model.clone()).or_default().push(r);
        }

        println!("日期范围: {} ~ {}", start, end);

        // 每个模型的汇总，按名称排序
        let mut models_sorted: Vec<_> = by_model.keys().collect();
        models_sorted.sort();

        let mut total_requests = 0u64;
        let mut total_cost = 0.0f64;

        for model in &models_sorted {
            let group = &by_model[*model];
            let reqs = group.iter().map(|r| r.requests).sum::<u64>();
            let input = group.iter().map(|r| r.input_tokens).sum::<u64>();
            let output = group.iter().map(|r| r.output_tokens).sum::<u64>();
            let cache_read = group.iter().map(|r| r.cache_read_tokens).sum::<u64>();
            let cache_creation = group.iter().map(|r| r.cache_creation_tokens).sum::<u64>();
            let cost = self.pricing.calculate_cost(
                model, input, output, cache_read, cache_creation,
            );

            total_requests += reqs;
            total_cost += cost;

            println!(
                "\n  {} — {} 次 | 输入 {:.1}K | 输出 {:.1}K | 缓存读 {:.1}K | 缓存建 {:.1}K | $ {:.2}",
                model, reqs,
                input as f64 / 1000.0, output as f64 / 1000.0,
                cache_read as f64 / 1000.0, cache_creation as f64 / 1000.0,
                cost
            );
        }

        println!("\n  合计: {} 次 | $ {:.2}", total_requests, total_cost);

        Ok(())
    }

    /// 详细报告
    pub fn report(&self, format: &str) -> Result<()> {
        let mut records = self.collect_records()?;
        records.sort_by(|a, b| b.date.cmp(&a.date));

        if records.is_empty() {
            println!("暂无数据");
            return Ok(());
        }

        match format {
            "json" => {
                let json = serde_json::to_string_pretty(&records)?;
                println!("{}", json);
            }
            "table" => {
                println!(
                    "Date       | Model          | Requests | Input   | Output  | Cache R | Cache W | Cost"
                );
                println!(
                    "-----------|----------------|----------|---------|---------|---------|---------|--------"
                );
                for r in &records {
                    let cost = self.pricing.calculate_cost(
                        &r.model,
                        r.input_tokens,
                        r.output_tokens,
                        r.cache_read_tokens,
                        r.cache_creation_tokens,
                    );
                    println!(
                        "{} | {:<14} | {:>8} | {:>7}K | {:>7}K | {:>7}K | {:>7}K | ${:>6.2}",
                        r.date,
                        r.model,
                        r.requests,
                        r.input_tokens / 1000,
                        r.output_tokens / 1000,
                        r.cache_read_tokens / 1000,
                        r.cache_creation_tokens / 1000,
                        cost
                    );
                }
            }
            _ => {
                return Err(anyhow::anyhow!("不支持的格式: {}", format));
            }
        }

        Ok(())
    }
}
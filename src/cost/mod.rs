//! 成本追踪模块

mod db;
mod pricing;
mod session;

use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};

pub use db::CostDb;
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
    db: CostDb,
    pricing: ModelPricing,
}

impl CostManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: CostDb::new()?,
            pricing: ModelPricing::load()?,
        })
    }

    /// 今日统计
    pub fn today(&self) -> Result<()> {
        let today = Local::now().date_naive();
        self.show_stats(today, today)?;
        Ok(())
    }

    /// 本月统计
    pub fn month(&self) -> Result<()> {
        let now = Local::now();
        let start = now.with_day(1).unwrap().date_naive();
        let end = now.date_naive();
        self.show_stats(start, end)?;
        Ok(())
    }

    /// 显示统计
    fn show_stats(&self, start: NaiveDate, end: NaiveDate) -> Result<()> {
        let records = self.db.get_range(start, end)?;

        if records.is_empty() {
            println!("暂无数据，使用 `cc-switcher cost sync` 同步会话日志");
            return Ok(())
        }

        // 汇总
        let total_requests = records.iter().map(|r| r.requests).sum::<u64>();
        let total_input = records.iter().map(|r| r.input_tokens).sum::<u64>();
        let total_output = records.iter().map(|r| r.output_tokens).sum::<u64>();
        let total_cache_read = records.iter().map(|r| r.cache_read_tokens).sum::<u64>();
        let total_cache_write = records.iter().map(|r| r.cache_creation_tokens).sum::<u64>();

        // 按模型分组计算成本
        let mut total_cost = 0.0;
        for record in &records {
            total_cost += self.pricing.calculate_cost(
                &record.model,
                record.input_tokens,
                record.output_tokens,
                record.cache_read_tokens,
                record.cache_creation_tokens,
            );
        }

        println!(
            "日期范围: {} ~ {}\n\
            请求次数: {}\n\
            输入 tokens: {} ({:.1}K)\n\
            输出 tokens: {} ({:.1}K)\n\
            缓存读取: {} ({:.1}K)\n\
            缓存写入: {} ({:.1}K)\n\
            估算成本: $ {:.2}",
            start, end,
            total_requests,
            total_input, total_input as f64 / 1000.0,
            total_output, total_output as f64 / 1000.0,
            total_cache_read, total_cache_read as f64 / 1000.0,
            total_cache_write, total_cache_write as f64 / 1000.0,
            total_cost
        );

        Ok(())
    }

    /// 详细报告
    pub fn report(&self, format: &str) -> Result<()> {
        let records = self.db.get_all()?;

        if records.is_empty() {
            println!("暂无数据");
            return Ok(())
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
                for r in records {
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

    /// 同步会话日志
    pub fn sync(&mut self) -> Result<()> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;

        let projects_dir = home.join(".claude").join("projects");

        if !projects_dir.exists() {
            println!("Claude Code 会话目录不存在: {}", projects_dir.display());
            return Ok(())
        }

        let mut imported = 0;
        let mut skipped = 0;

        // 遍历所有项目目录
        for entry in std::fs::read_dir(&projects_dir)? {
            let project_dir = entry?.path();

            // 遍历所有会话文件
            for session_file in std::fs::read_dir(&project_dir)? {
                let path = session_file?.path();

                if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                    match session::parse_session(&path, &self.pricing) {
                        Ok(records) => {
                            for record in records {
                                self.db.insert(&record)?;
                                imported += 1;
                            }
                        }
                        Err(e) => {
                            println!("跳过文件 {}: {}", path.display(), e);
                            skipped += 1;
                        }
                    }
                }
            }
        }

        println!("同步完成: 导入 {} 条记录，跳过 {} 个文件", imported, skipped);
        Ok(())
    }
}
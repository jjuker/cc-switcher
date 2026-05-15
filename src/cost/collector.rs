use anyhow::Result;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Once;
use walkdir::WalkDir;

use super::{AggregatedStats, CostRecord};
use crate::cost::pricing::ModelPricing;
use crate::cost::session;

static PARSE_WARNING: Once = Once::new();

/// 扫描会话目录，解析失败时警告（仅首次）
pub fn scan_session_files(dir: &Path) -> impl Iterator<Item = CostRecord> + '_ {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "jsonl").unwrap_or(false))
        .flat_map(|e| {
            session::parse_session(e.path()).unwrap_or_else(|_| {
                PARSE_WARNING.call_once(|| {
                    eprintln!("⚠️  部分会话日志解析失败，成本统计可能不完整");
                });
                vec![]
            })
        })
}

/// 聚合原始记录并计算成本
pub fn aggregate_with_cost(
    records: Vec<CostRecord>,
    pricing: &ModelPricing,
) -> Vec<AggregatedStats> {
    let mut grouped: HashMap<(NaiveDate, String), CostRecord> = HashMap::new();

    for r in records {
        let key = (r.date, r.model.clone());
        grouped
            .entry(key)
            .and_modify(|e| {
                e.requests += r.requests;
                e.input_tokens += r.input_tokens;
                e.output_tokens += r.output_tokens;
                e.cache_read_tokens += r.cache_read_tokens;
                e.cache_creation_tokens += r.cache_creation_tokens;
            })
            .or_insert(r);
    }

    grouped
        .into_values()
        .map(|r| {
            let cost = pricing.calculate_cost(
                &r.model,
                r.input_tokens,
                r.output_tokens,
                r.cache_read_tokens,
                r.cache_creation_tokens,
            );
            AggregatedStats {
                date: r.date,
                model: r.model,
                requests: r.requests,
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                cache_read_tokens: r.cache_read_tokens,
                cache_creation_tokens: r.cache_creation_tokens,
                cost,
            }
        })
        .collect()
}

/// 扫描并聚合所有统计数据
pub fn collect_stats(pricing: &ModelPricing) -> Result<Vec<AggregatedStats>> {
    let dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?
        .join(".claude")
        .join("projects");

    if !dir.exists() {
        return Ok(vec![]);
    }

    let records: Vec<CostRecord> = scan_session_files(&dir).collect();
    Ok(aggregate_with_cost(records, pricing))
}

use anyhow::Result;
use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::{AggregatedStats, CostRecord};
use crate::cost::pricing::ModelPricing;
use crate::cost::session;

/// Claude Code 会话日志目录
fn claude_projects_dir() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?
        .join(".claude")
        .join("projects");
    Ok(dir)
}

/// 目录扫描结果
struct ScanResult {
    records: Vec<CostRecord>,
    total_lines: usize,
    skipped_lines: usize,
    failed_files: usize,
}

/// 聚合结果（含未知模型列表）
struct AggregateResult {
    stats: Vec<AggregatedStats>,
    unknown_models: Vec<String>,
}

/// 聚合结果（含诊断信息，供调用方决定如何处理）
#[derive(Default)]
pub struct CollectResult {
    pub stats: Vec<AggregatedStats>,
    pub unknown_models: Vec<String>,
    pub total_lines: usize,
    pub skipped_lines: usize,
    pub failed_files: usize,
}

impl CollectResult {
    /// 打印采集过程中的警告信息
    pub fn print_warnings(&self) {
        if self.skipped_lines > 0 {
            eprintln!(
                "⚠️  解析了 {} 行，其中 {} 行格式不匹配",
                self.total_lines, self.skipped_lines
            );
        }
        if self.failed_files > 0 {
            eprintln!("⚠️  {} 个会话文件解析失败", self.failed_files);
        }
        if !self.unknown_models.is_empty() {
            eprintln!(
                "⚠️  以下模型未在定价表中找到，费用按 $0.00 计算: {}",
                self.unknown_models.join(", ")
            );
        }
    }

    /// 警告当前统计范围内出现的未知模型
    pub fn warn_unknown_models(&self, active_models: &HashSet<&str>) {
        let unknown: Vec<&str> = self
            .unknown_models
            .iter()
            .filter(|m| active_models.contains(m.as_str()))
            .map(|s| s.as_str())
            .collect();
        if !unknown.is_empty() {
            eprintln!(
                "⚠️  以下模型未在定价表中找到，费用按 $0.00 计算: {}",
                unknown.join(", ")
            );
        }
    }
}

/// 扫描会话目录，返回原始记录和解析统计
fn scan_session_files(dir: &Path) -> ScanResult {
    let mut all_records = Vec::new();
    let mut total_lines = 0;
    let mut skipped_lines = 0;
    let mut failed_files = 0;

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if !matches!(entry.path().extension(), Some(ext) if ext == "jsonl") {
            continue;
        }
        match session::parse_session(entry.path()) {
            Ok(result) => {
                total_lines += result.total_lines;
                skipped_lines += result.skipped_lines;
                all_records.extend(result.records);
            }
            Err(_) => {
                failed_files += 1;
            }
        }
    }

    ScanResult {
        records: all_records,
        total_lines,
        skipped_lines,
        failed_files,
    }
}

/// 聚合原始记录并计算成本
fn aggregate_with_cost(
    records: &[CostRecord],
    pricing: &ModelPricing,
) -> AggregateResult {
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
            .or_insert_with(|| r.clone());
    }

    let mut unknown_models = HashSet::new();

    let stats = grouped
        .into_values()
        .map(|r| {
            let (cost, found) = match pricing.get(&r.model) {
                Some(info) => (info.calculate_cost(&r), true),
                None => (0.0, false),
            };

            if !found {
                unknown_models.insert(r.model.clone());
            }

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
        .collect();

    let mut unknown_list: Vec<String> = unknown_models.into_iter().collect();
    unknown_list.sort();

    AggregateResult {
        stats,
        unknown_models: unknown_list,
    }
}

/// 扫描并聚合所有统计数据
pub fn collect_stats(pricing: &ModelPricing) -> Result<CollectResult> {
    let dir = claude_projects_dir()?;

    if !dir.exists() {
        return Ok(CollectResult::default());
    }

    let scan = scan_session_files(&dir);
    let agg = aggregate_with_cost(&scan.records, pricing);

    Ok(CollectResult {
        stats: agg.stats,
        unknown_models: agg.unknown_models,
        total_lines: scan.total_lines,
        skipped_lines: scan.skipped_lines,
        failed_files: scan.failed_files,
    })
}

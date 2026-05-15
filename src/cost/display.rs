use anyhow::Result;
use chrono::NaiveDate;
use std::collections::HashMap;

use super::AggregatedStats;

/// 按模型分组展示统计
pub fn show_stats(stats: &[AggregatedStats], start: NaiveDate, end: NaiveDate) -> Result<()> {
    if stats.is_empty() {
        println!("暂无数据");
        return Ok(());
    }

    let mut by_model: HashMap<String, Vec<&AggregatedStats>> = HashMap::new();
    for s in stats {
        by_model.entry(s.model.clone()).or_default().push(s);
    }

    println!("日期范围: {} ~ {}", start, end);

    let mut models_sorted: Vec<_> = by_model.keys().collect();
    models_sorted.sort();

    let mut total_requests = 0u64;
    let mut total_cost = 0.0f64;

    for model in &models_sorted {
        let group = &by_model[*model];
        let (reqs, input, output, cache_read, cache_creation, cost) = group.iter().fold(
            (0u64, 0u64, 0u64, 0u64, 0u64, 0.0f64),
            |(r, i, o, cr, cc, c), s| {
                (
                    r + s.requests,
                    i + s.input_tokens,
                    o + s.output_tokens,
                    cr + s.cache_read_tokens,
                    cc + s.cache_creation_tokens,
                    c + s.cost,
                )
            },
        );

        total_requests += reqs;
        total_cost += cost;

        println!(
            "\n  {} — {} 次 | 输入 {:.1}K | 输出 {:.1}K | 缓存读 {:.1}K | 缓存建 {:.1}K | $ {:.2}",
            model,
            reqs,
            input as f64 / 1000.0,
            output as f64 / 1000.0,
            cache_read as f64 / 1000.0,
            cache_creation as f64 / 1000.0,
            cost
        );
    }

    println!("\n  合计: {} 次 | $ {:.2}", total_requests, total_cost);

    Ok(())
}

/// 详细报告（json/table 格式）
pub fn report(stats: &[AggregatedStats], format: &str) -> Result<()> {
    if stats.is_empty() {
        println!("暂无数据");
        return Ok(());
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(stats)?;
            println!("{}", json);
        }
        "table" => {
            println!("Date       | Model          | Requests | Input   | Output  | Cache R | Cache W | Cost");
            println!("-----------|----------------|----------|---------|---------|---------|---------|--------");
            for s in stats {
                println!(
                    "{} | {:<14} | {:>8} | {:>7}K | {:>7}K | {:>7}K | {:>7}K | ${:>6.2}",
                    s.date,
                    s.model,
                    s.requests,
                    s.input_tokens / 1000,
                    s.output_tokens / 1000,
                    s.cache_read_tokens / 1000,
                    s.cache_creation_tokens / 1000,
                    s.cost
                );
            }
        }
        _ => return Err(anyhow::anyhow!("不支持的格式: {}", format)),
    }

    Ok(())
}

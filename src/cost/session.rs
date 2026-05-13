//! 会话日志解析

use anyhow::{Context, Result};
use chrono::{DateTime, Utc, NaiveDate};
use std::path::Path;
use std::collections::HashMap;

use super::CostRecord;

/// 会话文件中的单行消息
#[derive(Debug, serde::Deserialize)]
struct SessionMessage {
    #[serde(rename = "type")]
    msg_type: String,
    message: Option<MessageContent>,
    timestamp: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct MessageContent {
    model: Option<String>,
    usage: Option<Usage>,
}

#[derive(Debug, serde::Deserialize)]
struct Usage {
    input_tokens: u64,
    output_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
}

/// 解析会话文件
pub fn parse_session(path: &Path) -> Result<Vec<CostRecord>> {
    let content = std::fs::read_to_string(path)
        .context(format!("无法读取会话文件: {}", path.display()))?;

    let mut records: HashMap<(NaiveDate, String), CostRecord> = HashMap::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let msg: SessionMessage = match serde_json::from_str(line) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if msg.msg_type != "assistant" {
            continue;
        }

        let message = match msg.message {
            Some(m) => m,
            None => continue,
        };

        let usage = match message.usage {
            Some(u) => u,
            None => continue,
        };

        let model = message.model.unwrap_or_else(|| "unknown".into());

        let date = msg
            .timestamp
            .as_deref()
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc).date_naive())
            .unwrap_or_else(|| get_file_date(path).unwrap_or_default());

        let key = (date, model.clone());
        let entry = records.entry(key).or_insert_with(|| CostRecord {
            date,
            model,
            requests: 0,
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
        });

        entry.requests += 1;
        entry.input_tokens += usage.input_tokens;
        entry.output_tokens += usage.output_tokens;
        entry.cache_read_tokens += usage.cache_read_input_tokens;
        entry.cache_creation_tokens += usage.cache_creation_input_tokens;
    }

    Ok(records.into_values().collect())
}

/// 获取文件修改日期（fallback）
fn get_file_date(path: &Path) -> Result<NaiveDate> {
    let metadata = std::fs::metadata(path)?;
    let modified: DateTime<Utc> = metadata.modified()?.into();
    Ok(modified.date_naive())
}

//! SQLite 存储

use anyhow::Result;
use rusqlite::{Connection, params};
use chrono::NaiveDate;

use crate::utils::ensure_data_dir;
use super::CostRecord;

/// 成本数据库
pub struct CostDb {
    conn: Connection,
}

impl CostDb {
    /// 创建数据库
    pub fn new() -> Result<Self> {
        let dir = ensure_data_dir()?;
        let path = dir.join("costs.db");
        let conn = Connection::open(&path)?;

        // 创建表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cost_records (
                date TEXT NOT NULL,
                model TEXT NOT NULL,
                requests INTEGER NOT NULL,
                input_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                cache_read_tokens INTEGER NOT NULL,
                cache_creation_tokens INTEGER NOT NULL,
                PRIMARY KEY (date, model)
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    /// 插入记录（按日期+模型聚合）
    pub fn insert(&self, record: &CostRecord) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO cost_records (
                date, model, requests, input_tokens, output_tokens,
                cache_read_tokens, cache_creation_tokens
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.date.to_string(),
                record.model,
                record.requests,
                record.input_tokens,
                record.output_tokens,
                record.cache_read_tokens,
                record.cache_creation_tokens,
            ],
        )?;

        Ok(())
    }

    /// 获取日期范围内的记录
    pub fn get_range(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<CostRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT date, model, requests, input_tokens, output_tokens,
                    cache_read_tokens, cache_creation_tokens
             FROM cost_records
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date DESC"
        )?;

        let records = stmt.query_map(params![start.to_string(), end.to_string()], |row| {
            parse_row(row)
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// 获取所有记录
    pub fn get_all(&self) -> Result<Vec<CostRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT date, model, requests, input_tokens, output_tokens,
                    cache_read_tokens, cache_creation_tokens
             FROM cost_records
             ORDER BY date DESC"
        )?;

        let records = stmt.query_map([], |row| {
            parse_row(row)
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }
}

/// 从数据库行解析成本记录
fn parse_row(row: &rusqlite::Row) -> rusqlite::Result<CostRecord> {
    let date_str: String = row.get(0)?;
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|_| rusqlite::Error::InvalidParameterName("日期格式错误".into()))?;

    Ok(CostRecord {
        date,
        model: row.get(1)?,
        requests: row.get(2)?,
        input_tokens: row.get(3)?,
        output_tokens: row.get(4)?,
        cache_read_tokens: row.get(5)?,
        cache_creation_tokens: row.get(6)?,
    })
}
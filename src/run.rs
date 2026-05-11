//! Claude Code 执行器

use anyhow::{Context, Result};

/// 用指定配置运行 Claude Code
pub fn run_with_config(name: &str, args: &[String]) -> Result<()> {
    let mut manager = crate::config::ConfigManager::new()?;
    manager.switch(name)?;

    println!("启动 Claude Code...");

    let status = std::process::Command::new("claude")
        .args(args)
        .status()
        .context("无法启动 Claude Code")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Claude Code 执行失败"));
    }

    Ok(())
}

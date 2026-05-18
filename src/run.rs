//! Claude Code 执行器

use anyhow::{Context, Result};

/// 解析配置（优先级：用户指定 > 项目 pin > 全局 default）
/// 返回 &Config 引用，避免二次查找
pub fn resolve_config(
    user_input: Option<String>,
    manager: &crate::config::ConfigManager,
) -> Result<&crate::config::Config> {
    // 优先级 1: 用户直接指定
    if let Some(name) = user_input {
        return manager.get_config(&name);
    }

    // 优先级 2: 项目级 pin
    if let Some((path, pin_config)) = crate::pin::find_pin_config()? {
        return manager.get_config(&pin_config.config).with_context(|| {
            format!(
                "pin 文件 {} 指向的配置 '{}' 不存在",
                path.display(),
                pin_config.config
            )
        });
    }

    // 优先级 3: 全局 default
    if let Some(config) = manager.get_default_config() {
        return Ok(config);
    }

    Err(anyhow::anyhow!(
        "无可用配置：未指定配置名、未找到项目绑定(.cc-switcher.json)，且未设置默认配置。\n\
        请先添加配置: ccs add <name> <path>\n\
        然后设置默认: ccs default <name>\n\
        或项目绑定: ccs pin <name>"
    ))
}

/// 用指定配置运行 Claude Code（通过 --settings 参数传递配置文件）
pub fn run_with_config(config: &crate::config::Config, args: &[String]) -> Result<()> {
    println!("启动 Claude Code [{}]...", config.name);

    let mut cmd = std::process::Command::new("claude");
    cmd.arg("--settings").arg(&config.path);
    cmd.args(args);

    let status = cmd.status().context("无法启动 Claude Code")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Claude Code 执行失败"));
    }

    Ok(())
}

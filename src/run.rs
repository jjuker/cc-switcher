//! Claude Code 执行器

use anyhow::{Context, Result};

/// 自动选择配置名称（优先级：用户指定 > 项目 pin > 全局 default）
pub fn resolve_config_name(
    user_input: Option<String>,
    config_manager: &crate::config::ConfigManager,
) -> Result<String> {
    // 优先级 1: 用户直接指定
    if let Some(name) = user_input {
        if !config_manager.exists(&name) {
            return Err(anyhow::anyhow!("配置不存在: {}", name));
        }
        return Ok(name);
    }

    // 优先级 2: 项目级 pin
    if let Some((path, pin_config)) = crate::pin::find_pin_config()? {
        if !config_manager.exists(&pin_config.config) {
            return Err(anyhow::anyhow!(
                "pin 文件 {} 指向的配置 '{}' 不存在",
                path.display(),
                pin_config.config
            ));
        }
        return Ok(pin_config.config);
    }

    // 优先级 3: 全局 default
    if let Some(name) = config_manager.get_default() {
        // 验证配置存在（防止残留数据）
        if !config_manager.exists(&name) {
            return Err(anyhow::anyhow!(
                "默认配置 '{}' 不存在，请重新设置: ccs default <name>",
                name
            ));
        }
        return Ok(name);
    }

    // 无任何配置可用
    Err(anyhow::anyhow!(
        "无可用配置。\n\
        请先添加配置: ccs add <name> <path>\n\
        然后设置默认: ccs default <name>\n\
        或项目绑定: ccs pin <name>"
    ))
}

/// 用指定配置运行 Claude Code
pub fn run_with_config(name: &str, args: &[String]) -> Result<()> {
    let mut manager = crate::config::ConfigManager::new()?;
    manager.switch(name)?;

    println!("启动 Claude Code [{}]...", name);

    let status = std::process::Command::new("claude")
        .args(args)
        .status()
        .context("无法启动 Claude Code")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Claude Code 执行失败"));
    }

    Ok(())
}
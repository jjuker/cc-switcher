//! Claude Code 执行器

use anyhow::{Context, Result};
use std::collections::HashMap;

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
        if !config_manager.exists(&name) {
            return Err(anyhow::anyhow!(
                "默认配置 '{}' 不存在，请重新设置: ccs default <name>",
                name
            ));
        }
        return Ok(name);
    }

    Err(anyhow::anyhow!(
        "无可用配置。\n\
        请先添加配置: ccs add <name> <path>\n\
        然后设置默认: ccs default <name>\n\
        或项目绑定: ccs pin <name>"
    ))
}

/// 用指定配置运行 Claude Code（环境变量注入，不修改全局 settings.json）
pub fn run_with_config(
    name: &str,
    args: &[String],
    manager: &crate::config::ConfigManager,
) -> Result<()> {
    let config = manager.get_config(name)?;
    let env_vars = extract_env_vars(&config.path)?;

    println!("启动 Claude Code [{}]...", name);

    let mut cmd = std::process::Command::new("claude");
    cmd.args(args);

    for (key, value) in &env_vars {
        cmd.env(key, value);
    }

    let status = cmd.status().context("无法启动 Claude Code")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Claude Code 执行失败"));
    }

    Ok(())
}

/// 从配置文件的 "env" 块提取环境变量
fn extract_env_vars(path: &std::path::Path) -> Result<HashMap<String, String>> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("无法读取配置文件: {}", path.display()))?;

    let config: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("无法解析配置文件: {}", path.display()))?;

    let mut env_vars = HashMap::new();

    if let Some(env_obj) = config.get("env").and_then(|v| v.as_object()) {
        for (key, value) in env_obj {
            match value {
                serde_json::Value::String(s) if !s.is_empty() => {
                    env_vars.insert(key.clone(), s.clone());
                }
                serde_json::Value::Number(n) => {
                    env_vars.insert(key.clone(), n.to_string());
                }
                serde_json::Value::Bool(b) => {
                    env_vars.insert(key.clone(), b.to_string());
                }
                _ => {}
            }
        }
    }

    Ok(env_vars)
}

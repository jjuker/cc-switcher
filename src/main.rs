//! cc-switcher - Claude Code 配置管理器 + 成本追踪器

mod cli;
mod config;
mod cost;
mod pin;
mod run;
mod utils;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => handle_run(None, vec![])?,
        Some(cmd) => handle_command(cmd)?,
    }

    Ok(())
}

fn handle_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Use { name, args } => handle_run(name, args)?,
        Commands::Run(args) => handle_external_run(args)?,
        Commands::Default { name } => handle_default(&name)?,
        Commands::Pin { name } => handle_pin(&name)?,
        Commands::Unpin => handle_unpin()?,
        Commands::List => handle_list()?,
        Commands::New { name, description } => handle_new(name, description)?,
        Commands::Add {
            name,
            path,
            description,
        } => handle_add(name, path, description)?,
        Commands::Remove { name, delete } => handle_remove(&name, delete)?,
        Commands::Today => handle_today()?,
        Commands::Month => handle_month()?,
        Commands::Report { format } => handle_report(&format)?,
    }
    Ok(())
}

/// 处理 external_subcommand（如 ccs deepseek 或 ccs deepseek -- -p）
fn handle_external_run(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("需要指定配置名称"));
    }

    // 解析：第一个参数是配置名，"--" 后面的传给 claude
    let (name, claude_args) = parse_external_args(&args);
    handle_run(Some(name), claude_args)?;
    Ok(())
}

/// 解析 external_subcommand 参数
fn parse_external_args(args: &[String]) -> (String, Vec<String>) {
    let name = args[0].clone();
    let claude_args = if args.len() > 1 && args[1] == "--" {
        args[2..].to_vec()
    } else if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec![]
    };
    (name, claude_args)
}

fn handle_run(name: Option<String>, args: Vec<String>) -> Result<()> {
    let manager = config::ConfigManager::new()?;
    let config = run::resolve_config(name, &manager)?;
    run::run_with_config(config, &args)?;
    Ok(())
}

fn handle_default(name: &str) -> Result<()> {
    let mut manager = config::ConfigManager::new()?;
    manager.set_default(name)?;
    Ok(())
}

fn handle_pin(name: &str) -> Result<()> {
    // 验证配置存在（get_config 失败自带 "配置不存在" 错误）
    let manager = config::ConfigManager::new()?;
    manager.get_config(name)?;

    let pin_file = pin::pin_current_dir(name, None)?;
    println!("✅ 已绑定配置: {} → {}", pin_file.display(), name);
    Ok(())
}

fn handle_unpin() -> Result<()> {
    let removed = pin::unpin_current_dir()?;
    if removed {
        println!("✅ 已解除项目绑定");
    } else {
        println!("当前目录未绑定配置");
    }
    Ok(())
}

fn handle_list() -> Result<()> {
    let manager = config::ConfigManager::new()?;
    manager.list()?;
    Ok(())
}

fn handle_add(name: String, path: String, description: Option<String>) -> Result<()> {
    let mut manager = config::ConfigManager::new()?;
    let path = std::path::PathBuf::from(path);
    manager.add(name, path, description)?;
    Ok(())
}

fn handle_new(name: String, description: Option<String>) -> Result<()> {
    let mut manager = config::ConfigManager::new()?;
    manager.new_config(name, description)?;
    Ok(())
}

fn handle_remove(name: &str, delete: bool) -> Result<()> {
    let mut manager = config::ConfigManager::new()?;
    let info = manager.remove(name)?;

    // 输出状态
    if info.was_default {
        println!(
            "⚠️  已删除默认配置 '{}', 请重新设置: ccs default <name>",
            name
        );
    } else {
        println!("✅ 已删除配置: {}", name);
    }

    // 文件删除：仅删除内部管理的配置文件
    if delete {
        maybe_delete_config_file(&info.path)?;
    }

    Ok(())
}

/// 删除配置文件（仅删除内部管理的文件，需用户确认）
fn maybe_delete_config_file(path: &std::path::Path) -> Result<()> {
    let configs_dir = config::configs_dir()?;

    if !path.starts_with(&configs_dir) {
        if path.exists() {
            println!("⚠️  配置文件非内部管理，请手动处理: {}", path.display());
        }
        return Ok(());
    }

    if !path.exists() {
        return Ok(());
    }

    // 确认删除
    println!("确认删除文件 {}？[y/N]", path.display());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        std::fs::remove_file(path)?;
        println!("✅ 已删除文件: {}", path.display());
    } else {
        println!("已跳过文件删除");
    }

    Ok(())
}

fn handle_today() -> Result<()> {
    let manager = cost::CostManager::new()?;
    manager.today()?;
    Ok(())
}

fn handle_month() -> Result<()> {
    let manager = cost::CostManager::new()?;
    manager.month()?;
    Ok(())
}

fn handle_report(format: &str) -> Result<()> {
    let manager = cost::CostManager::new()?;
    manager.report(format)?;
    Ok(())
}

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
        Commands::Default { name } => handle_default(&name)?,
        Commands::Pin { name } => handle_pin(&name)?,
        Commands::Unpin => handle_unpin()?,
        Commands::List => handle_list()?,
        Commands::Add { name, path, description } => handle_add(name, path, description)?,
        Commands::Remove { name } => handle_remove(&name)?,
        Commands::Today => handle_today()?,
        Commands::Month => handle_month()?,
        Commands::Sync => handle_sync()?,
        Commands::Report { format } => handle_report(&format)?,
    }
    Ok(())
}

fn handle_run(name: Option<String>, args: Vec<String>) -> Result<()> {
    let manager = config::ConfigManager::new()?;
    let config_name = run::resolve_config_name(name, &manager)?;
    run::run_with_config(&config_name, &args)?;
    Ok(())
}

fn handle_default(name: &str) -> Result<()> {
    let mut manager = config::ConfigManager::new()?;
    manager.set_default(name)?;
    Ok(())
}

fn handle_pin(name: &str) -> Result<()> {
    // 验证配置存在
    let manager = config::ConfigManager::new()?;
    if !manager.exists(name) {
        return Err(anyhow::anyhow!("配置不存在: {}", name));
    }

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

fn handle_remove(name: &str) -> Result<()> {
    let mut manager = config::ConfigManager::new()?;
    manager.remove(name)?;
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

fn handle_sync() -> Result<()> {
    let mut manager = cost::CostManager::new()?;
    manager.sync()?;
    Ok(())
}

fn handle_report(format: &str) -> Result<()> {
    let manager = cost::CostManager::new()?;
    manager.report(format)?;
    Ok(())
}

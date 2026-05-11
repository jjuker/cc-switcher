//! cc-switcher - Claude Code 配置管理器 + 成本追踪器

mod cli;
mod config;
mod cost;
mod run;
mod utils;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands, ConfigCommands, CostCommands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { action } => handle_config(action)?,
        Commands::Cost { action } => handle_cost(action)?,
        Commands::Run { name, args } => run::run_with_config(&name, &args)?,
    }

    Ok(())
}

fn handle_config(action: ConfigCommands) -> Result<()> {
    let mut manager = config::ConfigManager::new()?;

    match action {
        ConfigCommands::Add { name, path, description } => {
            let path = std::path::PathBuf::from(path);
            manager.add(name, path, description)?;
        }
        ConfigCommands::List => {
            manager.list()?;
        }
        ConfigCommands::Switch { name } => {
            manager.switch(&name)?;
        }
        ConfigCommands::Remove { name } => {
            manager.remove(&name)?;
        }
        ConfigCommands::Current => {
            manager.current()?;
        }
    }

    Ok(())
}

fn handle_cost(action: CostCommands) -> Result<()> {
    let mut manager = cost::CostManager::new()?;

    match action {
        CostCommands::Today => {
            manager.today()?;
        }
        CostCommands::Month => {
            manager.month()?;
        }
        CostCommands::Report { format } => {
            manager.report(&format)?;
        }
        CostCommands::Sync => {
            manager.sync()?;
        }
    }

    Ok(())
}
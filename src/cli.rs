//! CLI 命令定义

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cc-switcher")]
#[command(about = "Claude Code 配置管理器 + 成本追踪器")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 配置管理
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    /// 成本统计
    Cost {
        #[command(subcommand)]
        action: CostCommands,
    },
    /// 用指定配置运行 Claude Code
    Run {
        /// 配置名称
        name: String,
        /// 传递给 claude 的参数
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// 添加配置
    Add {
        /// 配置名称
        name: String,
        /// 配置文件路径
        path: String,
        /// 描述
        #[arg(short, long)]
        description: Option<String>,
    },
    /// 列出所有配置
    List,
    /// 切换配置
    Switch {
        /// 配置名称
        name: String,
    },
    /// 删除配置
    Remove {
        /// 配置名称
        name: String,
    },
    /// 显示当前配置
    Current,
}

#[derive(Subcommand)]
pub enum CostCommands {
    /// 今日统计
    Today,
    /// 本月统计
    Month,
    /// 详细报告
    Report {
        /// 输出格式 (table/json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// 同步会话日志
    Sync,
}
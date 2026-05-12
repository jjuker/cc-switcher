//! CLI 命令定义

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ccs")]
#[command(about = "Claude Code 配置管理器 + 成本追踪器")]
#[command(after_help = "\
示例:
  ccs                    # 自动选择配置启动 Claude Code
  ccs work               # 用 work 配置启动
  ccs work -- -p         # 带参数启动
  ccs default work       # 设置全局默认配置
  ccs pin work           # 项目级绑定配置
")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 启动 Claude Code（无参数自动选择，有参数指定配置）
    #[command(visible_alias = "run")]
    Use {
        /// 配置名称（可选，空则自动选择：pin > default）
        name: Option<String>,
        /// 传递给 claude 的参数
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// 设置全局默认配置
    Default {
        /// 配置名称
        name: String,
    },
    /// 项目级绑定配置（写入当前目录 .cc-switcher.json）
    Pin {
        /// 配置名称
        name: String,
    },
    /// 解除项目绑定
    Unpin,
    /// 列出所有配置
    #[command(visible_alias = "ls")]
    List,
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
    /// 删除配置
    #[command(visible_alias = "rm")]
    Remove {
        /// 配置名称
        name: String,
    },
    /// 今日成本统计
    Today,
    /// 本月成本统计
    Month,
    /// 同步会话日志
    Sync,
    /// 详细成本报告
    Report {
        /// 输出格式 (table/json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}
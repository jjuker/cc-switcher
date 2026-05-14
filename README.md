# cc-switcher

> Claude Code 配置管理器 + 成本追踪器（Volta 风格）

## 核心概念

借鉴 Volta 工具链管理器的设计，实现三层配置选择机制：

| 命令 | 作用 | 示例 |
|------|------|------|
| `ccs` | 自动选择配置启动 | cd 进入项目自动识别 |
| `ccs default` | 设置全局默认 | `ccs default work` |
| `ccs pin` | 项目级绑定 | `ccs pin personal` |

**优先级**: 用户指定 > 项目 pin > 全局 default

## 快速开始

```bash
# 添加配置
ccs add work ~/.claude/configs/work.json
ccs add personal ~/.claude/configs/personal.json

# 设置全局默认
ccs default work

# 启动（自动选择 work 配置）
ccs

# 项目级绑定
cd ~/my-project
ccs pin personal
ccs  # → 使用 personal 配置

# 解除绑定
ccs unpin
```

## 命令一览

```
ccs                          # 自动选择配置启动 Claude Code
ccs <name>                   # 用指定配置启动
ccs <name> -- <args>         # 带参数启动

ccs default <name>           # 设置全局默认配置
ccs pin <name>               # 项目级绑定（写入 .cc-switcher.json）
ccs unpin                    # 解除项目绑定
ccs list                     # 列出所有配置
ccs new <name>               # 新建配置（自动生成模板并打开编辑器）
ccs add <name> <path>        # 添加已有配置文件
ccs remove <name>            # 删除配置

ccs today                    # 今日成本统计
ccs month                    # 本月成本统计
ccs report                   # 详细成本报告
```

## 项目绑定文件

项目目录下的 `.cc-switcher.json`：

```json
{
  "config": "work",
  "description": "此项目使用 work 配置"
}
```

cd 进入项目时自动识别，无需手动切换。

## 成本追踪

```bash
# 今日统计
ccs today

# 本月统计
ccs month

# 详细报告
ccs report
ccs report -f json
```

输出示例：
```
日期范围: 2026-05-11 ~ 2026-05-11
请求次数: 42
输入 tokens: 125000 (125.0K)
输出 tokens: 89000 (89.0K)
缓存读取: 340000 (340.0K)
缓存写入: 50000 (50.0K)
估算成本: $ 3.21
```

## 存储位置

- 配置索引：`~/.cc-switcher/configs.json`
- 成本数据：实时扫描 `~/.claude/projects/` 下 JSONL 会话文件
- 定价配置：`~/.cc-switcher/pricing.json`
- 项目绑定：`<project-dir>/.cc-switcher.json`

## 安装

```bash
cargo build --release
# 将 target/release/cc-switcher.exe 复制到 PATH
# 或创建别名: ccs = cc-switcher
```

## 架构

```
src/
├── main.rs           # CLI 入口
├── cli.rs            # 命令定义（clap）
├── pin.rs            # 项目绑定模块
├── run.rs            # Claude Code 执行器
├── utils.rs          # 工具函数
├── config/
│   ├── mod.rs        # 配置管理器
│   └── store.rs      # 配置存储（JSON）
└── cost/
    ├── mod.rs        # 成本管理器
    ├── session.rs    # JSONL 会话日志解析
    └── pricing.rs    # 模型定价
```
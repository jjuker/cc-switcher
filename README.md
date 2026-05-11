# cc-switcher

> Claude Code 配置管理器 + 成本追踪器（极简版）

## 核心问题

Claude Code 的配置文件在 `~/.claude/settings.json`。当你需要：
- 不同项目用不同配置（不同 MCP 服务器、不同权限设置）
- 切换配置时手动改文件，容易出错且麻烦
- 想知道花了多少钱（token 数量、请求次数）

这就是 **cc-switcher** 要解决的问题。

## 功能

### 1. 配置管理

```bash
# 添加配置
cc-switcher config add work ~/.claude/configs/work.json
cc-switcher config add personal ~/.claude/configs/personal.json

# 列出所有配置
cc-switcher config list

# 切换配置（写入 ~/.claude/settings.json）
cc-switcher config switch work

# 用指定配置运行 Claude Code
cc-switcher run work -- claude --dangerously-skip-permissions
```

### 2. 成本追踪

```bash
# 查看今日统计
cc-switcher cost today

# 查看本月统计
cc-switcher cost month

# 详细报告
cc-switcher cost report --format table
```

输出示例：
```
Date       | Requests | Input | Output | Cache Read | Cache Write | Est. Cost
2026-05-11 | 42       | 125K  | 89K    | 340K       | 50K         | $3.21
```

## 设计思路

### 数据结构

**配置实体**：
```rust
struct Config {
    name: String,      // 配置名称（work, personal）
    path: PathBuf,     // 配置文件路径
    description: String, // 描述（可选）
    is_current: bool,  // 是否当前激活
}
```

**成本记录**：
```rust
struct CostRecord {
    date: String,
    model: String,
    requests: u64,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
}
```

### 存储位置

- 配置索引：`~/.cc-switcher/configs.json`（存储所有配置元数据）
- 成本数据：`~/.cc-switcher/costs.db`（SQLite，按日期索引）
- 实际配置文件：用户指定路径（cc-switcher 不负责创建，只负责管理）

### 成本数据来源

**会话日志解析**（参考 cc-switch-main 的实现）：

Claude Code 的会话日志存储在：
- `~/.claude/projects/{project-hash}/session-{id}.jsonl`

每行包含 usage 信息：
```json
{
  "type": "assistant",
  "message": { "usage": { "input_tokens": 1234, "output_tokens": 567, ... } }
}
```

cc-switcher 解析这些日志文件，提取 token 使用量。

### 模型定价

内置定价表（参考 cc-switch-main 的 `model_pricing` 表）：
- Claude Opus 4.7: $5/$25 per 1M (input/output)
- Claude Sonnet 4.6: $3/$15 per 1M
- Claude Haiku 4.5: $1/$5 per 1M
- ...

## 架构

```
src/
├── main.rs           # CLI 入口
├── cli.rs            # 命令行参数解析（clap）
├── config/
│   ├── mod.rs        # 配置管理核心逻辑
│   └── store.rs      # 配置存储（JSON）
├── cost/
│   ├── mod.rs        # 成本计算逻辑
│   ├── session.rs    # 会话日志解析器
│   ├── pricing.rs    # 模型定价表
│   └── db.rs         # SQLite 存储
└── run.rs            # Claude Code 执行器
```

## 技术栈

- **CLI 框架**：`clap` - 标准的 Rust CLI 解析库
- **存储**：`serde_json`（配置）+ `rusqlite`（成本）
- **执行**：标准 `std::process::Command`

## 与 cc-switch-main 的区别

| 功能 | cc-switch-main | cc-switcher |
|------|----------------|-------------|
| GUI 界面 | ✅ Tauri 桌面应用 | ❌ 纯 CLI |
| 多应用支持 | ✅ Claude/Codex/Gemini/OpenCode/OpenClaw/Hermes | ❌ 仅 Claude Code |
| 代理模式 | ✅ 本地代理 + 故障转移 | ❌ 无 |
| MCP/Skills 管理 | ✅ 统一管理面板 | ❌ 无 |
| 配置切换 | ✅ | ✅ |
| 成本追踪 | ✅ | ✅ |
| 会话管理 | ✅ 浏览历史会话 | ❌ 无 |

简化版 = **配置切换 + 成本追踪**，200 行代码搞定核心功能。
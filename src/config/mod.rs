//! 配置管理模块

mod store;

use anyhow::{Context, Result};
use std::path::PathBuf;

pub use store::ConfigStore;

/// 配置实体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// 配置名称
    pub name: String,
    /// 配置文件路径
    pub path: PathBuf,
    /// 描述
    pub description: Option<String>,
    /// 是否全局默认
    #[serde(default)]
    pub is_default: bool,
}

impl Config {
    /// 创建新配置
    pub fn new(name: String, path: PathBuf, description: Option<String>) -> Self {
        Self {
            name,
            path,
            description,
            is_default: false,
        }
    }
}

/// 配置文件默认存放目录: ~/.cc-switcher/configs/
pub fn configs_dir() -> Result<PathBuf> {
    let dir = crate::utils::ensure_data_dir()?.join("configs");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .context(format!("无法创建配置目录: {}", dir.display()))?;
    }
    Ok(dir)
}

/// 默认配置模板
pub const DEFAULT_CONFIG_TEMPLATE: &str = r#"{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "env": {
    "ANTHROPIC_BASE_URL": "",
    "ANTHROPIC_AUTH_TOKEN": "",
    "ANTHROPIC_MODEL": "",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
    "CLAUDE_CODE_SKIP_BEDROCK_AUTH": "1",
    "CLAUDE_CODE_EFFORT_LEVEL": "max"
  },
  "language": "中文",
  "skipWebFetchPreflight": true,
  "theme": "dark-ansi",
  "verbose": true
}
"#;


/// 配置管理器
pub struct ConfigManager {
    store: ConfigStore,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            store: ConfigStore::load()?,
        })
    }

    /// 添加配置
    pub fn add(&mut self, name: String, path: PathBuf, description: Option<String>) -> Result<()> {
        // 相对路径转换为绝对路径
        let path = if path.is_relative() {
            std::env::current_dir()
                .context("无法获取当前目录")?
                .join(path)
        } else {
            path
        };

        // 验证路径存在
        if !path.exists() {
            return Err(anyhow::anyhow!("配置文件不存在: {}", path.display()));
        }

        let config = Config::new(name.clone(), path, description);
        self.store.add(config)?;
        println!("✅ 已添加配置: {}", name);
        Ok(())
    }

    /// 列出所有配置
    pub fn list(&self) -> Result<()> {
        let configs = self.store.list();

        if configs.is_empty() {
            println!("暂无配置，使用 `ccs add <name> <path>` 添加");
            return Ok(());
        }

        println!("配置列表:");
        for config in configs {
            let default = if config.is_default { "★" } else { " " };
            let desc = config.description.as_deref().unwrap_or("");
            println!(
                "  {} {} → {} {}",
                default,
                config.name,
                config.path.display(),
                desc
            );
        }
        println!("\n  ★ 全局默认");
        Ok(())
    }

    /// 删除配置（返回文件路径信息）
    pub fn remove(&mut self, name: &str) -> Result<store::RemoveInfo> {
        let info = self.store.remove(name)?;
        Ok(info)
    }

    /// 设置全局默认配置
    pub fn set_default(&mut self, name: &str) -> Result<()> {
        self.store.set_default(name)?;
        println!("✅ 已设置默认配置: {}", name);
        Ok(())
    }

    /// 获取默认配置名称
    pub fn get_default(&self) -> Option<String> {
        self.store.get_default().map(|c| c.name.clone())
    }

    /// 获取配置
    pub fn get_config(&self, name: &str) -> Result<&Config> {
        self.store.get(name)
    }

    /// 检查配置是否存在
    pub fn exists(&self, name: &str) -> bool {
        self.store.get(name).is_ok()
    }

    /// 新建配置（自动创建文件，打开编辑器）
    pub fn new_config(&mut self, name: String, description: Option<String>) -> Result<()> {
        // 验证名称
        let name = validate_config_name(&name)?;

        // 生成配置文件路径
        let configs_dir = configs_dir()?;
        let path = configs_dir.join(format!("{}.json", name));

        // 检查文件是否已存在
        if path.exists() {
            return Err(anyhow::anyhow!("配置文件已存在: {}", path.display()));
        }

        // 写入默认模板
        std::fs::write(&path, DEFAULT_CONFIG_TEMPLATE)
            .context(format!("无法写入配置文件: {}", path.display()))?;

        // 先打印，再添加到存储
        println!("✅ 已创建配置");
        println!("📝 配置文件: {}", path.display());

        let config = Config::new(name.clone(), path, description);
        self.store.add(config)?;

        // 打开编辑器（用保存的 name 获取）
        open_editor(&self.store.get(&name)?.path)?;

        Ok(())
    }
}

/// 验证配置名称（防止路径遍历和非法文件名）
fn validate_config_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(anyhow::anyhow!("配置名称不能为空"));
    }
    // 检查路径分隔符和 Windows 禁止字符
    if name.contains(['/', '\\', ':', '\0', '<', '>', '"', '|', '?', '*']) {
        return Err(anyhow::anyhow!(
            "配置名称不能包含特殊字符: / \\ : < > \" | ? *"
        ));
    }
    // 检查路径遍历（仅检查 . 和 .. 作为独立名称）
    if name == "." || name == ".." {
        return Err(anyhow::anyhow!("无效的配置名称"));
    }
    // Windows 文件名不能以空格或点结尾
    if name.ends_with(' ') || name.ends_with('.') {
        return Err(anyhow::anyhow!("配置名称不能以空格或点结尾"));
    }
    if name.len() > 64 {
        return Err(anyhow::anyhow!("配置名称过长（最多64字符）"));
    }
    Ok(name.to_string())
}

/// 打开编辑器编辑配置文件
fn open_editor(path: &std::path::Path) -> Result<()> {
    // 优先使用环境变量，默认 VS Code
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "code".to_string());

    // 安全验证：只允许简单的编辑器名称（字母、数字、下划线、连字符、点）
    // 拒绝路径和任何可能触发 shell 注入的字符
    if !editor.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
        println!("⚠️  编辑器名称包含非法字符，请手动编辑: {}", path.display());
        println!("   编辑器设置: {}", editor);
        return Ok(());
    }

    let result = std::process::Command::new(&editor).arg(path).status();

    match result {
        Ok(status) if status.success() => {}
        Ok(_) => println!("⚠️  编辑器启动失败，请手动编辑: {}", path.display()),
        Err(_) => println!("⚠️  未找到编辑器 '{}', 请手动编辑: {}", editor, path.display()),
    }

    Ok(())
}

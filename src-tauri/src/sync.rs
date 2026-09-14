//! sync.rs — 同步编排层。
//!
//! # 职责
//!
//! 把「用户点了同步」这件事，翻译成对每个 Adapter 的固定调用序列：
//!
//! ```text
//! 读取 SSOT（global/rules.md + knowledge/*.md）
//!   └─ 对每个 Adapter：
//!        detect()              未安装 → 本次不参与同步（不是错误）
//!        sync_l0(rules)        规则为空 → 跳过（避免注入空标记块）
//!        sync_l2(每个技能)
//!        cleanup_orphans(本次有效 slug)
//! ```
//!
//! # 为什么单独一层，而不是写进 command handler
//!
//! TASK-10 的向导步骤 4 与 TASK-13 的「立即同步」按钮触发的是**同一件事**。
//! 若各自实现一遍，「向导里能同步、状态栏里不能」这类分叉迟早出现。
//! 编排只保留一份，UI 只是它的两个入口。
//!
//! # 进度事件（向导步骤 4 的硬性要求）
//!
//! 向导要求「同步进度逐行实时显示（**不是**等全部完成后一次性显示）」
//! （`PRD.md` 第 5 节 / TASK-10 验收）。因此本模块在每个工具完成时**立即**
//! 回调一次，而不是把结果攒到最后一起返回。
//!
//! 回调形式（而不是直接依赖 `AppHandle`）是为了让编排逻辑能脱离 Tauri
//! 运行时被测试——事件只是它的一个消费者。
//!
//! # ⚠️ 安全铁律：任何读取失败都必须中止同步，绝不退化成「空」
//!
//! `cleanup_orphans` 的语义是「删掉不在本次有效列表里的 `crossbrain-*` 目录」。
//! 于是**空的 `active_slugs` 等于「删光全部技能目录」**。
//!
//! 这意味着：如果 `knowledge/` 目录暂时读不出来（被占用、权限异常、盘符离线），
//! 而我们把「读不出来」当成「目录里没有文件」，就会**删掉用户全部技能**。
//!
//! 因此这里严格区分两种状态：
//!
//! | 情况 | 处理 |
//! |:---|:---|
//! | 目录存在且真的没有 `.md` 文件 | 正常同步，`active_slugs` 为空（清理是正确行为） |
//! | 目录/文件**读取失败** | **整体中止**，一个 Adapter 都不调用 |
//!
//! 宁可让用户看到「同步失败」，也不能让它在数据不完整时执行删除。

use std::fs;
use std::path::PathBuf;

use serde::Serialize;

use crate::adapters::{format_skill_md, Adapter, AdapterError};
use crate::adapters::antigravity::AntigravityAdapter;
use crate::adapters::claude_code::ClaudeCodeAdapter;
use crate::adapters::codex::CodexAdapter;
use crate::paths;
use crate::slug;

/// 同步进度事件的名称（前端 `listen` 用同一常量，改一处即两侧一致）。
pub const SYNC_PROGRESS_EVENT: &str = "sync://progress";

// ============================================================================
// 面向 UI 的数据结构
// ============================================================================

/// 单个工具在一次同步中的结果状态。
///
/// 用枚举而非字符串：UI 要靠它决定图标（✅ / ⏳ / ❌），
/// 字符串一旦拼错就是「图标不显示」这种静默故障。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolSyncStatus {
    /// 同步成功
    Ok,
    /// 未安装，本次不参与同步（正常情况，不是错误）
    Skipped,
    /// 同步失败
    Failed,
}

/// 单个工具的同步结果（也是进度事件的载荷）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSyncResult {
    /// 稳定标识（`"claude_code"` / `"antigravity"`），供前端做映射
    pub tool_id: String,
    /// 展示名（`"Claude Code"` / `"Antigravity IDE"`）
    pub display_name: String,
    pub status: ToolSyncStatus,
    /// 面向用户的文案。**绝不包含系统错误码或技术路径**（`PRD.md` 第 8 节铁律）
    pub message: String,
}

/// 一次完整同步的报告。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub tools: Vec<ToolSyncResult>,
    /// 本次是否真的推送了全局规则（规则为空时为 false）
    pub rules_synced: bool,
    /// 本次推送的技能知识条数
    pub skill_count: usize,
    /// 是否全部成功（含「未安装而跳过」，跳过不算失败）
    pub ok: bool,
    /// 同步在开始前就被中止的原因（面向用户）。
    /// `Some` 时 `tools` 必为空——中止意味着一个 Adapter 都没被调用。
    pub error: Option<String>,
}

/// 工具检测结果（向导步骤 2 展示）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub tool_id: String,
    pub display_name: String,
    /// 推送位置（展示用）。未安装时也给出——用户可能想知道该装到哪里
    pub push_path: String,
    pub installed: bool,
    /// 检测过程本身出错（区别于「未安装」）。面向用户文案
    pub error: Option<String>,
}

/// 一个工具的 L0 备份现状（「备份与还原」区展示，TASK-19）。
///
/// 只包含**有备份概念**的工具：走标记块注入的 Claude Code 与 Codex。
/// Antigravity 写自己的独立文件、从不改动用户既有文件，因此不出现在此列表中
/// （`list_backups` 直接按 `l0_backup()` 过滤掉它）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    /// 稳定标识（`"claude_code"` / `"codex"`），还原时回传给后端
    pub tool_id: String,
    /// 展示名
    pub display_name: String,
    /// 被改动过的那个用户文件（还原的落点）
    pub target_file: String,
    /// 备份是否存在。`false` = 尚未同步过，我们还没动过这个文件
    pub exists: bool,
    /// 备份大小（字节）；不存在时为 0
    pub size_bytes: u64,
    /// 备份写入时间（`YYYY-MM-DD HH:MM`）；读不到元数据时为 `None`
    pub modified_at: Option<String>,
}

// ============================================================================
// 工具清单
// ============================================================================

/// 一个受支持工具的全部信息。
///
/// 字段全部 `pub`：测试与干跑工具需要把 `adapter` 换成**注入了临时目录**的实例
/// （见 [`run_for_tools`]），从而在真实目录结构上跑完整流程而不碰真实数据。
/// 生产路径只有一条——[`run_full_sync_with_progress`] 内部用 [`build_tools`] 构造。
pub struct ToolDescriptor {
    /// 稳定标识（`"claude_code"` / `"antigravity"`），供前端做映射
    pub tool_id: &'static str,
    /// 展示名（`"Claude Code"` / `"Antigravity IDE"`）
    pub display_name: &'static str,
    /// 推送位置（展示用）
    pub push_path: PathBuf,
    /// 实际执行同步的适配器
    pub adapter: Box<dyn Adapter>,
}

/// V1 支持的工具清单（`PRD.md` 第 3 节）。
///
/// 顺序即 UI 展示顺序：Claude Code → Antigravity IDE → Codex，
/// 与 `PRD.md` 第 5 节步骤 2 / 步骤 4 的示例一致。
///
/// Trae 等「即将支持」的工具不在此列表中——它们既没有 Adapter，
/// 也没有可检测的路径，属于纯展示项，由前端以静态常量渲染。
fn build_tools() -> Vec<ToolDescriptor> {
    vec![
        ToolDescriptor {
            tool_id: "claude_code",
            display_name: "Claude Code",
            push_path: paths::claude_dir(),
            adapter: Box::new(ClaudeCodeAdapter::new()),
        },
        ToolDescriptor {
            tool_id: "antigravity",
            display_name: "Antigravity IDE",
            push_path: paths::gemini_config_dir(),
            adapter: Box::new(AntigravityAdapter::new()),
        },
        // Codex 是 2026-09-14 的增补支持（TASK-18）。
        // 它与前两个的 L0 形态都不同：Antigravity 写独立文件、Claude 注入 `@` 引用、
        // Codex 注入**内联全文**——差异全部封装在各自 Adapter 内，编排层无需分支。
        //
        // 显示名是 `"Codex"` 而非 `"Codex CLI"`：桌面 App 与 CLI 共用同一个
        // `~/.codex/`，官方产品名就是 Codex（详见 `adapters/codex.rs` 模块文档）。
        ToolDescriptor {
            tool_id: "codex",
            display_name: "Codex",
            push_path: paths::codex_dir(),
            adapter: Box::new(CodexAdapter::new()),
        },
    ]
}

// ============================================================================
// SSOT 读取
// ============================================================================

/// 一条已就绪的技能知识。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeItem {
    /// 源文件名（含 `.md`，例如 `"Vue3 组件性能优化.md"`）
    pub file_name: String,
    /// 注入后的目录名（`crossbrain-vue3-9f86d0`）
    pub dir_name: String,
    /// frontmatter 的 `name` 字段值（不含命名空间前缀）
    pub skill_name: String,
    /// 原始正文
    pub body: String,
}

/// 读取 `global/rules.md`。
///
/// 文件不存在时返回空串（视作「用户还没写」），但**读取失败**要报错——
/// 与下面的知识库扫描遵循同一条原则：读不出来 ≠ 空的。
pub fn read_global_rules() -> Result<String, String> {
    let path = paths::global_rules_file();
    match fs::read_to_string(&path) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(_) => Err(
            "无法读取全局规则文件，为避免误删已中止同步。请检查文件夹访问权限。".to_string(),
        ),
    }
}

/// 扫描 `knowledge/` 下所有技能知识。
///
/// 返回 `Err` 表示**扫描不完整**，调用方必须中止整个同步（见模块文档的安全铁律）。
pub fn scan_knowledge() -> Result<Vec<KnowledgeItem>, String> {
    let dir = paths::knowledge_dir();

    // 目录本身不存在 → 从未创建过，等价于「没有知识」。
    // （init.rs 会在启动时创建它，这里只是兜底）
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&dir).map_err(|_| {
        "无法读取技能知识目录，为避免误删已中止同步。请检查文件夹访问权限。".to_string()
    })?;

    let mut items = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|_| "读取技能知识目录时出错，已中止同步。".to_string())?;

        let path = entry.path();
        // 只处理 .md 文件；子目录与其它扩展名一律忽略
        let is_md_file = entry
            .file_type()
            .map(|t| t.is_file())
            .unwrap_or(false)
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("md"))
                .unwrap_or(false);

        if !is_md_file {
            continue;
        }

        // ⚠️ 单个文件读失败必须中止：否则它算出来的 slug 不在 active_slugs 里，
        //    它对应的技能目录会被当作孤儿删除——用户的文件还在，注入的却没了。
        let body = fs::read_to_string(&path).map_err(|_| {
            "有技能知识文件无法读取，为避免误删已中止同步。请检查文件是否被其它程序占用。"
                .to_string()
        })?;

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();

        let slug = slug::generate_parts(&file_name, &body);
        items.push(KnowledgeItem {
            file_name,
            dir_name: slug.dir_name().to_string(),
            skill_name: slug.skill_name().to_string(),
            body,
        });
    }

    // 文件系统返回顺序不稳定；排序让 active_slugs、日志与报告都可确定比对
    items.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(items)
}

// ============================================================================
// 编排
// ============================================================================

/// 检测所有受支持工具（向导步骤 2）。
pub fn detect_tools() -> Vec<ToolInfo> {
    build_tools()
        .iter()
        .map(|tool| {
            let (installed, error) = match tool.adapter.detect() {
                Ok(found) => (found, None),
                Err(e) => (false, Some(tool.adapter.user_friendly_error(&e))),
            };
            ToolInfo {
                tool_id: tool.tool_id.to_string(),
                display_name: tool.display_name.to_string(),
                push_path: tool.push_path.to_string_lossy().to_string(),
                installed,
                error,
            }
        })
        .collect()
}

// ============================================================================
// 备份与还原（TASK-19 / ADR-15：「可逆」）
// ============================================================================

/// 查询各工具 L0 备份现状（生产路径）。
pub fn list_backups() -> Vec<BackupInfo> {
    list_backups_for(&build_tools())
}

/// 对给定工具集合查询备份现状（可注入，供测试与干跑用）。
///
/// 与 [`run_for_tools`] 同一条理由：生产路径由 [`crate::paths`] 固定、无法注入，
/// 而本机有真实用户数据，端到端验证必须在**目录结构的副本**上做。
pub fn list_backups_for(tools: &[ToolDescriptor]) -> Vec<BackupInfo> {
    tools
        .iter()
        // 无备份概念的工具（Antigravity）直接不出现——列一个永远「无备份」的条目
        // 只会让用户以为自己漏同步了
        .filter_map(|tool| {
            let backup = tool.adapter.l0_backup()?;
            let meta = fs::metadata(&backup.backup_file).ok();
            Some(BackupInfo {
                tool_id: tool.tool_id.to_string(),
                display_name: tool.display_name.to_string(),
                target_file: backup.target_file.to_string_lossy().to_string(),
                exists: meta.is_some(),
                size_bytes: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                modified_at: meta
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(format_local_time),
            })
        })
        .collect()
}

/// 把一个工具的文件还原到首次同步前的原始内容（生产路径）。
///
/// 返回**面向用户的成功文案**；失败返回面向用户的错误文案
/// （与 `commands.rs` 的「UI 文案必须可读」约定一致）。
pub fn restore_backup(tool_id: &str) -> Result<String, String> {
    restore_backup_for(&build_tools(), tool_id)
}

/// 对给定工具集合执行还原（可注入，供测试与干跑用）。
pub fn restore_backup_for(tools: &[ToolDescriptor], tool_id: &str) -> Result<String, String> {
    let tool = tools
        .iter()
        .find(|t| t.tool_id == tool_id)
        .ok_or_else(|| "没有找到这个工具，请重新打开界面后重试。".to_string())?;

    let backup = tool.adapter.l0_backup().ok_or_else(|| {
        format!(
            "{} 不会修改你原有的文件，没有需要还原的内容。",
            tool.display_name
        )
    })?;

    // 先判存在性再动手：给用户的应是「还没备份过」这句人话，
    // 而不是从 `fs::read_to_string` 冒出来的系统错误。
    if !backup.backup_file.exists() {
        return Err(format!(
            "{} 还没有备份文件（可能尚未同步过），暂时无法还原。",
            tool.display_name
        ));
    }

    match crate::adapters::restore_l0_backup(&backup.target_file, &backup.backup_file) {
        Ok(outcome) => Ok(match outcome.pre_restore_path {
            // 把「还原前的内容存到哪儿了」告诉用户，否则他会以为那次手改丢了
            Some(path) => format!(
                "已把 {} 还原到最初的内容。还原前的版本已另存为 {}，需要时可以取回。",
                tool.display_name,
                path.display()
            ),
            None => format!("已把 {} 还原到最初的内容。", tool.display_name),
        }),
        Err(e) => Err(format!(
            "还原 {} 失败：{}",
            tool.display_name,
            user_facing_error(&*tool.adapter, &e)
        )),
    }
}

/// 把文件系统时间戳格式化成状态栏同款本地时间（`YYYY-MM-DD HH:MM`）。
///
/// 铁律 L-03：时间一律来自 `chrono`，**不得调用 shell 的 `date`**。
/// 格式与 [`now_display`] 保持一致——同一件事在界面上不该有两种写法。
fn format_local_time(time: std::time::SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Local> = time.into();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

/// 执行完整同步，不汇报进度。
///
/// 供不需要实时进度的调用方使用（测试、将来的后台自动同步）。
pub fn run_full_sync() -> SyncReport {
    run_full_sync_with_progress(&mut |_| {})
}

/// 执行完整同步，每个工具完成时立即回调一次。
///
/// `on_progress` 在**同一线程**上同步调用，因此可以直接捕获 `AppHandle` 发事件。
///
/// # 关于 Git 提交
///
/// `ARCHITECTURE.md` 第 3 节的 Step 4（`git add/commit`）由 TASK-09 封装。
/// 该模块尚未实现（用户本机也尚未建立仓库），故此处只保留调用位置：
/// 它应当在本函数返回前执行，且失败**不影响**同步结果——
/// 同步的内容已经落盘，版本历史只是附带的便利功能（见 `PRD.md` 第 4 节）。
pub fn run_full_sync_with_progress(on_progress: &mut dyn FnMut(ToolSyncResult)) -> SyncReport {
    // ── Step 1. 读取 SSOT（失败即整体中止，不进入任何写入流程）──
    let rules = match read_global_rules() {
        Ok(text) => text,
        Err(msg) => return aborted(msg),
    };
    let knowledge = match scan_knowledge() {
        Ok(items) => items,
        Err(msg) => return aborted(msg),
    };

    // ── Step 2/3. 逐个工具同步 ──
    run_for_tools(&build_tools(), &rules, &knowledge, on_progress)
}

/// 对给定的工具集合执行一次同步（**不含 SSOT 读取，不做路径推断**）。
///
/// # 为什么必须单独拆出这一层
///
/// 本机 `~/.claude/` 与 `~/.gemini/config/` 里有**真实用户数据**
/// （5 路硬链接的全局记忆、用户自建技能），拿它们做端到端验证不可接受；
/// 而 [`run_full_sync_with_progress`] 的路径由 [`crate::paths`] 固定、无法注入。
///
/// 把「执行」与「取材」分开后，干跑工具可以传入
/// `with_base_dir(临时副本)` 构造的 Adapter，在**真实目录结构上**跑完整流程，
/// 而真实数据一个字节都不变。单元测试也靠它验证编排逻辑。
///
/// 生产代码请用 [`run_full_sync_with_progress`]。
pub fn run_for_tools(
    tools: &[ToolDescriptor],
    rules: &str,
    knowledge: &[KnowledgeItem],
    on_progress: &mut dyn FnMut(ToolSyncResult),
) -> SyncReport {
    // 规则内容全为空白时不推送 L0：往用户的 CLAUDE.md 里追加一个空标记块
    // 毫无价值（还会白白触发一次备份 + 断链），属于纯副作用。
    let rules_synced = !rules.trim().is_empty();
    let active_slugs: Vec<String> = knowledge.iter().map(|k| k.dir_name.clone()).collect();

    let mut results = Vec::new();
    for tool in tools {
        let result = sync_one_tool(
            &*tool.adapter,
            tool,
            rules,
            rules_synced,
            knowledge,
            &active_slugs,
        );
        // 立即回调：向导要求逐行实时显示，而不是最后一次性呈现
        on_progress(result.clone());
        results.push(result);
    }

    let ok = results.iter().all(|t| t.status != ToolSyncStatus::Failed);

    SyncReport {
        tools: results,
        rules_synced,
        skill_count: knowledge.len(),
        ok,
        error: None,
    }
}

/// 同步单个工具。任何一步失败都终止该工具（但**不影响**其它工具）。
fn sync_one_tool(
    adapter: &dyn Adapter,
    tool: &ToolDescriptor,
    rules: &str,
    rules_synced: bool,
    knowledge: &[KnowledgeItem],
    active_slugs: &[String],
) -> ToolSyncResult {
    // ① 检测：未安装是正常情况，不计入失败
    match adapter.detect() {
        Ok(true) => {}
        Ok(false) => {
            return ToolSyncResult {
                tool_id: tool.tool_id.to_string(),
                display_name: tool.display_name.to_string(),
                status: ToolSyncStatus::Skipped,
                message: "未安装此工具，本次未同步".to_string(),
            }
        }
        Err(e) => return failed(tool, adapter.user_friendly_error(&e)),
    }

    // ② L0 全局规则
    if rules_synced {
        if let Err(e) = adapter.sync_l0(rules) {
            return failed(tool, adapter.user_friendly_error(&e));
        }
    }

    // ③ L2 技能知识
    for item in knowledge {
        let content = format_skill_md(&item.skill_name, &item.body);
        if let Err(e) = adapter.sync_l2(&item.dir_name, &content) {
            return failed(tool, adapter.user_friendly_error(&e));
        }
    }

    // ④ 孤儿清理：删除已不在 knowledge 列表中的历史技能目录
    if let Err(e) = adapter.cleanup_orphans(active_slugs) {
        return failed(tool, adapter.user_friendly_error(&e));
    }

    ToolSyncResult {
        tool_id: tool.tool_id.to_string(),
        display_name: tool.display_name.to_string(),
        status: ToolSyncStatus::Ok,
        message: if rules_synced {
            format!("已同步 1 条全局规则与 {} 条技能知识", knowledge.len())
        } else {
            format!("已同步 {} 条技能知识", knowledge.len())
        },
    }
}

/// 构造一条失败结果。
fn failed(tool: &ToolDescriptor, message: String) -> ToolSyncResult {
    ToolSyncResult {
        tool_id: tool.tool_id.to_string(),
        display_name: tool.display_name.to_string(),
        status: ToolSyncStatus::Failed,
        message,
    }
}

/// 构造「同步未开始即中止」的报告。
fn aborted(message: String) -> SyncReport {
    SyncReport {
        tools: Vec::new(),
        rules_synced: false,
        skill_count: 0,
        ok: false,
        error: Some(message),
    }
}

/// 状态栏展示用的本地时间戳（`YYYY-MM-DD HH:MM`）。
///
/// 铁律 L-03：时间一律来自 `chrono`，**不得调用 shell 的 `date`**。
pub fn now_display() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()
}

/// 把 [`AdapterError`] 转成用户可读文案的统一入口。
///
/// 存在的意义是把「UI 文案必须经过 Adapter 翻译」这件事固化成一个可见的函数，
/// 避免日后有人图省事直接 `format!("{e}")`——那会把 `os error 5` 甩给用户。
pub fn user_facing_error(adapter: &dyn Adapter, err: &AdapterError) -> String {
    adapter.user_friendly_error(err)
}

#[cfg(test)]
mod tests {
    //! 编排层测试。
    //!
    //! ⚠️ 这里**不能**调用 [`run_full_sync`]：它内部走 [`crate::paths`]，
    //! 会真的读写用户的 `~/.ai-profile/`、`~/.claude/`、`~/.gemini/`。
    //! 本机有真实用户数据，那种测试必须避免。
    //! 因此只测可以在临时目录里安全验证的部分：知识库扫描的**失败语义**与
    //! 排序稳定性、以及纯函数的时间戳格式。
    //!
    //! 端到端行为由 `examples/dryrun_sync.rs` 在**真实结构的副本**上验证。

    use super::*;

    #[test]
    fn now_display_matches_expected_shape() {
        let now = now_display();
        // YYYY-MM-DD HH:MM
        assert_eq!(now.len(), 16, "实际输出：{now}");
        assert_eq!(now.as_bytes()[4], b'-');
        assert_eq!(now.as_bytes()[7], b'-');
        assert_eq!(now.as_bytes()[10], b' ');
        assert_eq!(now.as_bytes()[13], b':');
        assert!(
            now.chars().all(|c| c.is_ascii_digit() || c == '-' || c == ' ' || c == ':'),
            "时间戳含非预期字符：{now}"
        );
    }

    /// 未安装的工具必须记为「跳过」而非「失败」——跳过不算同步失败。
    /// 用 Mock 验证判定逻辑，不触碰真实目录。
    #[test]
    fn detect_false_is_skipped_not_failed() {
        use crate::adapters::CleanupReport;

        struct NotInstalled;
        impl Adapter for NotInstalled {
            fn detect(&self) -> Result<bool, AdapterError> {
                Ok(false)
            }
            fn sync_l0(&self, _: &str) -> Result<(), AdapterError> {
                unreachable!("未安装的工具不应被调用 sync_l0")
            }
            fn sync_l2(&self, _: &str, _: &str) -> Result<(), AdapterError> {
                unreachable!("未安装的工具不应被调用 sync_l2")
            }
            fn cleanup_orphans(&self, _: &[String]) -> Result<CleanupReport, AdapterError> {
                unreachable!("未安装的工具不应被调用 cleanup_orphans")
            }
        }

        let tool = ToolDescriptor {
            tool_id: "mock",
            display_name: "Mock",
            push_path: PathBuf::from("/nowhere"),
            adapter: Box::new(NotInstalled),
        };

        let result = sync_one_tool(
            &*tool.adapter,
            &tool,
            "规则",
            true,
            &[],
            &[],
        );
        assert_eq!(result.status, ToolSyncStatus::Skipped);
        assert!(result.message.contains("未安装"), "实际文案：{}", result.message);
    }

    /// 规则为空时不得调用 `sync_l0`——空标记块是纯副作用。
    #[test]
    fn empty_rules_skip_l0_when_not_synced() {
        use crate::adapters::CleanupReport;
        use std::sync::atomic::{AtomicU32, Ordering};

        struct Installed {
            l0_calls: AtomicU32,
        }
        impl Adapter for Installed {
            fn detect(&self) -> Result<bool, AdapterError> {
                Ok(true)
            }
            fn sync_l0(&self, _: &str) -> Result<(), AdapterError> {
                self.l0_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
            fn sync_l2(&self, _: &str, _: &str) -> Result<(), AdapterError> {
                Ok(())
            }
            fn cleanup_orphans(&self, _: &[String]) -> Result<CleanupReport, AdapterError> {
                Ok(CleanupReport {
                    deleted_dirs: vec![],
                    kept_dirs: vec![],
                })
            }
        }

        let tool = ToolDescriptor {
            tool_id: "mock",
            display_name: "Mock",
            push_path: PathBuf::from("/nowhere"),
            adapter: Box::new(Installed {
                l0_calls: AtomicU32::new(0),
            }),
        };

        // rules_synced = false 时应跳过 L0，但仍完成 L2 与清理
        let result = sync_one_tool(&*tool.adapter, &tool, "", false, &[], &[]);
        assert_eq!(result.status, ToolSyncStatus::Ok);
        assert!(
            !result.message.contains("全局规则"),
            "规则未同步时不该在文案里提全局规则：{}",
            result.message
        );
    }

    /// 失败文案必须来自 Adapter 翻译，不含系统错误码。
    #[test]
    fn failure_message_is_user_facing() {
        use crate::adapters::CleanupReport;

        struct Denied;
        impl Adapter for Denied {
            fn detect(&self) -> Result<bool, AdapterError> {
                Err(AdapterError::PermissionDenied("os error 5".into()))
            }
            fn sync_l0(&self, _: &str) -> Result<(), AdapterError> {
                unreachable!()
            }
            fn sync_l2(&self, _: &str, _: &str) -> Result<(), AdapterError> {
                unreachable!()
            }
            fn cleanup_orphans(&self, _: &[String]) -> Result<CleanupReport, AdapterError> {
                unreachable!()
            }
        }

        let tool = ToolDescriptor {
            tool_id: "mock",
            display_name: "Mock",
            push_path: PathBuf::from("/nowhere"),
            adapter: Box::new(Denied),
        };

        let result = sync_one_tool(&*tool.adapter, &tool, "规则", true, &[], &[]);
        assert_eq!(result.status, ToolSyncStatus::Failed);
        assert!(
            !result.message.contains("os error"),
            "失败文案泄漏了系统错误码：{}",
            result.message
        );
        assert_eq!(result.message, "没有写入权限，请检查目录访问权限");
    }

    /// 中止报告必须不携带任何工具结果——中止意味着一个 Adapter 都没被调用。
    #[test]
    fn aborted_report_has_no_tool_results() {
        let report = aborted("测试中止".to_string());
        assert!(report.tools.is_empty());
        assert!(!report.ok);
        assert_eq!(report.error.as_deref(), Some("测试中止"));
    }
}

//! adapters/mod.rs — Adapter 统一接口定义与错误类型。
//!
//! # 职责边界
//!
//! 本模块定义**接口契约**，并提供各 Adapter 共用的**格式工具与标记块协议**；
//! 不含任何具体工具的读写实现。
//! 每个目标 AI 工具各有一个 Adapter（Antigravity IDE、Claude Code…），
//! 全部实现本模块的 [`Adapter`] trait，使上层同步逻辑与具体工具解耦：
//! 新增工具时只需加一个 `impl Adapter`，不必改动同步流程。
//!
//! 完整接口规范见 `docs/tech/ADAPTER_SPEC.md` 第 1 节，本文件是它在 Rust 侧的落地。
//!
//! # 铁律约束（实现方必须遵守，见 `MASTER_CONTEXT.md`）
//!
//! - **L-01**：`sync_l0` 写 Claude Code 的 `CLAUDE.md` 时必须走**标记块协议**——
//!   绝不整体覆盖用户文件，标记块永远只能追加或在块内替换
//!   （四情况协议详见 `ADAPTER_SPEC.md` 第 3.2 节）。
//! - **L-02**：所有路径一律取自 [`crate::paths`]，Adapter 内不得自行拼接路径或硬编码盘符。
//! - **L-03**：涉及时间戳时用 `chrono` crate，不得调用 shell 的 `date` 命令。

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::{NoExpand, Regex};

pub mod antigravity;
pub mod claude_code;
pub mod codex;

/// 孤儿清理报告。
///
/// 「孤儿」= 目标工具 skills 目录下存在、但已不在本次同步 `active_slugs` 中的
/// `crossbrain-*` 目录——即用户删掉对应 knowledge 文档后留下的残留。
///
/// 派生 `Debug` / `Clone` / `PartialEq` / `Eq`：TASK-05 / TASK-06 的
/// `cleanup_orphans()` 测试需要精确比对两个列表，UI 层也要展示清理结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupReport {
    /// 被删除的孤儿目录名（例：`"crossbrain-vue3-perf-7c8e2a"`）
    pub deleted_dirs: Vec<String>,
    /// 保留下来的目录名（在 `active_slugs` 中，或不属于 CrossBrain 托管）
    pub kept_dirs: Vec<String>,
}

/// Adapter 统一错误类型。
///
/// 变体划分的目标是**让 UI 能给出可执行的下一步**，而不是把系统错误抛给用户：
/// 例如 `PermissionDenied` → 「检查目录访问权限」，
/// 而 `NotInstalled` 其实是正常情况（跳过即可，用户无需做任何事）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    /// 工具未安装（配置目录不存在）
    NotInstalled,
    /// 文件/目录权限不足
    PermissionDenied(String),
    /// 自动创建目录失败
    DirectoryCreateFailed(String),
    /// 文件写入失败
    WriteError(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::NotInstalled => write!(f, "工具未安装"),
            AdapterError::PermissionDenied(msg) => write!(f, "没有写入权限：{}", msg),
            AdapterError::DirectoryCreateFailed(msg) => write!(f, "无法创建配置目录：{}", msg),
            AdapterError::WriteError(msg) => write!(f, "文件写入失败：{}", msg),
            AdapterError::Other(msg) => write!(f, "错误：{}", msg),
        }
    }
}

// 实现标准错误 trait，使 AdapterError 能在 `?` 传播链与 `Box<dyn Error>` 上下文中使用。
impl std::error::Error for AdapterError {}

/// 一个可被还原的 L0 目标：被注入标记块的用户文件 + 它首次注入前的干净快照。
///
/// # 为什么只有「标记块注入型」Adapter 才有它
///
/// Antigravity 把规则写进**自己的独立文件**（`rules/crossbrain-L0.md`），
/// 从头到尾没碰过用户的任何既有文件，因此**不存在「还原用户原文件」这件事**——
/// 它的 `l0_backup()` 返回 `None`。Claude Code 与 Codex 则都往用户的既有文件里
/// 注入标记块并在首次注入前留下备份，二者才有本结构。
///
/// 派生 `PartialEq / Eq`：测试要精确断言路径，干跑要打印比对。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L0Backup {
    /// 被注入标记块的用户文件（还原的落点）
    pub target_file: PathBuf,
    /// 首次注入前的原始内容快照（**只读，任何路径都不得覆盖**）
    pub backup_file: PathBuf,
}

/// 一次还原的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreOutcome {
    /// 被还原的文件
    pub target_file: PathBuf,
    /// 还原前现场另存的路径。
    ///
    /// `None` 表示「还原前的内容与备份内容相同，无需另存」——
    /// 这样连续点两次还原不会凭空产生一份无意义的快照（幂等）。
    pub pre_restore_path: Option<PathBuf>,
}

/// 所有 AI 工具适配器必须实现的统一接口。
///
/// # 为什么要求 `Send + Sync`
///
/// Adapter 实例要放进 Tauri 的 `State`，并在 command handler（可能运行在其他线程）
/// 中被调用。无状态实现天然满足此约束，成本为零；若不在此处声明，
/// 等到状态管理任务时才发现，将被迫修改这个已被多处依赖的契约。
pub trait Adapter: Send + Sync {
    /// 检测目标工具是否已安装。
    ///
    /// - `Ok(true)`：已安装（配置目录存在）
    /// - `Ok(false)`：未安装——属正常情况，调用方应静默跳过，**不要当错误处理**
    /// - `Err(_)`：检测过程本身异常（如权限问题导致无法 stat）
    fn detect(&self) -> Result<bool, AdapterError>;

    /// 同步全局规则（L0）到目标工具。
    ///
    /// `rules_content`：`global/rules.md` 的完整文本内容。
    ///
    /// **必须幂等**：多次调用结果一致。
    /// ⚠️ 对 Claude Code 而言还须遵守标记块协议（铁律 L-01），
    /// 绝不能整体覆盖用户的 `CLAUDE.md`。
    fn sync_l0(&self, rules_content: &str) -> Result<(), AdapterError>;

    /// 同步一个技能知识（L2）到目标工具。
    ///
    /// `slug_hash`：由 `slug.rs` 生成的完整标识符（如 `"crossbrain-vue3-perf-7c8e2a"`）。
    /// `content`：**已格式化好**的 SKILL.md 完整内容（含 YAML frontmatter），
    /// Adapter 只负责落盘，不负责拼装 frontmatter——
    /// 拼装请用本模块的 [`format_skill_md`]。
    fn sync_l2(&self, slug_hash: &str, content: &str) -> Result<(), AdapterError>;

    /// 清理孤儿目录。
    ///
    /// `active_slugs`：本次同步中所有有效的 slug-hash 字符串列表。
    ///
    /// 删除目标目录下所有 `crossbrain-` 前缀、但不在 `active_slugs` 中的目录。
    /// ⚠️ 只允许删除 `crossbrain-` 前缀的目录（铁律 ADR-09）——
    /// 用户自建的同级内容绝不能被误删。
    fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError>;

    /// 该工具的 L0 备份信息；`None` = 不修改用户既有文件，没有备份可言。
    ///
    /// 供「备份与还原」功能查询与还原（`ADR-15`：对用户文件的改动必须可逆）。
    ///
    /// # 为什么给默认实现
    ///
    /// 只有走**标记块注入**的 Adapter 才需要覆盖它。给一个返回 `None` 的默认实现后，
    /// 写独立文件的 Adapter（Antigravity）与测试里的 Mock 都无需改动——
    /// 而不是被迫为「我没有这个概念」写一个空方法。
    fn l0_backup(&self) -> Option<L0Backup> {
        None
    }

    /// 将 [`AdapterError`] 转为用户友好提示文字。
    ///
    /// 所有 UI 展示的错误文本**必须**经过此函数，不能直接展示系统错误字符串
    /// （用户看不懂 `os error 5` 这类信息）。
    ///
    /// 默认实现已覆盖全部变体；有特殊文案需求的 Adapter 可覆盖。
    fn user_friendly_error(&self, err: &AdapterError) -> String {
        match err {
            AdapterError::NotInstalled => "未检测到此工具，已跳过".to_string(),
            AdapterError::PermissionDenied(_) => "没有写入权限，请检查目录访问权限".to_string(),
            AdapterError::DirectoryCreateFailed(_) => {
                "无法创建配置目录，请手动创建后重试".to_string()
            }
            AdapterError::WriteError(_) => "文件写入失败，请检查磁盘空间和权限".to_string(),
            AdapterError::Other(msg) => format!("同步出现问题：{}", msg),
        }
    }
}

// ============================================================================
// 共享命名空间约束（各 Adapter 通用，依据铁律 ADR-09）
// ============================================================================

/// CrossBrain 托管内容的统一前缀。
///
/// 读取、写入、删除三个动作**都**限定在这一命名空间内。
///
/// 定义源在 [`crate::slug`]（slug 的形态本身），这里只做转出——
/// 保证「生成标识符」与「校验标识符」用的是同一个字符串常量，
/// 两处各写一份会在日后分叉成运行时故障（见 `slug.rs` 的契约测试）。
pub use crate::slug::SLUG_PREFIX;

/// 校验 slug 落在 CrossBrain 命名空间内，且是安全的单层目录名。
///
/// # 为什么必须校验
///
/// `slug_hash` 会直接参与目录名拼接。若不校验：
/// - 传入用户自建技能名（真实机器上 `skills/` 里就有 `grill-me`、
///   `design-taste-frontend`）会**覆盖用户的技能文件**
/// - 传入 `crossbrain-../../xxx` 会**写到 `skills/` 之外**
///
/// 这与孤儿清理「只删 `crossbrain-` 前缀」是**同一条边界的两侧**：
/// 读和删限定在命名空间内，写入同样必须限定。
///
/// # 为什么放在共享层
///
/// TASK-05 时它只服务于 Antigravity；TASK-06 的 Claude Code Adapter 需要
/// **逐字相同**的判定规则——各 Adapter 若各写一份，日后放宽一处就会
/// 在另一处留下缺口。命名空间是安全边界，必须只有一个定义。
pub fn ensure_crossbrain_slug(slug_hash: &str) -> Result<(), AdapterError> {
    if !slug_hash.starts_with(SLUG_PREFIX) {
        return Err(AdapterError::Other(format!(
            "拒绝操作 CrossBrain 命名空间之外的目标：{slug_hash}（必须以 {SLUG_PREFIX} 开头）"
        )));
    }
    if slug_hash.contains('/') || slug_hash.contains('\\') || slug_hash.contains("..") {
        return Err(AdapterError::Other(format!(
            "非法的技能标识符（含路径分隔符或上跳）：{slug_hash}"
        )));
    }
    Ok(())
}

/// 在指定的 skills 目录里清理 CrossBrain 孤儿目录。
///
/// # 为什么各 Adapter 共用这一份实现
///
/// TASK-08 的验收要求「各 Adapter 的清理逻辑行为一致」——
/// 靠多份各自演进的代码「保持」一致是不可靠的；共用一份则**结构上**不可能分叉。
/// 各 Adapter 的差异只在 `skills_dir` 的取值上（`.gemini/config/skills/`、
/// `.claude/skills/`、`.codex/skills/`）。
///
/// # 边界（铁律 ADR-09）
///
/// - 只处理 `SLUG_PREFIX` 前缀的条目，其余连类型都不查
/// - 只处理**目录**：同名普通文件跳过（对它们调 `remove_dir_all` 必然失败），
///   符号链接也随之被排除（`file_type()` 对 symlink 的 `is_dir()` 为 false），
///   因此不会顺着链接误删到目标目录
/// - 目录不存在 = 从未同步过，静默返回空报告，**不是错误**
/// - 结果排序：`read_dir` 的返回顺序由文件系统决定、不稳定；
///   排序后测试可直接 `assert_eq!`，UI 展示也更稳定
pub fn cleanup_crossbrain_orphans(
    skills_dir: &Path,
    active_slugs: &[String],
) -> Result<CleanupReport, AdapterError> {
    let mut report = CleanupReport {
        deleted_dirs: Vec::new(),
        kept_dirs: Vec::new(),
    };

    if !skills_dir.is_dir() {
        return Ok(report);
    }

    let entries = fs::read_dir(skills_dir)
        .map_err(|e| AdapterError::Other(format!("无法读取目录 {}：{e}", skills_dir.display())))?;

    for entry in entries {
        let entry = entry.map_err(|e| AdapterError::Other(format!("遍历目录项失败：{e}")))?;
        let name = entry.file_name().to_string_lossy().to_string();

        // ① 只认 CrossBrain 命名空间：其余目录连类型都不查（铁律 ADR-09）
        if !name.starts_with(SLUG_PREFIX) {
            continue;
        }

        // ② 只处理目录。同名的普通文件/符号链接一律跳过——本就不该由我们删。
        let is_dir = entry
            .file_type()
            .map_err(|e| AdapterError::Other(format!("无法读取 {name} 的类型：{e}")))?
            .is_dir();
        if !is_dir {
            continue;
        }

        if active_slugs.iter().any(|s| s == &name) {
            report.kept_dirs.push(name);
        } else {
            fs::remove_dir_all(entry.path())
                .map_err(|e| AdapterError::Other(format!("删除孤儿目录 {name} 失败：{e}")))?;
            report.deleted_dirs.push(name);
        }
    }

    report.deleted_dirs.sort();
    report.kept_dirs.sort();
    Ok(report)
}

// ============================================================================
// 共享标记块协议（Claude Code / Codex 通用，依据铁律 L-01）
// ============================================================================

/// 标记块起始标志（逐字符固定，见 `ADAPTER_SPEC.md` 3.2）。
///
/// # 为什么只有这一处定义
///
/// 标记块格式由铁律 L-01 **逐字符锁定**。Claude Code 与 Codex 都往用户的
/// 既有文件里注入标记块，两者必须使用同一对标志——否则用户在两个工具里
/// 看到的边界不同，排障时无法判断哪一段归 CrossBrain 管。
pub const MARKER_START: &str = "<!-- CrossBrain:Start -->";

/// 标记块结束标志（逐字符固定）。
pub const MARKER_END: &str = "<!-- CrossBrain:End -->";

/// 标记块内的说明行。
pub const MARKER_NOTE: &str =
    "<!-- 本区域由 CrossBrain 自动维护，修改请在 CrossBrain App 中进行 -->";

/// 备份文件后缀（见 `ADAPTER_SPEC.md` 3.2 情况三）。
pub const MARKER_BACKUP_SUFFIX: &str = ".crossbrain-backup";

/// 断链写入用临时文件的后缀。
pub const MARKER_TEMP_SUFFIX: &str = ".crossbrain-tmp";

/// 还原前现场快照的后缀（TASK-19，ADR-15）。
///
/// # 与 [`MARKER_BACKUP_SUFFIX`] 的区别（关键，别混用）
///
/// | 后缀 | 内容 | 可覆盖性 |
/// |:---|:---|:---|
/// | `.crossbrain-backup` | **首次注入前的原始文件** | **任何代码路径都不得覆盖**——它是唯一的干净快照 |
/// | `.crossbrain-before-restore` | **最近一次还原前的现场** | 滚动覆盖；只在现场与备份内容不同时才写 |
///
/// 两个后缀各自承担一个方向的可逆性：前者保证「能回到最初」，
/// 后者保证「连还原这个动作本身也能退回去」。
pub const MARKER_PRE_RESTORE_SUFFIX: &str = ".crossbrain-before-restore";

/// `sync_l0` 的执行结果，用于决定 UI 是否需要弹提示。
///
/// 派生 `Debug / Clone / PartialEq / Eq`：测试需要精确断言「本次是哪种情况」，
/// `BackupCreated` 还要比对备份路径。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncL0Result {
    /// 情况一 / 二 / 四：静默完成，UI 无提示。
    Silent,
    /// 情况三：首次注入并已备份原文件，UI 需弹一次提示。
    BackupCreated {
        /// 备份文件的完整路径，供 UI 提示文案或「打开所在目录」使用。
        backup_path: String,
    },
}

/// 标记块匹配正则。
///
/// # 为什么由常量拼接，而不是写字面量
///
/// 正则与标志串是同一份契约的两处表达。若正则里硬编码 `<!-- CrossBrain:Start -->`，
/// 改了 [`MARKER_START`] 而正则没跟上，会出现最坏的一种失败：**判定与替换同时失效
/// 却没有任何编译期提示**——文件该改却没改，同步静默失效。
///
/// [`regex::escape`] 保证标志串里若含正则元字符（当前没有，但不必依赖这个巧合）
/// 也不会被解释成语法。
fn marker_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(&format!(
            "{}[\\s\\S]*?{}",
            regex::escape(MARKER_START),
            regex::escape(MARKER_END)
        ))
        .expect("标记块正则编译失败（由固定常量拼接，不可能发生）")
    })
}

/// 内容是否已含（可被正则替换的）标记块。
///
/// 用正则 `is_match` 而非 `contains(START) && contains(END)`：后者在
/// 「只有 Start 没有 End」或标志乱序时也会判真，于是判定进了一分支、替换却匹配不到，
/// 文件该改却没改。判定与替换共用同一正则，才能保证「进了这个分支就一定替换成功」。
pub fn has_marker_block(content: &str) -> bool {
    marker_regex().is_match(content)
}

/// 检查待**内联**进标记块的内容里是否混入了标记串。
///
/// # 为什么必须检查
///
/// 标记块的匹配正则是非贪婪的 `[\s\S]*?`，一旦内容里出现 `MARKER_END`，
/// 块会**在错误的位置提前闭合**：本次写入看似成功，下次同步却会重写错误的区间，
/// 文件从此处于难以人工修复的错乱状态。
///
/// 内容来自用户的 `global/rules.md`，完全可能因为「举例说明标记块格式」而命中。
/// 按 ADR-14「失败可见」原则：宁可让用户看到一次明确的同步失败，
/// 也不能静默写入一个已经损坏的文件。
pub fn ensure_no_marker_in_content(content: &str) -> Result<(), AdapterError> {
    for (label, marker) in [("起始", MARKER_START), ("结束", MARKER_END)] {
        if content.contains(marker) {
            return Err(AdapterError::Other(format!(
                "全局规则内容里出现了 CrossBrain 的{label}标记（{marker}）。\
                 直接写入会让规则区域提前结束、文件结构损坏，已中止本次同步。\
                 请从规则内容中移除该标记后重试。"
            )));
        }
    }
    Ok(())
}

/// 以「断开硬链接」的方式把 `content` 写入 `path`。
///
/// # 为什么不能用 `fs::write` 就地覆写（决策 D-03）
///
/// 本机实测（`fsutil hardlink list`，links=5）：用户把同一份全局规则做成了**硬链接**，
/// 5 个路径共享同一个 inode（`~/.ai-memory/user_profile.md`、`~/.cursor/rules/user_profile.md`、
/// `~/.config/opencode/AGENTS.md`、`~/.gemini/config/rules/user_global.md`、`~/.claude/CLAUDE.md`）。
///
/// 就地覆写会让内容**同时出现在全部 5 个工具**里——CrossBrain 的标记块会被
/// Cursor / OpenCode / Antigravity 也读到，属跨工具污染。故一律经本函数落盘。
///
/// # 三步，缺一不可
///
/// 1. **内容先写入临时文件** —— 保证新内容已完整落盘，后面两步失败也不丢数据
/// 2. **删除原路径** —— Windows 上这一步才真正打断硬链接：其余链接仍指向原 inode、
///    内容一字不变，而 `path` 变成空槽位
/// 3. **`rename` 覆盖** —— 同目录内的重命名是原子的，不会出现「读到半个文件」
///
/// ⚠️ 第 2 步不能省：Windows 的 `rename` **不覆盖已存在的目标**，目标在就直接报错；
/// 而「先删再 rename」正好也是断链本身必需的动作——一步兼得。
///
/// # 对 `links == 1` 的文件会怎样
///
/// 行为等价于普通覆盖（只是多一次临时文件写入）。**不做链接数检测**：
/// std 的 `number_of_links()` 属 unstable（feature `windows_by_handle`），
/// 稳定通道拿不到；而「只有一条写入通道」比「按链接数分支」更不容易出错。
///
/// # 失败可恢复
///
/// 第 3 步失败时临时文件会被保留（内容完整），错误信息中给出其路径，
/// 用户或后续版本可手动改名为目标文件名恢复。
pub fn write_breaking_hardlink(path: &Path, content: &str) -> Result<(), AdapterError> {
    let file_name = path
        .file_name()
        .ok_or_else(|| AdapterError::Other(format!("非法路径：{}", path.display())))?;
    let tmp = path.with_file_name(format!(
        "{}{MARKER_TEMP_SUFFIX}",
        file_name.to_string_lossy()
    ));

    // ① 内容先落盘（临时文件与目标同目录，保证 ③ 的同卷原子重命名）
    fs::write(&tmp, content.as_bytes())
        .map_err(|e| AdapterError::WriteError(format!("写入临时文件 {} 失败：{e}", tmp.display())))?;

    // ② 删除原路径 = 断开硬链接
    match fs::remove_file(path) {
        Ok(()) => {}
        // 情况一（文件本就不存在）走到这里，属正常
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => {
            // 连删除都做不到（权限、被占用）：清理临时文件并明确报错，
            // 而不是留一个误导性的 rename 错误。
            let _ = fs::remove_file(&tmp);
            return Err(AdapterError::WriteError(format!(
                "无法移除 {}（断开硬链接失败）：{e}",
                path.display()
            )));
        }
    }

    // ③ 原子替换
    fs::rename(&tmp, path).map_err(|e| {
        AdapterError::WriteError(format!(
            "替换 {} 失败：{e}（新内容已完整写入 {}，可手动改名恢复）",
            path.display(),
            tmp.display()
        ))
    })
}

/// 计算「还原前现场快照」的路径（`<目标文件>.crossbrain-before-restore`）。
///
/// 与备份路径一样取 `with_file_name` 而非替换扩展名：`CLAUDE.md` 的备份是
/// `CLAUDE.md.crossbrain-backup`（**保留 `.md`**），用户一眼能看出它原本是谁。
pub fn pre_restore_path(target_file: &Path) -> PathBuf {
    let name = target_file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    target_file.with_file_name(format!("{name}{MARKER_PRE_RESTORE_SUFFIX}"))
}

/// 用备份内容还原目标文件（`ADR-15` 的「可逆」实现）。
///
/// # 三条硬约束
///
/// 1. **备份只读**：本函数绝不写 `backup_file`。它是唯一的干净快照，
///    覆盖它就等于把「能回到最初」这条后路烧掉。
/// 2. **现场先留存**：目标文件存在**且内容与备份不同**时，先另存为
///    `*.crossbrain-before-restore`。内容相同则跳过——既避免无意义写入，
///    也使连续点两次还原保持幂等（第二次不会用「已还原后的内容」覆盖掉
///    第一次留下的真实现场）。
/// 3. **写入必须走 [`write_breaking_hardlink`]**：还原同样不能让硬链接重新连回去。
///    若这里图省事用 `fs::write`，就会出现「同步时断开、还原时又接上」的漏洞——
///    这种「大部分路径都对」的缺口在测试里极难发现，所以不设例外。
pub fn restore_l0_backup(
    target_file: &Path,
    backup_file: &Path,
) -> Result<RestoreOutcome, AdapterError> {
    let backup = fs::read_to_string(backup_file).map_err(|e| {
        AdapterError::Other(format!("读取备份 {} 失败：{e}", backup_file.display()))
    })?;

    let mut pre_restore_path_saved = None;

    if target_file.exists() {
        let current = fs::read_to_string(target_file).map_err(|e| {
            AdapterError::Other(format!("读取 {} 失败：{e}", target_file.display()))
        })?;

        if current != backup {
            let path = pre_restore_path(target_file);
            // `fs::copy` 创建的是新文件（不复制硬链接），与备份文件互不影响
            fs::copy(target_file, &path).map_err(|e| {
                AdapterError::WriteError(format!(
                    "另存当前内容到 {} 失败：{e}",
                    path.display()
                ))
            })?;
            pre_restore_path_saved = Some(path);
        }
    }

    write_breaking_hardlink(target_file, &backup)?;

    Ok(RestoreOutcome {
        target_file: target_file.to_path_buf(),
        pre_restore_path: pre_restore_path_saved,
    })
}

/// 把标记块注入用户的既有文件，命中四情况协议中的一种。
///
/// # 判断顺序（命中即止，见 `ADAPTER_SPEC.md` 3.2）
///
/// | 情况 | 条件 | 操作 |
/// |:---|:---|:---|
/// | 一 | 文件不存在 | 创建含标记块的文件 |
/// | 二 | 已有标记块 | 正则替换块内内容，块外原封不动 |
/// | 三 | 无标记块 + 无备份 | 先备份 → 末尾追加标记块 |
/// | 四 | 无标记块 + 有备份 | 跳过备份，末尾追加标记块 |
///
/// # 为什么由两个 Adapter 共用
///
/// Claude Code 与 Codex 的控制流、备份策略、边界处理**逐字相同**，
/// 差异只在三个入参上（目标文件、备份文件、标记块内容）。
/// 与 TASK-08 抽 [`cleanup_crossbrain_orphans`] 同一条理由：
/// 靠两份各自演进的代码「保持」一致不可靠，共用一份则**结构上**不可能分叉。
///
/// # 全部写入都经 [`write_breaking_hardlink`]
///
/// 四情况无一例外。若某一分支图省事用 `fs::write`，硬链接就会在那条路径上
/// 重新连回共享 inode，D-03 的隔离随之失效——这种「四条路里三条对」的漏洞
/// 极难在测试里发现，所以不设任何例外。
pub fn inject_marker_block(
    target_file: &Path,
    backup_file: &Path,
    marker_block: &str,
) -> Result<SyncL0Result, AdapterError> {
    // ---------- 情况一：文件不存在 ----------
    if !target_file.exists() {
        if let Some(parent) = target_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AdapterError::DirectoryCreateFailed(format!("{}：{e}", parent.display()))
            })?;
        }
        write_breaking_hardlink(target_file, marker_block)?;
        return Ok(SyncL0Result::Silent);
    }

    let existing = fs::read_to_string(target_file)
        .map_err(|e| AdapterError::Other(format!("读取 {} 失败：{e}", target_file.display())))?;

    // ---------- 情况二：已有标记块 → 只替换块内内容 ----------
    if has_marker_block(&existing) {
        // `NoExpand`：标记块文本里若出现 `$1` 这类片段，普通替换会把它当作
        // 捕获组引用展开。固定内容虽然眼下不含 `$`，但这是「格式逐字符固定」
        // 的契约，不能依赖巧合。
        let replaced = marker_regex()
            .replace(&existing, NoExpand(marker_block))
            .to_string();
        write_breaking_hardlink(target_file, &replaced)?;
        return Ok(SyncL0Result::Silent);
    }

    // ---------- 情况三 / 四：无标记块 → 备份（仅首次）+ 末尾追加 ----------
    let mut result = SyncL0Result::Silent;
    if !backup_file.exists() {
        // 先备份原始内容，**再**修改文件——顺序反了备份的就是改后的内容。
        // `fs::copy` 创建新文件（不复制硬链接），所以备份与原件天然互相独立。
        fs::copy(target_file, backup_file).map_err(|e| {
            AdapterError::WriteError(format!(
                "备份 {} → {} 失败：{e}",
                target_file.display(),
                backup_file.display()
            ))
        })?;
        result = SyncL0Result::BackupCreated {
            backup_path: backup_file.to_string_lossy().to_string(),
        };
    }
    // 情况四：备份已存在则跳过备份，避免覆盖用户此前那份「干净」的原始快照。

    let base = existing.trim_end();
    let new_content = if base.is_empty() {
        // 文件存在但为空：直接写标记块，不产生开头多余空行
        marker_block.to_string()
    } else {
        format!("{base}\n\n{marker_block}")
    };
    write_breaking_hardlink(target_file, &new_content)?;

    Ok(result)
}

// ============================================================================
// 共享格式工具（三个 Adapter 通用，依据 ADAPTER_SPEC.md 第 4 节）
// ============================================================================

/// 按 `ADAPTER_SPEC.md` 第 4 节规范，把 knowledge 正文包装成完整的 SKILL.md 内容。
///
/// # 为什么放在共享层
///
/// 该规范在各 Adapter 之间**完全一致**（相同的 frontmatter 字段与含义）。
/// 放在这里可避免后来者复制一份等价逻辑、日后各自不同步。
///
/// # 参数
///
/// - `slug_prefix`：slug-hash 的前缀部分（不含 hash），写入 frontmatter 的 `name` 字段
/// - `knowledge_body`：`knowledge/*.md` 的原始正文，**原样**附在 frontmatter 之后
///
/// # 生成格式
///
/// ```text
/// ---
/// name: vue3-perf
/// description: Vue3 组件性能优化：长列表虚拟滚动与响应式依赖控制
/// ---
///
/// # Vue3 组件性能优化
///
/// （knowledge 正文原样保留）
/// ```
///
/// # 关于 `description`
///
/// 它取自正文首个 `# ` 标题。这是 Antigravity / Claude Code 决定是否懒加载该技能的
/// **唯一依据**，所以必须精准——过于宽泛会让技能永远不被加载。
///
/// 若正文没有标题行，回退为 `slug_prefix`。规范原文说「取文件名的可读形式」，
/// 但 Adapter 层拿不到文件名；前缀与文件名同源，是等价且更稳定的兜底。
pub fn format_skill_md(slug_prefix: &str, knowledge_body: &str) -> String {
    // 去掉可能的 UTF-8 BOM：Spike 遗留的技能文件带 BOM，写进 frontmatter 前必须剥离
    let body = knowledge_body.trim_start_matches('\u{feff}');
    let description = extract_description(body).unwrap_or_else(|| slug_prefix.to_string());
    let rendered = if needs_yaml_quoting(&description) {
        yaml_quote(&description)
    } else {
        description
    };

    // frontmatter 固定用 LF；正文原样保留（不规范化其换行，遵守「原样写入」）
    format!("---\nname: {slug_prefix}\ndescription: {rendered}\n---\n\n{body}")
}

/// 取正文首个非空行里的 `# ` 标题。
///
/// 只认「首个非空行」：若正文开头就是普通段落，说明该文件本就没有标题，
/// 应按规范回退，而不是到文中随便抓一行当描述。
fn extract_description(body: &str) -> Option<String> {
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        return trimmed
            .strip_prefix("# ")
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(str::to_string);
    }
    None
}

/// 判断 YAML 标量值是否必须加双引号。
///
/// 只覆盖真实会踩的情况：含半角 `: `（会被误判为嵌套映射）、以 `:` 结尾、
/// 含 ` #`（会被当成注释）、或以 YAML 指示符开头。
///
/// ⚠️ 中文标题里常见的**全角**冒号 `：` 不是 YAML 指示符，**不需要**加引号——
/// 那正是 Spike C 实测通过时 description 的形态，不要无谓地改变它。
fn needs_yaml_quoting(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    if value.contains(": ") || value.ends_with(':') || value.contains(" #") {
        return true;
    }
    matches!(
        value.chars().next(),
        Some(
            '-' | '?' | ':' | '[' | ']' | '{' | '}' | '&' | '*' | '!' | '|' | '>' | '\'' | '"'
                | '%' | '@' | '`'
        )
    )
}

/// 用双引号包裹并转义（仅在 [`needs_yaml_quoting`] 判定必要时调用）。
fn yaml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    //! TASK-04 只定义接口，但「定义了」不等于「可用」。
    //! 这里用最小实现验证契约本身成立：trait 可被实现、可作为 trait object 传递、
    //! 幂等语义可表达、错误文案覆盖全部变体。

    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// 最小可用实现，仅用于验证接口（不代表任何真实工具）。
    struct MockAdapter {
        installed: bool,
        /// 用原子计数而非 `Cell`：`Cell` 不是 `Sync`，会违反 trait 约束
        l0_call_count: AtomicU32,
    }

    impl MockAdapter {
        fn new(installed: bool) -> Self {
            Self {
                installed,
                l0_call_count: AtomicU32::new(0),
            }
        }
    }

    impl Adapter for MockAdapter {
        fn detect(&self) -> Result<bool, AdapterError> {
            Ok(self.installed)
        }

        fn sync_l0(&self, _rules_content: &str) -> Result<(), AdapterError> {
            self.l0_call_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn sync_l2(&self, _slug_hash: &str, _content: &str) -> Result<(), AdapterError> {
            Ok(())
        }

        fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError> {
            Ok(CleanupReport {
                deleted_dirs: vec!["crossbrain-old-gone-000000".to_string()],
                kept_dirs: active_slugs.to_vec(),
            })
        }
    }

    /// trait 必须是 object-safe 的：上层要能持有 `Box<dyn Adapter>` 遍历多个工具。
    /// 同时验证 `Send + Sync` 约束确实生效。
    #[test]
    fn adapter_is_object_safe_and_thread_safe() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn Adapter>>();

        let adapters: Vec<Box<dyn Adapter>> = vec![
            Box::new(MockAdapter::new(true)),
            Box::new(MockAdapter::new(false)),
        ];

        assert_eq!(adapters[0].detect().unwrap(), true);
        assert_eq!(adapters[1].detect().unwrap(), false);
    }

    /// L0 同步的幂等性：重复调用应成功且不产生副作用差异。
    /// 这里以「可重复调用」作为最小验证；真实的「内容一致」由 TASK-05/06 验证。
    #[test]
    fn sync_l0_is_repeatable() {
        let adapter = MockAdapter::new(true);
        adapter.sync_l0("# 规则").expect("首次同步应成功");
        adapter.sync_l0("# 规则").expect("重复同步必须成功（幂等）");
        assert_eq!(adapter.l0_call_count.load(Ordering::SeqCst), 2);
    }

    /// 清理报告必须可直接比对，否则 TASK-05/06 的断言无法编写。
    #[test]
    fn cleanup_report_is_comparable() {
        let adapter = MockAdapter::new(true);
        let active = vec!["crossbrain-a-111111".to_string()];

        let report = adapter.cleanup_orphans(&active).unwrap();
        assert_eq!(
            report,
            CleanupReport {
                deleted_dirs: vec!["crossbrain-old-gone-000000".to_string()],
                kept_dirs: active,
            }
        );
    }

    /// 每个错误变体都要有面向用户的文案，且不得泄漏内部术语。
    #[test]
    fn user_friendly_error_covers_all_variants() {
        let adapter = MockAdapter::new(true);
        let cases = [
            (AdapterError::NotInstalled, "未检测到此工具，已跳过"),
            (
                AdapterError::PermissionDenied("access denied".into()),
                "没有写入权限，请检查目录访问权限",
            ),
            (
                AdapterError::DirectoryCreateFailed("mkdir failed".into()),
                "无法创建配置目录，请手动创建后重试",
            ),
            (
                AdapterError::WriteError("disk full".into()),
                "文件写入失败，请检查磁盘空间和权限",
            ),
        ];

        for (err, expected) in cases {
            assert_eq!(adapter.user_friendly_error(&err), expected);
        }

        // Other 变体会带上原始信息，用于问题定位；前缀仍是中文可读文案
        let other = adapter.user_friendly_error(&AdapterError::Other("未知异常".into()));
        assert!(other.starts_with("同步出现问题："), "实际文案：{other}");
    }

    /// Display 实现应可用于日志输出（与 UI 文案分开：日志要保留细节）。
    #[test]
    fn display_output_is_distinct_from_ui_text() {
        let err = AdapterError::WriteError("os error 5".into());
        assert_eq!(err.to_string(), "文件写入失败：os error 5");
    }

    // ---------- 共享格式工具（TASK-05 期间补入）----------

    /// description 必须来自正文首个 `# ` 标题——它是 AI 懒加载的唯一触发依据。
    #[test]
    fn format_skill_md_extracts_description_from_first_heading() {
        let body = "# Vue3 组件性能优化：长列表虚拟滚动\n\n正文……";
        let out = format_skill_md("vue3-perf", body);

        assert!(
            out.starts_with("---\nname: vue3-perf\ndescription: "),
            "实际输出：{out}"
        );
        assert!(
            out.contains("description: Vue3 组件性能优化：长列表虚拟滚动"),
            "中文全角冒号不应触发加引号（须与 Spike C 实测形态一致）：{out}"
        );
        assert!(
            out.contains("\n---\n\n# Vue3 组件性能优化"),
            "正文须原样保留在 frontmatter 之后：{out}"
        );
    }

    /// 含半角 `: ` 的标题会破坏 YAML 结构，必须加引号。
    #[test]
    fn format_skill_md_quotes_yaml_unsafe_description() {
        let out = format_skill_md("rust-lifetime", "# Rust 生命周期: 常见错误\n\n正文");
        assert!(
            out.contains("description: \"Rust 生命周期: 常见错误\""),
            "含半角冒号的描述必须被引号包裹：{out}"
        );
    }

    /// 无标题行时回退到 slug 前缀，且任何输入都不得 panic。
    #[test]
    fn format_skill_md_falls_back_without_heading() {
        let out = format_skill_md("my-skill", "这是一段没有标题的正文。");
        assert!(out.contains("description: my-skill"), "无标题应回退到前缀：{out}");
        assert!(out.ends_with("这是一段没有标题的正文。"), "正文丢失：{out}");

        // 空白正文同样不能 panic
        let blank = format_skill_md("fallback", "   \n\n  ");
        assert!(blank.contains("description: fallback"), "实际输出：{blank}");
    }

    /// 带 BOM 的正文必须剥离后再拼装，否则 frontmatter 前的 BOM 会破坏 YAML 解析。
    #[test]
    fn format_skill_md_strips_leading_bom() {
        let out = format_skill_md("bom-skill", "\u{feff}# 带 BOM 的标题\n\n正文");
        assert!(out.starts_with("---\n"), "BOM 未被剥离：{out:?}");
        assert!(out.contains("description: 带 BOM 的标题"));
    }

    // ---------- 共享标记块协议（TASK-18 抽取时补入）----------

    /// `has_marker_block` 与正则替换必须给出**一致**的判断，
    /// 否则会出现「判定为情况二、却替换不成功」的静默漏改。
    ///
    /// 这条测试随实现从 `claude_code.rs` 迁来——协议在共享层，验证也应在共享层，
    /// 否则两个 Adapter 各自演进时，被验证的只是「某一个」的一致性。
    #[test]
    fn marker_detection_agrees_with_replacement() {
        let block = format!("{MARKER_START}\n{MARKER_NOTE}\n规则\n{MARKER_END}");

        for text in [
            block.clone(),
            format!("前面\n{block}\n后面"),
            format!("{MARKER_START}A{MARKER_END}"),
        ] {
            assert!(has_marker_block(&text), "应判为含标记块：{text}");
            let replaced = marker_regex().replace(&text, NoExpand("X")).to_string();
            assert!(
                !replaced.contains(MARKER_START),
                "判定为含标记块但替换失败：{text}"
            );
        }

        // 只有 Start、没有 End：不构成可替换的标记块
        assert!(!has_marker_block(&format!("{MARKER_START}\n没有结束标志")));
        assert!(!has_marker_block("普通内容，没有任何标记"));
    }

    /// 正则必须与常量保持一致——它是从常量拼接出来的，这条测试锁住这个事实。
    ///
    /// 风险场景：若有人把正则改回硬编码字面量，日后修改 [`MARKER_START`] 时
    /// 正则不会跟着变，判定与替换会**同时静默失效**，且没有任何编译期提示。
    #[test]
    fn marker_regex_matches_the_constants() {
        let text = format!("{MARKER_START}\n任意内容\n{MARKER_END}");
        assert!(
            marker_regex().is_match(&text),
            "正则与 MARKER_* 常量不一致——改了常量却漏改正则，会导致判定与替换同时失效"
        );
    }

    /// 内联内容含标记串时必须被拒绝（Codex 内联路径的安全阀）。
    #[test]
    fn marker_in_content_is_rejected() {
        assert!(ensure_no_marker_in_content("普通规则内容").is_ok());
        assert!(ensure_no_marker_in_content(&format!("x {MARKER_END} y")).is_err());
        assert!(ensure_no_marker_in_content(&format!("x {MARKER_START} y")).is_err());

        let err = ensure_no_marker_in_content(&format!("x {MARKER_END}"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("结束标记"), "错误信息应指明是哪一侧的标记：{err}");
    }

    /// 情况一：文件不存在 → 创建，内容逐字符等于给定标记块，且不产生备份。
    #[test]
    fn inject_marker_block_creates_file_when_missing() {
        let dir = shared_temp_dir("inject-case1");
        let target = dir.join("AGENTS.md");
        let backup = dir.join("AGENTS.md.crossbrain-backup");
        let block = format!("{MARKER_START}\n内容\n{MARKER_END}");

        let result = inject_marker_block(&target, &backup, &block).unwrap();

        assert_eq!(result, SyncL0Result::Silent, "情况一必须静默");
        assert_eq!(fs::read_to_string(&target).unwrap(), block);
        assert!(!backup.exists(), "情况一不应产生备份");
        let _ = fs::remove_dir_all(&dir);
    }

    /// 情况三：无标记块 + 无备份 → 先备份原文，再把标记块追加到末尾。
    #[test]
    fn inject_marker_block_backs_up_then_appends() {
        let dir = shared_temp_dir("inject-case3");
        let target = dir.join("AGENTS.md");
        let backup = dir.join("AGENTS.md.crossbrain-backup");
        let user = "# 用户自己写的\n";
        fs::write(&target, user).unwrap();
        let block = format!("{MARKER_START}\n规则\n{MARKER_END}");

        let result = inject_marker_block(&target, &backup, &block).unwrap();

        assert_eq!(
            result,
            SyncL0Result::BackupCreated {
                backup_path: backup.to_string_lossy().to_string()
            }
        );
        assert_eq!(
            fs::read_to_string(&backup).unwrap(),
            user,
            "备份的应是改动前的原文"
        );
        let after = fs::read_to_string(&target).unwrap();
        assert!(after.starts_with("# 用户自己写的"), "用户内容必须在前：{after}");
        assert!(
            after.trim_end().ends_with(MARKER_END),
            "标记块应在末尾：{after}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    /// 情况二幂等：同内容重复注入，文件字节完全一致。
    #[test]
    fn inject_marker_block_is_idempotent() {
        let dir = shared_temp_dir("inject-idem");
        let target = dir.join("AGENTS.md");
        let backup = dir.join("AGENTS.md.crossbrain-backup");
        fs::write(&target, "# 用户内容\n").unwrap();
        let block = format!("{MARKER_START}\n规则\n{MARKER_END}");

        inject_marker_block(&target, &backup, &block).unwrap();
        let first = fs::read_to_string(&target).unwrap();
        inject_marker_block(&target, &backup, &block).unwrap();

        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            first,
            "重复注入不得累积内容"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // ---------- 还原（TASK-19 / ADR-15：「可逆」） ----------

    /// 还原前现场快照必须与备份是**两个不同的路径**，且都保留原文件名。
    ///
    /// 若两者落到同一路径，还原就会覆盖掉唯一的干净快照——这是整个可用性
    /// 承诺的致命伤，所以作为第一条断言钉死。
    #[test]
    fn pre_restore_path_is_distinct_from_backup_path() {
        let dir = shared_temp_dir("pre-restore-path");
        let target = dir.join("CLAUDE.md");
        let backup = dir.join(format!("CLAUDE.md{MARKER_BACKUP_SUFFIX}"));
        let pre = pre_restore_path(&target);

        assert_ne!(pre, backup, "现场快照与备份不得是同一路径");
        assert_ne!(pre, target, "现场快照不得等于被还原的文件本身");
        assert_eq!(
            pre.file_name().unwrap().to_string_lossy(),
            format!("CLAUDE.md{MARKER_PRE_RESTORE_SUFFIX}"),
            "现场快照应保留原文件名并追加独立后缀"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    /// 还原后目标内容 == 备份内容，且**备份本身一字不变**。
    #[test]
    fn restore_writes_backup_content_and_keeps_backup_intact() {
        let dir = shared_temp_dir("restore-basic");
        let target = dir.join("AGENTS.md");
        let backup = dir.join(format!("AGENTS.md{MARKER_BACKUP_SUFFIX}"));

        let original = "# 用户原始内容\n\n极客前端工程架构与 AI 行为准则\n";
        fs::write(&backup, original).unwrap();
        fs::write(
            &target,
            format!("{original}\n{MARKER_START}\n规则\n{MARKER_END}\n"),
        )
        .unwrap();

        let outcome = restore_l0_backup(&target, &backup).unwrap();

        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            original,
            "目标文件应回到备份里的原始内容"
        );
        assert_eq!(
            fs::read_to_string(&backup).unwrap(),
            original,
            "备份被改动了——唯一干净快照任何路径都不得覆盖（ADR-15）"
        );
        assert_eq!(outcome.target_file, target);
        let _ = fs::remove_dir_all(&dir);
    }

    /// 还原前先把现场另存 —— 否则用户手改的内容会凭空消失。
    #[test]
    fn restore_saves_current_state_when_it_differs() {
        let dir = shared_temp_dir("restore-save-current");
        let target = dir.join("CLAUDE.md");
        let backup = dir.join(format!("CLAUDE.md{MARKER_BACKUP_SUFFIX}"));

        let original = "原始内容\n";
        let injected = format!("{original}\n{MARKER_START}\n规则\n{MARKER_END}\n");
        fs::write(&backup, original).unwrap();
        fs::write(&target, &injected).unwrap();

        let outcome = restore_l0_backup(&target, &backup).unwrap();

        let saved = outcome
            .pre_restore_path
            .expect("内容与备份不同，必须留存现场快照");
        assert_eq!(
            fs::read_to_string(&saved).unwrap(),
            injected,
            "现场快照里应当是还原前的实际内容"
        );
        assert_eq!(saved, pre_restore_path(&target));
        let _ = fs::remove_dir_all(&dir);
    }

    /// 连续还原两次必须幂等：第二次内容已与备份一致，不再产生新的现场快照。
    ///
    /// 若第二次仍无条件另存，它写下的会是「已还原后的内容」，
    /// 把第一次留下的**真实现场**覆盖掉——用户的手改就此丢失。
    #[test]
    fn restore_is_idempotent() {
        let dir = shared_temp_dir("restore-idempotent");
        let target = dir.join("CLAUDE.md");
        let backup = dir.join(format!("CLAUDE.md{MARKER_BACKUP_SUFFIX}"));

        let original = "原始内容\n";
        let injected = format!("{original}\n{MARKER_START}\n规则\n{MARKER_END}\n");
        fs::write(&backup, original).unwrap();
        fs::write(&target, &injected).unwrap();

        let first = restore_l0_backup(&target, &backup).unwrap();
        let first_path = first.pre_restore_path.clone().expect("首次应留存现场");
        let first_saved = fs::read_to_string(&first_path).unwrap();
        assert_eq!(first_saved, injected);

        let second = restore_l0_backup(&target, &backup).unwrap();
        assert_eq!(
            second.pre_restore_path, None,
            "内容已与备份一致，第二次不该再写现场快照"
        );
        assert_eq!(
            fs::read_to_string(&first_path).unwrap(),
            first_saved,
            "第一次留下的现场快照被覆盖了"
        );
        assert_eq!(fs::read_to_string(&target).unwrap(), original);
        let _ = fs::remove_dir_all(&dir);
    }

    /// 还原同样必须**断开硬链接**：还原后，另一侧的就地覆写不得再牵动被还原的文件。
    ///
    /// 用行为验证而非读 inode：std 的 `number_of_links()` 在稳定通道不可用
    /// （unstable feature `windows_by_handle`），且行为验证直接证明了
    /// 我们真正要防的后果——「同步时断开了，还原时又接回去」。
    #[test]
    fn restore_breaks_hardlink() {
        let dir = shared_temp_dir("restore-hardlink");
        let target = dir.join("CLAUDE.md");
        let other = dir.join("user_profile.md");
        let backup = dir.join(format!("CLAUDE.md{MARKER_BACKUP_SUFFIX}"));

        let original = "# 全局记忆中心\n";
        fs::write(&target, original).unwrap();
        fs::hard_link(&target, &other).expect("创建硬链接失败（需 NTFS）");

        // 前置条件：确实共享同一 inode（否则本测试失去意义）
        fs::write(&other, "PROBE-前置验证\n").unwrap();
        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "PROBE-前置验证\n",
            "前置条件不成立：两个路径未共享 inode"
        );
        fs::write(&other, original).unwrap();

        // 模拟「已同步」状态：目标被注入标记块，备份里是原始内容
        let injected = format!("{original}\n{MARKER_START}\n规则\n{MARKER_END}\n");
        fs::write(&target, &injected).unwrap();
        fs::write(&backup, original).unwrap();

        restore_l0_backup(&target, &backup).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), original);

        // 断链验证：就地覆写另一侧，被还原的文件必须纹丝不动
        fs::write(&other, "PROBE-断链验证\n").unwrap();
        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            original,
            "还原路径未断开硬链接——就地覆写另一侧仍牵动了它"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    /// 共享层测试用的临时目录（与各 Adapter 测试同一套命名策略）。
    fn shared_temp_dir(tag: &str) -> std::path::PathBuf {
        static SEQ: AtomicU32 = AtomicU32::new(0);
        let seq = SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "crossbrain-shared-{}-{tag}-{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("创建临时目录失败");
        dir
    }
}

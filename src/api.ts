/**
 * api.ts — 前端与 Rust 之间唯一的调用出口。
 *
 * # 为什么要收在一处
 *
 * 组件里直接写 `invoke("xxx")` 会让三件事散落到各处：
 * 命令名、参数名、返回类型。Rust 侧改个名字，故障只会在运行时出现，
 * 而 `vue-tsc` 完全看不见。集中在这里后，类型不匹配会在**构建期**报错。
 *
 * 类型定义与 `src-tauri/src/{commands,sync}.rs` 中的 `Serialize` 结构一一对应
 * （Rust 侧用 `#[serde(rename_all = "camelCase")]` 转成前端习惯的命名）。
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ============================================================================
// 类型（对应 Rust 侧的 DTO）
// ============================================================================

/** 应用启动状态。 */
export interface StartupState {
  /** 是否应展示首次运行向导 */
  firstRun: boolean;
  /** 上次同步时间（`YYYY-MM-DD HH:MM`），从未同步过为 null */
  lastSyncAt: string | null;
  /** 上次同步是否全部成功 */
  lastSyncOk: boolean;
  /**
   * 「同步会断开文件共享关系」的说明是否已展示过。
   *
   * 为 false 时，首次同步前必须先弹一次说明（见 {@link LinkNoticeDialog}）。
   */
  linkNoticeShown: boolean;
}

/** 单个受支持工具的检测结果。 */
export interface ToolInfo {
  toolId: string;
  displayName: string;
  /** 推送位置（展示用） */
  pushPath: string;
  installed: boolean;
  /** 检测过程本身出错时的用户可读原因（区别于「未安装」） */
  error: string | null;
}

export type ToolSyncStatus = "ok" | "skipped" | "failed";

/** 单个工具在一次同步中的结果（也是进度事件的载荷）。 */
export interface ToolSyncResult {
  toolId: string;
  displayName: string;
  status: ToolSyncStatus;
  /** 面向用户的文案，绝不包含系统错误码 */
  message: string;
}

/** 一次完整同步的报告。 */
export interface SyncReport {
  tools: ToolSyncResult[];
  rulesSynced: boolean;
  skillCount: number;
  ok: boolean;
  /** 同步开始前即中止的原因（有值时 tools 必为空） */
  error: string | null;
}

/**
 * 一个工具的备份现状（「设置 → 备份与还原」区展示）。
 *
 * 只有**会改动用户既有文件**的工具才出现在这个列表里：Claude Code 与 Codex。
 * 像 Antigravity 那样写自己独立文件的工具没有备份概念，后端会直接过滤掉。
 */
export interface BackupInfo {
  toolId: string;
  displayName: string;
  /** 被改动过的那个用户文件（还原的落点） */
  targetFile: string;
  /** 备份是否存在。false = 尚未同步过，我们还没动过这个文件 */
  exists: boolean;
  /** 备份大小（字节）；不存在时为 0 */
  sizeBytes: number;
  /** 备份写入时间（`YYYY-MM-DD HH:MM`）；读不到为 null */
  modifiedAt: string | null;
}

/**
 * 技能知识卡片（「技能知识」页的列表数据，TASK-12）。
 *
 * `fileName` 是读写/删除操作的寻址键；`dirName` 必须展示——
 * 纯中文标题的目录名会退化成 `crossbrain-item-{hash}`，
 * 只看目录名无法对应回源文件（ADR-11 补偿）。
 */
export interface KnowledgeCard {
  /** 源文件名（含 `.md`） */
  fileName: string;
  /** 文件名去掉 `.md` */
  title: string;
  /** 第一行非空内容（去掉标题记号） */
  summary: string;
  /** 注入后的技能目录名（如 `crossbrain-vue3-9f86d0`） */
  dirName: string;
  /** frontmatter 的技能名 */
  skillName: string;
  /** 字符数 */
  charCount: number;
  /** 文件大小（字节） */
  sizeBytes: number;
  /** 最后修改时间（`YYYY-MM-DD HH:MM`），读不到为 null */
  modifiedAt: string | null;
}

// ============================================================================
// 命令封装
// ============================================================================

/** 读取启动状态（是否显示向导、上次同步信息）。 */
export function getStartupState(): Promise<StartupState> {
  return invoke<StartupState>("get_startup_state");
}

/** 检测各工具是否已安装。 */
export function detectTools(): Promise<ToolInfo[]> {
  return invoke<ToolInfo[]>("detect_tools");
}

/** 读取全局规则正文。 */
export function readGlobalRules(): Promise<string> {
  return invoke<string>("read_global_rules");
}

/**
 * 保存全局规则正文。
 *
 * ⚠️ 保存**不会**触发同步——「编辑」与「同步」是两个独立动作。
 */
export function saveGlobalRules(content: string): Promise<void> {
  return invoke<void>("save_global_rules", { content });
}

/** 标记首次运行向导已完成（含用户主动跳过）。 */
export function completeWizard(): Promise<void> {
  return invoke<void>("complete_wizard");
}

/**
 * 查询各工具的备份现状。
 *
 * 用于「设置 → 备份与还原」区：告诉用户我们改过哪些文件、还能不能还原。
 */
export function listBackups(): Promise<BackupInfo[]> {
  return invoke<BackupInfo[]>("list_backups");
}

/**
 * 把一个工具的文件还原到首次同步前的原始内容。
 *
 * 返回面向用户的说明文案（含「还原前的内容另存到了哪里」）。
 * ⚠️ 这是用户显式发起的动作，**绝不自动触发**——程序不替用户推断「他大概想还原了」。
 */
export function restoreBackup(toolId: string): Promise<string> {
  return invoke<string>("restore_backup", { toolId });
}

/**
 * 标记「同步会断开文件共享关系」的说明已展示过（只提示一次）。
 *
 * ⚠️ 必须在用户**确认继续**之后调用，而不是在对话框弹出的那一刻——
 * 否则用户关掉窗口或点了取消，这条说明就永远不会再出现了。
 */
export function markLinkNoticeShown(): Promise<void> {
  return invoke<void>("mark_link_notice_shown");
}

// ── 一键卸载 / 去痕（TASK-14）──

/** 单个工具的卸载结果。 */
export interface UninstallOutcome {
  toolId: string;
  displayName: string;
  /** 实际执行的清理动作（面向用户的描述）。空列表 = 本来就没有痕迹。 */
  actions: string[];
}

/**
 * 一键卸载：清理所有工具侧的 CrossBrain 痕迹（标记块 / 独立规则文件 /
 * 技能目录 / 备份文件），并删除运行状态文件。
 *
 * ⚠️ `~/.ai-profile/` 下的全局规则与技能知识是用户数据，**不在清理范围**。
 * 前端必须先弹二次确认，用户确认后才允许调用。
 */
export function uninstallCrossbrain(): Promise<UninstallOutcome[]> {
  return invoke<UninstallOutcome[]>("uninstall_crossbrain");
}

/** 执行完整同步。进度通过 {@link onSyncProgress} 推送。 */
export function runSync(): Promise<SyncReport> {
  return invoke<SyncReport>("run_sync");
}

// ── 技能知识库 CRUD（TASK-12）──

/** 列出技能知识卡片。 */
export function listKnowledge(): Promise<KnowledgeCard[]> {
  return invoke<KnowledgeCard[]>("list_knowledge");
}

/** 读取一篇技能知识的正文。 */
export function readKnowledge(fileName: string): Promise<string> {
  return invoke<string>("read_knowledge", { fileName });
}

/**
 * 新建一篇技能知识（预填标题模板），返回新文件名。
 *
 * ⚠️ 新建**不触发同步**——写完并保存后，由用户点「立即同步」推送。
 */
export function createKnowledge(title: string): Promise<string> {
  return invoke<string>("create_knowledge", { title });
}

/** 保存一篇技能知识的正文。⚠️ 保存**不会**触发同步。 */
export function saveKnowledge(fileName: string, content: string): Promise<void> {
  return invoke<void>("save_knowledge", { fileName, content });
}

/** 删除一篇技能知识。返回面向用户的成功说明（含工具内副本的清理时机）。 */
export function deleteKnowledge(fileName: string): Promise<string> {
  return invoke<string>("delete_knowledge", { fileName });
}

/**
 * 订阅同步进度。
 *
 * 向导要求「逐行实时显示」而不是等全部完成后一次性呈现，
 * 所以进度来自事件而非 `runSync()` 的返回值。
 *
 * 返回值是取消订阅函数，组件卸载时必须调用（否则重复进出会累积监听器）。
 */
export function onSyncProgress(
  handler: (progress: ToolSyncResult) => void,
): Promise<UnlistenFn> {
  // 事件名与 Rust 侧 `sync::SYNC_PROGRESS_EVENT` 必须完全一致
  return listen<ToolSyncResult>("sync://progress", (event) => handler(event.payload));
}

// ============================================================================
// 纯展示常量（前端本地知识，不来自 Rust）
// ============================================================================

/**
 * 「即将支持」的工具。
 *
 * 它们既没有适配器也没有可检测的路径，因此**不参与同步**、
 * 永远显示为「即将支持」。列表来自 `PRD.md` 第 5 节步骤 2。
 *
 * 2026-09-14 更新：原列表中的「ChatGPT IDE」是命名错误（实为 Codex，
 * 桌面 App 与 CLI 共用 `~/.codex/`），且已于 TASK-18 完成接入、移入真实检测列表，
 * 故从此处移除。
 */
export const UPCOMING_TOOLS: readonly string[] = ["Trae"];

/**
 * 把系统原始错误兜底成用户可读文案。
 *
 * Rust 侧返回的错误**本应**已经是中文提示；这里只是最后一道防线——
 * 万一有 `os error 5` 这类内容漏出来，也不能直接显示给用户
 * （`PRD.md` 第 8 节的铁律）。
 */
export function toUserMessage(error: unknown): string {
  const raw = typeof error === "string" ? error : error instanceof Error ? error.message : "";

  // 已经是中文提示 → 原样使用
  if (raw && /[\u4e00-\u9fa5]/.test(raw)) {
    return raw;
  }

  // 不像中文提示的，一律换成通用兜底，绝不外泄系统细节
  return "操作未完成，请检查文件夹访问权限后重试。";
}

/** 复制文本到剪贴板，返回是否成功。 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    // 剪贴板 API 在部分环境不可用（权限/非安全上下文），回退到传统方案
    try {
      const holder = document.createElement("textarea");
      holder.value = text;
      holder.setAttribute("readonly", "");
      holder.style.position = "fixed";
      holder.style.opacity = "0";
      document.body.appendChild(holder);
      holder.select();
      const ok = document.execCommand("copy");
      document.body.removeChild(holder);
      return ok;
    } catch {
      return false;
    }
  }
}

# CrossBrain 项目长期备忘

> 项目根 `D:\project\memory1.0`（Tauri v2 + Vue 3 + TS，Rust 侧 `src-tauri/`）。
> 只放跨会话长期约定；过程写 `YYYY-MM-DD.md`。细节见 `docs/`、`AGENTS.md`。

## 流程

- 权威顺序：`AGENTS.md` → `MASTER_CONTEXT.md` → `CURRENT_STATUS.md` → `docs/impl/TASK_BREAKDOWN.md`。
- 任务收尾三项：① TASK_BREAKDOWN 补「⚠️ 实施修正」（示例代码**经常是错的**，逐条写「原文→为何错→改成什么」）② 补「✅ 实施记录」③ `CURRENT_STATUS.md` 三处同步（时间戳用 `date` 取）。验收含糊就改写成可自动化形式。
- **版本控制**：远端 `git@github.com:JasonOracle/CrossBrain.git`，`main`。**一个 task 一个提交**，**先写文档与记忆再提交**（并入同一提交）。暂存文件数基线 90~100，暴涨= `target/` 漏进。`.workbuddy/` 已纳入；换行统一 LF。

## Rust 侧

- 入口 `src-tauri/src/lib.rs`（非 main.rs）；新模块必须在 lib.rs 挂 `mod`，否则**不参与编译**而 build 照样成功。
- 路径走 `crate::paths`；时间戳走 `chrono`；写用户既有文件走 `write_breaking_hardlink()`（**无例外**）。
- `Adapter` 约束 `Send + Sync`；Mock 用 `AtomicU32`（`Cell` 非 Sync）。

## 三个 Adapter 的 L0 形态（接新工具前必读）

| Adapter | L0 落点 | 写入方式 |
|:---|:---|:---|
| Antigravity | `~/.gemini/config/rules/`（整目录被读） | 独立文件 `crossbrain-L0.md` |
| Claude Code | `~/.claude/CLAUDE.md` 单文件 | 标记块内 `@~/.ai-profile/AGENTS.md` **引用** |
| Codex | `~/.codex/AGENTS.md` 单文件 | 标记块内规则**全文内联** |

- **两条致命照搬**：给单文件型工具写独立文件→永不被读；给非 Claude 工具用 `@` 引用（`@` 是 Claude 专有）。先判目录型/单文件型。
- **遮蔽文件**：Codex `AGENTS.override.md` 优先且同级只取第一个非空 → `reject_if_override_present()` 硬拦截。每接新工具反问：有没有优先级更高的文件遮蔽落点？
- 共享层在 `adapters/mod.rs`（`inject_marker_block` 四情况协议 / `write_breaking_hardlink` / `cleanup_crossbrain_orphans` / `format_skill_md` / `marker_regex()` 由常量拼接）。新 Adapter 只提供差异：目标文件/备份文件/标记块内容；`l0_backup()` 默认 `None`。
- 内联用户内容前必须 `ensure_no_marker_in_content()`，放在**构造内容那一层**。
- 7 步清单见项目技能 `.workbuddy/skills/crossbrain-add-adapter`（v1.3.0）。
- **Spike 优先用工具自带 dump 命令**（如 `codex debug prompt-input` 一次验 L0+L2）；须在**项目外**中立目录跑；没有 dump 才退回人工探针（用后清理+复跑回归）；文档说不清就二进制取证 `grep -a -o -E '.{0,140}AGENTS\.md.{0,140}' <tool>.exe`。

## IPC 层

依赖单向：`commands.rs` → `sync.rs` → `adapters/`。`commands.rs` 只做参数转发与错误文案转换。`run_full_sync_with_progress()` 取真实路径不可注入；`run_for_tools()` 由调用方给清单（干跑靠它注入副本）。`state.rs` 读失败**回退默认值不返回 Err**。字符串寻址由 `tests/ipc_contract.rs` 交叉锁定；过程可见的命令须 `async` + `spawn_blocking`；前端调用全收 `src/api.ts`。

## 同步安全（红线）

- **ADR-14**：SSOT 读取失败即整体中止，绝不退化为空——`cleanup_orphans()` 空列表 = 删光用户全部技能。`knowledge/` 任一文件读不出就不调任何 Adapter。失败可见可重试，误删不可逆。
- 空规则不推 L0（空标记块白触发备份+断链）。git 提交失败不得影响同步结果。
- **写入边界**：只写 L0 规则文件、`crossbrain-*` 技能目录、`*.crossbrain-backup`。`settings.json`/`config.toml`/`mcp_config.json`/`rules/default.rules`/`skills/.system/`/`~/.agents/` 零引用。
- **ADR-15（TASK-19 已实现）**：`.crossbrain-backup` = 首注前原始件**任何路径不得覆盖**；`.crossbrain-before-restore` = 还原前现场，滚动、仅当场 ≠ 备份才写。还原必须走 `write_breaking_hardlink()`。前端闸门 `useLinkNotice.ts`：标记「已展示」写在 `confirm()` 内（契约测试锁定）。

## 测试（本机有真实用户数据）

- 全部跑临时目录 `with_base_dir(temp)`（进程号 + AtomicU32，Drop 清理）。「不误伤真实数据」= 复制真实目录跑真代码再比对零改动。「A 与 B 一致」断言输出相等。删改断言：动作前采样 + 比差异清单。
- **源码文本断言必须反向验证一次**。
- **干跑不做机器状态假设**：`dryrun_sync` 已改预状态自适应（首注/更新分支）；新断言先问「真机上还成立吗」。
- 每轮改动后必做安全回归（基线见下节；CLAUDE.md 断链是设计行为）。

## 前端

- 图标 `~icons/lucide/*`（禁 Emoji）；名以 `@iconify-json/lucide/icons.json` 为准（`triangle-alert` 非 `alert-triangle`）。令牌 `src/style.css` `@theme`，Naive 覆盖在 `App.vue`，两处色值一致；语义色只有 `-500` 档。
- **界面文案禁内部术语**（L0/L2/SSOT/slug/Adapter）：`ipc_contract::wizard_copy_has_no_internal_terms` 扫 WizardView/MainView/LinkNoticeDialog/RuleEditor —— **新增界面文件必须加进列表**。
- 共用界面逻辑放 `src/composables/`。V1 无 vue-router（ADR-12）。
- 新增 Naive UI 组件后**先 `npx vite build`**（生成 components.d.ts）再 `pnpm build`。
- 有可编辑状态的 Tab 一律 `show:lazy`（默认 if 切页丢未保存内容）。`n-message-provider` 必须包所有视图外层。
- **编辑≠同步**：useRuleEditor/RuleEditor 内禁 `runSync`（契约锁定）；读不出磁盘内容就禁用保存。
- **Markdown 预览安全（红线）**：csp 仍为 null。markdown-it 默认 `html:false`，**严禁开 html**（`markdown_preview_must_escape_raw_html` 锁死）；预览链接拦截点击。v-html 排版（`.md-preview`）放全局 style.css。
- **编辑器状态机（TASK-11/12）**：`useRuleEditor(io)` 接 `EditorIO` 注入（load/save 通道），`openFile()`/`close()`；`RuleEditor.vue` 是 props 化的共用组件（templates/footnote/placeholder/lengthWarning）。`withDefaults` 的数组/对象默认值**必须工厂函数**，否则 vue-tsc TS2322。

## 真实数据基线（2026-09-15 00:18 真机同步后）

- 硬链接组 **4+1**：`~/.ai-memory/user_profile.md`、`~/.cursor/rules/user_profile.md`、`~/.config/opencode/AGENTS.md`、`~/.gemini/config/rules/user_global.md` 同一 inode（links=4，md5 `47645f60…`）；`~/.claude/CLAUDE.md` 已断成 links=1（标记块外 1908 字节原文完好，`.crossbrain-backup` 是原始件）。
- `~/.ai-profile/global/rules.md`：444 字节，md5 `f0b10675…`，**任何验证零改动**。真实快照 2949 项。
- 技能清单：`~/.claude/skills/`=agnes-scroll-world/huashu-design；`~/.gemini/config/skills/`=design-taste-frontend/grill-me；`~/.codex/skills/`=grill-me+.system。
- `~/.codex/`：App 与 CLI 共用；AGENTS.md 579 字节（标记块+内联，备份在 `.crossbrain-backup`）。`skills/.system/` 绝不能删。
- `~/.agents/`（59MB/86 技能）跨宿主生态**绝不可写入或清理**；靠「只扫 `~/.codex/skills/`」+「只处理 `crossbrain-` 前缀」收窄。
- 本机 20+ AI 工具（远超 PRD），完整表见 `docs/testing/SPIKE_RESULTS.md`。

## 知识库（TASK-12）

- 文件名 = **标题 kebab 段 + `.md`**（无 hash、无 `crossbrain-` 前缀——带前者「改一个字文件就改名」，带后者折算时叠成双前缀）；纯中文标题退化 `item`，同名按 `-2/-3` 去重（覆盖不可逆）。
- 前端传来的文件名必须过 `validated_knowledge_path`（穿越/分隔符/非 .md 拒绝）——它直接拼真实路径。
- 知识文件保存走 `write_breaking_hardlink()`（可能被用户硬链进记忆中心）；删除幂等（NotFound=成功），工具内副本等下次同步清理（UI 须明示）。
- `dryrun_sync` 快照已含 SSOT 侧（`global/rules.md` + `knowledge/`）。

## 本机坑

- cargo 需 `export PATH="$HOME/.cargo/bin:$PATH"`；走清华镜像（`~/.cargo/config.toml`）。
- `os error 5/拒绝访问` = 句柄释放延迟（非杀软），`rm -rf target/debug/build/<crate>-*` 重试。
- Edit 报 EBUSY → 直接重试，别改 Write。**同一文件禁止并行 Edit**（静默丢改）。
- 沙箱 git 远端引用偶发不落盘（`git status` 显 `[gone]` 但 push 实际成功）→ 判断同步一律 `git ls-remote origin` 直连。

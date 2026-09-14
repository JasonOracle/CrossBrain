# CrossBrain 项目长期备忘

> 项目根 `D:\project\memory1.0`（Tauri v2 + Vue 3 + TS，Rust 侧 `src-tauri/`）。
> 只放跨会话有长期价值的约定；过程写 `YYYY-MM-DD.md`。细节见 `docs/` 与 `AGENTS.md`。

## 流程约定

权威顺序：`AGENTS.md` → `MASTER_CONTEXT.md` → `CURRENT_STATUS.md` → `docs/impl/TASK_BREAKDOWN.md`。
每个任务收尾三项：① 文档补「⚠️ 实施修正」（示例代码**经常是错的**，逐条写「原文 → 为什么不对 →
改成什么」）② 补「✅ 实施记录」（交付文件 / 实测命令与结果 / 遗留 / 给后续任务的约束）
③ `CURRENT_STATUS.md` 三处同步（阶段表 / 任务状态 / 变更记录，时间戳用 `date` 取，**别自己算**）。
验收条目含糊就**改写为可自动化形式**并标出验证方式。

**版本控制**（2026-09-15 起）：远端 `git@github.com:JasonOracle/CrossBrain.git`，主分支 `main`。
**一个 task 一个提交**，且**先写文档与记忆再提交**（并入同一提交）；格式见 `AGENTS.md`。
提交前自检：`git diff --cached --name-only | wc -l` 正常在 90 上下，变成几千就是
`src-tauri/target/`（9.2 GB）漏进去了。`.workbuddy/` 已纳入版本控制。

## Rust 侧结构

- 入口是 **`src-tauri/src/lib.rs`**（不是 `main.rs`）；新模块必须在 `lib.rs` 挂 `mod`，
  否则整个目录**不参与编译**而 `cargo build` 照样成功。
- 路径走 `crate::paths`（L-02）；时间戳走 `chrono`（L-03）；写用户既有文件走
  `write_breaking_hardlink()`（L-01 + D-03，**无例外**）。
- 跨模块约定值（如 `SLUG_PREFIX`）只留**一处定义 + 一条契约测试**。
- `Adapter` 约束 `Send + Sync`；`CleanupReport`/`AdapterError` 派生 `PartialEq/Clone/Eq`；
  Mock 用 `AtomicU32`（`Cell` 非 Sync）。

## 三个 Adapter 的 L0 形态互不相同（接新工具前必读）

| Adapter | L0 落点 | 写入方式 |
|:---|:---|:---|
| Antigravity | `~/.gemini/config/rules/`（整目录都被读） | 写**独立文件** `crossbrain-L0.md` |
| Claude Code | `~/.claude/CLAUDE.md` 单文件 | 标记块注入，块内是 `@~/.ai-profile/AGENTS.md` **引用** |
| Codex | `~/.codex/AGENTS.md` 单文件 | 标记块注入，块内是规则**全文内联** |

- **两条致命照搬**：给单文件型工具「写独立文件」→ 永不被读到；给非 Claude 工具用 `@` 引用
  （`@` 是 Claude 专有语法）。**第一步永远先判目录型 / 单文件型。**
- **遮蔽文件**：落点对 ≠ 会被读到。Codex 的 `AGENTS.override.md` 优先于 `AGENTS.md` 且同级只取
  第一个非空 → 已用 `reject_if_override_present()` 首步硬拦截。**每接新工具都反问：有没有
  优先级更高的文件会遮蔽我们的落点？**
- **共享层已有，勿重写**（`adapters/mod.rs`）：`inject_marker_block()`（四情况协议）、
  `write_breaking_hardlink()`、`cleanup_crossbrain_orphans()`、`ensure_crossbrain_slug()`、
  `format_skill_md()`、`MARKER_*`、`marker_regex()`（由常量拼接，勿硬编标志串）。
  新 Adapter 只提供差异：**目标文件 / 备份文件 / 标记块内容**（TASK-19 起 `l0_backup()`
  默认 `None` = 无备份概念）。
- **内联型防线**：内联用户内容前必须先 `ensure_no_marker_in_content()`，且放在**构造内容那一层**
  （Claude 侧不内联，塞进 `inject_marker_block()` 是永不触发的死分支）。

> 完整「接入新工具」7 步清单见项目技能 `.workbuddy/skills/crossbrain-add-adapter`（v1.3.0）。

## 🎯 Spike：优先用工具自带的 dump 命令（TASK-18 经验，高复用）

不要默认「Spike 必须人工开会话」。先翻 CLI 找「dump 模型可见输入」类命令；
`codex debug prompt-input` 一条命令即 dump `<INSTRUCTIONS>` + `<skills_instructions>` 全文，
L0/L2 一次验完。三个要点：① 必须在**项目外**的中立目录执行（本仓库根就有 `AGENTS.md`，
项目内跑分不清全局/项目加载）；② 没有此类命令才退回人工探针（须可整份删除、写明临时用途），
验证后清理 + 复跑安全回归；③ 文档说不清就**二进制取证**：
`grep -a -o -E '.{0,140}AGENTS\.md.{0,140}' <tool>.exe`（`app.asar` 也能 `grep -a`），
命中里的 Rust 模块路径能直接定位加载逻辑。另：「磁盘上有、清单里没有」未必是落点问题，
带 `disable-model-invocation: true` 的技能会从清单消失。

## IPC 层（TASK-10）

依赖单向：`commands.rs` → `sync.rs` → `adapters/`。
`commands.rs` 是前端唯一入口，只做参数转发与错误文案转换；业务写别处才能脱离 Tauri 测试。
`sync.rs` 的 `run_full_sync_with_progress()` 取真实路径不可注入；`run_for_tools()` 由调用方给
清单与内容、不推断路径（干跑靠它注入 `with_base_dir(副本)`）。`state.rs` 状态存
`~/.ai-profile/.crossbrain-state.json`，读取失败**回退默认值、不返回 `Err`**。
Tauri IPC 字符串寻址 → `tests/ipc_contract.rs` 交叉比对源码锁定；UI 要「过程可见」时命令必须
`async` + `spawn_blocking`；前端调用全收在 `src/api.ts`。

## 同步安全约束

- **ADR-14：SSOT 读取失败即整体中止，绝不退化为「空」。** `cleanup_orphans()` 语义是
  「删除不在 active 列表里的 `crossbrain-*`」→ **空列表 = 删光用户全部技能目录**。
  `knowledge/` 任一文件读不出来时一个 Adapter 都不调用。**失败可见可重试，误删不可逆。**
- **空规则不推 L0**（追加空标记块会白触发备份 + 断链两个不可逆副作用）。
- `git` 提交失败不得影响同步结果。
- **写入边界（已核查，可对用户宣示）**：只写三类目标 —— L0 规则文件、L2 技能目录（仅
  `crossbrain-*` 前缀）、`*.crossbrain-backup`。**`settings.json` / `config.toml` /
  `mcp_config.json` / `rules/default.rules` / `skills/.system/` / `~/.agents/` 一律零引用**，
  新写 Adapter 时必须继续保持这个边界。
- **ADR-15：对用户文件的改动必须「知情且可逆」（TASK-19 已实现）。**
  两个快照后缀语义不同，**别混用**：`.crossbrain-backup` = 首次注入前的原始文件，
  **任何代码路径都不得覆盖**；`.crossbrain-before-restore` = 最近一次还原前的现场，
  滚动更新且**仅当现场 ≠ 备份时才写**（无条件写会让第二次还原覆盖掉真实现场）。
  还原写入**必须走** `write_breaking_hardlink()`（否则「同步时断开、还原时又接上」）。
  `Adapter::l0_backup() -> Option<L0Backup>` 返回**一对**路径（落点 + 快照）；
  默认 `None` = 无备份概念（Antigravity），自动从备份列表消失。
  前端闸门在 `src/composables/useLinkNotice.ts`；**标记「已展示」必须写在 `confirm()` 内**，
  写在 `guard()` 里会让点取消的用户永远不再看到告知（有契约测试锁定）。

## 测试约定（本机有真实用户数据）

- 所有测试跑临时目录：`with_base_dir(temp)`；目录名 = 进程号 + `static AtomicU32`；
  `Drop` 里忽略失败地 `remove_dir_all`。
- 验证「不误伤真实数据」：把真实目录（含硬链接结构本身）复制成副本，在副本上跑真实代码，
  再比对真实目录零改动。
- 验证「A 与 B 一致」：跑同一组用例断言输出**相等**，不能分别验证「A 对」「B 对」。
- 删改类断言：① 状态在动作**之前**采样；② 比前后差异清单，不硬编码真实文件名。
  （GUI 探针法：command 里临时 `eprintln!("[probe] …")` → `pnpm tauri dev` 读日志 → 立刻移除；
  细节见 2026-09-14 日志。）
- **源码文本断言必须反向验证一次**（改了实现看它是否真的失败），否则极易写成恒为真。
- **干跑不做「机器状态」假设**：`dryrun_sync` 原假设真实 CLAUDE.md 从未同步过，用户真机
  同步一次后就永久假红；已改为**预状态自适应**（跑前探测标记块与备份，走「首注」或
  「更新」分支）。以后写干跑断言先问：这个前提在用户真机上还成立吗？
- **每轮改动后必做安全回归**，当前基线见下节（CLAUDE.md 断链是设计行为，别按旧基线判失败）。

## 前端约定

- 图标一律 `~icons/lucide/*` SVG（**禁止 Emoji**）；令牌在 `src/style.css` 的 `@theme`，
  Naive UI 覆盖在 `src/App.vue`，**两处色值必须一致**。语义色**只有 `-500` 一档**，
  浅色底配透明度修饰符（`bg-warning-500/10`），别写 `warning-200` 这类不存在的档位。
- lucide 图标名以实际图标集为准（`node_modules/@iconify-json/lucide/icons.json`）：
  是 `triangle-alert` 而非 `alert-triangle`，也没有 `history`。用前先查，别猜。
- **界面文案不得出现内部术语**（L0 / L2 / SSOT / slug / Adapter），由
  `ipc_contract::wizard_copy_has_no_internal_terms` 用 `template_of()` 扫
  `WizardView.vue` / `MainView.vue` / `LinkNoticeDialog.vue` / `RuleEditor.vue` ——
  **每新增一个面向用户的界面文件都要加进这个列表**。
- **跨视图共用的界面逻辑放 `src/composables/`**（`useLinkNotice` / `useRuleEditor`）；
  两个同步入口（向导步骤 4 / 主工作台「立即同步」）必须共用同一份闸门逻辑。
- **V1 不引入 vue-router**（ADR-12）：向导 / 主工作台两个顶层视图，内部用 Tab。
- `pnpm build` = `vue-tsc && vite build`，而 `src/components.d.ts` 要等 `vite build` 才生成 →
  **新增 Naive UI 组件后先跑一次 `npx vite build`**，再 `pnpm build`。
- **`n-tab-pane` 默认 `display-directive="if"` 会在切 Tab 时卸载面板**——未保存修改跟着丢。
  凡有可编辑状态的 Tab 一律 `show:lazy`。
- **`n-message-provider` 必须包在所有视图外层**（`App.vue`）：放进某个 `v-if` 分支，
  另一分支里的 `useMessage()` 运行时报错，`pnpm build` 看不出来。
- **「编辑」与「同步」是两个独立动作**：`useRuleEditor.ts` / `RuleEditor.vue` 里不得出现
  `runSync`（契约测试锁定）；磁盘内容读不出来（baseline 未建立）就**禁用保存**，
  防止空文件覆盖用户规则。
- **Markdown 预览安全（TASK-11，红线级）**：`tauri.conf.json` 的 `csp` 仍是 `null`，
  webview 里的脚本有 `invoke` 权限。渲染用户/远端内容必须走 markdown-it 默认
  `html:false`（原始 HTML 被转义），**严禁打开 `html` 开关**——
  `markdown_preview_must_escape_raw_html` 已锁死；预览区链接要拦截点击（webview 导航
  会离开应用）。收紧 CSP 是待办。
- **v-html 注入的内容不受 scoped 样式影响**，预览排版（`.md-preview`）必须放全局 `style.css`。

## 本机既有用户数据（别假设是空目录）＝ 安全回归基线

- **硬链接组（用户「全局 AI 记忆中心」）——2026-09-15 00:18 用户真机同步后为 4+1**：
  `~/.ai-memory/user_profile.md`、`~/.cursor/rules/user_profile.md`、
  `~/.config/opencode/AGENTS.md`、`~/.gemini/config/rules/user_global.md` 四路仍同一 inode
  （**links=4**，md5 全等 `47645f603b7bfd2ce5eeba650ed09981`）；`~/.claude/CLAUDE.md` 已被
  `write_breaking_hardlink()` 断成 **links=1**（标记块已注入，标记块外 1908 字节原文完整保留，
  `.crossbrain-backup` 仍是原始内容）。D-03 语义实测正确：只断 CLAUDE.md 侧。
- **`~/.ai-profile/global/rules.md`**：444 字节（前端模板内容），
  md5 `f0b1067570845c72b5e3acb929c5b90a`，任何验证都必须零改动。
- **真实快照 2949 项**跑前跑后逐项一致。
- **真实技能目录清单（跑前跑后必须一致）**：`~/.claude/skills/` = `agnes-scroll-world` /
  `huashu-design`；`~/.gemini/config/skills/` = `design-taste-frontend` / `grill-me`；
  `~/.codex/skills/` = `grill-me` + `.system`。Spike 遗留孤儿 `crossbrain-spike-test`
  已被 00:18 的真机同步按设计正确删除（验收样本已消费，`cleanup_orphans()` 实测有效）。
- **`~/.codex/`（桌面 App 与 CLI 共用，是一款工具）**：`AGENTS.md` 现为 579 字节
  （标记块 + 内联全文，00:18 首注；原始 0 字节件在 `AGENTS.md.crossbrain-backup`）；
  `AGENTS.override.md` 优先级更高（已硬拦截）。技能根四类：`skills/.system/`（内置，
  **绝不能被删**）、`skills/{name}/`（**我们的 L2 落点，已实测可见**）、`plugins/cache/`、
  `~/.agents/skills/`（第三方）。`rules/default.rules` 是命令白名单不是规则文件。
  `project_doc_max_bytes` 默认 32 KiB，全局 + 项目合并超限会截断。
- **`~/.agents/`（59 MB / 86 个技能）是跨宿主技能生态，极易误伤**：含 `.skill-lock.json`(v3)、
  `agents/openai.yaml`、3349 字节的 `~/.agents/AGENTS.md`（Codex **不读**它）。
  **任何 Adapter 都不得对 `~/.agents/` 写入或清理**；本项目靠「只扫 `~/.codex/skills/`」+
  「只处理 `crossbrain-` 前缀」收窄。
- 本机装了 **20+ 个 AI 工具**（远超 PRD）：`~/.workbuddy/`、`~/.codebuddy/rules/*.mdc`、
  `~/.cursor/`、`~/.zcode/`、`~/.qoder/`（后两者暂无全局落点）。完整表见
  `docs/testing/SPIKE_RESULTS.md`。

## 本机系统性坑

- bash 里 cargo 不在 PATH：跑前先 `export PATH="$HOME/.cargo/bin:$PATH"`。
- `cargo build` 报 `os error 5 / 拒绝访问` → 句柄释放延迟（**不是杀软**），
  `rm -rf target/debug/build/<crate>-*` 重试。
- cargo 走清华镜像：配置在 `~/.cargo/config.toml`，删文件还原官方源。
- Edit 报 `EBUSY` → 直接重试，别改写成 Write 覆盖。
- 换行符统一 LF（`.gitattributes` `* text=auto eol=lf`）；仓库内**没有** `.bat/.cmd/.ps1`，
  新增这类脚本要单独豁免 CRLF。
- **同一文件禁止并行 Edit**（会静默丢改）；跨文件才可并行。

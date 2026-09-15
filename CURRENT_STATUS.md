# CrossBrain — 当前项目状态（CURRENT STATUS）

> **此文件是活文档，每次有任务完成或状态变化，必须更新本文件。**  
> AI 工具接手时，先读本文件，确认当前进度，再开始工作。

---

## 📍 当前阶段

| 项目 | 状态 |
|:---|:---|
| **当前阶段** | 🔄 **第 4 阶段：系统验证与打包** — **TASK-15 ✅（端到端测试）**、**TASK-16 ✅（离线测试）**、**TASK-20 ✅（增补：探针检测，工具读取验证）**——进入 TASK-17（生产构建 + 打包） |
| **当前日期** | 2026-09-15 |
| **计划交付** | 2026-09-28（两周） |
| **下一个任务** | `TASK-17`：✅ `TASK-20`（增补）**已于 2026-09-15 12:01 完成**：设置页「工具读取检测」——写临时探针技能 + Codex 全自动取证（`codex debug prompt-input`）+ 移除/同步/卸载三重清理；真机端到端通过<br>`pnpm tauri build` 生产构建（验收：无 error、MSI 生成、可安装、可启动）<br>✅ `TASK-16` **已于 2026-09-15 11:24 完成**：T7-1/T7-2 通过（CDP 离线模拟）；T7-3 与 T8-1/T8-3 依赖暂缓的 TASK-09，随其回补<br>✅ `TASK-15` 已于 2026-09-15 11:12 完成：T9-1/T9-2 真机端到端通过（含 Codex `debug prompt-input` 实证技能加载）+ 积压 GUI 走查；**发现并修复 MainView 未加载规则内容的 BUG**；T10 真机确认卸载留用户执行<br>✅ `TASK-14` 已于 2026-09-15 10:30 完成：三工具去痕 + 标记块外内容保留 + 状态文件删除回向导<br>✅ `TASK-13` 已于 2026-09-15 09:58 完成：离线指示红色化 + 失败原因可点击展开 + report.ok 权威判定；git commit 一项顺延 TASK-09<br>✅ `TASK-12` 已于 2026-09-15 09:48 完成：知识卡片列表 + 逐篇编辑 + 2000 字警告 + 新建/删除<br>✅ `TASK-11` 已于 2026-09-15 00:55 完成：左右分栏编辑器 + 实时预览 + 保存与同步分离<br>✅ `TASK-19`（P1，数据安全红线）已于 2026-09-15 00:10 完成：断链首次同步前告知 + 备份一键还原 |
| **环境状态** | ✅ Rust 1.98.1 (MSVC) + VS Build Tools 2022；`~/.ai-profile/` SSOT 底座就绪；**IPC 层已打通，首次运行向导已端到端验证**；**支持工具数 2 → 3**（Claude Code / Antigravity IDE / Codex） |

---

## ✅ 已完成

### 文档体系（2026-09-14）

- ✅ `MASTER_CONTEXT.md` — AI 工具接手总纲
- ✅ `CURRENT_STATUS.md` — 本文件（当前状态追踪）
- ✅ `README.md` — 项目门面
- ✅ `docs/product/PRD.md` — 产品需求文档
- ✅ `docs/product/CHANGELOG.md` — 版本演进记录
- ✅ `docs/tech/ARCHITECTURE.md` — 系统架构文档
- ✅ `docs/tech/ADAPTER_SPEC.md` — Adapter 接口规范
- ✅ `docs/tech/TECH_STACK.md` — 技术栈选型说明
- ✅ `docs/tech/WINDOWS_COMPATIBILITY.md` — Windows 兼容性备忘
- ✅ `docs/testing/TEST_PLAN.md` — 测试计划（38 个用例）
- ✅ `docs/testing/SPIKE_RESULTS.md` — Spike A~D 验证归档
- ✅ `docs/impl/DEV_SETUP.md` — 开发环境搭建
- ✅ `docs/impl/TASK_BREAKDOWN.md` — 详细任务拆解与验收标准
- ✅ `docs/impl/RELEASE.md` — 发布打包流程
- ✅ `docs/impl/CONTRIBUTING.md` — 贡献者指南
- ✅ `docs/impl/DECISION_LOG.md` — 设计决策记录
- ✅ `docs/impl/HANDOFF_PROTOCOL.md` — 交接协议

### 技术预研（Spike）

- ✅ Spike A：Claude Code `@~` 路径展开 — Windows 可用
- ✅ Spike B：Claude Code Skills 懒加载 — 可用
- ✅ Spike C：Antigravity Skills 懒加载 — 可用
- ✅ Spike D：Antigravity L0 规则路径 — 确认
- ✅ Spike E：**Codex L0 读取 / L2 技能加载 / 作用域全局** — 三项全通过（2026-09-14）
  - 方法突破：`codex debug prompt-input` 可 dump 模型可见输入 → Spike 全自动化，
    不必再人工开会话（后续接入自带 CLI 的工具时优先复用此思路）

### 工程骨架（2026-09-14）

- ✅ `TASK-01`：Tauri v2 + Vue 3 + TypeScript 骨架初始化完成（6/6 验收项通过）
  - 实测版本：pnpm 9.15.9 / Node v22.22.2 / vue 3.5.42 / vite 8.3.0 / TS 6.0.3 / @tauri-apps/cli 2.11.4
  - 验证结果：`pnpm build`（含 `vue-tsc --noEmit` 类型检查）零报错；`pnpm dev` 返回 HTTP 200

- ✅ `TASK-02`：Tailwind CSS v4 + Naive UI + SVG 图标链路打通（13/13 验收项通过）
  - 实测版本：tailwindcss 4.3.3 / @tailwindcss/vite 4.3.3 / naive-ui 2.45.3 / vfonts 0.0.3 / unplugin-icons 24.0.0 / unplugin-vue-components 32.1.0 / @iconify-json/lucide 1.2.132
  - 决策：采用 **Tailwind v4 + 官方 Vite 插件**（原文档为 v3 写法，已同步改写 TASK_BREAKDOWN.md）
  - 验证结果：`pnpm build` 零报错；构建产物 CSS 中确认产出 `.text-blue-500` 与自定义令牌 `.text-brand-500{color:var(--color-brand-500)}`；`~icons/lucide/sparkles` 解析为真实 SVG 组件

- ✅ `TASK-03`：封装 Rust 路径模块 + 初始化 `~/.ai-profile/` SSOT 目录底座（10/10 验收项通过）
  - 新建 `src-tauri/src/paths.rs`（6 个路径函数，全部经 `dirs` crate + `PathBuf::join`，零字符串拼接）
  - 新建 `src-tauri/src/init.rs`（`ensure_profile_dirs()` 幂等初始化，自带单元测试）
  - 修改 `src-tauri/src/lib.rs`（注册 `mod init` / `mod paths`，Builder 启动前调用初始化）
  - 修改 `src-tauri/Cargo.toml`（新增 `dirs = "5"`、`chrono = "0.4"`，依据铁律 L-02 / L-03）
  - 实测版本：`dirs 5.0.1`
  - 验证结果：`cargo build` **0 error**；`cargo test` **1 passed**（幂等断言）；
    `pnpm tauri dev` 两次启动，`~/.ai-profile/{global,knowledge,.gitignore}` 自动创建，
    二次启动所有文件 mtime / md5 **零变化**（幂等实测）；已存在的 `AGENTS.md` 全程未被覆盖

- ✅ `TASK-04`：定义 `Adapter` trait 统一接口（7/7 验收项通过）
  - 新建 `src-tauri/src/adapters/mod.rs`（`Adapter` trait：`detect` / `sync_l0` / `sync_l2` / `cleanup_orphans` 四个必需方法 + `user_friendly_error` 默认实现；`CleanupReport`；`AdapterError` 5 个变体；含 5 个契约测试）
  - 新建 `src-tauri/src/adapters/antigravity.rs` / `claude_code.rs`（占位模块，待 TASK-05 / TASK-06 填充）
  - 修改 `src-tauri/src/lib.rs`（挂载 `pub mod adapters;`）
  - 验证结果：`cargo build` **0 error**（产物 `crossbrain.exe` 12.88 MB）；`cargo test` **6 passed / 0 failed**
  - 接口补充：trait 增加 `Send + Sync` 约束（Tauri `State` 需要）；`CleanupReport` / `AdapterError` 补派生 `PartialEq` / `Clone` / `Eq`（TASK-05/06 测试断言需要）

- ✅ `TASK-05`：实现 `AntigravityAdapter`（13/13 验收项通过）
  - 新建 `src-tauri/src/adapters/antigravity.rs`（`detect` / `sync_l0` / `sync_l2` / `cleanup_orphans` 完整实现 + 9 个测试）
  - 修改 `src-tauri/src/adapters/mod.rs`（新增共享 `format_skill_md()`，落实 ADAPTER_SPEC 第 4 节的 frontmatter 规范 + 4 个测试）
  - 新建 `src-tauri/examples/dryrun_antigravity.rs`（真实数据安全干跑，开发期工具，不参与发布产物）
  - 验证结果：`cargo test` **19 passed / 0 failed**（TASK-04 的 6 个 + 本次新增 13 个）
  - 关键加固：`base_dir` 注入（测试绝不碰真实目录）、`ensure_crossbrain_slug()` 写入命名空间校验、清理跳过非目录项
  - **真实数据干跑**：在真实 `~/.gemini/config` 副本上执行最激进清理，用户的 `grill-me` / `design-taste-frontend` / `rules/user_global.md` **全部完好**，真实目录**零改动**

- ✅ `TASK-06`：实现 `ClaudeCodeAdapter`（标记块四情况协议 + 备份机制 + D-03 断链写入）
  - 改写 `src-tauri/src/adapters/claude_code.rs`（四情况协议完整实现 + 19 个测试）
  - 修改 `src-tauri/src/adapters/mod.rs`（新增共享 `SLUG_PREFIX` / `ensure_crossbrain_slug()` / `cleanup_crossbrain_orphans()`）
  - 修改 `src-tauri/src/adapters/antigravity.rs`（改用上述共享实现，消除与 Claude 侧的重复）
  - 新建 `src-tauri/examples/dryrun_claude.rs`（真实 5 链接结构副本干跑，开发期工具，不参与发布产物）
  - 修改 `src-tauri/Cargo.toml`（新增 `regex = "1"`，标记块正则替换用）
  - 验证结果：`cargo test` **38 passed / 0 failed**（TASK-05 的 19 个 + 本次新增 19 个）；0 error
  - **真实数据干跑**：在真实 5 硬链接结构的副本上跑完四情况 + 孤儿清理，**其余 4 个工具路径全程一字未变**；
    真实 5 个文件 md5 与链接数（links=5）运行前后**完全一致**，零改动
  - 关键加固：`write_breaking_hardlink()` 三步写入（tmp → remove → rename）成为 `CLAUDE.md` 的**唯一**写入通道

- ✅ `TASK-07`：实现 Slug-Hash 生成算法（6/6 验收项通过）
  - 新建 `src-tauri/src/slug.rs`（`generate_parts() -> Slug` + `generate()` + 18 个单元测试）
  - 修改 `src-tauri/src/adapters/mod.rs`（`SLUG_PREFIX` 改为转出自 `crate::slug`，**单一真实源**）
  - 修改 `src-tauri/src/lib.rs`（挂载 `pub mod slug;`）
  - 修改 `src-tauri/Cargo.toml`（新增 `sha2 = "0.10"`）
  - 新建 `src-tauri/examples/dryrun_slug.rs`（验收脚本，开发期工具，不参与发布产物）
  - 验证结果：`cargo test` **63 passed / 0 failed**（TASK-06 的 38 + slug 18 + 集成一致性 7）
  - **外部交叉验证**：hash 段与系统 `sha256sum` 逐位一致
    （`sha256("test")` → `9f86d0`、`sha256("")` → `e3b0c4`）——证明是标准 SHA-256 而非私有实现
  - 关键加固：`.md` 剥离大小写不敏感且字符边界安全、空 kebab 段兜底 `item`、
    hash 用 SHA-256（`DefaultHasher` 跨版本不稳定会导致全量目录被判孤儿而重建）、
    4 条跨模块契约测试钉死「生成」与「命名空间校验」两侧不分叉
  - ⚠️ 已知局限：纯中文文件名的 kebab 段退化为 `item`（功能无影响，仅目录名可读性下降），
    三条改进路径记录在 TASK_BREAKDOWN.md

- ✅ `TASK-08`：孤儿清理一致性验证（4/4 验收项通过）
  - 新建 `src-tauri/tests/adapter_consistency.rs`（7 个集成测试用例）
  - 设计要点：**同一组输入同时驱动两个 Adapter**，断言 `CleanupReport` 与清理后目录树
    **逐项相等**（原计划的「分别测试」只能证明各自符合自己的预期，证不出行为一致）
  - 验证结果：`cargo test` **63 passed / 0 failed**（lib 56 + 集成 7）
  - 真实数据安全：全部用例跑在系统临时目录，真实 5 个硬链接目标 md5 与链接数
    （links=5）零改动，用户技能（`grill-me` / `design-taste-frontend` / `agnes-scroll-world`
    / `huashu-design`）全部完好

- ✅ `TASK-10`：首次启动 5 步向导 UI + **IPC 层**（2026-09-14 完成）
  - 新建 Rust：`commands.rs`（6 个 `#[tauri::command]` 前端唯一入口）、
    `sync.rs`（同步编排 + DTO + 5 个单测）、`state.rs`（跨启动状态 + 5 个单测）
  - 新建测试与工具：`tests/ipc_contract.rs`（4 个跨语言契约测试）、
    `examples/dryrun_sync.rs`（真实结构副本干跑）
  - 新建前端：`api.ts`（类型与命令封装）、`views/WizardView.vue`（5 步向导）、
    `views/MainView.vue`（主工作台外壳）；改写 `App.vue`（替换 TASK-02 占位页）
  - 修改：`paths.rs`（`STATE_FILE_NAME` / `state_file()`）、`init.rs`（`.gitignore` 改幂等补齐）、
    `lib.rs`（注册命令、移除脚手架的 `greet`）、`tauri.conf.json`（窗口 → 1024×720）
  - 验证结果：`cargo test` **82 passed / 0 failed**（lib 70 + 一致性 7 + IPC 契约 5），
    0 error 0 warning；`pnpm build`（含 `vue-tsc --noEmit`）**零报错**
  - **端到端实测**：`pnpm tauri dev` 启动后，探针确认前端确实调用了
    `get_startup_state` 与 `detect_tools`（返回 2 项）——IPC 链路通、向导已渲染
  - **真实结构干跑**：`cargo run --example dryrun_sync` **20 项检查全绿**，
    含「两个 `crossbrain-spike-test` 遗留孤儿被正确清理」与「真实目录 **2885 项快照零改动**」
  - 关键加固：**SSOT 读取失败即中止同步**（把「读不出来」当「空的」会误删用户全部技能，
    见 ADR-14）；同步进度走 `sync://progress` 事件 + `spawn_blocking`
    （否则主线程被占住，前端收得到事件也渲染不出来，「实时显示」会退化成假实时）
  - ⚠️ 遗留两项：**向导视觉效果需人工确认**（TASK-02 记录的 Tailwind preflight
    与 Naive UI 样式优先级冲突是待观察项）；**git 提交（TASK-09）未接入**

### 开发环境（2026-09-14）

- ✅ `BLOCKER-01` 解除：Rust 1.98.1 (MSVC) + VS Build Tools 2022 + Windows SDK 全部安装完成
  - 真实编译链接测试通过（不是只看 `--version`，而是真的编译并运行了 exe）
  - 详见 `docs/impl/DEV_SETUP.md`「实测安装记录与踩坑复盘」

---

## 🔲 待完成任务（按优先级排序）

> **接手工具：从第一个 🔲 开始执行，完成后改为 ✅ 并更新「本次变更记录」。**

### 第 1 阶段：工程底座初始化（Day 1 - Day 2）

- ✅ **TASK-01**：初始化 Tauri v2 + Vue 3 + TypeScript 项目骨架（2026-09-14 19:54 完成）
- ✅ **TASK-02**：配置 Tailwind CSS **v4** + Naive UI 组件库 + SVG 图标（2026-09-14 完成）
- ✅ **TASK-03**：封装 Rust `dirs` 路径模块，初始化 `~/.ai-profile/` 目录底座（2026-09-14 21:34 完成）

### 第 2 阶段：Rust 核心分发引擎（Day 3 - Day 5）

- ✅ **TASK-04**：定义 `Adapter` trait（`detect` / `sync_l0` / `sync_l2` / `cleanup_orphans`）（2026-09-14 21:39 完成）
- ✅ **TASK-05**：实现 `AntigravityAdapter`（含 Plan B 降级）（2026-09-14 21:53 完成）
- ✅ **TASK-06**：实现 `ClaudeCodeAdapter`（含标记块四情况协议、备份机制、D-03 断链写入策略）（2026-09-14 完成）
- ✅ **TASK-07**：实现 Slug-Hash 生成算法（`slug.rs`）（2026-09-14 完成）
- ✅ **TASK-08**：孤儿清理一致性验证（2026-09-14 完成）
- 🔲 **TASK-09**：封装本地 Git 提交（`git.rs`，含 `nothing to commit` 静默处理）— ⏸️ **暂缓**（用户本机尚未建仓库；接入位置已在 `sync.rs` 预留）

### 增补任务（2026-09-14 用户触发）

- ✅ **TASK-18**：Codex Adapter 接入（**支持工具数 2 → 3**）（2026-09-14 完成）
  - 触发：用户发现向导只识别 2 个工具，要求接入 Codex。经路径预探测确认成本最低后提入 V1
  - 关键差异：Codex 的 L0 走**标记块注入 + 内联规则全文**（Claude 是 `@` 引用、Antigravity 是独立文件）
  - **第二轮修正（同日）**：探测到用户装的是 **Codex 桌面应用**（`D:\soft\win-x64`，MSIX 包 `OpenAI.Codex`），
    确认 App / CLI / Chrome 扩展**共用 `~/.codex/`** → ① 显示名 `Codex CLI` → **`Codex`**；
    ② 新增 `AGENTS.override.md` 遮蔽检测（该文件优先级更高，会让我们的规则完全失效），命中即中止报错；
    ③ 干跑新增第 [10] 节端到端断言「该错误确实上报为失败」
  - ✅ **Spike 闭环（2026-09-14 23:4x）**：三项全部通过 —— ① L0 读取（探针口令出现在
    模型上下文的 `<INSTRUCTIONS>` 块）② L2 技能加载（探针技能以 `name + description +
    (file: …)` 出现）③ 作用域**全局**（在中立目录下即被加载）。
    关键方法：`codex debug prompt-input` 可直接 dump 模型可见输入，**Spike 从此可自动化**
  - 附带发现：Codex 技能根共四类（含第三方 `~/.agents/skills/`，86 个技能 / 59 MB，
    **不得写入或清理**）；`disable-model-invocation: true` 会让技能从清单消失。

- ✅ **TASK-19**：改动知情与可逆（**P1，数据安全红线**）（2026-09-15 00:10 完成）
  - 触发：用户提问「会不会破坏用户原有编程软件的用户偏好配置」→ 逐行核查写入边界
  - **核查结论（可作为对用户的承诺）**：偏好配置类文件 —— `~/.claude/settings.json`、
    `~/.codex/config.toml`、`~/.gemini/config/mcp_config.json`、`~/.gemini/config/AGENTS.md`、
    `~/.codex/rules/default.rules`、`skills/.system/`、`~/.agents/` ——
    **代码里零引用，不存在破坏路径**。写入目标只有三类：L0 规则文件、L2 技能目录（仅
    `crossbrain-*` 前缀）、`*.crossbrain-backup`
  - **缺口 1（可逆性）已补**：新增「设置 → 备份与还原」区 + `list_backups` / `restore_backup`
    两条命令。还原前先把当前版本另存为 `*.crossbrain-before-restore`（**仅当内容与备份不同时**，
    因而幂等），**绝不覆盖** `.crossbrain-backup` 这份唯一干净快照；还原写入同样走
    `write_breaking_hardlink()`
  - **缺口 2（知情）已补**：新增 `LinkNoticeDialog.vue`，首次同步前必须确认一次
    「共享关系将被断开、内容不变」。闸门逻辑抽到 `src/composables/useLinkNotice.ts`
    供向导与主工作台共用；**标记「已展示」写在确认之后**，点取消不会吞掉这条告知
  - 依据：`DECISION_LOG.md` **ADR-15**；任务定义与「实施修正 8 条 + 实施记录」见
    `TASK_BREAKDOWN.md` **TASK-19**
  - **遗留（3 项，均不阻塞）**：① GUI 未做点击穿过验证（由用户目视或 TASK-15 覆盖）
    ② 还原后不自动重新同步，界面上未提示这一点 ③ `*.crossbrain-before-restore` 无清理入口

### 第 3 阶段：前端控制台（Day 6 - Day 8）

- ✅ **TASK-10**：首次启动 5 步向导 UI（工具检测 → 规则填写 → 同步 → 验证）（2026-09-14 完成）
- ✅ **TASK-11**：全局规则编辑器工作台（Markdown 编辑 + 实时预览）（2026-09-15 00:55 完成）
  - 左右分栏（编辑 + 预览同时可见）；保存与同步严格分离，契约测试锁死「保存不触发同步」
  - **安全要点**：预览渲染选 markdown-it（默认 `html:false` 转义原始 HTML）——
    `tauri.conf.json` 的 `csp: null` 下，渲染原始 HTML 等于把 webview 的
    `invoke` 权限交给任意脚本；`markdown_preview_must_escape_raw_html` 锁死这个开关
  - 连带发现并修复：**用户 00:18 的真机同步推翻了干跑「从未同步过」的假设**，
    干跑改为预状态自适应（首注 / 更新两分支各自的不变式）
  - 详见 `TASK_BREAKDOWN.md` TASK-11「实施修正 10 条 + 实施记录」

- ✅ **TASK-12**：技能知识卡片列表（新建 / 编辑 / 删除，超 2000 字警告）（2026-09-15 09:48 完成）
  - 卡片网格：标题（文件名去 `.md`）+ 第一行摘要 + **注入目录名**（PRD §6.2 / ADR-11 补偿）+ 字数与修改时间
  - 新建：文件名 = 标题 kebab 段 + `.md`（无 hash、无前缀），同名自动 `-2/-3` 去重；预填 `# [技术/语言] · [具体场景]` 模板
  - 编辑：复用 `RuleEditor.vue`（props 化）与 `useRuleEditor`（`EditorIO` 注入），超 2000 字黄色警告条（PRD §6.2 原文）
  - 删除：二次确认 + 明示「工具内副本下次同步清理」；文件名校验防路径穿越；重复删除幂等
  - 详见 `TASK_BREAKDOWN.md` TASK-12「实施修正 10 条 + 实施记录」

- ✅ **TASK-13**（2026-09-15 09:58）：同步状态栏收尾——离线指示红色化、
  失败原因可点击展开（PRD §7）、成功判定改用 `report.ok` 权威字段。
  验收「+ git commit」顺延 TASK-09（架构归属使然，见 TASK_BREAKDOWN 修正 1）
- ✅ **TASK-14**（2026-09-15 10:30）：一键卸载 / 去痕——三工具 `uninstall()`
  （移标记块 → 删备份 → 清 `crossbrain-*` 技能目录），标记块外内容原样保留，
  状态文件删除后下次启动回向导；用户数据（`~/.ai-profile`）明确不碰

### 第 4 阶段：系统验证与打包（Day 9 - Day 10）

- ✅ **TASK-15**（2026-09-15 11:12）：端到端测试——T9-1/T9-2 真机全链路通过
  （UI 新建→同步→三工具落盘→Codex 实证加载；删除→同步→孤儿清理）；
  积压 GUI 走查完成；**发现并修复 MainView 未调用 `ruleEditor.load()` 的 BUG**
  （「全局规则」页此前永远显示空内容且无法保存）。T10 真机确认卸载留用户执行
  （弹窗/取消路径已验，文件语义由 dryrun [13] 覆盖）
- ✅ **TASK-16**（2026-09-15 11:24）：离线测试——CDP `emulateNetworkConditions`
  模拟断网（零风险可逆）：T7-1 断网同步正常、T7-2 离线指示正确且恢复在线自动消失、
  顺带 T8-2 连续同步静默成功。T7-3 与 T8-1/T8-3 依赖暂缓的 TASK-09，随其回补
- 🔲 **TASK-17**：Tauri 生产构建 + 打包（MSI + 便携版 zip）

---

## 🚫 阻塞项（Blockers）

> **目前无阻塞项。** BLOCKER-01（Rust toolchain 未安装）已于 2026-09-14 21:00 解除，
> 详见下方「已决策事项 D-02」与 `docs/impl/DEV_SETUP.md` 的实测安装章节。

---

## 🧭 已决策事项（Decisions）

> **D-01：Tailwind CSS 采用 v4（已决策，2026-09-14）**
>
> - 背景：`tailwindcss` 最新版为 **4.3.3**，而原 `TASK_BREAKDOWN.md` TASK-02 写的是 v3 用法
>   （`npx tailwindcss init -p`、`@tailwind base/components/utilities`），在 v4 下**全部失效**
> - 决策：**采用 v4 + `@tailwindcss/vite` 插件**
> - 影响：不再需要 `tailwind.config.js` / `postcss.config.js` / `postcss` / `autoprefixer`；
>   设计令牌统一写在 `src/style.css` 的 `@theme { }` 中
> - 已同步更新：`docs/impl/TASK_BREAKDOWN.md`（TASK-02 全节改写为 v4）、`docs/tech/TECH_STACK.md`
>
> **D-02：Rust 工具链走 MSVC 官方路线（已决策并执行完毕，2026-09-14）**
>
> - 备选 GNU + MinGW-w64（体积小但 Tauri 兼容性非官方保证）
> - 决策：走 MSVC，牺牲安装体积换取官方支持与 CI 一致性
> - **实际安装结果（已实测验证）**：
>   - rustup-init.exe（官方，SHA256 与官方 `.sha256` 校验一致）
>   - Rust toolchain：**stable-x86_64-pc-windows-msvc 1.98.1**（profile = minimal）
>   - VS Build Tools 2022：**MSVC 14.44.35207**（`cl.exe` + `link.exe`）+ **Windows SDK 10.0.26100.0**
>   - Tauri CLI：`pnpm tauri --version` → **tauri-cli 2.11.4**（npm 版，本项目实际使用）
>   - `~/.cargo/bin` 已写入用户 PATH（新终端直接可用）
> - **真实编译链接测试通过**：`rustc hello.rs` 编译出 148,992 字节 exe 并成功运行
>   （`CrossBrain rust link ok: [1, 2, 3, 4, 5]`）
> - ⚠️ 安装过程中踩到 3 个坑（大小写重复环境变量、VS installer 版本缺陷、rustup 下载卡死），
>   完整复盘见 `docs/impl/DEV_SETUP.md`「实测安装记录与踩坑复盘」

> **D-03：`~/.claude/CLAUDE.md` 的写入策略 = 「断开硬链接 / 原子替换」（已决策并落地，2026-09-14）**
>
> **实测事实**（`fsutil hardlink list` 权威输出）：下列 5 个路径指向**同一个 inode**
> （`links=5`，内容 md5 完全一致，1908 字节）：
>
> ```
> ~/.ai-memory/user_profile.md            ← 原始
> ~/.cursor/rules/user_profile.md
> ~/.config/opencode/AGENTS.md
> ~/.gemini/config/rules/user_global.md
> ~/.claude/CLAUDE.md                     ← TASK-06 的目标
> ```
>
> 文件首行即自称「AI 个人中心记忆与工程偏好中心 (Global AI Memory Hub)」，
> 明示它是「全机所有 AI 编程工具（Antigravity、OpenCode、Claude Code、Cursor 等）的
> **单一真实可信源 (Single Source of Truth)**」。
> 也就是说，用户**已经用硬链接实现了 CrossBrain 的目标场景**：一份内容，5 个工具生效。
>
> **⚠️ 决策（用户拍板，2026-09-14 21:56）**：采用 **选项 2 —— 断开硬链接 / 原子替换**。
>
> 理由：不能把 Claude 专有的 `@` 语法通过硬链接**污染到 Cursor 和 Antigravity**，必须隔离。
>
> **执行要求（已写死在实现里）**：`sync_l0_with_result` 中所有对 `CLAUDE.md` 的写入
> （情况一 / 二 / 三 / 四，**无一例外**）一律**不得**就地 `fs::write`，必须走
> `write_breaking_hardlink()`：
>
> 1. 新内容先写入临时文件 `CLAUDE.md.crossbrain-tmp`（内容先落盘，失败也不丢数据）
> 2. **`fs::remove_file()` 删除原路径** ← Windows 上这一步才真正打断硬链接
> 3. `fs::rename()` 覆盖为原路径（同卷原子替换，不会读到半个文件）
>
> *（第 2 步不可省：Windows 的 `rename` 不覆盖已存在目标；而「先删再 rename」
> 正好也是断链本身所必需的动作，一步兼得。）*
>
> **已接受的代价**：用户手工搭的「一处修改、五处生效」机制在 `CLAUDE.md` 这一侧被打破，
> `~/.claude/CLAUDE.md` 从此是独立文件。
>
> **另需正视（未处理，留给后续）**：CrossBrain 设计的 SSOT 在 `~/.ai-profile/global/rules.md`，
> 而用户正在用的 SSOT 是 `~/.ai-memory/user_profile.md`，两者定位重叠。
> 建议在 TASK-10（首次启动向导）阶段向用户提示「本机已存在同类机制」。

---

## 📝 本次变更记录（倒序，最新在上）

> **每次 AI 工具完成工作后，必须在此追加记录。格式如下：**
>
> ```
> ### YYYY-MM-DD HH:MM:SS — {工具名 or 人工}
> - 完成了：{TASK-XX} 做了什么
> - 修改了文件：列出所有修改过的文件
> - 注意事项：如果遇到了坑或做了特殊决定，在这里说明
> ```

---

### 2026-09-15 12:01:14 — WorkBuddy（GLM-5.3-Flash）

- 完成：**TASK-20 探针检测（增补任务）**。设置页新增「工具读取检测」区：
  向各工具技能目录写 `crossbrain-probe/` 临时技能（frontmatter description 含
  唯一标记 `cb-probe-<时间戳>`）；**Codex 全自动取证**（运行 `codex debug
  prompt-input`、30 秒超时、输出落临时文件判标记，通过即当场清理探针）；
  Claude Code / Antigravity 半自动（返回人工验证问句与标记）。移除有三重兜底：
  手动按钮、下次同步的孤儿清理、卸载。
- 关键改动：`Adapter` trait 新增必选方法 `skills_root()`（写与删同根）；
  共享函数 `remove_crossbrain_skill_dir()`（只删一个，不可用 cleanup_orphans
  反向表达）；`commands.rs` 两条新命令（`run_tool_probe` 走 spawn_blocking）；
  设置页复用 `detectTools` 出清单。
- 验证：cargo test **141 passed / 0 failed**（+4 探针单测、+1 契约测试
  `probe_wiring_matches`）；pnpm build 通过；**真机端到端**——手工写入探针后
  `codex debug prompt-input` 输出中出现标记，清理后零残留。
- 文件：`adapters/mod.rs`、三个 Adapter、`sync.rs`、`commands.rs`、`lib.rs`、
  `api.ts`、`MainView.vue`、`ipc_contract.rs`、TASK_BREAKDOWN（TASK-20 节）、本文件。

### 2026-09-15 11:33:08 — WorkBuddy（GLM-5.3-Flash）

- 完成：**用户反馈布局 BUG 修复（RuleEditor 分栏）**。症状：初始态输入框下方
  边框不可见、footnote 被盖（两框缝隙露出「推送」两字）；多行输入后面板顶出
  视口、顶部内容被盖、textarea 无法内部滚动。根因：`n-spin` 的插槽层
  `.n-spin-content` 高度默认 auto，切断了 flex 高度链——内部 `h-full` 全部失效，
  窗格高度变成随内容撑开。修复：`:content-style="{ height: '100%' }"`（一行）。
- 修改文件：`src/components/RuleEditor.vue`（+5 行）、
  `docs/impl/TASK_BREAKDOWN.md`（TASK-11 实施修正追加第 5 条）、本文件、记忆库。
- 验证：CDP 实测初始态四边边框完整、footnote 可见；40 行内容 textarea 内部
  滚动正常（scrollTop 可设）、框高固定 364px、文档零溢出；测试内容已回填原文；
  pnpm build 通过；`tauri.conf.json` 调试参数已还原。
- 教训（已入 TASK_BREAKDOWN）：**Naive UI 包裹型组件会把 flex/grid 高度链
  切断在自己的 DOM 层**——内部传 `h-full` 必须用 `content-style` 显式接续。

### 2026-09-15 11:24:32 — WorkBuddy（GLM-5.3-Flash）

- 完成：**TASK-16 离线测试**。方法：CDP `Network.emulateNetworkConditions(offline:true)`
  浏览器级离线模拟（零风险可逆，`navigator.onLine` 变 false、`offline` 事件派发，
  无需真禁网卡）。T7-1 断网同步正常完成（三工具 ok、无网络报错）、T7-2 离线指示
  正确显示且恢复在线自动消失、顺带 T8-2 连续两次同步第二次静默成功。
  **T7-3 及 T8-1/T8-3 依赖暂缓的 TASK-09（`~/.ai-profile` 尚无 git 仓库），随其回补**。
- 修改文件：`docs/impl/TASK_BREAKDOWN.md`（TASK-16 实施记录）、本文件三处、
  记忆库。`tauri.conf.json` 调试参数已还原（仅行尾归一，内容零差异）。
- 注意事项：测试期间同步执行了 3 次（内容等价重写），真实数据核对无变化
  （rules.md md5 `f0b10675…`、knowledge 0 项）。测试截图与走查脚本在
  `src-tauri/target/e2e/`（不进 git）。

### 2026-09-15 11:12:01 — WorkBuddy（GLM-5.3-Flash）

- 完成：**TASK-15 端到端测试**。方法：`tauri.conf.json` 临时加
  `additionalBrowserArgs --remote-debugging-port=9223`（跑完即还原）→
  Playwright `connectOverCDP` 驱动真实 WebView2 窗口（wry 不读
  `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`，环境变量方案无效，已写入 TASK_BREAKDOWN）。
  T9-1 全链路通过（UI 新建技能→保存→同步→三工具 `crossbrain-vue3-aaee58`
  落盘→`codex debug prompt-input` 实证技能进入模型可见输入）；T9-2 通过
  （UI 删除→同步→三工具孤儿全清）。积压 GUI 走查（TASK-11/12/13/19）完成。
- 修改文件：`src/views/MainView.vue`（**BUG 修复**：`onMounted` 补
  `ruleEditor.load()`——此前「全局规则」页永远空内容、保存禁用）、
  `docs/impl/TASK_BREAKDOWN.md`（TASK-15 实施修正 5 条 + 实施记录）、本文件三处、
  记忆库。`tauri.conf.json` 的调试参数已还原（git diff 零残留）。
- 注意事项：
  - **GUI 走查抓到单测/类型检查都发现不了的「忘记接线」BUG**——「修复」与「加载」
    分属两层，`load()` 只挂在错误重试按钮上，正常路径无人调用。
  - 预览 HTML 转义在真实窗口复验通过（`<b onclick>` / `<img onerror>` 探针均未执行）。
  - T10 真机确认卸载、TASK-19 真机还原、TASK-13 失败展开（需制造失败/断网）
    留用户或 TASK-16（T7）。
  - 验证：cargo test 136 passed / 0 failed；pnpm build 通过；真实数据零改动
    （rules.md md5 `f0b10675…`、`~/.ai-memory` links=4、knowledge 0 项、备份完好）。

### 2026-09-15 10:30:55 — WorkBuddy（DeepSeek-V4.1-Flash）





- 完成：**TASK-14 一键卸载 / 去痕**。三工具实现 `Adapter::uninstall()`：
  Claude/Codex = 移标记块（块外内容原样保留）→ 删备份 → 清 `crossbrain-*`
  技能目录；Antigravity = 删独立规则文件 → 清技能目录。编排 `uninstall_all_for`
  幂等、单工具失败即中止可重试。命令层在全部成功后删状态文件
  （下次启动回向导），前端二次确认弹窗 + `uninstalled` 事件切回向导。
- 修改文件：`adapters/mod.rs`、`adapters/{claude_code,codex,antigravity}.rs`、
  `state.rs`、`sync.rs`、`commands.rs`、`lib.rs`、`src/api.ts`、
  `src/views/MainView.vue`、`src/App.vue`、`tests/ipc_contract.rs`、
  `examples/dryrun_sync.rs`（[13] 节）。
- 注意事项：
  - 删备份必须在标记块**成功移除之后**——备份是用户唯一还原手段（修正 2）。
  - 整文件皆由我方创建时移除后删整个文件；有备份的空原文件写回空。
  - 反向验证抓到弱断言：子串 `uninstall_crossbrain` 连
    `uninstall_crossbrain_x` 变体都防不住 → 改断言完整调用形态（修正 8）。
  - 用户数据 `~/.ai-profile` 不在清理范围，弹窗已明示。
  - 验证：cargo test 136 passed / 0 failed / 0 warning；dryrun 13 节 0 未通过；
    pnpm build 通过；真实数据零改动。

### 2026-09-15 09:58:48 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成：**TASK-13 同步状态栏 + 立即同步（收尾）**。TASK-10 外壳审计后 6 条验收
  已有 4 条达标，本任务补齐缺口：① 离线指示按 PRD §7 红色化
  （`text-error-500` + `wifi-off`，不用 Emoji）；② 「上次同步未完全成功」
  改为可点击 `<button>`，展开各失败工具的面向用户原因（PRD「点击可查看原因」）；
  ③ 成功判定改用 `SyncReport.ok` 权威字段并用报告覆盖进度行。
- 修改文件：`src/views/MainView.vue`、`src-tauri/tests/ipc_contract.rs`
  （新增 `status_bar_matches_prd_contract`，已反向验证）。
- 注意事项：
  - 验收项「+ git commit」**顺延 TASK-09**：架构注释明确 git add/commit 属
    TASK-09（SSOT 版本化，暂缓）的封装范围，不往本任务硬塞（同 TASK-18 先例）。
  - 失败原因不跨启动持久化（state 只存成功与否），重启后展开区诚实提示。
  - 验证：cargo test 128 passed / 0 failed / 0 warning；dryrun 12 节 0 未通过；
    pnpm build 通过；真实数据零改动。

### 2026-09-15 09:48:51 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成：**TASK-12 技能知识卡片列表**（列表 / 新建 / 编辑 / 删除 + 2000 字警告）
- 修改文件：
  - Rust：`sync.rs`（KnowledgeCard + CRUD 五组函数 + 文件名校验 + 测试×5）、
    `slug.rs`（`kebab_from_title`）、`commands.rs` / `lib.rs`（5 条命令）、
    `examples/dryrun_sync.rs`（新增 [12] 节 + SSOT 侧纳入快照）、
    `tests/ipc_contract.rs`（字段对 +4 / 参数名比对 / never_triggers_sync / 术语列表）
  - 前端：`api.ts`、`composables/useRuleEditor.ts`（EditorIO 注入 + openFile/close）、
    `components/RuleEditor.vue`（props 化 + 2000 字警告条）、
    `components/KnowledgeManager.vue`（新）、`views/MainView.vue`
  - 文档：`TASK_BREAKDOWN.md`（修正 10 条 + 实施记录）、本文件三处、记忆库
- 验证：cargo test **127 passed / 0 failed / 0 warning**；dryrun_sync 12 节 **0 项未通过**；
  `pnpm build` 通过；两条新文本断言反向验证；真实数据零改动
  （4+1 硬链接组 / knowledge 0 项 / rules.md md5 `f0b10675…`）
- 注意事项：① 验收项「{slug_without_hash}.md」歧义已修正为「标题 kebab 段 + `.md`」，
  同名自动去重（覆盖不可逆）② 保存走 `write_breaking_hardlink`（知识文件可能被用户
  硬链）③ 再次踩中「同消息并行编辑同一文件丢改」——被反向验证抓出，教训已记档

### 2026-09-15 00:55:35 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：**`TASK-11` 全局规则编辑器工作台（左右分栏 + 实时预览）—— 代码 + 测试 + 前端 + 文档全部落地**
  - 左右分栏（编辑与预览同时可见）；保存与同步严格分离（契约测试
    `rule_editor_saving_never_triggers_sync` 锁死，**反向验证过**）；
    保存成功走轻提示 Toast（`n-message-provider` 包住所有视图），不是弹窗
  - **安全要点**：预览渲染选 markdown-it（默认 `html:false`，原始 HTML 被转义）。
    `tauri.conf.json` 的 `csp: null` 下若渲染原始 HTML，脚本可在 webview 内执行，
    而 webview 有 `invoke` 权限，等同本地文件任意读写；
    `markdown_preview_must_escape_raw_html` 锁死「不许打开 html 开关」
  - **交互正确性**：三个 Tab 改 `display-directive="show:lazy"`（Naive UI 默认 `if`
    会在切 Tab 时卸载面板 → 未保存修改丢失）；「未保存」按内容基线判断
    （改了又改回原样不算）；**读取失败禁用保存**（读不出来 ≠ 空，与 ADR-14 同源）；
    模板入口只在内容为空时出现（主工作台里一键套模板 = 静默覆盖用户规则）
  - **连带修复**：用户 00:18 真机同步推翻了干跑「从未同步过」的假设（2 项假失败），
    干跑改为预状态自适应（首注 / 更新分支各自的不变式）；
    顺带修了 `copy_codex_subset` 不复制 Codex 备份文件的保真度缺口
  - **真实机首测（用户真机同步，全部符合设计）**：断链只断 CLAUDE.md 侧
    （links=5 → 1+4，其余 4 路 md5 全等）、用户 1908 字节原文与备份完好、
    Codex 0 → 579 字节、L0 444 字节、真实目录 2949 项零改动
- 修改了文件：
  - 新建 `src/composables/useRuleEditor.ts`（读写状态机 + `RULE_TEMPLATES` + markdown-it 单例）
  - 新建 `src/components/RuleEditor.vue`（左右分栏编辑器）
  - 修改 `src/views/MainView.vue`（接入编辑器 + Toast；Tab 改 `show:lazy`）
  - 修改 `src/views/WizardView.vue`（步骤 3 模板改为引用共享常量，60 行字符串不再复制）
  - 修改 `src/App.vue`（`<n-message-provider>` 包住所有视图）
  - 修改 `src/style.css`（`.md-preview` 排版；v-html 内容不受 scoped 样式影响，必须放全局）
  - 修改 `package.json`（新增 `markdown-it 15.0.2` + `@types/markdown-it`）
  - 修改 `src-tauri/tests/ipc_contract.rs`（新增 2 条契约测试 + 术语检查扩围）
  - 修改 `src-tauri/examples/dryrun_sync.rs`（预状态自适应 + Codex 备份复制）
  - 修改 `docs/tech/TECH_STACK.md` / `docs/product/PRD.md`（依赖与交互细节同步）
- 注意事项：
  - **验证方式**：`pnpm build` 零报错；`cargo test` **120 passed / 0 failed / 0 warning**；
    干跑 **0 项未通过**（含真实目录 2949 项零改动）；新增文本断言全部反向验证过
  - **GUI 未做点击走查**（不擅自开窗口）：建议用户跑一次 `pnpm tauri dev` 目视确认，
    可并入 TASK-15
  - **`csp: null` 仍是纵深防御缺口**，建议独立小任务收紧
  - 发现用户 `~/.claude/CLAUDE.md` 第 39 行手写了一行带中文逗号的
    `@~/.ai-profile/AGENTS.md，`（在标记块外）——与标记块内的引用重复，
    建议用户自行清理；CrossBrain 不代改用户标记块外的内容

---

### 2026-09-15 00:10:45 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：**`TASK-19` 改动知情与可逆（P1 数据安全红线）—— 代码 + 测试 + 前端 + 文档全部落地**
  - **Part A 知情**：新增 `LinkNoticeDialog.vue`，首次同步前一次性告知「文件共享关系将被断开、
    内容一字不变」。闸门逻辑抽到 `src/composables/useLinkNotice.ts`（本项目首个组合式函数），
    向导步骤 4 与主工作台「立即同步」共用；**「已展示」标记写在用户确认之后**
  - **Part B 可逆**：新增 `list_backups` / `restore_backup` 两条命令 + 设置页「备份与还原」区
    + 二次确认弹窗。核心语义：`.crossbrain-backup` 是**唯一不可覆盖的干净快照**，
    还原前先把现场另存为 `*.crossbrain-before-restore`（仅当内容与备份不同时，因而幂等），
    还原写入同样走 `write_breaking_hardlink()` 保持断链语义
  - **架构**：`Adapter` trait 新增 `fn l0_backup(&self) -> Option<L0Backup>`（**默认 `None`**）——
    Antigravity 写独立文件、没有备份概念，返回 `NONE` 即自动从备份列表消失；Claude/Codex 覆盖它
- 修改了文件：
  - Rust：`adapters/mod.rs`（+`MARKER_PRE_RESTORE_SUFFIX`/`L0Backup`/`RestoreOutcome`/`pre_restore_path`/`restore_l0_backup`）、
    `adapters/{claude_code,codex}.rs`、`sync.rs`（+`BackupInfo` 与 4 个函数）、`state.rs`（+`link_notice_shown`）、
    `commands.rs`（+3 条命令）、`lib.rs`（注册）
  - 前端：`api.ts`、`composables/useLinkNotice.ts`（新）、`components/LinkNoticeDialog.vue`（新）、
    `views/WizardView.vue`、`views/MainView.vue`、`App.vue`、`components.d.ts`（重新生成）
  - 测试/干跑：`tests/ipc_contract.rs`、`examples/dryrun_sync.rs`（+第 [11] 节）
  - 文档：`docs/impl/TASK_BREAKDOWN.md`（+实施修正 8 条、+实施记录）、本文件三处、记忆
- 注意事项：
  - **验收数据**：`cargo test` **118 passed / 0 failed / 0 warning**；`dryrun_sync` **11 节 0 项失败**；
    `pnpm build` 零报错；真实数据零改动（5 路硬链接 md5 全等 `47645f60…`、links=5、
    `~/.codex/AGENTS.md` 仍 0 字节、**状态文件仍未创建**、用户技能完好）
  - **反向验证了一条契约测试**：把 `markLinkNoticeShown()` 挪进 `guard()` → 测试如期失败，
    确认 `link_notice_is_marked_only_after_confirm` 不是永不失败的空断言
  - **术语检查已扩围**：`wizard_copy_has_no_internal_terms` 从只扫向导扩到
    `WizardView.vue` / `MainView.vue` / `LinkNoticeDialog.vue` 三个模板
  - ⚠️ **遗留**：GUI 未做点击穿过验证（`cargo build` + 单测 + 干跑 + `vue-tsc` + 产物核对已做，
    但没真的开窗口点一遍）。可选：用户跑 `pnpm tauri dev` 目视确认

---

### 2026-09-14 23:53:35 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：**安全边界核查 → 新增 `TASK-19`（P1）与 `ADR-15`**
  - 触发：用户提问「我们的软件是否会破坏用户原有编程软件的用户偏好配置」
  - 核查方式：逐行读三个 Adapter 的全部写路径 + 共享层 `inject_marker_block` /
    `write_breaking_hardlink` / `cleanup_crossbrain_orphans` 实现，并对照真实目录实况
  - **结论（可对用户宣示）**：偏好配置类文件**零引用、无破坏路径** ——
    `~/.claude/settings.json`、`~/.codex/config.toml`、`~/.gemini/config/mcp_config.json`、
    `~/.gemini/config/AGENTS.md`、`~/.codex/rules/default.rules`、`skills/.system/`、
    `~/.agents/`（86 技能 / 59 MB）以及 backups/history/sessions 等工具内部数据全不碰。
    写入目标仅三类：L0 规则文件、L2 技能目录（仅 `crossbrain-*` 前缀）、`*.crossbrain-backup`
  - **发现缺口 1**：备份只写不读 → 无还原入口（`commands.rs` 6 条命令无 restore）
  - **发现缺口 2**：D-03 断链改变用户既有结构（links 5 → 1）却未在界面提示
  - 用户拍板：① 备份还原**立项补上** ② 断链**首次同步前告知**
- 修改了文件：
  - `docs/impl/DECISION_LOG.md` — 新增 **ADR-15**（对用户文件的改动必须「知情且可逆」；
    含明确不做的边界：不回滚整次同步、不自动还原、`.crossbrain-backup` 是唯一干净快照不得覆盖）
  - `docs/impl/TASK_BREAKDOWN.md` — 新增 **TASK-19** 完整任务定义
    （Part A 断链知情 + Part B 备份可见性与一键还原 + 可自动化验收清单）
  - 本文件 — 阶段表 / 下一个任务 / 增补任务节 / 本记录
  - `.workbuddy/memory/2026-09-14.md` — 记录核查结论与缺口
- 注意事项：
  - **没有改任何代码**。本轮只做核查 + 立任务。
  - TASK-19 里已写死两条硬约束：还原前先把当前版本另存为 `*.crossbrain-before-restore`
    （**不得覆盖** `.crossbrain-backup`）；还原写入**必须走** `write_breaking_hardlink()`。
  - 断链告知**不做硬链接数检测** —— Windows 上 Rust std 拿不到 links（Unix 专有 API），
    且断链不是 Claude 独有，故改用「首次同步前一次性通用告知 + state 标记控制」。

---

### 2026-09-14 23:44:41 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：**`TASK-18` 真实 Spike 闭环 —— Codex 三项实测全部通过，遗留清零**
  - 触发：用户贴回真实 Codex 会话的回答 —— 探针口令「紫色鳄鱼正在吃第七个饺子」
    被**原样复述**，且列出了它加载的指令来源
  - **方法突破**：从用户回答里发现 `~/.agents/skills/` 这一新路径 → 转为**二进制取证**
    （对 `D:\soft\win-x64\app\resources\codex.exe` 297 MB 做字符串检索），
    从而发现官方调试命令 **`codex debug prompt-input`**（dump 模型可见输入为 JSON）
    → 三项 Spike 全部变成「一条命令 + grep」，**不再需要人工开会话**
- **实测结果（三项全通过）**：① L0 —— 探针口令出现在 `<INSTRUCTIONS>` 块内；
  ② L2 —— 探针技能出现在 `<skills_instructions>` 清单，条目含 `name + description +
  (file: C:/Users/Administrator/.codex/skills/…)`；③ 作用域——在 `%TEMP%\cbspike`
  **中立目录**下执行仍被加载，确认为**全局**（L0 策略成立）
- **新发现（已写入 `SPIKE_RESULTS.md`）**：
  1. Codex 技能根共**四类**：`~/.codex/skills/.system/`（内置，禁碰）、
     `~/.codex/skills/{name}/`（**我们的 L2 落点，实测可见** ✅）、
     `~/.codex/plugins/cache/`（插件）、`~/.agents/skills/`（**第三方跨宿主库：
     86 个技能 / 59 MB，不得写入或清理**）
  2. 二进制证据（`codex-home/src/instructions/mod.rs` 只硬编 `AGENTS.override.md` /
     `AGENTS.md` 一对文件名）表明 Codex **不读** `~/.agents/AGENTS.md`（3349 字节的用户规则文件）
  3. `disable-model-invocation: true` 会让技能从清单中消失 —— 解释了用户自建
     `grill-me` 两处副本都不在清单里，**「磁盘上有、清单里没有」未必是落点问题**
- **探针清理（零残留）**：删除 `~/.codex/skills/crossbrain-spike-load/`、
  恢复 `~/.codex/AGENTS.md` 为 0 字节、`~/.agents/` 零改动、5 路硬链接 md5 全等
- 修改文件：
  - `docs/testing/SPIKE_RESULTS.md`（TASK-18 节改为「✅ 实测完成」，补方法 / 结果 /
    新发现 / 清理记录；V1.5 节补「先找 dump 命令」执行提示 + `~/.agents/` 资产警示）
  - `docs/impl/TASK_BREAKDOWN.md`（新增「TASK-18 Spike 完成（第三次）」全节，遗留清零）
  - `CURRENT_STATUS.md`（阶段表下一任务、Spike 清单加 Spike E、TASK-18 任务节、本记录）
- **结论**：TASK-18 **无未闭环遗留**；下一个主线任务为 `TASK-11` 全局规则编辑器

---

### 2026-09-14 23:33:28 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：**`TASK-18` 第二轮修正 —— 探测到 Codex 桌面应用后的命名与安全加固**
  - 触发：用户提示「codex ide 安装了啊，目录是 `D:\soft\win-x64\app`」
  - 实测确认：那是 **OpenAI 官方 Codex 桌面应用**（MSIX 包 `OpenAI.Codex`，DisplayName = `Codex`，
    版本 26.623.13972.0，自带 `resources/codex.exe` 297 MB），且**桌面 App / 内置 CLI /
    Chrome 扩展三者共用同一个 `~/.codex/`**（官方文档确认）→ 它是一款工具，不是两款
- **三处实质改动**：
  1. **显示名 `Codex CLI` → `Codex`**（`sync.rs` 的 `display_name`；`tool_id` 保持 `"codex"` 不变）
  2. **新增 `AGENTS.override.md` 遮蔽检测**：该文件优先级高于 `AGENTS.md` 且同级只取第一个
     非空文件，它一存在我们的规则就**完全不生效** → `CodexAdapter::reject_if_override_present()`
     在同步首步检测，命中即中止并报明确错误（`AdapterError::Other`，含路径与原因）
  3. **干跑新增第 [10] 节**：断言该错误能**端到端**走到同步报告（`status == Failed`、
     `report.ok == false`、且未写出 `AGENTS.md`）——只有它才能证明防线真实存在
- 修改文件：
  - `src-tauri/src/adapters/codex.rs`（模块文档补 App/CLI 同源 + override 风险；新增常量、
    `agents_override_file()`、`reject_if_override_present()`；`sync_l0_with_result()` 加首步检测；
    新增 **4 个**测试 → 该文件测试数 19 → 23）
  - `src-tauri/src/sync.rs`（`display_name` 更名）
  - `src-tauri/src/paths.rs`（`codex_dir()` 注释补 App 共用 + override）
  - `src-tauri/src/adapters/mod.rs`、`claude_code.rs`（注释统一为「Codex」）
  - `src-tauri/tests/adapter_consistency.rs`（注释与断言文案）
  - `src-tauri/examples/dryrun_sync.rs`（标签统一 + 新增第 [10] 节）
  - `src/api.ts`（注释）
  - `docs/product/PRD.md`（工具矩阵行改为 `Codex`；命名修正改为「两次」）
  - `docs/testing/SPIKE_RESULTS.md`（新增「官方文档已确认的部分」表；待 Spike 项 3 → 1）
  - `docs/impl/TASK_BREAKDOWN.md`（新增「补充修正（第二次）」全节 + 第二轮实施记录）
  - 本文件 `CURRENT_STATUS.md`（阶段表、任务状态、本次记录）
- 实测结果：`cargo test` **110 passed / 0 failed**（0 warning，较上轮 +4）；
  `dryrun_sync` **10 节 0 项失败**；`pnpm build` 零报错
- 真实数据安全回归：5 个硬链接目标 md5 全等 `47645f60…`、`~/.codex/AGENTS.md` 仍
  **0 字节 / links=1**、`AGENTS.override.md` **不存在**、用户技能完好、无新增 crossbrain 残留
- 注意事项：
  1. **`AGENTS.override.md` 一律按「存在即中止」处理，不特判空文件**。「跳过空文件」是
     Codex 的实现细节；空 override 恰恰说明用户正要用它，放行等于埋「某天写一行 → 规则无声失效」的雷。
  2. **不要为「App」和「CLI」各做一个 Adapter**——它们共用 `~/.codex/`，分开做就是同一份内容写两遍。
  3. ⏳ **真实 Spike 收窄为 1 项**：作用域（全局）与 skills 懒加载已由官方文档解答，
     仅剩「写入真实 `~/.codex/AGENTS.md` 后 Codex 会话能否读到」需用户配合实测。

---

### 2026-09-14 23:04:33 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：**`TASK-18` Codex CLI Adapter 接入**（增补任务，用户触发）
  - 用户反馈向导只识别 2 个工具 → 探测本机 6 款 AI 工具的真实落点 → 用户拍板先接 Codex
  - **支持工具数 2 → 3**（Claude Code / Antigravity IDE / Codex CLI）
- 新建文件：
  - `src-tauri/src/adapters/codex.rs`（`CodexAdapter` 完整实现 + 19 个测试）
- 修改文件：
  - `src-tauri/src/adapters/mod.rs`（**共享标记块协议抽取**：常量、`marker_regex()`、
    `has_marker_block()`、`ensure_no_marker_in_content()`、`write_breaking_hardlink()`、
    `SyncL0Result`、`inject_marker_block()` + 6 个协议测试）
  - `src-tauri/src/adapters/claude_code.rs`（删除本地副本，改用共享实现）
  - `src-tauri/src/paths.rs`（新增 `codex_dir()`）
  - `src-tauri/src/sync.rs`（`build_tools()` 注册第三个工具）
  - `src-tauri/tests/adapter_consistency.rs`（对比列表扩为三个 Adapter）
  - `src-tauri/examples/dryrun_sync.rs`（覆盖 Codex 全流程 + `copy_codex_subset()`）
  - `src/api.ts`（`UPCOMING_TOOLS` 移除 `"ChatGPT IDE"`）
  - `docs/product/PRD.md`（工具矩阵：Codex 提入 V1、修正「ChatGPT IDE」命名）
  - `docs/testing/SPIKE_RESULTS.md`（新增「待接入工具路径预探测」+「TASK-18 待完成 Spike」）
  - `docs/impl/TASK_BREAKDOWN.md`（新增 TASK-18 全节 + 实施修正 5 条 + 实施记录）
  - 本文件 `CURRENT_STATUS.md`（阶段表、任务状态、本次记录）
- 实测结果：`cargo test` **106 passed / 0 failed**（0 warning）；`cargo build --examples` 通过；
  `dryrun_sync` **0 项失败**；`pnpm build` 零报错
- 真实数据安全回归：5 个硬链接目标 md5 全等 `47645f60…`、`CLAUDE.md` links=5、
  `~/.codex/AGENTS.md` 仍 0 字节、用户技能全部完好、无 crossbrain 残留
- 注意事项：
  1. **Codex 的 L0 与 Claude 是两套语义**：Claude 写 `@~/.ai-profile/AGENTS.md` 引用，
     Codex **内联规则全文**（Codex 不认识 `@` 语法）。`sync_l0(rules_content)` 同一签名
     在两侧含义不同，是刻意的，不要为了「统一」而删参数。
  2. **内联带来注入风险**：规则内容若含 `<!-- CrossBrain:End -->` 会提前闭合标记块。
     已在 `CodexAdapter::build_marker_block()` 拦截并报错（Claude 侧无需此检查，内容固定）。
  3. **`marker_regex()` 原先硬编码了标志串**（与常量重复），本轮改为从常量拼接——
     否则改常量时正则不跟，判定与替换会同时静默失效。
  4. ⏳ **真实 Spike 未做**：已验证「代码按预期写入副本」，**未验证**「Codex 真能读到
     `~/.codex/AGENTS.md`」。见 `docs/testing/SPIKE_RESULTS.md` 的待办表。
  5. 本机 `%TEMP%\crossbrain-setup\` 有 TASK-03 装 Rust 时留下的安装日志与
     `rustup-init.exe`（约 68 MB），**未擅自清理**，待用户确认。

---

### 2026-09-14 22:37:41 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：
  1. `TASK-10` 首次启动 5 步向导 UI + **整个 IPC 层**（10/10 验收项通过）
  2. **ADR-11 拍板**：UI 侧展示「文件名 → 目录名」映射（用户选择，不引拼音、不改规格）
  3. **TASK-09 暂缓**（用户决策：本机尚未为 `~/.ai-profile/` 建立 git 仓库）
- 新建文件：
  - Rust：`src-tauri/src/commands.rs`（6 个命令）/ `sync.rs`（编排 + DTO）/ `state.rs`（跨启动状态）
  - 测试与工具：`src-tauri/tests/ipc_contract.rs`（4 个跨语言契约测试）、
    `src-tauri/examples/dryrun_sync.rs`（真实结构副本干跑）
  - 前端：`src/api.ts`、`src/views/WizardView.vue`、`src/views/MainView.vue`
- 改写文件：`src/App.vue`（替换 TASK-02 的技术验证占位页）
- 修改文件：
  - `src-tauri/src/paths.rs`（新增 `STATE_FILE_NAME` 常量与 `state_file()`）
  - `src-tauri/src/init.rs`（`.gitignore` 由「写死内容」改为「幂等逐行补齐」）
  - `src-tauri/src/lib.rs`（注册 6 个命令、移除脚手架遗留的 `greet`、`paths` 改 `pub`）
  - `src-tauri/tauri.conf.json`（窗口 800×600 → 1024×720，并加最小尺寸）
  - 本文件 `CURRENT_STATUS.md`（阶段表、任务状态、本次记录）
  - `docs/impl/TASK_BREAKDOWN.md`（TASK-10 实施修正 8 条 + 实施记录 + TASK-09 暂缓注记）
  - `docs/product/PRD.md`（§6.2 落实 ADR-11 的展示要求）
  - `docs/impl/DECISION_LOG.md`（ADR-11 决策确认 + 新增 ADR-12 / ADR-13 / ADR-14）
- 实测证据：
  - `cargo test` → **82 passed; 0 failed**，0 error 0 warning
    （lib 70 + Adapter 一致性 7 + IPC 契约 5）
  - `pnpm build`（含 `vue-tsc --noEmit`）→ **零报错**
  - `cargo run --example dryrun_sync` → **20 项检查全绿**，
    含「两个 `crossbrain-spike-test` 孤儿被正确清理」与「真实目录 **2885 项快照零改动**」
  - `pnpm tauri dev` → 探针确认前端实际调用了 `get_startup_state` 与
    `detect_tools`（返回 2 项）——IPC 链路通、向导已渲染
  - **真实数据零改动**：5 个硬链接目标 md5 全等 `47645f60…`、`CLAUDE.md` 链接数仍为 **5**
- 注意事项：
  1. **⚠️ TASK-10~14 全都漏了同一个前置：IPC 层**。任务清单里没有任何一条交代
     「前端怎么调用 Rust」。已新增 `commands.rs`（6 个命令）+ `sync.rs`（编排）+ `state.rs`。
     后续新增任何 UI 能力，先确认对应的 command 是否已存在。
  2. **⚠️ Tauri 的 IPC 是字符串寻址**：命令名 / 事件名 / 字段名写错，两侧编译**都会通过**
     （`cargo test` 与 `pnpm build` 双双成功），只在运行期失败，且表现为「按钮没反应」
     这类最难排查的形态。已用 `tests/ipc_contract.rs` 把两侧**真实源码**交叉比对锁死。
  3. **⚠️ 同步的删除语义决定了「失败即中止」**（ADR-14）：`cleanup_orphans` 收到空列表
     等于删光全部技能目录，因此 SSOT 读取失败**绝不能**降级为「当作空」。
     这是本任务最重要的安全约束，日后改同步流程必须保持。
  4. **`run_sync` 必须 `async` + `spawn_blocking`**：否则主线程被文件 IO 占住，
     前端收得到进度事件也渲染不出来，「逐行实时显示」会退化成假实时。
  5. **向导触发条件不能只看 `rules.md`**：用户可一字不写就点「跳过，直接进入」，
     该条件将永远成立。已加 `~/.ai-profile/.crossbrain-state.json` 记 `wizard_completed`，
     它同时是 TASK-13 状态栏「上次同步时间」的数据源。
  6. **`init.rs` 的 `.gitignore` 改为幂等逐行补齐**：状态文件需要被忽略，
     但那是用户的文件，覆盖写入会删掉用户自己加的忽略规则；
     条目齐全时**完全不写盘**，以保证 TASK-03 的「二次启动 mtime 零变化」断言仍成立。
  7. **干跑工具自身出过两个假断言（已修）**：spike 孤儿的存在性在清理**之后**才检查
     （于是永远为真）；硬编码 4 个技能名（真实机器每个目录只装 2 个 → 假失败）。
     现改为「跑前采样、跑后比对差异」。
  8. **向导视觉效果需人工确认**：TASK-02 记录的「Tailwind preflight 与 Naive UI
     样式优先级冲突」是待观察项，界面我无法目视。请在本机 `pnpm tauri dev` 看一眼。
  9. **TASK-09 暂缓的影响面**：`sync.rs` 已预留 git 提交的调用位置与注释，
     接入时不需要改编排结构；但 TASK-13 的「含 git commit」验收项在 TASK-09 完成前无法勾选。

---

### 2026-09-14 22:17:23 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：
  1. `TASK-07` 实现 Slug-Hash 生成算法（`slug.rs`，6/6 验收项通过）
  2. `TASK-08` 孤儿清理一致性验证（`tests/adapter_consistency.rs`，4/4 验收项通过）
- 新建文件：
  - `src-tauri/src/slug.rs`（`generate_parts() -> Slug` / `generate()` / 18 个单元测试）
  - `src-tauri/tests/adapter_consistency.rs`（7 个集成测试：同一组输入同时驱动两个 Adapter）
  - `src-tauri/examples/dryrun_slug.rs`（验收脚本，开发期工具，不参与发布产物）
- 修改文件：
  - `src-tauri/src/adapters/mod.rs`（`SLUG_PREFIX` 改为转出自 `crate::slug`，单一真实源）
  - `src-tauri/src/lib.rs`（挂载 `pub mod slug;`）
  - `src-tauri/Cargo.toml`（新增 `sha2 = "0.10"`）
  - `docs/impl/TASK_BREAKDOWN.md`（TASK-07「实施修正」6 条 + 两份实施记录）
  - `docs/tech/ARCHITECTURE.md`（第 4 节新增「实现说明」，澄清不做拼音）
  - `docs/tech/ADAPTER_SPEC.md`（第 4 节 `name` 字段说明改为明确不含命名空间前缀）
  - `docs/testing/TEST_PLAN.md`（T1 补 T1-5 边界表、T4 补一致性用例映射）
  - 本文件 `CURRENT_STATUS.md`
- 实测证据：
  - `cargo test` → **63 passed; 0 failed**，0 error
    （lib 56 = TASK-06 的 38 + slug 18；集成 7）
  - `cargo run --example dryrun_slug` → **全部通过**，含外部交叉验证：
    `sha256("test")` 前 6 位 `9f86d0`、`sha256("")` 前 6 位 `e3b0c4`，
    与本程序输出**逐位一致**（证明是标准 SHA-256）
  - **真实数据零改动**：5 个硬链接目标 md5 仍全等（`47645f60…`）、
    `CLAUDE.md` 链接数仍为 **5**，用户技能目录全部完好
- 注意事项：
  1. **原 TASK-07 示例代码有 6 处需修正**，已在 TASK_BREAKDOWN 的「实施修正」中逐条写明。
     最关键的一条：**ARCHITECTURE 第 4 节的示例含人工拼音折算**（`vue3-perf`），
     V1 不做拼音，实际输出是 `crossbrain-vue3-{hash}`。TEST_PLAN T1-1 的期望输出已同步修正。
  2. **`strip_md_extension()` 的字节切片必须用 `is_char_boundary` 守卫**：
     `"优化"` 是 6 字节，`len - 3 = 3` 恰好落在第二个汉字中间，直接切片会 panic。
     已加专门用例踩这个点——这是纯中文环境下最容易漏的崩溃点。
  3. **hash 必须用 SHA-256，不能用 `DefaultHasher`**：后者的输出不保证跨 Rust 版本稳定，
     会导致升级工具链后**全量技能目录被判为孤儿而删除重建**。内容寻址的标识符必须可复现。
  4. **`SLUG_PREFIX` 的单一真实源移到 `slug.rs`**（原在 `adapters/mod.rs`）：
     前缀是 slug 的形态本身，Adapter 侧只是引用。已加 4 条跨模块契约测试——
     若两侧分叉，表现是**运行时** `sync_l2` 全部失败，编译期发现不了。
  5. **新增 `generate_parts() -> Slug`（文档未要求）**：frontmatter 的 `name` 需要
     「不含 `crossbrain-` 前缀」的 kebab 段，而目录名需要完整标识符；若只交付一个字符串，
     上层只能靠字符串切割反推，而 kebab 段自身含 `-`，切割位置无从判断。
  6. **⚠️ 已知局限（不阻塞验收，待产品侧决策）**：纯中文文件名的 kebab 段退化为 `item`，
     目录名如 `crossbrain-item-7a6883`，人工排障时看不出对应哪个知识点。
     功能无影响（内容不同则 hash 不同）。推荐方案：UI 侧始终展示「文件名 → 目录名」映射。
  7. **⚠️ 本机已存在 2 个 `crossbrain-spike-test` 孤儿目录**
     （`~/.claude/skills/` 与 `~/.gemini/config/skills/`），TASK-13 首次真实同步时
     它们会被自动清理——这是**正确行为**，但需在 TASK-10 向导里告知用户。
  8. 编译仅剩 1 个预期 warning（`agents_md_file`），由 TASK-13 生成 AGENTS.md 时消耗。

---

### 2026-09-14 22:03:23 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：
  1. **决策 D-03 拍板**：`~/.claude/CLAUDE.md` 写入策略 = **断开硬链接 / 原子替换**（用户选择）
  2. `TASK-06` 实现 `ClaudeCodeAdapter`（标记块四情况协议 + 备份机制 + 断链写入）
- 新建文件：
  - `src-tauri/examples/dryrun_claude.rs`（真实 5 链接结构副本干跑，开发期工具，不参与发布产物）
- 改写文件：
  - `src-tauri/src/adapters/claude_code.rs`（从占位模块 → 完整实现 + 19 个测试）
- 修改文件：
  - `src-tauri/src/adapters/mod.rs`（新增共享 `SLUG_PREFIX` / `ensure_crossbrain_slug()` / `cleanup_crossbrain_orphans()`）
  - `src-tauri/src/adapters/antigravity.rs`（改用上述共享实现，两个 Adapter 不再各持一份）
  - `src-tauri/Cargo.toml`（新增 `regex = "1"`）
  - 本文件 `CURRENT_STATUS.md`（D-03 定稿、进度更新、本记录）
  - `docs/impl/TASK_BREAKDOWN.md`（TASK-06 实施修正 + 实施记录）
  - `docs/tech/ADAPTER_SPEC.md`（3.2 节补充硬链接隔离写入策略）
- 实测证据：
  - `cargo test` → **38 passed; 0 failed**，0 error（TASK-05 的 19 个 + 本次新增 19 个）
  - `cargo run --example dryrun_claude` → **干跑通过**。在真实 5 链接结构的副本上跑完
    情况三 → 情况二 → 情况四 → 孤儿清理，**其余 4 个工具路径全程一字未变**，
    且「就地覆写 CLAUDE.md 不再牵动兄弟路径」证明断链确实生效
  - **真实数据零改动**：运行前后 5 个文件 md5 全等（`47645f60…`）、
    `CLAUDE.md` 链接数仍为 **5**（`ls -la` 实测），无 `.crossbrain-backup` / `.crossbrain-tmp` 残留
- 注意事项：
  1. **⚠️ D-03 是本任务的核心约束，已写死在代码里**：`CLAUDE.md` 的**唯一**写入通道是
     `write_breaking_hardlink()`（tmp 落盘 → `remove_file` 断链 → `rename` 原子替换）。
     四情况全部经它，**没有例外**。日后新增任何写 `CLAUDE.md` 的代码，都必须走这里。
  2. **`std` 的 `file_index()` / `number_of_links()` 是 unstable**（feature `windows_by_handle`），
     稳定通道编译不过（`error[E0658]`）。已改用**行为验证**替代读 inode：
     就地覆写一处、看另一处内容是否跟着变——这比读 inode 更硬，因为它直接证明了
     「就地覆写会互相污染」这个我们真正要防的后果。
  3. **情况二的判定改用正则 `is_match`**（原文档代码用 `contains(Start) && contains(End)`）：
     后者在「只有 Start 没有 End」或标志乱序时也判真，于是进了情况二分支却替换不到，
     文件该改而没改、还静默返回成功。现在判定与替换共用同一个正则，保证「进分支即替换成功」。
     替换时用 `regex::NoExpand` 而非普通替换——避免标记块文本里的 `$1` 之类被当作捕获组展开。
  4. **孤儿清理与命名空间校验提升到 `adapters/mod.rs` 共享**：TASK-08 要求
     「两个 Adapter 清理行为一致」，靠两份各自演进的代码「保持」一致不可靠，共用一份才是
     结构上不会分叉。Antigravity 侧的 13 个测试重跑通过，确认重构无回归。
  5. **情况三的备份先于修改**（顺序反了备份的就是改后的内容）；`fs::copy` 创建新文件、
     不复制硬链接，所以备份与原件天然独立——已加测试断言。
  6. **Plan B（`@~` 展开失败降级）V1 不实现**：Spike A 已实测 Windows 下 `@~` 可用，
     触发条件不存在。`sync_l0_with_result` 仍保留 `rules_content` 参数作为入口，
     但**不造「永远触发不到、也测不到」的死代码**。
  7. `~/.claude/` 下**没有** `.crossbrain-backup` 文件——说明此前从未有工具注入过标记块，
     TASK-06 是该文件的第一次改动（且至今未实际执行，只跑了副本）。

---

### 2026-09-14 21:53:18 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：`TASK-05` 实现 `AntigravityAdapter`（13/13 验收项通过）
- 新建文件：
  - `src-tauri/src/adapters/antigravity.rs`（`detect` / `sync_l0` / `sync_l2` / `cleanup_orphans` 完整实现 + 9 个测试）
  - `src-tauri/examples/dryrun_antigravity.rs`（真实数据安全干跑工具）
- 修改文件：
  - `src-tauri/src/adapters/mod.rs`（新增共享 `format_skill_md()` + 4 个测试；trait 的 `sync_l2` 文档指向该函数）
  - `docs/impl/TASK_BREAKDOWN.md`（TASK-05 新增「实施修正」4 条、验收清单改写为可自动化的 13 项、补实施记录）
  - 本文件 `CURRENT_STATUS.md`（含新增 **D-03** 决策项）
- 实测证据：
  - `cargo test` → **19 passed; 0 failed**，0 error（TASK-04 的 6 个 + 新增 13 个）
  - `cargo run --example dryrun_antigravity` → **干跑通过**。在真实 `~/.gemini/config` 的
    临时副本上执行 `cleanup_orphans(&[])`（最激进场景）后：用户的
    `design-taste-frontend`、`grill-me`、`rules/user_global.md` **全部完好**；
    真实 `~/.gemini/config` 目录清单**逐一比对一致，零改动**
- 注意事项：
  1. **原文档的验收方式（在真实目录上「手动测试」删除）不可执行**——本机
     `~/.gemini/config/skills/` 下真实存在用户自建的 `grill-me`、`design-taste-frontend`。
     已加 `base_dir` 注入，所有测试一律跑在临时目录。
  2. **`sync_l2` 补了命名空间校验**（原文档没有）：`slug_hash` 直接拼进目录名，
     不校验就会用 `grill-me` 这类名字**覆盖用户技能**。现在要求必须以 `crossbrain-`
     开头且不含 `/`、`\`、`..`。这与清理时「只删 `crossbrain-` 前缀」是同一条边界。
  3. **`cleanup_orphans` 补了非目录项跳过**：`remove_dir_all` 对同名普通文件会失败；
     顺带使符号链接也被排除（`file_type()` 对 symlink 返回的 `is_dir()` 为 false），
     不会顺着链接误删到目标目录。
  4. **frontmatter 拼装此前无任务覆盖**（原文档只说「由调用方负责」，但那个调用方不存在），
     已在 `adapters/mod.rs` 补 `format_skill_md()`，落 `ADAPTER_SPEC` 第 4 节规范。
     中文全角冒号 `：` **不加引号**（与 Spike C 实测形态一致），半角 `: ` 才加引号。
  5. **⚠️ 发现 D-03**：本机 `~/.claude/CLAUDE.md` 通过硬链接与另外 4 个路径共享同一 inode，
     用户实际上已用硬链接实现了「一份内容、5 个工具生效」。
     这直接决定 TASK-06 的写入策略，**开工前必须先确认**。
  6. 遗留 warning 降到 2 个（`agents_md_file` / `claude_dir`），
     分别由 TASK-06 / TASK-09 使用，属预期。

---

### 2026-09-14 21:39:23 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：`TASK-04` 定义 `Adapter` trait 统一接口（7/7 验收项通过）
- 新建文件：
  - `src-tauri/src/adapters/mod.rs`（`Adapter` trait：四个必需方法 + `user_friendly_error` 默认实现；`CleanupReport`；`AdapterError` 5 个变体；含 5 个契约测试）
  - `src-tauri/src/adapters/antigravity.rs`（占位模块，含目标路径对照表）
  - `src-tauri/src/adapters/claude_code.rs`（占位模块，含标记块协议警告）
- 修改文件：
  - `src-tauri/src/lib.rs`（挂载 `pub mod adapters;`）
  - `docs/impl/TASK_BREAKDOWN.md`（TASK-04 新增「实施修正」注记、验收清单补 3 条、补「实施记录」小节）
  - 本文件 `CURRENT_STATUS.md`
- 实测证据：
  - `cargo build` → **退出码 0**，0 error，产物 `target/debug/crossbrain.exe`（12,884,480 bytes）
  - `cargo test` → **6 passed; 0 failed**（5 个新增接口契约测试 + TASK-03 的幂等测试）
- 注意事项：
  1. **原步骤「只创建 `mod.rs`」跑不通**：`mod.rs` 里的 `pub mod antigravity;` / `pub mod claude_code;`
     声明要求对应文件存在，否则 `cargo build` 报 `file not found for module`——
     与验收清单第 7 条「编译通过」**直接冲突**。已同时创建两个占位文件。
  2. **必须在 `lib.rs` 挂载 `pub mod adapters;`**：否则 `adapters/` 只是磁盘上的孤儿目录、
     **根本不参与编译**，`cargo build` 照样返回成功，但验收第 1~6 条全是空转。
     这是本任务最容易漏掉的一步。
  3. **trait 补了 `Send + Sync` 约束**（原文档未写）：Adapter 实例要放进 Tauri `State`，
     并在 command handler（可能运行在其他线程）中被调用。无状态实现天然满足，成本为零；
     若拖到状态管理任务才补，将被迫修改这个已被多处依赖的契约。
  4. **`CleanupReport` / `AdapterError` 补派生 `PartialEq` / `Clone` / `Eq`**（原文档只写 `Debug`）：
     TASK-05 / TASK-06 的 `cleanup_orphans()` 测试需要精确比对目录列表，缺 `PartialEq` 无法写断言。
  5. 编译仍有 3 个 `never used` warning（`agents_md_file` / `gemini_config_dir` / `claude_dir`），
     正是 TASK-05 / TASK-06 要用的预留路径函数，属**预期**。

---

### 2026-09-14 21:34:12 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：`TASK-03` 封装 Rust 路径模块 + 初始化 `~/.ai-profile/` SSOT 目录底座（10/10 验收项通过）
- 新建文件：
  - `src-tauri/src/paths.rs`（`profile_root` / `global_rules_file` / `knowledge_dir` / `agents_md_file` / `gemini_config_dir` / `claude_dir`，共 6 个函数；内部统一走一个私有 `home_dir()`，只处理一次失败分支）
  - `src-tauri/src/init.rs`（`ensure_profile_dirs()`，幂等；含 `#[cfg(test)]` 单元测试）
- 修改文件：
  - `src-tauri/src/lib.rs`（新增 `mod init; mod paths;`，在 `tauri::Builder` 之前调用 `init::ensure_profile_dirs()`，失败仅 `eprintln!` 不 panic）
  - `src-tauri/Cargo.toml`（新增 `dirs = "5"` 与 `chrono = { version = "0.4", features = ["serde"] }`，均带铁律出处注释）
  - `docs/impl/TASK_BREAKDOWN.md`（TASK-03 步骤 4 改写为 `lib.rs` 入口；验收清单新增 `.gitignore` 项；补充依赖版本与 os error 5 注记）
  - `docs/impl/DEV_SETUP.md`（目录结构补 `paths.rs` / `init.rs` 并标注入口是 `lib.rs`；踩坑复盘新增坑 4「os error 5」、坑 5「cargo 国内镜像」；首次编译耗时由 2~5 min 更正为实测 12 min）
  - 本文件 `CURRENT_STATUS.md`
- 实测证据：
  - `cargo build` → **退出码 0**，0 error（3 个 `never used` warning，属预期，路径函数为后续任务预留）
  - `cargo test` → **1 passed; 0 failed**（测试内连续调用两次 `ensure_profile_dirs()`，幂等成立）
  - `pnpm tauri dev` 第 1 次 → 应用启动后自动创建 `~/.ai-profile/.gitignore`、`global/rules.md`、`knowledge/`
  - `pnpm tauri dev` 第 2 次 → **0 error**，所有文件 mtime 仍为 `21:30:33`、md5 完全一致（**零重写**，幂等实测）
  - 起始状态已存在的 `~/.ai-profile/AGENTS.md`（Spike A 遗留）**全程未被覆盖**，md5 前后一致
- 注意事项：
  1. **Rust 侧入口是 `lib.rs` 不是 `main.rs`**。Tauri v2 官方模板把逻辑放在 `lib.rs`，`main.rs` 只有 `fn main() { crossbrain_lib::run() }` 一行（移动端入口固定结构）。原文档 TASK-03 步骤 4 写的是改 `main.rs`，已修正。
  2. **首次 `cargo build` 失败过一次**：`proc-macro2` 构建脚本清理临时目录时报
     `拒绝访问 (os error 5)`。**根因不是杀软**（本机 Defender 实时保护本来就关闭、也无第三方杀软），
     而是 Windows 上刚执行过的 exe 句柄释放延迟。清理 `target/debug/build/proc-macro2-*` 后重试即过。
     同类错误还会以 warning 形式出现（增量元数据复制），属本机系统性现象，**遇错先怀疑它、不要先怀疑代码**。
  3. **新配置了用户级 `~/.cargo/config.toml`**（新建文件，未覆盖任何已有配置）：直连 crates.io 实测仅 ~170 KB/s，
     而依赖树有 484 个 crate，故改用清华 sparse 镜像 + 直连（实测 rustup / cargo 走代理反而卡死）。
     删除该文件即可恢复官方源。
  4. **`.gitignore` 是超出原验收清单的补充**：依据 `docs/tech/ARCHITECTURE.md` 第 2 节，
     SSOT 的 `.gitignore` 是目录结构固定组成部分（排除 `AGENTS.md` 这个动态中间产物），
     且 TASK-09（Git 封装）并不负责创建它，故在「初始化目录底座」阶段一并创建。
  5. **`dirs` 锁 5.x**（实际最新 7.x）。验收清单明确要求 `dirs = "5"`，且 5.x 的 `home_dir()` 已实测定稿。
  6. 已知非阻塞现象：`cargo build` 有 3 个 `never used` warning（`agents_md_file` / `gemini_config_dir` / `claude_dir`）。
     它们是 TASK-05 / TASK-06 的 Adapter 要用的，届时自然消除，**不要为了消 warning 去加 `#[allow(dead_code)]`**。

---

### 2026-09-14 21:03:23 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：安装 Rust 工具链 + VS Build Tools（解除 BLOCKER-01，TASK-03 起可开工）
- 环境安装结果：
  - Rust **1.98.1**（`stable-x86_64-pc-windows-msvc`，profile = minimal）
  - VS Build Tools 2022：**MSVC 14.44.35207**（`cl.exe` / `link.exe`）+ **Windows SDK 10.0.26100.0**
  - `vswhere.exe` 就位；`~/.cargo/bin` 已写入用户 PATH
  - 真实编译链接测试：`rustc hello.rs` → 148,992 字节 exe，运行输出正常
- 修改文件：
  - 修改 `CURRENT_STATUS.md`（BLOCKER-01 关闭、D-02 补全实测结果、新增「开发环境」完成项、本记录）
  - 修改 `docs/impl/DEV_SETUP.md`（将预估步骤替换为实测安装记录 + 踩坑复盘，见下方）
  - 未改动任何项目源码
- 注意事项（**三个坑，都已解决，务必看 DEV_SETUP.md**）：
  1. **VS 安装器 5008/5002 的真正根因是本机环境变量存在大小写重复**
     （`http_proxy`+`HTTP_PROXY`、`https_proxy`+`HTTPS_PROXY`、`Path`+`PATH`）。
     VS 安装器用忽略大小写的 .NET 字典装载环境变量，遇到重复键直接抛
     `0x80070057 已添加项` 并在启动 `setup.exe` 前崩溃。
     **解法**：启动子进程前在进程内把所有大小写重复项去重（本次去掉了
     `HTTPS_PROXY / http_proxy / Path` 三个冗余项），去重后一次通过。
     注意 `Path`(853字符) 与 `PATH`(962字符) 值不同，保留的是 `PATH`（超集）。
  2. **另有一个 VS 安装器自身缺陷**：bootstrapper 需先自我更新 installer，
     首次运行会停在「Unexpected Installer Version」。解法是先
     `curl https://aka.ms/vs/install/latest/installer` 下载最新 `vs_installer.opc`，
     再用 `vs_BuildTools.exe --quiet --wait --update --offline <opc>` 把 installer 本体更新到位，
     最后才跑真正的安装（不要加 `--nocache`，会把下载的 opc 清掉）。
  3. **rustup 走代理会卡死**（实测下载 60 秒零增量）。改用国内镜像 + 直连（不用代理）后速度正常：
     `RUSTUP_DIST_SERVER=https://mirrors.tuna.tsinghua.edu.cn/rustup`。
     另外 **default profile 里的 `rust-docs` 在 Windows 上解压极慢**
     （数万个小文件 + Defender 扫描，实测 0.1 MB/s），
     需先 `rustup set profile minimal` 再装，否则会卡很久。
  4. 附注：`where link.exe` 会命中 PortableGit 自带的同名 coreutils 工具，
     这是无害的名义冲突——rustc 通过 vswhere/注册表定位 MSVC linker，不受 PATH 影响
     （已用真实链接测试证明）。

---

### 2026-09-14 20:10:23 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：`TASK-02` 配置 Tailwind CSS v4 + Naive UI + SVG 图标链路（13/13 验收项通过）
- 修改/新建文件：
  - 新建 `src/style.css`（`@import "tailwindcss";` + `@theme` 设计令牌：brand 色阶 50~900、success/warning/error）
  - 改写 `src/main.ts`（引入 `./style.css`，Naive UI 走自动导入不做全局注册）
  - 改写 `src/App.vue`（移除 Tauri 官方 demo，替换为无内部术语的占位页，同时验证 Tailwind + Naive UI `n-button` + lucide 图标三者打通）
  - 改写 `vite.config.ts`（新增 `tailwindcss()`、`Icons()`、`Components({ resolvers: [NaiveUiResolver()], dts: 'src/components.d.ts' })`）
  - 改写 `src/vite-env.d.ts`（加入 `unplugin-icons/types/vue` 引用；**删除模板自带的 `declare module "*.vue"` 兜底声明**）
  - 自动生成 `src/components.d.ts`（unplugin-vue-components 产出的组件类型声明）
  - 修改 `docs/impl/TASK_BREAKDOWN.md`：TASK-02 整节改写为 Tailwind v4 写法，新增 v3↔v4 对照表、步骤 8（TS 类型引用）、13 项 v4 版验收清单
  - 修改 `docs/tech/TECH_STACK.md`：Tailwind `v3.x` → `v4.x`、TS 版本补实测值、pnpm 版本补实测值、图标方案补版本约束
  - 修改 `CURRENT_STATUS.md`（含新增「已决策事项」章节）
- 注意事项：
  1. **Tailwind v4 与文档原内容完全不兼容**，已按决策 D-01 整节改写。v4 不需要 `postcss`/`autoprefixer`/`tailwind.config.js`/`postcss.config.js`，也没有 `tailwindcss init -p` 命令。已在验收清单中把「tailwind.config.js 必须存在」反向改为「**不应存在**」。
  2. **模板自带的 `declare module "*.vue"` 是铁律 L-04 的违规点**：它把组件类型擦成 `DefineComponent<{}, {}, any>`。vue-tsc/Volar 原生解析 `.vue`，该兜底声明纯属有害，已删除。删后 `pnpm build` 通过。
  3. **`~icons/*` 类型需要显式引用**：不加 `/// <reference types="unplugin-icons/types/vue" />` 会报 `TS2307: Cannot find module '~icons/lucide/sparkles'`。
  4. **首次构建存在顺序陷阱**：`pnpm build` = `vue-tsc --noEmit && vite build`，而 `src/components.d.ts` 要等 `vite build` 才生成。首次跑 `pnpm build` 前建议先单独跑一次 `npx vite build` 生成该文件。
  5. **图标集改为本地安装**（`@iconify-json/lucide`），不再依赖 `Icons({ autoInstall: true })` 的按需联网下载，符合「零网络依赖」定位。已在 TASK_BREAKDOWN 与 TECH_STACK 中写明。
  6. 已知待观察项：Tailwind preflight（CSS reset）与 Naive UI 自带样式存在潜在优先级冲突，目前 `n-button` 渲染正常，后续 TASK-10 做真实界面时需重点回归。

---

### 2026-09-14 19:54:12 — WorkBuddy（DeepSeek-V4.1-Flash）

- 完成了：`TASK-01` 初始化 Tauri v2 + Vue 3 + TypeScript 项目骨架（6/6 验收项通过）
- 修改/新建文件：
  - 新建 `package.json`（`name: crossbrain`，补 `packageManager: pnpm@9.15.9`）
  - 新建 `pnpm-lock.yaml`、`index.html`（标题改 CrossBrain）、`vite.config.ts`、`tsconfig.json`、`tsconfig.node.json`、`.gitignore`、`.vscode/`
  - 新建 `src/`（`main.ts`、`App.vue`、`vite-env.d.ts`、`assets/vue.svg`）、`public/`（`vite.svg`、`tauri.svg`）
  - 新建 `src-tauri/`（`Cargo.toml`、`tauri.conf.json`、`build.rs`、`src/main.rs`、`src/lib.rs`、`capabilities/default.json`、`icons/`）
  - 修改 `src-tauri/tauri.conf.json`：`productName` 与窗口 `title` 由 `crossbrain` 改为 `CrossBrain`
  - 修改 `docs/impl/TASK_BREAKDOWN.md`：修正 TASK-01 步骤 2 命令（原 `--app-name` / `--no-open-browser` 参数不存在，会直接失败）、Node 版本改为 18/20/22 LTS、新增第 5 项前置检查（WebView2）
  - 修改 `docs/impl/DEV_SETUP.md`：修正创建命令、新增「Rust toolchain 安装」与「WebView2 Runtime」章节、FAQ Node 版本同步为 18/20/22
  - 修改 `CURRENT_STATUS.md`
- 注意事项：
  1. **文档里的创建命令是错的**。`create-tauri-app@4.7.4` 无 `--app-name` / `--no-open-browser` 参数；且 `PROJECTNAME` 传 `./` 会把包名取成目录名 `memory1.0`，不满足「name 必须是 crossbrain」的验收要求。已改为：在临时目录 `.scaffold` 用项目名 `crossbrain` 生成 → 逐项 copy 回根目录（跳过 `README.md`）→ 删除临时目录。全程未覆盖任何已有文档。
  2. **create-tauri-app 不会写 `packageManager` 字段**，已手动补 `pnpm@9.15.9`。
  3. **跳过前置检查第 2/4 项继续执行的理由**：Rust 与 `cargo tauri` 缺失，但 TASK-01 的 6 项验收全部不需要编译 Rust（`pnpm dev` 走的是 Vite，`pnpm build` 走 `vue-tsc` + Vite）。前端链路已实测通过。Rust 命名为 BLOCKER-01，影响 TASK-03 起。
  4. **TASK-02 存在版本分叉**：`tailwindcss` 最新为 4.3.3，文档 TASK-02 是 v3 写法，两者不兼容，已记为待决策 D-01。
  5. 实测依赖版本：pnpm 9.15.9、Node v22.22.2、vue 3.5.42、vite 8.3.0、typescript 6.0.3、vue-tsc 3.3.11、@tauri-apps/cli 2.11.4。

---

### 2026-09-14 19:40:33 — 人工决策

- 完成了：选定开发工具
- 修改文件：CURRENT_STATUS.md
- 注意事项：**确定使用 WorkBuddy + DeepSeek-V4.1-Flash 执行 TASK-01 起的全部开发任务**。Rust 核心阶段（TASK-04~09）优先由该模型完成，如遇复杂 Rust 推理卡壳，可切换 GLM-5.3 完整版辅助分析。

---

### 2026-09-14 19:21:32 — Antigravity IDE（Gemini）

- 完成了：全套文档体系生成（17 份文档）
- 修改了文件：
  - 新建 `MASTER_CONTEXT.md`
  - 新建 `CURRENT_STATUS.md`（本文件）
  - 新建 `README.md`
  - 新建 `docs/product/PRD.md`
  - 新建 `docs/product/CHANGELOG.md`
  - 新建 `docs/tech/ARCHITECTURE.md`
  - 新建 `docs/tech/ADAPTER_SPEC.md`
  - 新建 `docs/tech/TECH_STACK.md`
  - 新建 `docs/tech/WINDOWS_COMPATIBILITY.md`
  - 新建 `docs/testing/TEST_PLAN.md`
  - 新建 `docs/testing/SPIKE_RESULTS.md`
  - 新建 `docs/impl/DEV_SETUP.md`
  - 新建 `docs/impl/TASK_BREAKDOWN.md`
  - 新建 `docs/impl/RELEASE.md`
  - 新建 `docs/impl/CONTRIBUTING.md`
  - 新建 `docs/impl/DECISION_LOG.md`
  - 新建 `docs/impl/HANDOFF_PROTOCOL.md`
- 注意事项：项目代码尚未创建，所有文件均为文档。下一步执行 TASK-01。

# CrossBrain (跨脑) - 项目实施蓝图 V2.3

> **版本历史**：
> - V2.0：废弃 Symlink/JSON，Hub & Spoke 架构
> - V2.1：L2 挂原生懒加载，Claude 桩文件定死
> - V2.2：修复 15 处漏洞，更名 CrossBrain，砍掉 V1 Watcher
> - **V2.3（本版）**：修复 7 处设计冲突/遗漏，新增用户易用性设计规范
>
> 架构无硬伤，可开工。

---

## 🎯 核心定位

在 2026 年 `AGENTS.md` 成为行业标准的大背景下，"项目级规则同步"已被 Git 原生解决。
CrossBrain 的核心价值定位为：**个人全局记忆层** 与 **按需经验库** 的跨工具统一分发。

- **全局规则**（内部称 L0）：你的编码风格、语言偏好、思维框架——每次对话都生效
- **技能知识**（内部称 L2）：特定领域的调试经验、模式库——AI 需要时自动加载，不污染上下文

我们不接管具体项目，我们接管开发者的**数字大脑**，并让它跨越所有 AI 工具生效。

> ⚠️ **UI 语言规范**：L0/L2/Adapter/SSOT/slug/frontmatter 等内部术语**严禁出现在 UI 界面**。
> 对用户只说"全局规则"和"技能知识"，技术细节全部收进代码。

---

## 👤 用户体验设计（UX 规范）

### 目标用户画像

**重度 AI 编程工具用户**：每天使用 Cursor 或 Claude Code，懂 AI 规则的概念，但不一定懂 git 操作，不一定熟悉 Markdown 高级语法。期望"写好规则，一键同步，就能生效"。

### 首次运行向导（5 步，不可跳过）

```
Step 1 欢迎页
  └─ 一句话：CrossBrain 让你的 AI 配置跨工具同步生效
  └─ 按钮：开始配置

Step 2 检测已安装工具
  └─ 自动扫描：✅ Cursor（已检测） / ✅ Claude Code（已检测） / ⬜ Windsurf（未安装，跳过）
  └─ 每个工具显示推送目标路径（可修改）

Step 3 写第一条全局规则
  └─ 提供 3 个模板选择：Python 开发者 / 前端工程师 / 通用开发者
  └─ 选模板后右侧自动填充，用户可直接修改
  └─ 说明文字："全局规则会在每次 AI 对话中自动生效"

Step 4 一键同步
  └─ 点击【立即同步】，显示进度：Cursor ✅ / Claude Code ✅
  └─ 同步完成后显示：已推送 1 条全局规则到 2 个工具

Step 5 验证生效
  └─ UI 引导："现在打开 Cursor，新建对话，输入以下内容："
  └─ 展示一个验证用的问题（不是验血暗号，而是 UI 主动引导的验证流程）
  └─ 用户点击"我已验证" 或 "跳过"
```

### 验血暗号的 UX 改造

原方案"在规则末尾藏一条 `若读到此条请回复 OK`"对普通用户很突兀（AI 莫名其妙回复 OK）。

**改为主动验证模式**：
- 设置页有【验证注入】按钮
- 点击后弹出对话框："请在 Cursor 新建对话，输入以下文字，看 AI 是否按你的规则回应"
- 底部展示期望的 AI 行为示例
- 验血暗号仍然生成，但只作为引导验证的触发器，不是用户自己察觉到的

### 错误提示规范

所有错误信息**禁止展示原始系统错误**，必须转译为用户可操作的建议：

| 系统错误 | 用户看到的提示 |
|---|---|
| Permission denied | "没有写入权限，请检查 ~/.cursor/rules/ 目录权限" |
| Directory not found | "目录不存在，CrossBrain 将自动创建" （自动处理，不报错） |
| Git: nothing to commit | 不报错，静默处理 |
| Git not found | "未检测到 Git，规则仍会同步，但不会保存版本历史。[安装 Git]" |

### L2 技能知识的用户引导

用户不知道一篇知识该写多长、该分几个文件。需要在 UI 里给出明确建议：
- **每个文件 = 一个具体技能主题**（好例子：`Rust 生命周期调试`，坏例子：`我的所有经验`）
- 文件过大（> 2000字）时 UI 显示黄色警告："这篇知识可能过长，建议拆分为多个主题"
- 新建时默认提供标题模板：`# [工具/语言] + [具体场景]`

### 同步状态可视化

同步按钮区域应显示：
```
上次同步：2026-09-14 17:30  ✅ Cursor  ✅ Claude Code
[立即同步]
```
失败时：
```
上次同步：2026-09-14 17:30  ✅ Cursor  ❌ Claude Code（点击查看原因）
```

---

## 🏗️ 架构：中心源 + Hub & Spoke 推送

**彻底废弃**：Symlink 方案、JSON 注入方案、File Watcher 回写方案。
**采用**：中心化存储 + 适配器普通文件覆盖推送。

### 数据源（SSOT）

所有用户数据只存在于 `~/.ai-profile/`（Windows: `%USERPROFILE%\.ai-profile\`）：

```
~/.ai-profile/
├── .git/
├── .gitignore                   # 内容：AGENTS.md（生成物不进 git）
├── global/
│   └── rules.md                 # 全局规则（用户编辑的唯一源文件）
├── knowledge/
│   ├── debug-rust.md            # 技能知识（每篇 = 一个技能/经验主题）
│   ├── vue3-patterns.md
│   └── ...
└── AGENTS.md                    # ⚠️ 推送引擎的中间产物，不进 git
                                 # 每次同步从 global/rules.md 重新生成
                                 # 仅当 @import Spike 验证通过时使用
```

**AGENTS.md 生成逻辑**：每次同步时，Rust 引擎将 `global/rules.md` 内容写入 `~/.ai-profile/AGENTS.md`，列入 `.gitignore`，不被 git 追踪，用户不应直接编辑（下次同步覆盖）。

**Spike 失败退化路径**：若 `@import` 验证失败，Claude Adapter 直接将 `global/rules.md` 内容以标记块格式写入 `~/.claude/CLAUDE.md`（不经过 AGENTS.md 中转），SSOT 目录里的 AGENTS.md 不再生成。

> ⚠️ `~/.ai-profile/` 从 **Day 1 即 `git init`**，不是 V3 路标。多机同步是核心卖点，事后 retrofit 代价极高。

### 适配器推送（Push Adapters）

点击【立即同步】时，Rust 底层读取 SSOT，以**普通文件读写**推送到目标目录。

**两套幂等策略（按文件性质区分）**：

| 文件类型 | 管理方 | 幂等策略 |
|---|---|---|
| `crossbrain-L0.mdc` | CrossBrain 完全管理 | **直接全量覆盖**（整个文件是生成物） |
| `crossbrain-L2-*.mdc` | CrossBrain 完全管理 | **直接全量覆盖** |
| `~/.claude/skills/crossbrain-*/SKILL.md` | CrossBrain 完全管理 | **直接全量覆盖** |
| `~/.claude/CLAUDE.md` | 用户可能手写过 | **标记块协议**（保护用户原有内容） |

> ⚠️ **关键决策**：对 CrossBrain 完全管理的文件用标记块是不必要的复杂度，会引发 frontmatter 位置冲突问题。直接全量覆盖更简单可靠。标记块协议**只用于 `CLAUDE.md`**。

#### Adapter A：Cursor

**目标路径检测逻辑**（三级处理，不是简单的弹填写框）：
1. 默认路径存在 → 直接推送
2. 默认路径不存在但检测到 Cursor 已安装 → **自动创建目录**，再推送，UI 提示"已自动创建目录"
3. 无法检测到 Cursor（便携版、自定义安装） → UI 弹出路径填写框
4. 用户未安装 Cursor → UI 标注"未检测到 Cursor，已跳过"，不报错

**Windows 目标路径**：`%USERPROFILE%\.cursor\rules\`
**macOS 目标路径**：`~/.cursor/rules/`

| 文件类型 | 来源 | 推送目标（文件名） | 生成策略 |
|:---|:---|:---|:---|
| 全局规则 | `global/rules.md` | `crossbrain-L0.mdc` | 全量覆盖，含 frontmatter `alwaysApply: true` |
| 技能知识 | `knowledge/*.md`（每篇） | `crossbrain-L2-{slug}.mdc` | 全量覆盖，含 frontmatter `description: {标题}` |

**Slug 生成规则**：文件名转 kebab-case，仅保留 ASCII 字母+数字+连字符，截断至 40 字符，**末尾追加文件内容 hash 前 6 位**防止冲突。
例：`debug-rust.md` → `crossbrain-L2-debug-rust-a3f2b1.mdc`；`Vue3组件规范.md` → `crossbrain-L2-vue3-c8d0e2.mdc`。

**description 取值规则**：取文件第一行 `# 标题`（去掉 `# ` 前缀）。若无 h1 标题，fallback 为文件名去扩展名。空文件跳过并 UI 警告。

**完整 .mdc 文件格式**（无标记块，frontmatter 必须在文件绝对行首）：

```
---
alwaysApply: false
description: "Rust 调试经验：内存溢出与生命周期边界问题"
---

[规则正文内容]
```

> ⚠️ **Spike 必验**：frontmatter 在绝对行首时，Cursor 是否正确解析并按 description 按需加载。

#### Adapter B：Claude Code

**目标路径检测**（同 Adapter A 三级逻辑）：
**Windows 目标路径**：`%USERPROFILE%\.claude\`
**macOS 目标路径**：`~/.claude/`

**`CLAUDE.md` 写入策略（标记块协议）**：

```
情况一：文件不存在
  → 直接创建桩文件

情况二：文件存在且含 <!-- CrossBrain:Start -->
  → 正则替换标记块内容（幂等）

情况三：文件存在，不含标记块，且 .crossbrain-backup 不存在
  → 备份原文件为 .crossbrain-backup，写入桩文件，UI 弹提示

情况四：文件存在，不含标记块，且 .crossbrain-backup 已存在
  → 跳过备份（已有备份），直接覆盖写入桩文件，UI 不重复提示
```

**Spike 通过时的桩文件内容**：

```markdown
<!-- CrossBrain:Start -->
<!-- 本文件由 CrossBrain 生成，请在 CrossBrain App 中编辑 ~/.ai-profile/ -->
@~/.ai-profile/AGENTS.md
<!-- CrossBrain:End -->
```

**Spike 失败时的退化内容**（直接写全量规则，同样用标记块包裹）：

```markdown
<!-- CrossBrain:Start -->
<!-- 本文件由 CrossBrain 生成，请在 CrossBrain App 中编辑 ~/.ai-profile/ -->

[global/rules.md 的完整内容]
<!-- CrossBrain:End -->
```

**L2 技能知识**通过 Claude Code Skills 机制推送，复用 Cursor 同一套 slug 生成器：

```
~/.claude/skills/
└── crossbrain-debug-rust-a3f2b1/   # slug 带 hash 后缀，与 Cursor 保持一致
    └── SKILL.md                    # 全量覆盖，CrossBrain 完全管理
```

**SKILL.md 格式**：

```markdown
---
name: debug-rust
description: Rust 调试经验：内存溢出与生命周期边界问题
---

[knowledge/debug-rust.md 的完整内容]
```

> ⚠️ **高风险假设（Spike 必验）**：
> 1. `@~/.ai-profile/AGENTS.md` 中 `~` 是否在 Claude Code 里被展开（尤其 Windows）
> 2. Claude Code Skills `~/.claude/skills/` 是否为官方支持路径
> 3. Windows 路径形式：`~` / `%USERPROFILE%` / 绝对路径哪种有效

#### 架构优势

- ❌ 跨盘软链接断裂？消失了
- ❌ Windows UAC 提权？消失了
- ❌ 云盘同步死循环？消失了
- ❌ JSON 解析竞争防抖？消失了
- ❌ L2 靠祈祷模型自觉？消失了——**挂上各工具原生懒加载机制**

---

## 🔄 L2 技能知识：原生懒加载（核心差异化）

| 工具 | 原生机制 | CrossBrain 做什么 |
|:---|:---|:---|
| **Cursor** | Agent Requested Rules（`description` 驱动，模型按需加载） | 每条 `knowledge/*.md` 推成一个 `.mdc`，description 取 h1 标题 |
| **Claude Code** | Skills（`~/.claude/skills/*/SKILL.md`，渐进披露） | 每条 `knowledge/*.md` 推成 `crossbrain-{slug-hash}/SKILL.md` |

两套 Adapter 复用**同一套 slug 生成器**，只是生成不同格式的文件模板。

---

## 🔪 V1 MVP 功能范围（两周落地）

> 原则：V1 只打通数据流，证明核心价值闭环。能让第一个真实用户用起来并信任产品即为达标。

1. **纯白板 UI（Vue3 + Tailwind，Tauri v2）**：
   - 左侧：工具检测状态栏 + 全局规则/技能知识双列表
   - 右侧：纯文本编辑器（Markdown，不渲染）
   - 顶部：【立即同步】按钮 + 上次同步时间 + 每个工具状态

2. **首次运行 5 步向导**（见 UX 规范章节）：
   - 检测已安装工具（Cursor/Claude Code）
   - 自动创建 `~/.ai-profile/` + `.gitignore` + `git init` + 写入示例文件
   - 检测 git：未安装→弹引导链接，降级无版本历史模式（不阻断）
   - 检测 SSOT 是否在云盘目录：是→弹警告（不阻断）
   - 规则模板选择 + 引导首次同步 + 验证引导

3. **核心推送引擎（Rust）**：
   - 基类 Adapter trait
   - Cursor Adapter（全局规则全量覆盖 + L2 技能全量覆盖）
   - Claude Adapter（CLAUDE.md 标记块协议 + L2 Skills 全量覆盖）
   - 三级路径检测逻辑（见 Adapter A 章节）

4. **两套幂等策略**：
   - CrossBrain 完全管理的文件 → 直接全量覆盖
   - `CLAUDE.md` → 标记块正则替换（4 种情况处理，见 Adapter B 章节）

5. **SSOT git 自动提交**：
   - Rust 用 `std::process::Command` 调用系统 git（V1 选型，简单）
   - 用 `chrono` crate 格式化时间（**禁止 shell `$(date)`，Windows 不兼容**）
   - **git commit 退出码处理**：退出码非零且 stderr 含 "nothing to commit" → 视为成功，不报错

6. **错误处理（用户友好）**：
   - 所有系统错误必须转译为用户可操作的提示（见 UX 规范章节）
   - **严禁展示原始系统错误码**
   - 目标目录不存在 → 自动创建，不报错

7. **同步 + 主动验证引导**：
   - 同步完成后显示摘要：推送了几条规则到几个工具
   - 设置页提供【验证注入】按钮（见 UX 规范章节），不再是藏在规则里的验血暗号

8. **优雅卸载（去无痕）**：
   - 设置页【清除所有注入】按钮
   - 遍历所有 Adapter 执行删除/剥离逻辑，还原 IDE 纯净状态

> ❌ **V1 不做 SSOT 自动 Watcher**：V1 保持"手动点同步"，Watcher 移至 V2。
> 理由：跨平台 FS 事件 + 去抖动 + 写入竞争，复杂度与两周目标不匹配。

---

## 🛡️ V1 工程底线（P0/P1）

| 问题域 | 解决方案 |
|:---|:---|
| **安装包信任危机** | 必须购买代码签名证书（OV 级）。**签名才是唯一解**；便携版 `.exe` 带 MoTW 依旧弹 SmartScreen 红窗，两者不可混淆 |
| **幂等性（.mdc/SKILL.md）** | 直接全量覆盖（CrossBrain 完全管理，无需标记块） |
| **幂等性（CLAUDE.md）** | 标记块协议，4 种情况处理（见 Adapter B） |
| **已有文件保护** | 检测到用户手写 CLAUDE.md 时，备份为 `.crossbrain-backup`，UI 弹提示 |
| **backup 二次运行** | `.crossbrain-backup` 已存在时跳过备份，直接覆盖，不重复提示 |
| **优雅卸载** | 【清除所有注入】按钮，还原所有 IDE 纯净状态 |
| **`.mdc` frontmatter** | frontmatter 必须在文件绝对行首，列入 Day 1 Spike 验证 |
| **路径三级检测** | 存在→推送；不存在但工具已装→自动创建；无法检测→弹填写框；未安装→跳过 |
| **L2 加载机制** | 挂 Cursor Agent Requested + Claude Skills 原生机制 |
| **错误处理** | 系统错误转译为用户友好提示，严禁展示原始错误 |
| **git 无变更提交** | 捕获 "nothing to commit"，视为成功，不触发 UI 错误 |
| **Windows 路径兼容** | Rust 代码统一用 `dirs` crate 解析 home 目录 |

---

## 📅 Day 1 Spike 清单（半天，开工前必验）

**原则：所有验证必须在零上下文的全新对话里进行，防止假阳性。**

- [ ] `~/.cursor/rules/` 下的 `.mdc` frontmatter 在文件绝对行首时是否正确解析
- [ ] Cursor Agent Requested `description` 字段是否真正驱动按需加载（vs always loaded）
- [ ] 裸 `.md` vs 带 frontmatter `.mdc` 是否存在加载行为差异
- [ ] `@~/.ai-profile/AGENTS.md`（含 `~`）在 Claude Code 里是否正确展开
- [ ] Windows 上 `@` 语法使用绝对路径是否可行
- [ ] Claude Code Skills `~/.claude/skills/` 是否官方支持
- [ ] `SKILL.md` frontmatter 格式是否正确，description 是否驱动按需加载
- [ ] git commit "nothing to commit" 时退出码验证（确认为 1，非 0）

**Spike 通过标准**：手工创建测试文件，空白新对话里触发，行为符合预期即通过。
**Spike 失败预案**：每条 Spike 对应一个 Plan B（例：@import 不支持 `~` → 退化为直接写全量内容进 CLAUDE.md）。

---

## ⚠️ 发布前工程底线

- **代码签名**：OV 级证书，签名才能绕过 SmartScreen，便携版 ≠ 免签名
- **README 第一节预埋反驳**：必须回答"shell 脚本也能干"的质疑（答案：幂等安全、旧数据备份保护、L2 原生路由、GUI 降低门槛、多机同步）
- **License**：建议 **Apache-2.0**（含专利保护条款，适合与商业 IDE 深度集成的工具类软件）
- **项目名查重**：CrossBrain 在 GitHub/npm/crates.io 均无占用，EU 同名科研项目领域不同，可用

---

## 🔧 技术选型锁定

| 层 | 选型 | 版本 | 理由 |
|:---|:---|:---|:---|
| 桌面框架 | Tauri | **v2**（锁定） | v1/v2 API 完全不兼容，必须明确 |
| 前端 | Vue 3 + Tailwind | latest stable | 计划已定 |
| Rust 时间处理 | `chrono` | latest | 替代 shell `$(date)`，跨平台 |
| Rust 路径解析 | `dirs` | latest | 跨平台 home 目录，不用 `~/` 字符串拼接 |
| **Rust git 集成** | **`std::process::Command`** | **系统 git** | V1 简单选型；依赖系统 git，未安装时降级无版本历史模式。V2 可迁移至 `git2` crate（libgit2，不依赖系统 git） |

---

## 🔮 V2/V3 演进路标

- **V2**：SSOT 文件变动自动触发重推（单向 Watcher，`tauri-plugin-fs`）
- **V2**：迁移 git 集成至 `git2` crate，消除系统 git 依赖
- **V2**：`~/.ai-profile/` git remote 配置，实现多机跨设备同步（push/pull）
- **V2**：支持 Windsurf、Copilot、Zed 等更多工具 Adapter
- **V2**：L0/L2 内容的 UI 引导（知识文件大小警告、拆分建议）
- **V3**：Adapter 抽象为可通过 UI 编写的自定义插件
- **V3**：接入轻量本地 LLM 或 BYOK API，实现经验库自动结构化与压缩

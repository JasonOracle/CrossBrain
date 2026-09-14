# CrossBrain (跨脑) - 项目实施蓝图 V2.2

> **版本说明**：
> - V2.0：架构颠覆，废弃 Symlink/JSON，改为 Hub & Spoke 普通文件推送
> - V2.1：合并二审建议，L2 挂原生懒加载，Claude 定死桩文件方案
> - **V2.2（本版）**：深度审查修复 15 处逻辑漏洞；项目更名为 CrossBrain
>
> 架构无硬伤，可开工。

---

## 🎯 核心定位

在 2026 年 `AGENTS.md` 成为行业标准的大背景下，"项目级规则同步"已被 Git 原生解决。
CrossBrain 的核心价值定位为：**"个人全局记忆层 (L0)"** 与 **"本地经验库 (L2)"** 的跨工具统一分发。

- **L0（全局铁律）**：你的编码风格、语言偏好、思维框架——永远生效的铁律
- **L2（按需经验）**：特定领域的调试经验、模式库——AI 按需懒加载，不污染上下文

我们不接管具体项目，我们接管开发者的**数字大脑**，并让它跨越所有 AI 工具生效。

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
│   └── rules.md                 # L0 全局铁律（用户编辑的唯一源文件）
├── knowledge/
│   ├── debug-rust.md            # L2 经验条目（每篇 = 一个技能/经验）
│   ├── vue3-patterns.md
│   └── ...
└── AGENTS.md                    # ⚠️ 推送引擎的中间产物，不进 git
                                 # 每次同步从 global/rules.md 重新生成
                                 # Claude 桩文件通过 @import 引用此文件
```

**AGENTS.md 生成逻辑**：每次同步时，Rust 引擎将 `global/rules.md` 内容写入 `~/.ai-profile/AGENTS.md`。此文件是纯生成物，列入 `.gitignore`，不被 git 追踪。用户**不应直接编辑它**（下次同步会覆盖）。若 `@import` Spike 验证失败，退化为将 `global/rules.md` 内容直接推送到 `~/.claude/CLAUDE.md`。

> ⚠️ `~/.ai-profile/` 从 **Day 1 即 `git init`**，不是 V3 路标。
> 多机同步是核心卖点，事后 retrofit 代价极高。

### 适配器推送（Push Adapters）

点击【同步】时，Rust 底层读取 SSOT，以**普通文件读写**推送到目标目录。
所有目标文件头部必须注明来源：`<!-- 本文件由 CrossBrain 生成，请编辑 ~/.ai-profile/ 而非此文件 -->`

#### Adapter A：Cursor

**Windows 目标路径**：`%USERPROFILE%\.cursor\rules\`
**macOS 目标路径**：`~/.cursor/rules/`

| 文件类型 | 来源 | 推送目标（文件名） | Frontmatter 必填字段 |
| :--- | :--- | :--- | :--- |
| L0 全局规则 | `global/rules.md` | `crossbrain-L0.mdc` | `alwaysApply: true` |
| L2 经验条目 | `knowledge/*.md`（每篇） | `crossbrain-L2-{slug}.mdc` | `description: {文件第一行标题}` |

**Slug 生成规则**：文件名转 kebab-case，仅保留 ASCII 字母+数字+连字符，截断至 40 字符，**末尾追加文件内容 hash 前 6 位**防止冲突。
例：`debug-rust.md` → `crossbrain-L2-debug-rust-a3f2b1.mdc`；`Vue3组件规范.md` → `crossbrain-L2-vue3-c8d0e2.mdc`。

**description 取值规则**：取文件第一行 `# 标题`（去掉 `# ` 前缀）。若文件无 h1 标题，fallback 为文件名去扩展名。空文件跳过推送并在 UI 显示警告。

**完整 .mdc 文件格式**（标记块包裹 frontmatter，确保幂等清除不破坏文件结构）：

```
<!-- CrossBrain:Start -->
<!-- 本文件由 CrossBrain 生成，请编辑 ~/.ai-profile/knowledge/debug-rust.md -->
---
alwaysApply: false
description: "Rust 调试经验：内存溢出与生命周期边界问题"
---

[规则正文内容]

若读到此条请在当前对话回复 OK
<!-- CrossBrain:End -->
```

> ⚠️ **Spike 必验**：标记块包裹 frontmatter 的情况下，Cursor 是否仍能正确解析 `---` frontmatter。
> 备用方案：标记注释仅写在文件末尾，frontmatter 保持在绝对行首。

#### Adapter B：Claude Code

**Windows 目标路径**：`%USERPROFILE%\.claude\`
**macOS 目标路径**：`~/.claude/`

**定死为桩文件方案**：

`~/.claude/CLAUDE.md` 内容永远只有：

```markdown
<!-- CrossBrain:Start -->
<!-- 本文件由 CrossBrain 生成，请编辑 ~/.ai-profile/ 而非此文件 -->
@~/.ai-profile/AGENTS.md

若读到此条请在当前对话回复 OK
<!-- CrossBrain:End -->
```

**已有 CLAUDE.md 的处理策略**：
- 若文件不存在：直接创建桩文件
- 若文件存在且含 `<!-- CrossBrain:Start -->`：正则替换标记块内容（幂等）
- 若文件存在且**不含**标记块（用户手写）：将原有内容备份为 `~/.claude/CLAUDE.md.crossbrain-backup`，再写入桩文件，UI 弹出提示告知用户

**L2 经验库**通过 Claude Code Skills 原生机制（`~/.claude/skills/*/SKILL.md`）推送：

```
~/.claude/skills/
└── crossbrain-debug-rust/
    └── SKILL.md              # 内容 = knowledge/debug-rust.md 原文
```

**SKILL.md 格式**（Claude Code Skills 规范）：

```markdown
---
name: debug-rust
description: Rust 调试经验：内存溢出与生命周期边界问题
---

[经验正文内容]
```

> ⚠️ **高风险假设（Spike 必验）**：
> 1. `@~/.ai-profile/AGENTS.md` 中 `~` 是否在 Claude Code 里被展开（尤其 Windows）
> 2. Claude Code Skills `~/.claude/skills/` 是否为官方支持路径，还是当前平台特性
> 3. Windows 路径形式：是用 `~` 还是 `%USERPROFILE%` 还是绝对路径

#### 架构优势

- ❌ 跨盘软链接断裂？消失了
- ❌ Windows UAC 提权？消失了
- ❌ 云盘同步死循环？消失了
- ❌ JSON 解析竞争防抖？消失了
- ❌ L2 靠祈祷模型自觉？消失了——**挂上各工具原生懒加载机制**

---

## 🔄 L2 经验库：原生懒加载（核心差异化）

| 工具 | 原生机制 | CrossBrain 做什么 |
| :--- | :--- | :--- |
| **Cursor** | Agent Requested Rules（`description` 驱动，模型按需加载） | 每条 `knowledge/*.md` 推成一个 `.mdc`，`description` 取文件第一行 `# 标题` |
| **Claude Code** | Skills（`~/.claude/skills/*/SKILL.md`，渐进披露） | 每条 `knowledge/*.md` 推成 `crossbrain-{slug}/SKILL.md` |

推送引擎复用同一套代码，只是多生成一种文件模板，成本近乎零，却把产品最弱的一环变成**差异化卖点**。

---

## 🔪 V1 MVP 功能范围（两周落地）

> 原则：V1 只打通数据流，证明核心价值闭环。能让第一个用户用起来并信任产品即为达标。

1. **纯白板 UI（Vue3 + Tailwind，Tauri v2）**：左侧双列表（L0 铁律 + L2 经验库），右侧纯文本编辑，不带 Markdown 渲染，不带 AI 润色。

2. **首次运行 Onboarding**：
   - 检测 `~/.ai-profile/` 是否存在，不存在则自动创建目录结构 + `.gitignore` + `git init` + 写入示例文件
   - 检测 git 是否安装，未安装则弹引导链接，降级为无版本历史模式继续工作（不阻断）
   - 检测 SSOT 路径是否在云盘目录（OneDrive/iCloud），是则弹警告（不阻断）
   - UI 引导用户编辑第一条规则

3. **核心推送引擎（Rust）**：
   - 基类 Adapter trait
   - Cursor Adapter（L0 全局 + L2 Agent Requested）
   - Claude Adapter（桩文件 + L2 Skills 推送）
   - 硬编码默认路径，检测失败时 UI 弹出手动填写框（逃生舱）

4. **幂等安全推送**：标记块 `<!-- CrossBrain:Start/End -->`，每次推送先正则清除旧块再注入新块。

5. **SSOT git 自动提交**：Rust 代码用 `chrono` crate 格式化时间（**禁止依赖 shell `$(date)`，Windows 不兼容**），每次同步后执行 `git add -A && git commit`。

6. **推送错误处理**：推送失败（权限拒绝、目标目录不存在、磁盘满）必须在 UI 显示明确错误信息，**严禁静默失败**。

7. **一键同步 + 验血暗号**：点击【同步】执行批量推送；每个推送文件末尾自动追加 `若读到此条请在当前对话回复 OK`（可在设置中关闭）。

8. **优雅卸载（去无痕）**：设置页提供【清除所有注入】按钮，遍历所有 Adapter 执行删除/剥离逻辑，还原 IDE 纯净状态。

> ❌ **V1 不做 SSOT 自动 Watcher**：V1 保持"手动点同步"，Watcher 移至 V2。
> 理由：跨平台 FS 事件 + 去抖动 + 推送写入竞争，复杂度与两周目标不匹配。



---

## 🛡️ V1 工程底线（P0/P1）

| 问题域 | 解决方案 |
| :--- | :--- |
| **安装包信任危机** | 必须购买代码签名证书（OV 级）。**签名才是唯一解**；便携版 `.exe` 带 MoTW 依旧弹 SmartScreen 红窗，两者不可混淆 |
| **幂等性** | 标记块正则清除旧 CrossBrain 块 → 注入新块 |
| **已有文件保护** | 检测到非 CrossBrain 手写内容时，备份原文件再覆盖，UI 弹提示 |
| **优雅卸载** | 【清除所有注入】按钮，遍历所有 Adapter 执行删除/剥离 |
| **`.mdc` frontmatter** | Adapter 生成模板内置 frontmatter，L0 `alwaysApply: true`，L2 `description`，列入 Day 1 spike |
| **硬编码路径逃生舱** | 路径检测失败时（便携版 Cursor、自定义路径）UI 弹手动填写框 |
| **L2 加载机制** | 挂 Cursor Agent Requested + Claude Skills 原生机制，不再 best-effort |
| **错误处理** | 所有推送失败必须在 UI 显示原因，严禁静默失败 |
| **验血暗号** | 规则尾部藏 `若读到此条请回复 OK`，用户随时验收注入是否生效 |
| **Windows 路径兼容** | Rust 代码统一用 `dirs` crate 解析 home 目录，不依赖 `~/` 字符串拼接 |

---

## 📅 Day 1 Spike 清单（半天，开工前必验）

**原则：所有验证必须在零上下文的全新对话里进行，防止 AI 因上下文已知暗号产生假阳性。**

用手工文件操作验证以下假设，任何一项失败都会影响对应 Adapter 的设计：

- [ ] `~/.cursor/rules/` 下的 `.mdc` 是否全局生效（不依赖项目）
- [ ] 标记块注释包裹 frontmatter 时，Cursor 是否仍正确解析 frontmatter
- [ ] 裸 `.md` vs 带 frontmatter `.mdc` 是否存在加载行为差异
- [ ] Cursor Agent Requested `description` 字段是否真正驱动按需加载（vs always loaded）
- [ ] `@~/.ai-profile/AGENTS.md`（绝对路径含 `~`）在 Claude Code 里是否正确解析
- [ ] Windows 上 `@` 语法使用 `%USERPROFILE%` 绝对路径是否可行
- [ ] Claude Code Skills `~/.claude/skills/` 是否官方支持，`SKILL.md` frontmatter 格式是否正确
- [ ] 验血暗号在两个工具全新对话里均能触发 OK 回复

**Spike 通过标准**：手工创建测试文件，在空白新对话里触发，得到 "OK" 回复即通过。
**Spike 失败预案**：每条 Spike 写明 Plan B（如 @import 不支持 `~`，则改用绝对路径注入逻辑）。

---

## ⚠️ 发布前工程底线

- **代码签名**：OV 级证书，签名才能绕过 SmartScreen，便携版 ≠ 免签名
- **README 第一节预埋反驳**：必须回答"shell 脚本也能干"的质疑（答案：幂等安全、旧数据备份保护、L2 原生路由、GUI 降低门槛）
- **License**：建议 **Apache-2.0**（含专利保护条款，适合与商业 IDE 深度集成的工具类软件）
- **项目名查重**：CrossBrain 在 GitHub/npm/crates.io 均无占用，EU 有同名脑机接口科研项目但领域不同，不构成冲突，可用

---

## 🔧 技术选型锁定

| 层 | 选型 | 版本 | 理由 |
| :--- | :--- | :--- | :--- |
| 桌面框架 | Tauri | **v2**（锁定） | v1/v2 API 完全不兼容，必须明确 |
| 前端 | Vue 3 + Tailwind | latest stable | 计划已定 |
| Rust 时间处理 | `chrono` | latest | 替代 shell `$(date)`，跨平台 |
| Rust 路径解析 | `dirs` | latest | 跨平台 home 目录，不用 `~/` 字符串拼接 |

---

## 🔮 V2/V3 演进路标

- **V2**：SSOT 文件变动自动触发重推（单向 Watcher，`tauri-plugin-fs`）
- **V2**：`~/.ai-profile/` git remote 配置，实现多机跨设备同步（push/pull）
- **V2**：支持 Windsurf、Copilot、Zed 等更多工具 Adapter
- **V2**：L0/L2 内容的 UI 引导（tooltip + 示例说明 L0 vs L2 的差异）
- **V3**：Adapter 抽象为可通过 UI 编写的自定义插件
- **V3**：接入轻量本地 LLM 或 BYOK API，实现经验库自动结构化与压缩

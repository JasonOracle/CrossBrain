# OmniBrain (全知记忆总线) - 项目实施蓝图 V2.1

> **版本说明**：V2 基础上合并二审（内部评审 + kim3 评审）全部 9 条建议。架构无硬伤，可开工。

---

## 🎯 核心定位

在 2026 年 `AGENTS.md` 成为行业标准的大背景下，"项目级规则同步"已被 Git 原生解决。
OmniBrain 的核心价值定位为：**"个人全局记忆层 (L0)"** 与 **"本地经验库 (L2)"** 的跨工具统一分发。

我们不接管具体项目，我们接管开发者的**数字大脑**。

---

## 🏗️ 架构：中心源 + Hub & Spoke 推送

**彻底废弃**：Symlink 方案、JSON 注入方案、File Watcher 回写方案。
**采用**：中心化存储 + 适配器普通文件覆盖推送。

### 数据源（SSOT）

所有用户数据只存在于 `~/.ai-profile/`：

```
~/.ai-profile/
├── .git/                      # Day 1 自动 git init，每次同步自动 commit
├── global/
│   └── rules.md               # L0 全局铁律（永远 alwaysApply）
└── knowledge/
    ├── debug-rust.md          # L2 经验条目（每篇对应一个技能/经验）
    ├── vue3-patterns.md
    └── ...
```

> ⚠️ `~/.ai-profile/` 从 **Day 1 即 `git init`**，不是 V3 路标。每次【同步】自动执行 `git add -A && git commit`。多机同步是核心卖点，事后 retrofit 代价极高。

### 适配器推送（Push Adapters）

点击【同步】时，Rust 底层读取 SSOT，以**普通文件读写**推送到目标目录：

#### Adapter A：Cursor

| 文件类型 | 来源 | 推送目标 | Frontmatter |
| :--- | :--- | :--- | :--- |
| L0 全局规则 | `global/rules.md` | `~/.cursor/rules/omnibrain-L0.mdc` | `alwaysApply: true` |
| L2 经验条目 | `knowledge/*.md`（每篇） | `~/.cursor/rules/omnibrain-L2-{slug}.mdc` | `description: {文件标题}` |

**关键约束**：`.mdc` 文件**必须携带 YAML frontmatter**，否则 Cursor 可能静默忽略。
- L0 → `alwaysApply: true`
- L2 → `description: "..."` 驱动 **Agent Requested** 原生懒加载机制

```
---
alwaysApply: false
description: "Rust 调试经验：内存溢出与生命周期边界问题"
---
[规则正文]
<!-- OmniBrain:End -->
```

#### Adapter B：Claude Code

**定死为桩文件方案（不再二选一）**：

`~/.claude/CLAUDE.md` 内容永远只有：

```markdown
<!-- OmniBrain:Start -->
<!-- 本文件由 OmniBrain 生成，请编辑 ~/.ai-profile/ 而非此文件 -->
@~/.ai-profile/AGENTS.md
<!-- OmniBrain:End -->
```

桩内容永不变，追加/覆盖/防重复逻辑对 Claude 适配器**直接消失**。

> ⚠️ **Spike 必验**：`@import` 是否支持 home 目录绝对路径。

L2 经验库通过 **Claude Code Skills 原生机制**（`~/.claude/skills/*/SKILL.md`）实现渐进披露懒加载，每条 `knowledge/*.md` 推成一个 skill 目录。

#### 架构优势

- ❌ 跨盘软链接断裂？消失了
- ❌ Windows UAC 提权？消失了
- ❌ 云盘同步死循环？消失了
- ❌ JSON 解析竞争防抖？消失了
- ❌ L2 靠祈祷模型自觉？消失了——**挂上各工具原生懒加载机制**

---

## 🔄 L2 经验库：原生懒加载（最重要的架构升级）

**V2 的原话"依赖大模型自身的检索意愿"是整个方案里唯一还在靠概率的地方，V2.1 彻底解决它。**

| 工具 | 原生机制 | OmniBrain 做什么 |
| :--- | :--- | :--- |
| **Cursor** | Agent Requested Rules（`description` 驱动，模型按需加载） | 每条 `knowledge/*.md` 推成一个 `.mdc`，`description` 取文件标题 |
| **Claude Code** | Skills（`~/.claude/skills/*/SKILL.md`，渐进披露） | 每条 `knowledge/*.md` 推成一个 skill 目录 |

推送引擎**复用同一套代码**，只是多生成一种文件模板，成本近乎零，却把产品最弱的一环变成**差异化卖点**。

---

## 🔪 V1 MVP 功能范围（两周落地）

1. **纯白板 UI（Vue3 + Tailwind）**：左侧双列表（全局规则 L0、经验库 L2），右侧纯文本编辑，不带 Markdown 渲染，不带 AI 润色。
2. **核心推送引擎（Rust）**：基类 Adapter + Cursor Adapter + Claude Adapter，硬编码默认路径，路径检测失败时 UI 允许手动填写（逃生舱）。
3. **幂等安全推送**：标记块 `<!-- OmniBrain:Start -->` / `<!-- OmniBrain:End -->`，每次推送先正则清除旧块再注入新块，天然幂等。
4. **SSOT git 自动提交**：每次同步后自动 `git add -A && git commit -m "sync: $(date)"`。
5. **一键同步 + 验血暗号**：点击【同步】执行批量推送；规则尾部自动追加一行 `若读到此条请回复 OK`，用户问一句即可验收注入是否生效。
6. **单向回流契约**：SSOT 侧轻量 watcher（SSOT 文件变动 → 自动触发重推），所有生成的目标文件头部注明"本文件由 OmniBrain 生成，请编辑 `~/.ai-profile/`"。

---

## 🛡️ V1 工程底线（P0/P1）

| 问题域 | 解决方案 |
| :--- | :--- |
| **安装包信任危机** | 必须购买代码签名证书（签名才是唯一解；便携版 `.exe` 带 MoTW 依旧弹 SmartScreen 红窗，两者不可混淆） |
| **幂等性导致数据重复** | 标记块正则清除旧 OmniBrain 块 → 注入新块，Push 前检测非 OmniBrain 手写内容，标记并追加而非覆盖 |
| **优雅卸载（去无痕）** | 软件内提供【清除所有注入】按钮，遍历所有 Adapter 执行删除/剥离逻辑，还原 IDE 纯净状态 |
| **`.mdc` frontmatter** | Adapter 生成模板内置 frontmatter，L0 用 `alwaysApply: true`，L2 用 `description`，列入 Day 1 spike |
| **硬编码路径逃生舱** | 路径检测失败时（便携版 Cursor、自定义 `%USERPROFILE%`）UI 弹出手动填写框而非报错崩溃 |
| **L2 经验库加载** | 挂 Cursor Agent Requested + Claude Skills 原生机制，不再 best-effort |
| **Claude @import 验证** | Day 1 spike：验证 `@~/.ai-profile/AGENTS.md` 绝对路径是否被 Claude Code 正确解析 |
| **验血暗号** | 规则尾部藏 `若读到此条请回复 OK`，用户随时可做端到端注入验收 |

---

## 📅 Day 1 Spike 清单（半天，开工前必验）

在写任何正式代码前，用手工文件操作验证以下假设：

- [ ] `~/.cursor/rules/` 下的 `.mdc` 文件是否全局生效（不依赖任何项目）
- [ ] 裸 `.md` vs 带 frontmatter `.mdc` 是否存在行为差异
- [ ] `@~/.ai-profile/AGENTS.md` 在 Claude Code 里绝对路径是否正确解析
- [ ] Cursor Agent Requested 规则的 `description` 字段是否真的驱动按需加载
- [ ] Claude Skills `~/.claude/skills/` 目录结构是否符合预期
- [ ] 验血暗号是否在两个工具里均能触发

**Spike 通过标准**：手工创建测试文件，问 AI 一句话，能得到 "OK" 回复即为通过。

---

## ⚠️ 发布前工程底线

- **代码签名**：OV 级证书，签名才能绕过 SmartScreen，便携版≠免签名
- **README 第一节预埋反驳**：必须回答"shell 脚本也能干"的质疑（答案：幂等安全、旧数据保护、L2 原生路由、GUI 降低门槛）
- **License**：选定 MIT 或 Apache-2.0 并写入仓库
- **项目名查重**：OmniBrain 存在撞名项目，发布前确认 GitHub / npm / crates.io 命名可用性

---

## 🔮 V2/V3 演进路标

- **V2**：`~/.ai-profile/` git remote 配置，实现多机跨设备同步（push/pull）
- **V2**：更多工具 Adapter（Windsurf、Copilot、Zed 等）
- **V3**：Adapter 抽象为可通过 UI 编写的自定义插件
- **V3**：接入轻量本地 LLM 或 BYOK API，实现经验库自动结构化与压缩

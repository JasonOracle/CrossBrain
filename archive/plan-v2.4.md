# CrossBrain (跨脑) - 项目实施蓝图 V2.4

> **版本历史**：
> - V2.0：废弃 Symlink/JSON，Hub & Spoke 架构
> - V2.1：L2 挂原生懒加载，Claude 桩文件定死
> - V2.2：修复 15 处漏洞，更名 CrossBrain，砍掉 V1 Watcher
> - V2.3：修复 7 处设计冲突，新增 UX 规范
> - **V2.4（本版）**：新增国内工具支持矩阵（Trae/WorkBuddy/Qoder/Codex）；新增离线/内网兜底策略
>
> 架构无硬伤，可开工。

---

## 🎯 核心定位

CrossBrain 的核心价值：**个人全局规则** 与 **按需技能知识** 的跨工具统一分发。

- **全局规则**（内部 L0）：编码风格、语言偏好、思维框架——每次 AI 对话都生效
- **技能知识**（内部 L2）：特定领域经验模式库——AI 需要时自动加载，不污染上下文

我们不接管具体项目，我们接管开发者的**数字大脑**，并让它跨越所有 AI 工具生效。

> ⚠️ **UI 语言规范**：L0/L2/Adapter/SSOT/slug/frontmatter 等内部术语**严禁出现在 UI 界面**。

---

## 🛠️ 工具支持矩阵

### V1 支持（两周内）

| 工具 | 公司 | 配置机制 | Spike 状态 |
|:---|:---|:---|:---|
| **Cursor** | Anysphere | `~/.cursor/rules/*.mdc` + Agent Requested | 已规划，需 Spike 验证 frontmatter |
| **Claude Code** | Anthropic | `~/.claude/CLAUDE.md` + `~/.claude/skills/` | 已规划，需 Spike 验证 @import |

### V1.5 支持（V1 发布后优先追加）

| 工具 | 公司 | 推测配置机制 | Spike 要点 |
|:---|:---|:---|:---|
| **Trae** | 字节跳动 | 可能支持 `AGENTS.md` 或自有 rules 目录（待验证） | 检测 `~/.trae/rules/` 或全局 `AGENTS.md` 位置 |
| **WorkBuddy** | - | 全局规则路径待确认（项目级 `.workbuddy/` 不是全局） | Spike：WorkBuddy 是否有全局配置目录 |

> **WorkBuddy 特殊说明**：项目目录下的 `.workbuddy/memory/` 是**会话记忆**，不是规则文件。CrossBrain 要找的是 WorkBuddy 的**全局用户规则**配置路径（可能是 `~/.workbuddy/rules/` 或类似位置）。V1.5 Spike 前暂不支持。

### V2 支持

| 工具 | 公司 | 配置机制 | 说明 |
|:---|:---|:---|:---|
| **Codex CLI** | OpenAI | `AGENTS.md` 标准 | 天然兼容，CrossBrain 的 `~/.ai-profile/AGENTS.md` 可直接挂接 |
| **Qoder** | - | 待 Spike | 需要调研配置文件位置 |
| **Windsurf** | Codeium | `~/.codeium/windsurf/memories/` 或类似 | 配置路径待验证 |

### 工具支持的通用原则

1. **AGENTS.md 标准工具**（Codex CLI 等）：直接引用 `~/.ai-profile/AGENTS.md`，成本趋近于零
2. **Cursor-like 工具**（基于 VS Code 的 IDE，如 Trae）：大概率支持类似 rules 目录，Spike 后复用 Cursor Adapter 模板
3. **自研配置体系**（WorkBuddy、Qoder）：需要独立 Spike，可能需要专用 Adapter

---

## 🔌 离线 / 内网兜底策略

### V1 核心功能：天然 100% 离线

CrossBrain V1 的所有核心操作都是**本地文件读写**，无需任何网络连接。这是面向中国企业内网用户的核心卖点。

| 功能 | 是否需要网络 | 说明 |
|:---|:---|:---|
| 规则编辑 | ❌ 不需要 | 纯本地文件 |
| 推送到 Cursor / Claude | ❌ 不需要 | 本地文件写入 |
| git 本地提交 | ❌ 不需要 | 本地 git 仓库 |
| 首次安装 App | ✅ 一次性 | 可提供离线安装包 |
| 代码签名 OCSP 验证 | ✅ 首次需要 | 之后系统缓存，断网不影响已装应用 |
| **多机同步（V2）** | ✅ 需要网络 | **但可以用内网 git 服务** |
| V3 BYOK API | ✅ 需要网络 | 可配置内网代理或本地模型 |

### 完全离线环境（无任何网络）

V1 功能**完整可用**。用户唯一无法使用的是 V2 的多机同步。

降级策略：
- UI 明确标注"多机同步需要配置 git remote，当前为离线模式"
- 用户可手动拷贝 `~/.ai-profile/` 目录到其他机器（U 盘迁移）
- UI 提供【导出配置包】按钮：将 `~/.ai-profile/` 打包为 zip，用于手动迁移

### 内网 git 服务支持（V2 多机同步）

多机同步**不依赖 GitHub**，支持任何标准 git 服务：

```
支持：
  ✅ GitHub / GitLab.com（公网）
  ✅ 企业内网 GitLab（自托管）
  ✅ Gitea（轻量自托管，推荐内网部署）
  ✅ Bitbucket Server
  ✅ 任何支持 HTTPS/SSH 的 git 服务
```

UI 配置项：
```
多机同步设置
  [git remote URL] ：http://gitlab.internal.company.com/username/ai-profile.git
  [认证方式]：HTTPS (用户名/密码 / Token) 或 SSH 密钥
  [代理设置]：可选，支持 HTTP 代理（企业内网常见场景）
```

### 离线安装包策略

国内企业内网场景，用户可能无法直接从公网下载安装包：

- **提供离线安装包**：打包所有依赖，不依赖在线资源安装
- **安装包托管**：除 GitHub Releases 外，同步提供国内镜像（如阿里云 OSS）
- **代码签名**：签名证书验证在首次安装后缓存，离线环境下不影响已安装的应用运行

---

## 👤 用户体验设计（UX 规范）

### 目标用户画像

**重度 AI 编程工具用户**：每天使用 Cursor/Claude Code/Trae，懂 AI 规则概念，但不一定懂 git，不一定熟悉 Markdown 高级语法。期望"写好规则，一键同步，就能生效"。

### 首次运行向导（5 步，不可跳过）

```
Step 1 欢迎页
  └─ 一句话：CrossBrain 让你的 AI 配置跨工具同步生效
  └─ 按钮：开始配置

Step 2 检测已安装工具
  └─ 自动扫描：✅ Cursor（已检测）/ ✅ Claude Code（已检测）/ ✅ Trae（已检测）
  └─ 每个工具显示推送目标路径（可修改）
  └─ 网络状态：🔴 无网络连接（同步功能不受影响，版本历史需要 git）

Step 3 写第一条全局规则
  └─ 提供 3 个模板选择：Python 开发者 / 前端工程师 / 通用开发者
  └─ 选模板后右侧自动填充，用户可直接修改
  └─ 说明文字："全局规则会在每次 AI 对话中自动生效"

Step 4 一键同步
  └─ 显示进度：Cursor ✅ / Claude Code ✅ / Trae ✅
  └─ 同步完成：已推送 1 条全局规则到 3 个工具

Step 5 验证生效
  └─ UI 主动引导验证流程（见下方"验证注入"设计）
```

### 验证注入的 UX 设计

不在规则里藏奇怪文字，改为 UI 主动引导：
- 设置页有【验证注入】按钮
- 点击后弹出："请在 Cursor 新建对话，输入以下文字：" + 展示一个测试问题
- 展示期望的 AI 行为示例，用户自己判断是否生效

### 错误提示规范

| 系统错误 | 用户看到的提示 |
|---|---|
| Permission denied | "没有写入权限，请检查目录权限" |
| Directory not found | 自动创建，不报错 |
| Git: nothing to commit | 静默处理，不报错 |
| Git not found | "未检测到 Git，规则仍会同步，版本历史不可用。[安装 Git]" |
| No network | "当前无网络，规则同步正常，多机同步暂不可用" |

### L2 技能知识的用户引导

- **每个文件 = 一个具体技能主题**（好例子：`Rust 生命周期调试`；坏例子：`我的所有经验`）
- 文件 > 2000 字时 UI 显示警告："建议拆分为多个技能主题，有助于 AI 精准加载"
- 新建时默认标题模板：`# [工具/语言] + [具体场景]`

### 同步状态可视化

```
上次同步：2026-09-14 17:30  ✅ Cursor  ✅ Claude Code  ✅ Trae
[立即同步]                   离线模式：版本历史不可用
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

**AGENTS.md 生成逻辑**：每次同步时从 `global/rules.md` 生成，列入 `.gitignore`，不被 git 追踪。
**Spike 失败退化**：直接将规则内容以标记块格式写入 `~/.claude/CLAUDE.md`，不经过 AGENTS.md 中转。

> ⚠️ `~/.ai-profile/` 从 **Day 1 即 `git init`**，多机同步是核心卖点，事后 retrofit 代价极高。

### 两套幂等策略（按文件性质区分）

| 文件类型 | 管理方 | 幂等策略 |
|:---|:---|:---|
| `crossbrain-L0.mdc` | CrossBrain 完全管理 | **直接全量覆盖** |
| `crossbrain-L2-*.mdc` | CrossBrain 完全管理 | **直接全量覆盖** |
| `~/.claude/skills/crossbrain-*/SKILL.md` | CrossBrain 完全管理 | **直接全量覆盖** |
| `~/.claude/CLAUDE.md` | 用户可能手写过 | **标记块协议**（保护用户内容） |

### Adapter A：Cursor

**目标路径三级检测**：
1. 默认路径存在 → 直接推送
2. 路径不存在但检测到 Cursor 已安装 → **自动创建目录**，推送
3. 无法检测 Cursor → UI 弹路径填写框
4. 未安装 → 标注"未检测到 Cursor"，跳过

**Windows 路径**：`%USERPROFILE%\.cursor\rules\` / **macOS 路径**：`~/.cursor/rules/`

| 文件类型 | 推送目标 | frontmatter |
|:---|:---|:---|
| 全局规则 | `crossbrain-L0.mdc` | `alwaysApply: true`，frontmatter 在文件绝对行首 |
| 技能知识（每篇） | `crossbrain-L2-{slug}.mdc` | `description: {h1标题}`，frontmatter 在文件绝对行首 |

**Slug 规则**：kebab-case ASCII + hash 前 6 位防冲突。例：`debug-rust.md` → `crossbrain-L2-debug-rust-a3f2b1.mdc`
**description 取值**：h1 标题（去掉 `# `）；无标题 fallback 为文件名；空文件跳过并 UI 警告。

### Adapter B：Claude Code

**目标路径**：`%USERPROFILE%\.claude\`（Win） / `~/.claude/`（macOS）

**CLAUDE.md 标记块协议（4 种情况）**：
```
情况一：文件不存在 → 直接创建桩文件
情况二：含 CrossBrain:Start → 正则替换（幂等）
情况三：用户手写，.crossbrain-backup 不存在 → 备份，写入桩文件，UI 提示
情况四：用户手写，.crossbrain-backup 已存在 → 跳过备份，直接覆盖，不重复提示
```

**Spike 通过时**（@import 有效）：
```markdown
<!-- CrossBrain:Start -->
<!-- 本文件由 CrossBrain 生成，请在 CrossBrain App 中编辑 -->
@~/.ai-profile/AGENTS.md
<!-- CrossBrain:End -->
```

**Spike 失败时**（直接写全量内容）：
```markdown
<!-- CrossBrain:Start -->
<!-- 本文件由 CrossBrain 生成，请在 CrossBrain App 中编辑 -->
[global/rules.md 的完整内容]
<!-- CrossBrain:End -->
```

**L2 Skills 推送**（复用 Cursor 同一套 slug 生成器，带 hash 后缀）：
```
~/.claude/skills/
└── crossbrain-debug-rust-a3f2b1/
    └── SKILL.md   # 全量覆盖，frontmatter + knowledge/*.md 原文
```

> ⚠️ **高风险假设（Spike 必验）**：
> 1. `@~/.ai-profile/AGENTS.md` 中 `~` 在 Claude Code 里是否展开（尤其 Windows）
> 2. Claude Code Skills `~/.claude/skills/` 是否官方支持
> 3. Windows 路径形式哪种有效

### Adapter C：Trae（V1.5，Spike 通过后实现）

Trae 是字节跳动的 AI IDE（VS Code fork），国内用户占比高，优先支持。

**Spike 验证要点**：
- [ ] Trae 是否支持全局规则目录（`~/.trae/rules/` 或 `%APPDATA%\Trae\rules\`）
- [ ] 是否原生支持 AGENTS.md（若支持，则天然兼容，成本趋近于零）
- [ ] rules 文件格式：是 `.mdc`（同 Cursor）还是 `.md` 还是其他

**推测方案**（Spike 通过后确认）：
- 若支持 AGENTS.md：推送 `~/.ai-profile/AGENTS.md` 路径给 Trae
- 若与 Cursor 兼容：复用 Cursor Adapter，修改目标路径

### AGENTS.md 标准工具的通用兼容（Codex CLI 等）

遵循 AGENTS.md 标准的工具（OpenAI Codex CLI 等）可以天然受益，因为 CrossBrain 已生成 `~/.ai-profile/AGENTS.md`。

但这些工具通常读取**项目级** AGENTS.md，不是全局。V2 可研究是否支持全局 AGENTS.md 路径配置（如 `$AGENTS_MD` 环境变量）。

---

## 🔄 L2 技能知识：原生懒加载

| 工具 | 原生机制 | CrossBrain 做什么 |
|:---|:---|:---|
| **Cursor** | Agent Requested Rules（description 驱动） | 每条 `knowledge/*.md` 推成 `.mdc`，description 取 h1 标题 |
| **Claude Code** | Skills（`~/.claude/skills/`，渐进披露） | 每条 `knowledge/*.md` 推成 `crossbrain-{slug-hash}/SKILL.md` |
| **Trae**（V1.5） | 待 Spike 确认 | Spike 通过后确认 |

两套已实现 Adapter 复用**同一套 slug 生成器**。

---

## 🔪 V1 MVP 功能范围（两周落地）

> 原则：V1 只打通数据流，证明核心价值闭环。

1. **纯白板 UI（Vue3 + Tailwind，Tauri v2）**：
   - 左侧：工具检测状态栏 + 全局规则/技能知识双列表
   - 右侧：纯文本编辑器
   - 顶部：【立即同步】+ 上次同步时间 + 每个工具状态 + 网络状态指示

2. **首次运行 5 步向导**（见 UX 规范章节）

3. **核心推送引擎（Rust）**：Cursor Adapter + Claude Adapter，三级路径检测

4. **两套幂等策略**：全量覆盖（CrossBrain 管理的文件）+ 标记块（CLAUDE.md）

5. **SSOT git 自动提交**：
   - `chrono` crate 格式化时间（禁止 shell `$(date)`）
   - 捕获 "nothing to commit"，视为成功，不报错
   - git 未安装时降级无版本历史模式，不阻断工作流

6. **离线模式自动识别**：
   - 检测网络连接，无网络时 UI 显示"离线模式"标注
   - 明确告知哪些功能受影响（只有多机同步需要网络，V1 核心不受影响）
   - 提供【导出配置包】按钮：将 `~/.ai-profile/` 打包为 zip，支持手动迁移

7. **推送错误处理**：系统错误转译为用户友好提示，严禁展示原始错误码

8. **同步 + 主动验证引导**：同步完成后显示摘要；设置页提供【验证注入】按钮

9. **优雅卸载（去无痕）**：【清除所有注入】按钮，还原所有 IDE 纯净状态

> ❌ **V1 不做**：SSOT Watcher（移至 V2）；Trae/Qoder Adapter（移至 V1.5）

---

## 🛡️ V1 工程底线（P0/P1）

| 问题域 | 解决方案 |
|:---|:---|
| **安装包信任危机** | OV 级代码签名证书；同时提供国内镜像下载（阿里云 OSS） |
| **幂等性（.mdc/SKILL.md）** | 直接全量覆盖 |
| **幂等性（CLAUDE.md）** | 标记块协议，4 种情况处理 |
| **已有文件保护** | 备份为 `.crossbrain-backup`，UI 弹提示 |
| **backup 二次运行** | `.crossbrain-backup` 已存在则跳过备份 |
| **路径三级检测** | 存在→推送；不存在但已装→自动创建；无法检测→弹填写框；未安装→跳过 |
| **完全离线支持** | V1 所有核心功能本地运行，无网络依赖 |
| **内网 git 支持** | V2 git remote 支持自定义 URL（GitLab/Gitea 等内网服务） |
| **错误处理** | 系统错误转译为用户友好提示 |
| **git 无变更提交** | 捕获 "nothing to commit"，视为成功 |
| **Windows 路径兼容** | `dirs` crate 解析 home 目录 |

---

## 📅 Day 1 Spike 清单（半天，开工前必验）

**所有验证在零上下文新对话里进行，防止假阳性。**

**Cursor：**
- [ ] `~/.cursor/rules/` 下的 `.mdc` frontmatter 在文件绝对行首时是否正确解析
- [ ] Cursor Agent Requested `description` 是否真正驱动按需加载
- [ ] 裸 `.md` vs 带 frontmatter `.mdc` 是否存在加载行为差异

**Claude Code：**
- [ ] `@~/.ai-profile/AGENTS.md`（含 `~`）在 Claude Code 里是否正确展开
- [ ] Windows 上 `@` 语法使用绝对路径是否可行
- [ ] Claude Code Skills `~/.claude/skills/` 是否官方支持，SKILL.md frontmatter 格式

**Trae（V1.5 Spike，优先于 Qoder）：**
- [ ] Trae 全局规则配置文件位置
- [ ] 是否支持 AGENTS.md 标准，还是有自己的 rules 机制
- [ ] 文件格式与 Cursor .mdc 是否兼容

**Spike 通过标准**：手工创建测试文件，空白新对话触发，行为符合预期即通过。
**Spike 失败预案**：每条 Spike 对应明确 Plan B（@import 不支持 `~` → 退化为直接写全量内容）。

---

## ⚠️ 发布前工程底线

- **代码签名**：OV 级证书；便携版 ≠ 免签名
- **国内下载镜像**：除 GitHub Releases 外，同步到国内 CDN/OSS，解决企业内网下载问题
- **README 第一节预埋反驳**：答案：幂等安全、旧数据保护、L2 原生路由、GUI、多机同步、**离线 100% 可用**
- **License**：建议 **Apache-2.0**（含专利保护条款）
- **项目名查重**：CrossBrain 在 GitHub/npm/crates.io 均无占用，可用

---

## 🔧 技术选型锁定

| 层 | 选型 | 版本 | 理由 |
|:---|:---|:---|:---|
| 桌面框架 | Tauri | **v2**（锁定） | v1/v2 API 不兼容 |
| 前端 | Vue 3 + Tailwind | latest stable | 计划已定 |
| Rust 时间处理 | `chrono` | latest | 替代 shell `$(date)`，跨平台 |
| Rust 路径解析 | `dirs` | latest | 跨平台 home 目录 |
| Rust git 集成 | `std::process::Command` | 系统 git | V1 简单选型；V2 迁移至 `git2`（libgit2，消除系统 git 依赖） |

---

## 🔮 V2/V3 演进路标

- **V1.5**：Trae Adapter（国内用户优先）；WorkBuddy Adapter（Spike 通过后）
- **V2**：SSOT 文件变动自动触发重推（单向 Watcher，`tauri-plugin-fs`）
- **V2**：迁移 git 集成至 `git2` crate（消除系统 git 依赖）
- **V2**：`~/.ai-profile/` git remote 配置，多机同步；支持内网 GitLab/Gitea + SSH/HTTP 代理
- **V2**：Qoder Adapter；Windsurf Adapter；Codex CLI（AGENTS.md 标准接入）
- **V2**：L0/L2 内容 UI 引导（文件大小警告、拆分建议）
- **V3**：Adapter 抽象为可通过 UI 编写的自定义插件
- **V3**：接入轻量本地 LLM 或 BYOK API（支持内网私有化部署模型）

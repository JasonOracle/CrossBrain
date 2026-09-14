# CrossBrain (跨脑) - 项目实施蓝图 V3.0 (最终实测定稿版)

> **版本演进**：
> - V2.0：废弃 Symlink/JSON，确定 Hub & Spoke 架构
> - V2.1：L2 挂原生懒加载，Claude 桩文件定死
> - V2.2：更名 CrossBrain，砍掉 V1 Watcher
> - V2.3：修复 7 处设计冲突，确立严苛 UX 规范（禁泄露内部术语）
> - V2.4：确立国内工具矩阵，确定离线 100% 兜底策略
> - V2.5：P0 孤儿清理机制 + 安装检测方法 + 三个高风险 Spike Plan B
> - **V3.0（本版·实测定稿）**：
>   1. **Spike 全线贯通**：实测验证 Claude Code 与 Antigravity IDE 的 Skills 懒加载 100% 成立。
>   2. **V1 核心矩阵修正**：剔除未安装的 Cursor，将 **Claude Code + Antigravity IDE (Gemini)** 作为 V1 双王牌，Cursor 移入 V2。
>   3. **Antigravity 规则机制固化**：L0 写入 `~/.gemini/config/rules/crossbrain-L0.md`（或全局 AGENTS.md），L2 写入 `~/.gemini/config/skills/crossbrain-{slug}/SKILL.md`。
>   4. **保护既有规则**：确认用户已有完善的 `CLAUDE.md` 与 `user_global.md`，适配器必须采用**标记块注入或备份**策略，严禁粗暴清空。

---

## 🎯 1. 核心定位与业务边界

CrossBrain 的核心价值：**个人全局规则** 与 **按需技能知识** 的跨工具统一分发。

- **全局规则**（内部 L0）：编码风格、语言偏好、思维框架——每次 AI 对话都必定生效。
- **技能知识**（内部 L2）：特定领域经验库——AI 按需加载，杜绝上下文污染与 Token 浪费。

> ⚠️ **UI 语言铁律**：L0/L2/Adapter/SSOT/slug/frontmatter 等内部研发术语**严禁在 UI 界面展示**。对外统一称为「全局规则」与「技能知识」。

---

## 🛠️ 2. 工具支持矩阵（依据真实环境定稿）

### V1 核心支持（两周内交付）

| 工具 | 公司/生态 | 全局规则 (L0) 路径与机制 | 技能知识 (L2) 路径与机制 | 实测状态 |
|:---|:---|:---|:---|:---|
| **Claude Code CLI** | Anthropic | `~/.claude/CLAUDE.md`（标记块或桩文件引用） | `~/.claude/skills/crossbrain-{slug}/SKILL.md` | ✅ 100% 实测通过 |
| **Antigravity IDE** | Google / Gemini | `~/.gemini/config/rules/crossbrain-L0.md` | `~/.gemini/config/skills/crossbrain-{slug}/SKILL.md` | ✅ 100% 实测通过 |
| **Gemini CLI** | Google | 共用 `~/.gemini/config/` 配置体系 | 共用 `~/.gemini/config/skills/` | ✅ 天然顺带支持 |

### V1.5 支持（国内高频与主流工具 Spike 拓展）

| 工具 | 生态 | 预估机制 | 状态 |
|:---|:---|:---|:---|
| **Trae** | 字节跳动 | VS Code 架构，探测全局 rules 目录或全局 AGENTS.md | 待 Spike |
| **ChatGPT IDE (原 Codex)** | OpenAI | 探索用户级全局 AGENTS.md 挂载 | 待 Spike |
| **OpenCode IDE** | 开源/国内 | 标准 AGENTS.md 体系 | 待 Spike |
| **WorkBuddy / Codebuddy** | 国内厂商 | 探测全局配置路径（避免混淆项目级 `.workbuddy/`） | 待 Spike |

### V2 拓展支持

| 工具 | 说明 |
|:---|:---|
| **Cursor** | 原 V1 候选，因用户未装移入 V2。使用 `.cursor/rules/*.mdc` 体系。 |
| **Windsurf / Qoder / Zcode** | 待后续环境就绪后逐个 Spike 接入。 |

---

## 🔌 3. 离线与内网保障策略

CrossBrain V1 的核心生命线是**纯本地文件 I/O**，天然 100% 离线可用，专为企业隔离内网与无网开发优化。

| 业务动作 | 网络依赖 | 机制保障 |
|:---|:---|:---|
| 规则编辑与存储 | ❌ 0 依赖 | 纯本地 Markdown 文件处理 |
| 分发到 Claude / Antigravity | ❌ 0 依赖 | 本地磁盘直接覆盖/写入 |
| 本地版本历史 | ❌ 0 依赖 | 本地 `git commit`（未装 git 时降级无历史模式，不阻断） |
| 跨设备多机同步（V2） | ✅ 需要 | 支持内网自建 GitLab/Gitea，不绑公网 GitHub |

---

## 🏗️ 4. 架构规范：单一事实源 (SSOT) 与分发引擎

### 4.1 本地数据仓库 (`~/.ai-profile/`)

```
~/.ai-profile/
├── .git/
├── .gitignore                   # 固定忽略 AGENTS.md 等动态中间产物
├── global/
│   └── rules.md                 # 全局规则（用户编辑的唯一源头）
├── knowledge/
│   ├── debug-rust.md            # 技能知识（每篇 = 一个独立经验主题）
│   ├── vue3-patterns.md
│   └── ...
└── AGENTS.md                    # 动态生成中间产物，包含 @import 或 rules.md 汇总
```

### 4.2 幂等分发流程与孤儿清理 (Orphan Cleanup)

```
用户在 UI 点击【立即同步】
  │
  ├─ 1. 读取 SSOT
  │      ├─ 解析 global/rules.md
  │      └─ 扫描 knowledge/*.md 并计算 slug-hash（如 crossbrain-vue3-a3f2b1）
  │
  ├─ 2. 执行 Adapter A: Antigravity IDE
  │      ├─ 写入 L0: ~/.gemini/config/rules/crossbrain-L0.md（直接覆盖）
  │      ├─ 写入 L2: ~/.gemini/config/skills/crossbrain-{slug-hash}/SKILL.md
  │      └─ 孤儿清理: 遍历 ~/.gemini/config/skills/ 下所有 crossbrain-* 目录，
  │                 删除不属于当前 knowledge 列表的历史技能目录
  │
  ├─ 3. 执行 Adapter B: Claude Code
  │      ├─ 写入 L0: ~/.claude/CLAUDE.md（标记块协议：保护用户既有规则）
  │      ├─ 写入 L2: ~/.claude/skills/crossbrain-{slug-hash}/SKILL.md
  │      └─ 孤儿清理: 遍历 ~/.claude/skills/ 下所有 crossbrain-* 目录，
  │                 删除不属于当前 knowledge 列表的历史技能目录
  │
  └─ 4. SSOT 本地 Git 提交
         执行 git add -A && git commit -m "sync: {ISO-8601-Time}"
         静默处理 "nothing to commit"，不弹报错
```

---

## ⚙️ 5. Adapter 详细实现规范

### 5.1 通用 Slug-Hash 生成算法
为了杜绝中文文件名、特殊字符导致的路径解析崩溃，并解决文件名修改后的精准追踪：
```
源文件: "Vue3 组件性能优化.md"
1. 提取英文字符/拼音并转小写 kebab-case，截断最多 30 字符: "vue3-perf"
2. 计算文件内容 SHA256，取前 6 位: "7c8e2a"
3. 组合: "crossbrain-vue3-perf-7c8e2a"
```

### 5.2 Antigravity Adapter
- **安装检测**：检查目录 `%USERPROFILE%\.gemini\config\` 是否存在。
- **L0 规则文件**：`%USERPROFILE%\.gemini\config\rules\crossbrain-L0.md`
  - 格式：纯 Markdown，完全由 CrossBrain 托管，直接覆盖。
- **L2 技能目录**：`%USERPROFILE%\.gemini\config\skills\crossbrain-{slug-hash}\SKILL.md`
  - 格式遵循 YAML Frontmatter 规范：
    ```markdown
    ---
    name: vue3-perf
    description: Vue3 组件性能优化：长列表虚拟滚动与响应式依赖控制
    ---
    [正文内容]
    ```

### 5.3 Claude Code Adapter
- **安装检测**：检查目录 `%USERPROFILE%\.claude\` 是否存在。
- **L0 规则文件**：`%USERPROFILE%\.claude\CLAUDE.md`
  - **采用标记块协议**（严禁直接清空，保护用户原有配置）：
    ```markdown
    <!-- CrossBrain:Start -->
    <!-- 本区域由 CrossBrain 自动维护，修改请在 CrossBrain App 中进行 -->
    @~/.ai-profile/AGENTS.md
    <!-- CrossBrain:End -->
    
    [用户既有的原有规则保持在标记块之外，互不干扰]
    ```
  - **备份保障**：初次发现无标记块且含内容时，先自动创建 `CLAUDE.md.crossbrain-backup`。
- **L2 技能目录**：`%USERPROFILE%\.claude\skills\crossbrain-{slug-hash}\SKILL.md`（格式与 Antigravity 保持 100% 一致）。

---

## 🔪 6. V1 工程任务拆解与交付工期 (两周落地)

### 第 1 阶段：工程底座初始化 (Day 1 - Day 2)
- [ ] 基于 `pnpm create tauri-app` 初始化 Tauri v2 + Vue 3 + TypeScript 骨架。
- [ ] 配置 Tailwind CSS 与 Naive UI 组件库。
- [ ] 封装 Rust `dirs` 与路径检测模块，初始化 `%USERPROFILE%\.ai-profile\` 目录底座。

### 第 2 阶段：Rust 核心分发引擎 (Day 3 - Day 5)
- [ ] 实现 `Adapter` trait 接口规范（`detect()`, `sync_l0()`, `sync_l2()`, `cleanup_orphans()`）。
- [ ] 实现 `AntigravityAdapter`。
- [ ] 实现 `ClaudeCodeAdapter`（含标记块读写与备份机制）。
- [ ] 实现孤儿清理与本地 Git 提交封装（`chrono` 时间戳）。

### 第 3 阶段：前端控制台与交互 (Day 6 - Day 8)
- [ ] 首次启动 5 步向导（工具自检 → 填写首条规则 → 一键同步 → 引导验证）。
- [ ] 规则管理工作台（全局规则编辑器 + 技能卡片列表与 Markdown 预览）。
- [ ] 同步状态指示（离线模式指示灯、上次同步结果摘要）。
- [ ] 设置页「一键卸载/去痕」功能（移除所有注入标记与技能目录）。

### 第 4 阶段：系统验证与打包交付 (Day 9 - Day 10)
- [ ] Windows 本地端到端流程跑通（新建技能 → 同步 → Claude Code / Antigravity 实际调用触发）。
- [ ] 离线模式断网实测。
- [ ] Tauri 生产构建打包为免安装便携版 / MSI 安装包。

---

## 🔧 7. 技术栈清单

| 分层 | 选型 | 说明 |
|:---|:---|:---|
| 桌面框架 | **Tauri v2** | 轻量高效，安全调用 Rust 系统底层 |
| 前端语言/框架 | **Vue 3 + TypeScript** | 严格类型约束，禁用 `any` |
| 样式系统 | **Tailwind CSS** | 现代原子化样式 |
| UI 组件库 | **Naive UI** | 完善的 Vue3 原生组件支持 |
| 图标方案 | **SVG (unplugin-icons / 本地 SVG)** | 严禁使用 Emoji 作为功能图标 |
| 系统底层依赖 | `dirs` crate + `chrono` crate | 路径与跨平台时间处理 |
| 包管理器 | **pnpm** | 强制使用，禁止 npm/yarn |
| Rust Git 集成 | `std::process::Command` (V1) → `git2` crate (V2) | V1 依赖系统 git，V2 内嵌 libgit2 消除外部依赖 |

---

## 🖥️ 8. UX 规范（用户体验铁律）

### 8.1 首次运行向导（5 步强制）

> **普通用户关键设计原则**：不要求用户理解任何技术细节，每一步只问一件事，每个按钮只做一件事。

```
步骤 1 / 欢迎
  └─ 主标题：「CrossBrain 让你的 AI 配置跨所有工具同步生效」
  └─ 副标题：「写一次，处处可用。全程在本机运行，无需联网。」
  └─ 唯一按钮：「开始配置」

步骤 2 / 检测你的 AI 工具
  └─ 展示检测到的工具（V1 只展示 Claude Code / Antigravity IDE / Gemini CLI）：
      ✅ Claude Code       推送位置：C:\Users\xxx\.claude\
      ✅ Antigravity IDE   推送位置：C:\Users\xxx\.gemini\config\
      ⏳ Trae              即将支持（不参与本次同步）
      ⏳ ChatGPT IDE       即将支持（不参与本次同步）
  └─ 每个已支持工具的推送路径可点击修改（高级用户）
  └─ 网络状态指示：🔴 无网络（同步完全不受影响，版本历史需 Git）

步骤 3 / 写你的第一条 AI 规则
  └─ 提供 3 个规则模板：「前端工程师」「后端/全栈开发者」「通用开发者」
  └─ 选择模板后右侧文本框自动填充，用户可直接修改
  └─ 提示文字：「这段规则会在所有 AI 对话中自动生效，用于告知 AI 你的编码偏好和工作习惯」

步骤 4 / 一键同步
  └─ 同步进度逐行显示：
      正在推送到 Claude Code... ✅ 完成
      正在推送到 Antigravity IDE... ✅ 完成
  └─ 完成摘要：「已将 1 条全局规则同步至 2 个工具」

步骤 5 / 验证生效（自助验证，非强制）
  └─ 引导语：「请在 Claude Code 新建对话，复制以下内容发送」
  └─ 展示测试提示词（可一键复制）
  └─ 两个按钮：「已验证，进入主界面」 /「跳过，直接进入」
```

### 8.2 系统错误→用户提示翻译表（强制执行，严禁展示原始错误码）

| 系统层错误 | 用户看到的提示 |
|:---|:---|
| `Permission denied` | 「没有写入权限，请检查目录访问权限」|
| `Directory not found` | 静默自动创建目录，无感知 |
| 自动创建目录失败 | 「无法创建配置目录，请手动创建后重试」|
| Git `nothing to commit` | 静默处理，视为成功，不弹提示 |
| Git 未安装 / 不在 PATH | 「未检测到 Git，规则同步正常，版本历史功能暂不可用 [如何安装 Git]」|
| 无网络连接 | 「当前无网络，规则同步正常，多机同步功能暂不可用」|
| `CLAUDE.md` 已备份，写入桩文件 | 「已将你的原有 Claude 配置安全备份到 CLAUDE.md.crossbrain-backup，如需恢复请查看此文件」|

### 8.3 技能知识（L2）用户引导规则

- **一篇 = 一个具体主题**（✅ 好例子：`Vue3 虚拟列表性能优化`；❌ 坏例子：`我的所有经验`）
- **超过 2000 字时**，编辑器顶部黄色警告条：「此技能知识偏长，建议按主题拆分为多篇，有助于 AI 精准加载」
- **新建时默认标题模板**：`# [技术/语言] · [具体场景]`（例：`# Rust · 生命周期与借用检查器常见错误`）

### 8.4 同步状态可视化

```
主界面顶部状态栏示例：

上次同步：2026-09-14 19:00   ✅ Claude Code   ✅ Antigravity   🔴 离线模式
[立即同步]
```

---

## 🛡️ 9. Adapter 降级（Plan B）策略

> Spike 已验证两个 Adapter 的核心路径均成立。以下 Plan B 仅在极端环境下触发，需在运行时自动探测并降级，用户透明感知。

### 9.1 Claude Code Adapter Plan B

| 功能 | 主路径（实测通过） | 降级条件 | Plan B |
|:---|:---|:---|:---|
| L0 全局规则 | `CLAUDE.md` 写入标记块 + `@~/.ai-profile/AGENTS.md` 引用 | `@~/.ai-profile/AGENTS.md` 路径展开失败 | 将 `global/rules.md` 完整内容直接写入 `CLAUDE.md` 标记块，不生成 AGENTS.md |
| L2 技能知识 | `~/.claude/skills/crossbrain-*/SKILL.md` | `skills/` 目录不支持懒加载 | 将所有知识标题追加到 `CLAUDE.md` 标记块内作为目录索引，降级为全量加载，放弃懒加载特性 |

### 9.2 Antigravity Adapter Plan B

| 功能 | 主路径（实测通过） | 降级条件 | Plan B |
|:---|:---|:---|:---|
| L0 全局规则 | `~/.gemini/config/rules/crossbrain-L0.md` | `rules/` 目录不被读取 | 将内容追加写入 `~/.gemini/config/AGENTS.md` 的标记块（该文件已确认存在且被读取） |
| L2 技能知识 | `~/.gemini/config/skills/crossbrain-*/SKILL.md` | 极端情况下懒加载失效 | 降级为 `alwaysApply: true` 格式，全量注入（需警告用户上下文将增加） |

---

## 🔒 10. CLAUDE.md 4 种处理情况完整协议

> 这是处理用户现有 `CLAUDE.md` 最关键的业务逻辑，**顺序判断，命中即止**。

```
情况一：文件不存在
  → 直接创建桩文件（内含 CrossBrain:Start 标记块 + @import 引用）
  → 无任何 UI 提示（静默完成）

情况二：文件存在，且已含 <!-- CrossBrain:Start --> 标记块
  → 用正则精准替换标记块内的内容（幂等）
  → 标记块外的用户内容原封不动
  → 无任何 UI 提示（静默完成）

情况三：文件存在，不含标记块，且 .crossbrain-backup 不存在
  → 先备份原文件为 CLAUDE.md.crossbrain-backup
  → 将 CrossBrain 标记块**追加到文件末尾**（不覆盖，不删除用户已有规则）
  → UI 弹出提示：「已将 CrossBrain 规则追加到你的 Claude 配置末尾。原文件已备份至
    CLAUDE.md.crossbrain-backup，如需恢复请查看此文件。」

情况四：文件存在，不含标记块，且 .crossbrain-backup 已存在
  → 跳过备份步骤（防止覆盖已有备份）
  → 同样执行追加标记块到末尾
  → 静默完成，不重复提示
```

> ⚠️ **核心原则**：CrossBrain **绝不删除、绝不覆盖**用户既有规则内容。标记块永远是**追加**，而非替换整个文件。

---

## ✅ 11. Day 1 Spike 最终结果（归档）

> 全部验证在零上下文全新对话中进行，防假阳性。

| Spike 项目 | 验证方式 | 结果 |
|:---|:---|:---|
| A. Claude Code `@~/.ai-profile/AGENTS.md` 中 `~` 路径展开 | 创建桩文件 + 新对话验证规则读取 | ✅ 通过，Windows `~` 可正确展开 |
| B. Claude Code Skills 目录懒加载（description 驱动） | 创建测试 SKILL.md + 新对话触发 | ✅ 通过，`SKILL_LOADED` 打印确认 |
| C. Antigravity IDE Skills 目录懒加载 | 创建测试 SKILL.md + 新对话触发 | ✅ 通过，`Analyzed SKILL.md #L1-7` 确认 |
| D. Antigravity L0 全局规则路径 | 探查目录结构 | ✅ 确认：`~/.gemini/config/rules/user_global.md` 已生效，可写入同目录 |

> 所有核心 Spike 全部通过，**可立即开工**。

---

## ⚠️ 12. V1 发布前工程底线

| 问题域 | 解决方案 |
|:---|:---|
| **安装包信任问题（Windows 蓝屏警告）** | 必须申请 OV 级代码签名证书；便携版同样需要签名（MoTW 机制触发） |
| **国内下载渠道** | GitHub Releases 之外，同步提供国内 OSS/CDN 镜像下载链接 |
| **幂等性（工具配置文件）** | CrossBrain 完全托管的文件直接全量覆盖；`CLAUDE.md` 用标记块追加协议 |
| **孤儿文件清理** | 每次同步后扫描推送目录，删除不在当前 `knowledge/` 列表中的旧 `crossbrain-*` 条目 |
| **路径三级检测** | 存在→推送；检测到安装但目录缺失→自动创建；无法确认→弹填写框；未安装→跳过不报错 |
| **git 提交健壮性** | 使用 `chrono` 生成时间戳（**禁用 shell `$(date)`，Windows 完全不兼容**）；捕获 `nothing to commit` 静默成功 |
| **Windows 路径** | 全程使用 `dirs` crate 解析 home 目录；路径拼接使用 Rust `Path` API，不用字符串拼接 |
| **安全声明** | README 首屏明确：CrossBrain 仅写入用户自己创建的内容；V1/V2 不支持从 URL 导入规则（防供应链攻击） |
| **License** | **Apache-2.0**（含专利保护条款，对开源生态友好） |

### README 需预埋的反驳声明

> **「Shell 脚本不也能干这个？」**
>
> 不一样的地方：CrossBrain 提供的是 Shell 脚本几乎必定缺失的能力组合——
> 幂等安全（标记块协议） + 既有配置保护（备份机制）+ 孤儿清理（避免幽灵规则残留） + 跨平台 GUI + 多机同步 + **100% 离线可用** + 懒加载路由（L2 精准分发，不污染上下文）。

---

## 🔮 13. V2 / V3 演进路标

### V1.5（V1 上线后立即启动）
- Trae Adapter（国内用户高频，VS Code 架构，Spike 后优先接入）
- ChatGPT IDE / OpenCode Adapter（AGENTS.md 标准体系，Spike 后接入）
- WorkBuddy / Codebuddy Adapter（探测全局配置路径后接入）
- 导出配置包功能（含 `.git/` 的完整迁移包 + 纯规则包双选）
- UI 优化：本地版本历史时间轴查看与回滚

### V2
- **SSOT 文件变动自动触发重推**（单向 Watcher，`tauri-plugin-fs`，无需手动点同步）
- **Rust Git 内嵌**：迁移至 `git2` crate（libgit2），消除对系统已装 Git 的依赖
- **多机同步 UI**：配置 `~/.ai-profile/` git remote（支持 GitHub、内网 GitLab、Gitea 等任意 git 服务）
- Cursor Adapter 正式接入（`.cursor/rules/*.mdc`，Agent Requested 懒加载）
- Windsurf / Qoder / Zcode Adapter（各需独立 Spike）

### V3
- Adapter 抽象为用户可通过 UI 配置的自定义插件（无代码接入任意新工具）
- 接入轻量本地 LLM 或 BYOK API（支持私有化部署模型，内网 Ollama 等）
- 团队版：多人共享规则集与权限管理

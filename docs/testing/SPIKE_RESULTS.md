# CrossBrain Spike 验证归档

> 归档时间：2026-09-14  
> 说明：全部验证在**零上下文全新对话**中进行，防止上下文污染导致假阳性

---

## V1 核心 Spike 结果（全部通过）

| Spike | 验证项目 | 结果 |
|:---|:---|:---|
| A | Claude Code `@~/.ai-profile/AGENTS.md` 中 `~` 路径展开 | ✅ 通过 |
| B | Claude Code Skills 目录懒加载（description 驱动） | ✅ 通过 |
| C | Antigravity IDE Skills 目录懒加载 | ✅ 通过 |
| D | Antigravity L0 全局规则路径确认 | ✅ 通过 |

---

## Spike A — Claude Code `~` 路径展开

**验证项**：在 `CLAUDE.md` 中写入 `@~/.ai-profile/AGENTS.md`，`~` 是否在 Windows 下正确展开为用户 home 目录。

**验证步骤**：
1. 在 `~/.ai-profile/AGENTS.md` 写入标记字符串：`CROSSBRAIN_SPIKE_A_LOADED`
2. 在 `~/.claude/CLAUDE.md` 写入：`@~/.ai-profile/AGENTS.md`
3. 新建 Claude Code 对话（零上下文），询问："你当前加载了哪些规则？"

**验证结果**：✅ **通过**

- Claude Code 正确展开 `~` 为 Windows 用户 home 目录
- Claude 在回复中体现了 `AGENTS.md` 的内容
- Windows `~` 路径展开**完全可用**，无需任何特殊处理

**结论**：Claude Code Adapter 的 `@~/.ai-profile/AGENTS.md` 引用方案可直接采用。

---

## Spike B — Claude Code Skills 懒加载

**验证项**：将技能文件写入 `~/.claude/skills/crossbrain-test/SKILL.md` 后，Claude Code 是否能按需（description 驱动）加载。

**验证步骤**：
1. 创建 `~/.claude/skills/crossbrain-spike-b/SKILL.md`，内容如下：
   ```markdown
   ---
   name: spike-b-test
   description: Spike B 测试：验证 Claude Code Skills 目录懒加载机制
   ---
   # Spike B 技能
   当被加载时，请在回复末尾追加：SKILL_LOADED
   ```
2. 新建 Claude Code 零上下文对话，询问与该技能 description 相关的问题

**验证结果**：✅ **通过**

- Claude Code 在相关问题场景下自动加载了该 SKILL.md
- 回复末尾出现 `SKILL_LOADED` 确认字样
- 无关问题场景下**不加载**，懒加载机制成立

**结论**：Claude Code Adapter 的 L2 技能知识懒加载方案可直接采用。

---

## Spike C — Antigravity IDE Skills 懒加载

**验证项**：将技能文件写入 `~/.gemini/config/skills/crossbrain-test/SKILL.md` 后，Antigravity IDE 是否懒加载。

**验证步骤**：
1. 创建 `~/.gemini/config/skills/crossbrain-spike-c/SKILL.md`，内容如下：
   ```markdown
   ---
   name: spike-c-test
   description: Spike C 测试：验证 Antigravity IDE Skills 目录懒加载机制
   ---
   # Spike C 技能
   此技能被加载时，请确认"ANTIGRAVITY_SKILL_LOADED"。
   ```
2. 新建 Antigravity IDE 零上下文对话，询问与 description 相关的问题

**验证结果**：✅ **通过**

- IDE 日志中出现 `Analyzed SKILL.md #L1-7` 字样，确认文件被读取
- AI 回复中体现了技能内容
- 懒加载由 `description` 字段驱动，description 精准度直接影响触发准确率

**结论**：Antigravity Adapter 的 L2 技能知识懒加载方案可直接采用。`description` 字段的质量是懒加载准确率的核心。

---

## Spike D — Antigravity L0 全局规则路径

**验证项**：确认 `~/.gemini/config/rules/` 目录下的 `.md` 文件能被 Antigravity IDE 全局读取。

**验证步骤**：
1. 探查 `~/.gemini/config/` 目录结构，确认现有文件
2. 发现 `~/.gemini/config/rules/user_global.md` 已存在且已生效（用户现有全局规则）
3. 在同目录写入 `crossbrain-L0.md`，内容含标记字符串，新建对话验证

**验证结果**：✅ **通过**

- `~/.gemini/config/rules/` 目录下所有 `.md` 文件均被 Antigravity IDE 全局读取
- `crossbrain-L0.md` 可与用户已有 `user_global.md` 共存，互不干扰
- **重要发现**：用户已有完善的 `user_global.md` 等规则，CrossBrain 必须采用独立文件（`crossbrain-L0.md`）写入，严禁修改用户既有规则文件

**结论**：Antigravity Adapter 的 L0 路径 `~/.gemini/config/rules/crossbrain-L0.md` 完全成立，独立文件策略正确。

---

## 待接入工具路径预探测（2026-09-14）

> ⚠️ **这是「文件系统结构探测」，不是 Spike。** 只回答了「目标路径在哪、长什么样」，
> 未验证「写入后工具能否读到」。按 `ADAPTER_SPEC` 第 6 节，正式接入前**仍须完成 Spike 实测**。

探测动机：用户反馈向导只识别 2 个工具，逐一核对其余工具在本机的真实落点。

| 优先级 | 工具 | 本机路径 | 路径形态 | 待验证 |
|:---|:---|:---|:---|:---|
| P0 | **Codex** | `~/.codex/AGENTS.md` | **0 字节空文件**（links=1）；同目录另有 `rules/default.rules`、`skills/`（含 `.system/` 内置 6 个 + 用户 `grill-me`）。**已确认桌面 App/CLI/扩展共用此目录** | 写入后是否被读取（预期最易，走「情况一」无备份） |
| P0 | **OpenCode** | `~/.config/opencode/AGENTS.md` | 1908 字节，**links=5**（在硬链接组内）；**无 `skills/` 目录** | 写入后是否被读取；技能落点是否存在 |
| P1 | **CodeBuddy** | `~/.codebuddy/rules/*.mdc` | `.mdc` 格式；已有用户自建文件（7810 字节，links=1）；`skills/` 存在 | `.mdc` frontmatter 规范；能否只追加不覆盖 |
| P1 | **WorkBuddy** | `~/.workbuddy/MEMORY.md`（13622 字节）+ `skills/` | **无 `rules/` 目录**；「记忆文件 + 技能库」模型 | 有无规则注入点；`MEMORY.md` 能否安全追加 |
| P2 | **Cursor** | `~/.cursor/rules/user_profile.md` | 1908 字节，**links=5**（在硬链接组内） | `.mdc` 体系差异 |
| P2 | **ZCode** | `~/.zcode/cli/`、`AppData/Roaming/ZCode/` | 智谱 GLM 系（`setting.json` 含 `bigmodel`/`glm`）；记忆**按项目**分目录存（`cli/memories/projects/{项目名}-{hash}/memory`） | **未见全局规则落点**，须确认是否存在全局 prompt 机制 |
| P2 | **Qoder** | `~/.qoder/logs/` | **仅 2 个日志文件**，无任何配置 | 工具侧暂无可写落点，无法接入 |

### 关键观察

1. **Codex 是性价比最高的下一个 Adapter**：`~/.codex/AGENTS.md` 是**空文件** →
   接入时命中「情况一」（不备份、不断链、零冲突），且 `~/.codex/` 的
   「全局 AGENTS.md + `skills/`」形态与 Antigravity **完全同构**，Adapter 可高度复用。
2. **用户的 OpenCode 与 Cursor 已在共享全局记忆**：这两个工具各自路径（`~/.config/opencode/AGENTS.md`、
   `~/.cursor/rules/user_profile.md`）都在那 5 路硬链接组内 —— 即用户**已用硬链接手工实现了
   CrossBrain 的目标场景**。D-03 决策（断开 `~/.claude/CLAUDE.md` 一侧）会切断 CrossBrain 对这条通路的利用，
   接入这两个工具前须重新评估该通路是否应保留（见 `DECISION_LOG.md` ADR-02）。
3. **PRD 的「ChatGPT IDE」命名不准确，且「Codex CLI」也不准确**：2026-09-14 二次探测确认，
   本机装的是 **Codex 桌面应用**（MSIX 包身份 `OpenAI.Codex`，官方 DisplayName = `Codex`，
   装在 `D:\soft\win-x64\`，自带 `resources/codex.exe` CLI 本体），而桌面 App、内置 CLI、
   Chrome 扩展三者**共用同一个 Codex home `~/.codex/`**。故正确工具名是 **Codex**，
   PRD 第 3 节工具矩阵已两次更正（「ChatGPT IDE」→「Codex CLI」→「Codex」）。
   附加发现：`~/.codex/AGENTS.override.md` 优先级**高于** `AGENTS.md`，
   它存在会让 CrossBrain 的规则完全不生效 → 已加入同步前强制检测（`reject_if_override_present`）。
4. **`~/.trae/` 本机不存在**（用户未安装 Trae），但向导仍按 PRD 展示该项占位。
5. **本机另有 PRD 未列出的 AI 工具目录**（是否纳入待定）：
   `.antigravity-ide/`、`.joycode/`、`.joycoder/`、`.joycode-editor/`、`.marscode/`、
   `.codebuddycn/`、`.claude-code-router/`、`.cc-switch/`、`.agents/`、`.mem0/`、`.mnemon/`。

---

## ✅ TASK-18 Spike（Codex — 实测完成，2026-09-14 23:4x）

2026-09-14 已完成 Codex 的 **Adapter 实现、干跑验证与真实 Spike**
（`cargo test` 110 passed、`dryrun_sync` 10 节 0 项失败、真实数据零改动）。

### 🎯 关键方法：用官方调试命令无头验证（无需人工开会话）

实测中发现了 Codex CLI 自带的一个**官方调试入口**，可把「模型实际看到的全部输入」
以 JSON 形式打印出来 —— 这让 Spike 从「必须人工开一次 GUI 会话」变成**完全自动化**：

```bash
# 在任意目录执行；输出为 JSON，含 system prompt / <INSTRUCTIONS> / <skills_instructions>
"/d/soft/win-x64/app/resources/codex.exe" debug prompt-input > pi.json
```

- `codex debug prompt-input` —— *Render the model-visible prompt input list as JSON*
- 同族命令还有 `codex debug models`（模型目录）、`codex debug app-server`

> **这是本任务最有复用价值的发现。** 后续接入任何**自带 CLI** 的工具（OpenCode、
> CodeBuddy、WorkBuddy…），都应先翻它的 `--help` / `debug --help`，找这类
> 「dump 模型可见输入」的命令 —— 能把 `ADAPTER_SPEC.md` 第 6 节的 Spike 从
> 「需要用户配合」降级为「一条命令」。若工具没有，再退回人工会话验证。

### 实测结果（三项全部通过）

| # | 验证项 | 方法 | 结果 |
|:---|:---|:---|:---|
| 1 | `~/.codex/AGENTS.md` 写入后是否被读取 | 写入探针口令 → `debug prompt-input` → grep 口令 | ✅ **通过**，口令出现在 `<INSTRUCTIONS>` 块内 |
| 2 | `~/.codex/skills/{name}/SKILL.md` 是否被纳入技能清单 | 写入探针技能 → `debug prompt-input` → grep 技能名 | ✅ **通过**，条目含 name + description + `file:` 定位器 |
| 3 | `AGENTS.md` 作用域（全局 vs 项目级） | 在**中立目录**（`%TEMP%\cbspike`，非任何项目根）下执行 | ✅ **全局**，无需项目上下文即被加载 → L0 策略成立 |

> 方法论上补一句：第 1、3 项必须**在项目目录之外**执行，否则无法区分「全局加载」
> 与「项目根 AGENTS.md 加载」——本机项目根本身就有 `AGENTS.md`。

**L2 落点最终确认**：`~/.codex/skills/{slug}/SKILL.md` 出现在模型可见清单中，
条目形如
`- crossbrain-spike-load: <description> (file: C:/Users/Administrator/.codex/skills/crossbrain-spike-load/SKILL.md)`
—— 与本项目 `format_skill_md()` 生成的格式（`name` + `description` frontmatter，
**不带** `disable-model-invocation`）完全吻合，说明生成的技能会被模型看到且可隐式调用。

### 本次探测挖出的新事实（影响后续 Adapter）

**① Codex 的技能根目录不止一个（共三类）。** 完整清单的 `file:` 定位器暴露了：

| 技能根 | 内容 | 归属 |
|:---|:---|:---|
| `~/.codex/skills/.system/` | 6 个内置技能（imagegen / openai-docs / plugin-creator / skill-creator / skill-installer / review-agent） | Codex 内置，**严禁触碰** |
| `~/.codex/skills/{name}/` | 用户技能（`grill-me`） | **CrossBrain 的 L2 落点** ✅ |
| `~/.codex/plugins/cache/` | 插件技能（openai-bundled / openai-primary-runtime） | 插件系统管理，不碰 |
| `~/.agents/skills/{name}/` | **86 个用户技能，59 MB**（第三方跨宿主技能库） | **不是我们的落点**，不得写入 |

> `~/.agents/` 是另一套**跨宿主技能生态**的目录：内有 `.skill-lock.json`（v3，记录
> GitHub 来源如 `obra/superpowers`）、按宿主分的 `agents/openai.yaml`、以及一份
> **3349 字节的 `~/.agents/AGENTS.md`**（「极客前端工程架构与 AI 行为准则」）。
> 已用二进制证据确认：Codex 的全局指令加载器是 `codex-home/src/instructions/mod.rs`，
> 其中只硬编了 `AGENTS.override.md` / `AGENTS.md` **一对文件名**，**不读 `~/.agents/AGENTS.md`**。
> 该文件的归属者待查（不影响 V1）。
>
> ⚠️ **对后续 Adapter 的警示**：`~/.agents/skills/` 体量大且被多个工具共享，
> 任何「清理孤儿」逻辑都必须**严格限定在自己写入的那一个根**（本项目靠
> 「只处理 `crossbrain-` 前缀 + 只扫 `~/.codex/skills/`」双重收窄），
> 否则会误删用户 86 个技能。

**② 技能包可以带宿主专属配置。** `~/.agents/skills/grill-me/agents/openai.yaml`
是 Codex 专属的技能元数据（`interface.display_name` / `policy.allow_implicit_invocation`）。
本项目只写 `SKILL.md`，**不写** `agents/` 子目录 —— 属有意收窄，避免越权。

**③ `disable-model-invocation: true` 会让技能从清单中消失。** 用户自建的 `grill-me`
两处副本（`~/.codex/skills/grill-me`、`~/.agents/skills/grill-me`）**都不在**清单里，
原因就是该标记 —— 这解释了为什么「磁盘上有、清单里没有」不一定是落点问题。

### 探针清理记录

| 动作 | 结果 |
|:---|:---|
| 删除 `~/.codex/skills/crossbrain-spike-load/` | ✅ 已删，目录回到 `.system` + `grill-me` |
| 恢复 `~/.codex/AGENTS.md` 为 0 字节 | ✅ 0 bytes / links=1 |
| `~/.agents/` 是否被触碰 | ✅ 零改动（`AGENTS.md` md5 `cb47d1ce…` 不变、`grill-me` mtime 保持 `2026-09-05 17:53`） |
| 5 路硬链接组 | ✅ md5 全等 `47645f60…` |

### 官方文档结论（其中「全局作用域」「技能目录」两项已由上述实测复核）

来源：<https://developers.openai.com/codex/guides/agents-md> 与
<https://developers.openai.com/codex/concepts/customization/>

| 结论 | 内容 |
|:---|:---|
| **全局作用域成立** | 全局指令就在 Codex home 目录（默认 `~/.codex/`，可用 `CODEX_HOME` 覆盖）——**不是**项目级，L0 策略成立 |
| **桌面 App 与 CLI 同源** | 「在桌面 App 里配置自定义指令，本质上也是写入 Codex home 的 AGENTS.md，与写入 `~/.codex/AGENTS.md` 等价」 |
| **空文件被跳过** | Codex 会忽略空文件 → 当前 0 字节的 `AGENTS.md` 尚未生效，写入后即生效 |
| **`AGENTS.override.md` 优先级更高** | 同一层级**只取第一个非空文件**：先 override，没有再回落 `AGENTS.md` |
| **合并上限 32 KiB** | `project_doc_max_bytes` 默认 32768，超出会截断（全局 + 项目合并计算） |
| **技能目录成立** | 全局技能即 `~/.codex/skills/{name}/SKILL.md`（用户自建的 `grill-me` 就在这里），与官方「skills can be global」一致 |
| **`AGENTS.md` 检索顺序** | 全局 → 项目根 → 沿目录树向下至 cwd，越靠近 cwd 优先级越高 |

### 干跑已验证的部分（代码行为，非工具行为）

标记块注入、内联规则全文、未混入 `@` 引用、`skills/.system/` 未被触碰、
三个 Adapter 行为一致、`AGENTS.override.md` 遮蔽时端到端上报为失败（`report.ok == false`
且未写出 `AGENTS.md`）。

---

## V1.5 待 Spike 项目

以下 Spike 在 V1 上线后立即启动（本机路径已由上一节预探测确认）：

| 优先级 | 工具 | 验证项 | 目标路径 |
|:---|:---|:---|:---|
| P0 | **OpenCode** | 写入 `AGENTS.md` 后是否被读取；确认技能落点 | `~/.config/opencode/AGENTS.md` |
| P1 | **CodeBuddy** | `.mdc` 规范确认 + 追加而非覆盖的可行性 | `~/.codebuddy/rules/` |
| P1 | **WorkBuddy** | 全局规则注入点确认（勿与项目级 `.workbuddy/` 混淆） | `~/.workbuddy/MEMORY.md` |
| P2 | **Trae**（字节跳动） | 全局 rules 目录路径确认 + 懒加载验证 | 本机未安装，需用户侧安装后再探 |
| P2 | **ZCode / Qoder** | 确认是否存在任何可写的全局规则落点 | 见上表（当前均未见落点） |

> 💡 **执行提示（来自 TASK-18 的经验）**：若待测工具**自带 CLI**，先跑
> `<cli> --help` 与 `<cli> debug --help`，找是否有「dump 模型可见输入 / 已加载指令」
> 这类命令。Codex 的 `codex debug prompt-input` 就是这样把 Spike 变成一条命令的。
> 另外务必**在项目目录之外**执行，否则无法区分「全局加载」与「项目根 AGENTS.md 加载」。

### ⚠️ 本机一处体量大、易误伤的既有资产

`~/.agents/`（59 MB，其中 `skills/` 有 **86 个技能**）是**另一套跨宿主技能生态**的目录，
含 `.skill-lock.json`（v3，来源 `obra/superpowers` 等 GitHub 仓库）、按宿主分的
`agents/openai.yaml`，以及一份 3349 字节的 `~/.agents/AGENTS.md`。

- Codex **会**把 `~/.agents/skills/` 作为技能根之一扫描（见上文实测）
- 但 Codex **不会**把 `~/.agents/AGENTS.md` 当全局指令（加载器只认 `~/.codex/` 下那一对文件名）
- **任何工具接入时都不得对 `~/.agents/` 做任何写入或清理**

### 新 Spike 验证模板

```markdown
## Spike X — {工具名} {验证项}

**验证项**：...

**验证步骤**：
1. ...
2. ...

**验证结果**：[ ] 通过 / [ ] 失败 / [ ] 部分通过

**结论与 Plan B**：...
```

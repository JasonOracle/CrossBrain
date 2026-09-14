# CrossBrain — AI 工具接手总纲（MASTER CONTEXT）

> **你是一个 AI 编程工具。请先完整读完本文件，再读其他任何文件。**  
> 本文件是你理解项目的唯一起点。读完后，按第 5 节的指引找到你的下一步任务。

---

## 1. 项目是什么（3 句话版本）

CrossBrain 是一个 **Windows 桌面应用**（Tauri v2 + Vue 3 + Rust），帮助用户把自己的 AI 编程规则（编码偏好、语言习惯）和技能知识（特定领域经验）**一次写好，自动同步到多个 AI 工具**（Claude Code、Antigravity IDE 等）。

所有操作均为**纯本地文件读写**，零网络依赖，天然支持企业内网。

V1 目标：支持 Claude Code + Antigravity IDE，两周内交付可用版本。

---

## 2. 当前项目状态

> **查看最新状态前，先读这里。**

请立即读取：[`CURRENT_STATUS.md`](./CURRENT_STATUS.md)

它告诉你：
- 当前进行到第几天 / 第几阶段
- 哪些任务已完成（打 ✅ 的）
- 你现在需要做什么（打 🔲 的，排在最前面的就是你的任务）
- 哪些任务有阻塞（打 🚫 的）

---

## 3. 铁律（不可违反，任何情况下都不能改）

以下规则由人工验证并写死，**AI 工具不得以任何理由修改、绕过或"优化"**：

| 编号 | 铁律内容 | 违反后果 |
|:---|:---|:---|
| 🔴 L-01 | 包管理器只能用 **pnpm**，禁止 npm / yarn | 构建失败，锁文件冲突 |
| 🔴 L-02 | 路径处理只能用 `dirs` crate + Rust `Path` API，禁止字符串拼接 | Windows 非 C 盘用户路径崩溃 |
| 🔴 L-03 | 时间戳只能用 `chrono` crate，禁止调用 shell `date` 命令 | Windows 完全不兼容 |
| 🔴 L-04 | TypeScript 禁止 `any`，禁止 `as any` | 类型安全崩溃 |
| 🔴 L-05 | 图标只能用 SVG，禁止 Emoji 作为功能图标 | 设计规范违反 |
| 🔴 L-06 | 对 CLAUDE.md 操作必须遵循四情况协议，禁止直接覆盖整个文件 | 用户数据丢失 |
| 🔴 L-07 | UI 界面禁止出现 L0/L2/Adapter/SSOT/slug/frontmatter 等内部术语 | 产品规范违反 |
| 🔴 L-08 | Antigravity 的 crossbrain-L0.md 只能直接覆盖（不用标记块），Claude Code 的 CLAUDE.md 必须用标记块 | 两者机制不同，混用出错 |
| 🔴 L-09 | 孤儿清理只能删除 `crossbrain-` 前缀的目录，严禁误删用户自建目录 | 用户数据丢失 |
| 🔴 L-10 | 引入新依赖前必须先检查现有 package.json / Cargo.toml | 禁止重复造轮子 |

---

## 4. 文档地图（你需要读哪些文件）

### 必读顺序

```
1. 本文件（MASTER_CONTEXT.md）           ← 你现在在这里
2. CURRENT_STATUS.md                     ← 确认当前进度和你的任务
3. docs/impl/TASK_BREAKDOWN.md           ← 你的任务的详细实现步骤
4. （按需）以下参考文档
```

### 按需参考文档

| 你需要了解 | 读这个文件 |
|:---|:---|
| 整体架构、SSOT 目录、分发流程图 | [`docs/tech/ARCHITECTURE.md`](./docs/tech/ARCHITECTURE.md) |
| Adapter trait 定义、两个 Adapter 实现规范、CLAUDE.md 四情况协议 | [`docs/tech/ADAPTER_SPEC.md`](./docs/tech/ADAPTER_SPEC.md) |
| Tauri / Vue / Rust 技术栈选型和约束 | [`docs/tech/TECH_STACK.md`](./docs/tech/TECH_STACK.md) |
| Windows 路径、时间戳、签名等专项陷阱 | [`docs/tech/WINDOWS_COMPATIBILITY.md`](./docs/tech/WINDOWS_COMPATIBILITY.md) |
| 产品功能、5步向导、错误提示翻译表 | [`docs/product/PRD.md`](./docs/product/PRD.md) |
| 测试用例、验收标准 | [`docs/testing/TEST_PLAN.md`](./docs/testing/TEST_PLAN.md) |
| 已验证的 Spike 结果（不要重复验证这些） | [`docs/testing/SPIKE_RESULTS.md`](./docs/testing/SPIKE_RESULTS.md) |
| 为什么做这个技术决策（防止你推翻它） | [`docs/impl/DECISION_LOG.md`](./docs/impl/DECISION_LOG.md) |
| 人工改了代码后需要更新哪些文档 | [`docs/impl/HANDOFF_PROTOCOL.md`](./docs/impl/HANDOFF_PROTOCOL.md) |
| 开发环境搭建 | [`docs/impl/DEV_SETUP.md`](./docs/impl/DEV_SETUP.md) |

---

## 5. 你的下一步操作

### Step A：确认你的身份

你是谁？你接手了什么？

- 如果你是**第一个启动项目的工具** → 读 `docs/impl/DEV_SETUP.md`，初始化项目骨架
- 如果你是**中途接手的工具** → 读 `CURRENT_STATUS.md`，找到第一个 🔲 任务
- 如果你被要求**修复某个 Bug** → 读 `docs/impl/HANDOFF_PROTOCOL.md` → 找到对应章节

### Step B：找到你的具体任务

打开 [`docs/impl/TASK_BREAKDOWN.md`](./docs/impl/TASK_BREAKDOWN.md)，找到对应任务编号（如 `TASK-03`），按照里面的步骤逐一执行。

### Step C：完成任务后必须做的事

1. 在 `CURRENT_STATUS.md` 中将该任务标记为 ✅
2. 如果你修改了任何现有代码，检查 `docs/impl/HANDOFF_PROTOCOL.md` → 确认哪些文档需要同步更新
3. 在 `CURRENT_STATUS.md` 的「本次变更记录」中追加你做的内容

---

## 6. 关键路径（绝对不能走错的设计）

以下是这个项目最核心的 3 个设计，理解它们你就理解了 90% 的代码逻辑：

### 6.1 SSOT 单一事实源

用户只在 `~/.ai-profile/global/rules.md` 和 `~/.ai-profile/knowledge/*.md` 编辑内容。  
**所有 AI 工具的配置目录都是只写目标，用户不直接操作它们。**

### 6.2 两种写入策略（不能搞混）

| 目标工具 | 写入策略 | 原因 |
|:---|:---|:---|
| **Antigravity IDE** L0 | 直接全量覆盖 `crossbrain-L0.md` | 该文件 100% 由 CrossBrain 托管 |
| **Claude Code** L0 | 标记块注入到 `CLAUDE.md`（四情况协议） | 用户可能有自己的规则在此文件 |

### 6.3 孤儿清理的边界

只清理 `crossbrain-` 前缀的目录。  
永远不碰没有 `crossbrain-` 前缀的目录（可能是用户自建的）。

---

## 7. 已完成的预研工作（不需要再验证）

以下 4 项技术可行性已通过人工实测，直接按结论编码即可，不需要再做 Spike：

| 项目 | 结论 |
|:---|:---|
| Claude Code `@~/.ai-profile/AGENTS.md`，Windows `~` 路径展开 | ✅ 可用 |
| Claude Code `~/.claude/skills/` 懒加载（description 驱动） | ✅ 可用 |
| Antigravity IDE `~/.gemini/config/skills/` 懒加载 | ✅ 可用 |
| Antigravity IDE `~/.gemini/config/rules/` 全局规则路径 | ✅ 可用 |

详细验证报告：[`docs/testing/SPIKE_RESULTS.md`](./docs/testing/SPIKE_RESULTS.md)

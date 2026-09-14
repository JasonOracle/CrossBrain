# CrossBrain — AI 工具自动指令（AGENTS.md）

> 本文件由 AI 编程工具（Claude Code / OpenCode / WorkBuddy 等）**自动加载**。  
> 你读到这里，说明你已进入 CrossBrain 项目。**立即按以下步骤执行，不得跳过。**

---

## ⚡ 你的第一个动作

**现在立即读取以下两个文件，完整读完后再做任何事：**

1. [`MASTER_CONTEXT.md`](./MASTER_CONTEXT.md) — 项目总纲、铁律、文档地图
2. [`CURRENT_STATUS.md`](./CURRENT_STATUS.md) — 当前进度、你的任务、阻塞项

读完后，从 `CURRENT_STATUS.md` 中找到第一个 🔲 任务，打开 `docs/impl/TASK_BREAKDOWN.md` 找到对应任务编号，逐步骤执行。

---

## 🔴 安全红线（任何情况下不可妥协）

以下涉及**用户数据安全**，绝对不能违反，没有任何例外：

1. **绝不**直接覆盖整个 `~/.claude/CLAUDE.md` 文件 → 必须用标记块协议（原因见 DECISION_LOG.md ADR-01）
2. **绝不**删除非 `crossbrain-` 前缀的目录 → 可能误删用户自建内容（原因见 ADR-09）
3. **绝不**在 UI 界面出现 L0/L2/Adapter/SSOT/slug 等内部术语 → 用户完全看不懂
4. **绝不**在未查阅 `docs/impl/DECISION_LOG.md` 的情况下推翻现有设计决策
5. **绝不**把 `node_modules/`、`dist/`、`src-tauri/target/`、任何密钥提交进 git

---

## 🟡 优先级规则（优先执行，不行再降级，不要卡死）

以下是工程规范，**优先遵守，但环境不支持时有兜底方案，不要为此卡死流程**：

| 规则 | 首选 | 兜底（环境不支持时） |
|:---|:---|:---|
| 包管理器 | **pnpm**（`pnpm --version` 验证） | pnpm 不可用时降级用 npm，并在 `CURRENT_STATUS.md` 变更记录中注明 |
| Rust 路径处理 | `dirs` crate + `Path` API | 若 dirs crate 有兼容问题，用 `std::env::var("USERPROFILE")` 替代，不用字符串硬编码盘符 |
| 时间戳生成 | `chrono` crate | 若 chrono 不可用，用 `std::time::SystemTime`，绝对不调用 shell `date` |
| TypeScript 类型 | 严格类型，禁 `any` | 临时 `any` 必须加注释 `// TODO: 补充类型`，不能静默留下 |

---

## 📋 每次完成任务后必须做

1. 将 `CURRENT_STATUS.md` 中完成的任务标记为 ✅
2. 在「本次变更记录」中追加你的操作，**时间精确到时分秒**，格式：
   ```
   ### YYYY-MM-DD HH:MM:SS — {工具名}
   - 完成：TASK-XX，做了什么
   - 修改文件：列出所有改动文件
   - 注意事项：遇到的坑或特殊决定
   ```
3. 如果你修改了代码，检查 `docs/impl/HANDOFF_PROTOCOL.md` — 确认哪些文档需要同步更新
4. **git 提交一次**（见下节）

---

## 🔀 版本控制

- 远端：`git@github.com:JasonOracle/CrossBrain.git`，主分支 `main`
- **每完成一个 task 提交一次**，不要把多个 task 混进同一个提交
- 提交信息（中文，Conventional Commits 前缀）：

  ```
  <type>: TASK-XX <一句话说明>

  <做了什么、为什么这么做；踩到的坑>

  验证：cargo test <N> passed / dryrun_sync <N> section 0 失败 / pnpm build 通过
  ```

  `type` 取 `feat` / `fix` / `docs` / `chore` / `refactor` / `test`
- 提交前先跑一遍「真实数据安全回归」（清单见 `.workbuddy/memory/MEMORY.md`），**确认用户真实目录零改动再提交**
- `.gitignore` / `.gitattributes` 已就绪；新增构建产物目录时同步补规则，别让 9 GB 的 `target/` 溜进去

---

## ℹ️ 项目一句话说明

CrossBrain 是一个 Windows 桌面应用（Tauri v2 + Vue 3 + Rust），把用户的 AI 编程规则从一个地方同步到 Claude Code、Antigravity IDE 等多个 AI 工具。全程本地文件操作，零网络依赖。

**当前状态**：见 [`CURRENT_STATUS.md`](./CURRENT_STATUS.md)。  
> 此处**不要**再复制一份进度描述——两处必然不一致。进度只维护在 `CURRENT_STATUS.md` 一处。

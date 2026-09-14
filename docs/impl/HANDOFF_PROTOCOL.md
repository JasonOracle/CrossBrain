# CrossBrain — 交接协议（HANDOFF PROTOCOL）

> **本文件解决一个核心问题**：  
> 当人工（或某个 AI 工具）修改了代码后，**哪些文档必须同步更新**？  
> 不同步 → 下一个 AI 工具读到错误的文档 → 按错误文档实现 → 出 bug → 反复返工

---

## Part 1：人工修改代码后的文档同步规则

### 规则 1：修改了 Adapter 的任何逻辑

| 修改内容 | 必须同步更新的文档 |
|:---|:---|
| 修改了 `sync_l0()` 或 `sync_l0_with_result()` 的行为 | `docs/tech/ADAPTER_SPEC.md` 对应 Adapter 章节 |
| 修改了标记块格式（`<!-- CrossBrain:Start -->` 等） | `docs/tech/ADAPTER_SPEC.md` §3.2 + `docs/impl/TASK_BREAKDOWN.md` TASK-06 |
| 修改了四情况协议的任何一种情况的行为 | `docs/tech/ADAPTER_SPEC.md` §3.2 + `docs/impl/TASK_BREAKDOWN.md` TASK-06 + `docs/testing/TEST_PLAN.md` T3 |
| 修改了孤儿清理的过滤逻辑（crossbrain- 前缀判断） | `docs/tech/ADAPTER_SPEC.md` + `docs/testing/TEST_PLAN.md` T4 |
| 修改了 Plan B 降级触发条件或行为 | `docs/tech/ADAPTER_SPEC.md` Plan B 章节 |
| 新增了第三个 Adapter | `docs/tech/ADAPTER_SPEC.md`（新增章节）+ `docs/product/PRD.md` 工具矩阵 + `docs/testing/TEST_PLAN.md`（新增测试模块）+ `CURRENT_STATUS.md` |

### 规则 2：修改了路径相关代码（`paths.rs`）

| 修改内容 | 必须同步更新的文档 |
|:---|:---|
| 修改了任何路径函数的返回值 | `docs/tech/ARCHITECTURE.md` 对应路径说明 + `docs/tech/ADAPTER_SPEC.md` 路径说明 |
| 新增了路径函数 | `docs/tech/ARCHITECTURE.md`（说明用途）|

### 规则 3：修改了 Slug-Hash 算法（`slug.rs`）

| 修改内容 | 必须同步更新的文档 |
|:---|:---|
| 修改了 slug 截断长度（现为 30） | `docs/tech/ARCHITECTURE.md` §4 |
| 修改了 hash 长度（现为前 6 位） | `docs/tech/ARCHITECTURE.md` §4 |
| 修改了 slug 字符过滤逻辑 | `docs/tech/ARCHITECTURE.md` §4 |

### 规则 4：修改了 Git 提交逻辑（`git.rs`）

| 修改内容 | 必须同步更新的文档 |
|:---|:---|
| 修改了时间戳格式 | `docs/tech/ARCHITECTURE.md` §6 + `docs/testing/TEST_PLAN.md` T8-3 |
| 修改了 `nothing to commit` 的处理方式 | `docs/testing/TEST_PLAN.md` T8-2 |

### 规则 5：修改了 UI 任何可见文字

| 修改内容 | 必须同步更新的文档 |
|:---|:---|
| 修改了错误提示文字 | `docs/product/PRD.md` §8 错误翻译表 |
| 修改了 5 步向导的文案 | `docs/product/PRD.md` §5 |
| 修改了同步状态栏格式 | `docs/product/PRD.md` §7 |
| 修改了技能知识超长警告文字 | `docs/product/PRD.md` §6.2 |

### 规则 6：修改了任何技术选型或依赖

| 修改内容 | 必须同步更新的文档 |
|:---|:---|
| 升级了 Tauri / Vue / Tailwind 主版本 | `docs/tech/TECH_STACK.md` |
| 升级了 Rust 依赖（dirs / chrono / sha2 等） | `docs/tech/TECH_STACK.md` + `docs/impl/DEV_SETUP.md` |
| 改变了包管理器配置 | `docs/impl/DEV_SETUP.md` |

---

## Part 2：任务完成后的必做检查

每完成一个 TASK，在更新 `CURRENT_STATUS.md` 之前，先对照以下清单：

```
完成 TASK-XX 后，检查是否：

[ ] CURRENT_STATUS.md 中该任务已改为 ✅
[ ] CURRENT_STATUS.md 的「本次变更记录」已追加本次操作
[ ] 本次实现中是否有任何与 TASK_BREAKDOWN.md 描述不一致的地方？
    → 如有，更新 TASK_BREAKDOWN.md，并说明为什么不同
[ ] 本次实现中是否推翻了 DECISION_LOG.md 中的某项决策？
    → 如有，先停下来，在 DECISION_LOG.md 中补充新的 ADR 说明原因
[ ] 本次实现引入了新的路径、文件格式或接口变化？
    → 按 Part 1 的规则检查需要同步哪些文档
```

---

## Part 3：接手新工具时的 Onboarding 检查

当一个新的 AI 工具被安排接手项目时，按以下顺序确认：

```
Step 1. 读 MASTER_CONTEXT.md（完整读完，不跳过）
Step 2. 读 CURRENT_STATUS.md（确认当前任务）
Step 3. 读 docs/impl/TASK_BREAKDOWN.md（找到你的任务）
Step 4. 读 docs/impl/DECISION_LOG.md（了解不能推翻的决策）
Step 5. 按需读参考文档（见 MASTER_CONTEXT.md 第 4 节）

❌ 不允许的行为：
- 不读文档直接开始写代码
- 认为某个设计"有问题"就直接改掉（先查 DECISION_LOG.md）
- 完成后不更新 CURRENT_STATUS.md
- 修改了代码但不同步相关文档
```

---

## Part 4：发现文档与代码不一致时的处理

**当实际代码与文档描述不符时**，按以下流程处理：

```
情况 A：代码是对的，文档落后了
  → 更新文档，在 CURRENT_STATUS.md 变更记录中注明"文档已对齐实际代码"

情况 B：文档是对的，代码实现有误
  → 修复代码，使其符合文档，在变更记录中注明"修复了实现与文档的偏差"

情况 C：不确定哪个是对的
  → 查 DECISION_LOG.md
  → 如果 DECISION_LOG.md 有相关 ADR，按 ADR 的决策执行
  → 如果 DECISION_LOG.md 没有，停下来等人工确认，在 CURRENT_STATUS.md 中记录这个阻塞项
```

---

## Part 5：文档版本说明

所有文档均基于 `plan-v2.5.md`（实际标题 V3.0 实测定稿版）生成。  
`plan-v2.5.md` 是只读参考原始蓝图，不应修改。  
如果 plan-v2.5.md 内容与其他文档有冲突，**以其他文档为准**（因为其他文档是从 plan 中提取并细化的，更精确）。

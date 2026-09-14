# CrossBrain 测试计划（V1）

> 版本：V1.0（依据 plan-v2.5.md V3.0 实测定稿版）  
> 执行时间：Day 9 - Day 10  
> 更新时间：2026-09-14

---

## 测试原则

1. 每个用例必须有明确的**通过标准**（Pass Criteria），可被客观验证
2. 所有测试在 Windows 环境下执行（V1 仅支持 Windows）
3. Adapter 相关测试必须在真实工具环境下执行（非 Mock），复现线上状态

---

## 测试模块总览

| 模块 | 用例数 | 执行阶段 |
|:---|:---|:---|
| T1. Slug-Hash 算法 | 4（实现拆为 18 个测试） | Day 3（单元测试） |
| T2. Antigravity Adapter | 6 | Day 5 |
| T3. Claude Code Adapter | 8 | Day 5 |
| T4. 孤儿清理机制 | 3（+7 一致性用例） | Day 5 |
| T5. 幂等性验证 | 2 | Day 5 |
| T6. 首次运行向导 | 5 | Day 8 |
| T7. 离线模式 | 3 | Day 9 |
| T8. Git 健壮性 | 3 | Day 9 |
| T9. 端到端流程 | 2 | Day 9 - Day 10 |
| T10. 一键卸载 / 去痕 | 2 | Day 10 |

---

## T1. Slug-Hash 算法（Rust 单元测试）

> **状态：全部通过（2026-09-14，TASK-07）**，实现拆为 18 个测试。
> 完整清单与边界用例见 `TASK_BREAKDOWN.md` 的 TASK-07「实施记录」。

### T1-1 基础生成

- **输入**：`"Vue3 组件性能优化.md"`，固定测试内容（SHA256 前 6 位已知）
- **期望输出**：`"crossbrain-vue3-{hash}"` 格式正确
- **通过标准**：输出字符串以 `crossbrain-` 开头，包含 `-`，总长 ≤ 50 字符
- **实测**：`generate("Vue3 组件性能优化.md", "test")` → `"crossbrain-vue3-9f86d0"`
  （`9f86d0` 与系统 `sha256sum` 输出逐位一致，见 `examples/dryrun_slug.rs`）
- ⚠️ **原期望输出 `crossbrain-vue3-perf-{hash}` 已修正**：V1 不做拼音转换，
  `perf` 是 ARCHITECTURE 第 4 节示例里的人工伤示例。详见该节「实现说明」。

### T1-2 幂等性

- **操作**：对同一文件路径+内容执行算法 3 次
- **通过标准**：3 次输出完全相同

### T1-3 内容变更触发 Hash 变更

- **操作**：同一文件名，内容增加一行后重新计算
- **通过标准**：新 slug-hash 的后 6 位与原 hash 不同

### T1-4 中文/特殊字符安全

- **输入**：`"Rust · 生命周期与借用检查器.md"`、`"C++ #include 踩坑记录.md"`
- **通过标准**：输出不含中文、空格、特殊字符，格式合法（只含 `a-z0-9-`）

### T1-5 补充边界（文档未覆盖，实施时补入）

| 用例 | 通过标准 |
|:---|:---|
| 纯中文文件名（`组件性能优化.md`、`！！！.md`、空串） | 回退为 `item`，不 panic，格式仍合法 |
| 2 字符中文名（命中 `len - 3` 的字节切片陷阱） | 不 panic |
| `.MD` / `.Md` | 大小写不敏感剥离，kebab 段不含 `md` |
| 300 字符超长文件名 | kebab 段截断至 30，末尾无残留 `-`，总长 ≤ 50 |
| `../../etc/passwd.md` | 斜杠与 `..` 被折叠，产出仍通过命名空间校验 |
| `CON.md` / `NUL.md` / `COM1.md` | Windows 保留名由 `crossbrain-` 前缀规避 |

> **T1 之外的一致性验证**：`tests/adapter_consistency.rs`（7 个用例）覆盖 T2/T3 的
> 清理行为一致性，见 T4。

---

## T2. Antigravity Adapter

### T2-1 安装检测（已安装）

- **前置条件**：`%USERPROFILE%\.gemini\config\` 目录存在
- **通过标准**：`detect()` 返回 `Ok(true)`

### T2-2 安装检测（未安装）

- **前置条件**：重命名 `.gemini` 目录（模拟未安装）
- **通过标准**：`detect()` 返回 `Ok(false)`，同步流程跳过此 Adapter，不报错

### T2-3 L0 全局规则写入

- **操作**：调用 `sync_l0()`，内容为「测试规则-Antigravity-T2-3」
- **通过标准**：
  - `%USERPROFILE%\.gemini\config\rules\crossbrain-L0.md` 存在
  - 文件内容与传入内容完全一致

### T2-4 L0 多次写入幂等

- **操作**：连续调用 `sync_l0()` 两次，第二次内容相同
- **通过标准**：`rules/crossbrain-L0.md` 内容与第二次写入一致，目录下无重复文件

### T2-5 L2 技能知识写入

- **操作**：调用 `sync_l2("crossbrain-vue3-perf-7c8e2a", content)`
- **通过标准**：
  - `%USERPROFILE%\.gemini\config\skills\crossbrain-vue3-perf-7c8e2a\SKILL.md` 存在
  - 文件包含合法的 YAML frontmatter（`name` 和 `description` 字段不为空）
  - 文件内容的正文部分与传入 content 一致

### T2-6 L2 懒加载真实触发验证

- **操作**：写入测试 SKILL.md 后，在 Antigravity IDE 新建对话，询问该技能相关问题
- **通过标准**：IDE 日志或回复中出现 `Analyzed SKILL.md` 字样（复现 Spike C 验证方法）

---

## T3. Claude Code Adapter

### T3-1 安装检测

- **前置条件**：`%USERPROFILE%\.claude\` 目录存在
- **通过标准**：`detect()` 返回 `Ok(true)`

### T3-2 情况一：CLAUDE.md 不存在

- **前置条件**：删除（或重命名）`~/.claude/CLAUDE.md`
- **操作**：调用 `sync_l0()`
- **通过标准**：
  - 创建了新的 `CLAUDE.md`
  - 文件包含 `<!-- CrossBrain:Start -->` 和 `<!-- CrossBrain:End -->` 标记块
  - 无任何 UI 弹窗

### T3-3 情况二：CLAUDE.md 存在且已含标记块

- **前置条件**：`CLAUDE.md` 中已含 CrossBrain 标记块（旧内容）
- **操作**：用新内容调用 `sync_l0()`
- **通过标准**：
  - 标记块内容被精准替换为新内容
  - 标记块外的用户内容原封不动
  - 无任何 UI 弹窗

### T3-4 情况三：CLAUDE.md 存在，无标记块，无备份文件

- **前置条件**：`CLAUDE.md` 含用户自定义内容，无标记块，无 `.crossbrain-backup`
- **操作**：调用 `sync_l0()`
- **通过标准**：
  - 创建了 `CLAUDE.md.crossbrain-backup`，内容与原始 `CLAUDE.md` 完全一致
  - `CLAUDE.md` 末尾追加了 CrossBrain 标记块
  - `CLAUDE.md` 中用户原有内容未被删除
  - UI 弹出备份提示（含备份文件路径）

### T3-5 情况四：CLAUDE.md 存在，无标记块，备份文件已存在

- **前置条件**：`CLAUDE.md.crossbrain-backup` 已存在
- **操作**：调用 `sync_l0()`
- **通过标准**：
  - 旧备份文件未被覆盖（内容不变）
  - 标记块追加到 `CLAUDE.md` 末尾
  - 无 UI 弹窗（静默完成）

### T3-6 L2 技能知识写入

- **操作**：调用 `sync_l2("crossbrain-test-abc123", content)`
- **通过标准**：`%USERPROFILE%\.claude\skills\crossbrain-test-abc123\SKILL.md` 存在，格式合法

### T3-7 L2 懒加载真实触发验证

- **操作**：写入测试 SKILL.md 后，在 Claude Code 新建对话，询问该技能相关问题
- **通过标准**：Claude Code 回复内容体现了 SKILL.md 中的知识（复现 Spike B 验证方法）

### T3-8 @import 路径展开验证

- **操作**：在 `CLAUDE.md` 中写入 `@~/.ai-profile/AGENTS.md`，新建 Claude 对话
- **通过标准**：Claude 读取到 `AGENTS.md` 内容（复现 Spike A 验证方法）

---

## T4. 孤儿清理机制

### T4-1 正常孤儿清理

- **前置条件**：在 `~/.gemini/config/skills/` 下预置 3 个 `crossbrain-*` 目录（其中 1 个在当前 knowledge 列表中，2 个不在）
- **操作**：执行一次同步
- **通过标准**：不在列表中的 2 个目录被删除，在列表中的 1 个目录保留

### T4-2 空 knowledge 列表

- **前置条件**：`knowledge/` 目录为空，skills 目录下有 3 个 `crossbrain-*` 目录
- **操作**：执行同步
- **通过标准**：3 个 `crossbrain-*` 目录全部被删除，不报错

### T4-3 非 CrossBrain 目录不被误删

- **前置条件**：`~/.gemini/config/skills/` 下有 1 个用户自建的目录（非 `crossbrain-` 前缀）
- **操作**：执行同步
- **通过标准**：用户自建目录完整保留，未被误删

> **状态：全部通过（2026-09-14，TASK-08）**。三个用例均已实现为
> `tests/adapter_consistency.rs` 中「**同一组输入同时驱动两个 Adapter**」的集成测试——
> 原计划的「两个 Adapter 分别测试」只能证明各自符合自己的预期，
> 无法证明**行为一致**：
>
> | 用例 | 对应测试 |
> |:---|:---|
> | T4-1 正常孤儿清理 | `t8_both_delete_orphans_and_keep_active` |
> | T4-2 空 knowledge 列表 | `t8_both_never_touch_non_crossbrain_dirs`（active 传空 = 最激进场景） |
> | T4-3 非 CrossBrain 目录不被误删 | `t8_both_never_touch_non_crossbrain_dirs` |
>
> 另补 4 个边界用例：skills 目录缺失返回 `Ok`（且不顺手创建该目录）、
> 同名前缀普通文件跳过、全 active 时一个都不删、报告按名确定性排序。
> 实现细节见 `TASK_BREAKDOWN.md` 的 TASK-08「实施记录」。

---

## T5. 幂等性验证

### T5-1 连续两次同步内容一致

- **操作**：不修改任何内容，连续执行同步两次
- **通过标准**：
  - 第二次同步无报错
  - 所有目标文件的内容与第一次同步后完全一致
  - git 第二次提交触发 `nothing to commit`，静默处理，UI 不报错

### T5-2 同步后内容比对

- **操作**：同步完成后，对比 SSOT 内容与 Antigravity、Claude 对应文件内容
- **通过标准**：内容完全一致（忽略 YAML frontmatter 部分）

---

## T6. 首次运行向导

### T6-1 步骤 2 工具检测准确

- **通过标准**：已安装工具显示 ✅，未安装工具显示 ⏳，无错误弹窗

### T6-2 规则模板填充

- **操作**：点击「前端工程师」模板
- **通过标准**：编辑器内容自动填充，可继续编辑

### T6-3 步骤 4 同步进度实时显示

- **通过标准**：进度逐行出现（非一次性全部显示），完成后显示摘要

### T6-4 步骤 5 验证提示词可复制

- **通过标准**：点击复制按钮后，剪贴板内容与展示提示词一致

### T6-5 跳过向导直接进入主界面

- **操作**：点击「跳过，直接进入」
- **通过标准**：直接进入主界面，无崩溃，下次启动不再触发向导

---

## T7. 离线模式

### T7-1 断网后本地同步正常

- **前置条件**：断开网络连接（或禁用网卡）
- **操作**：执行同步
- **通过标准**：同步正常完成，文件写入成功，UI 显示「当前无网络，规则同步正常」

### T7-2 离线状态指示

- **通过标准**：主界面状态栏显示 🔴 离线模式，同步按钮可正常点击

### T7-3 断网后 git 提交处理

- **通过标准**：本地 git commit 成功（git 不需要网络），UI 无网络相关报错

---

## T8. Git 健壮性

### T8-1 git 未安装时降级

- **前置条件**：从 PATH 中临时移除 git
- **操作**：执行同步
- **通过标准**：同步正常完成，文件写入成功，UI 显示「未检测到 Git，版本历史功能暂不可用」

### T8-2 `nothing to commit` 静默处理

- **操作**：连续同步两次（第二次无内容变更）
- **通过标准**：第二次同步无报错弹窗，UI 显示同步成功

### T8-3 时间戳格式正确

- **操作**：执行同步，查看 git log
- **通过标准**：commit message 格式为 `sync: 2026-09-14T19:00:00+0800`（ISO-8601 格式）

---

## T9. 端到端流程

### T9-1 完整流程：新建技能 → 同步 → AI 工具实际触发

- **操作**：在 CrossBrain UI 新建技能知识「# Vue3 · 虚拟列表」→ 点击同步 → 在 Antigravity IDE 新对话询问相关问题
- **通过标准**：IDE 加载了该 SKILL.md 并在回答中体现知识内容

### T9-2 完整流程：删除技能 → 同步 → 孤儿清理确认

- **操作**：删除上一条技能 → 点击同步
- **通过标准**：对应的 `crossbrain-*` 目录从两个 Adapter 目录中均被删除

---

## T10. 一键卸载 / 去痕

### T10-1 卸载后所有注入内容清除

- **操作**：在设置页点击「一键卸载 / 去痕」
- **通过标准**：
  - `~/.gemini/config/rules/crossbrain-L0.md` 文件已删除
  - `~/.gemini/config/skills/` 下所有 `crossbrain-*` 目录已删除
  - `~/.claude/CLAUDE.md` 中的 CrossBrain 标记块已移除，用户原有内容保留
  - `~/.claude/skills/` 下所有 `crossbrain-*` 目录已删除

### T10-2 卸载不影响用户既有 CLAUDE.md 内容

- **前置条件**：`CLAUDE.md` 中含用户自定义内容（在标记块之外）
- **通过标准**：卸载后标记块被移除，但标记块外的用户内容完整保留

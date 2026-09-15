---
name: crossbrain-add-adapter
description: 为 CrossBrain（D:\project\memory1.0）接入一个新的 AI 工具 Adapter，把支持的工具数从 N 提到 N+1。当用户说「接入 X 工具 / 让向导识别 X / 加一个 X adapter / TASK-NN 是接 X」时使用。
description_zh: "CrossBrain 新增 AI 工具 Adapter 的完整流程：从落点探测到文档同步与安全回归"
version: 1.4.0
agent_created: true
allowed-tools: Read,Edit,Write,Grep,Glob,Bash,PowerShell
display_name: "CrossBrain 接入新工具 Adapter"
---

# CrossBrain 新增 AI 工具 Adapter

把一款新 AI 工具接进 CrossBrain 的统一分发引擎。**这不是「照着 codex.rs 复制一份」的活**——
每个工具的 L0 落点形态都可能不同，照搬会静默失效（编译通过、测试通过、用户规则就是没生效）。

项目根：`D:\project\memory1.0`，Rust 侧在 `src-tauri/`。

---

## 0. 先决条件：落点探测（**不可跳过**）

**没有落点就不能接入。** 必须先确认目标工具「有没有一个全局的规则文件/目录可写」，
以及「它读不读这个文件」。只答第一个问题的叫**文件系统预探测**，够用来写代码；
第二个问题必须靠 Spike 实测（见第 6 节）。

探测要回答：

| 问题 | 怎么查 |
|---|---|
| 配置目录在哪 | 列 home 下的 `.<tool>/`、`AppData/Roaming/<Tool>/` |
| L0 落点是文件还是目录 | `ls <dir>`，看有没有 `AGENTS.md` / `CLAUDE.md` / `rules/` / `*.mdc` |
| 该落点是否在**硬链接组**内 | `fsutil hardlink list <path>`——若在，说明是用户「全局记忆中心」，改动影响面极大 |
| 有没有内置内容不能被删 | 典型：`skills/.system/`（Codex 内置）、命令白名单 `rules/default.rules` |
| **落点有没有「遮蔽文件」** | 查官方文档的**规则发现优先级**（如 Codex 的 `AGENTS.override.md` 高于 `AGENTS.md`）。见第 1 节 |
| 该工具是「App + CLI 两款」还是**一款** | 很多工具桌面版与 CLI **共用同一个 home**（Codex 即如此）——共用就只做一个 Adapter、一个显示名 |
| **技能根（L2）有几个** | 有些工具会扫描**多个**技能根。Codex 实测有 4 类：自身 `skills/.system/`（内置）、自身 `skills/{name}/`（**我们的落点**）、`plugins/cache/`（插件）、以及第三方跨宿主 `~/.agents/skills/`（86 个技能 / 59 MB）。**只能往自己那一个根写**，其余一律不碰 |
| 用户有没有已存在的自建内容 | 有 → 走标记块注入（保留原内容），无 → 仍走标记块注入（统一行为） |

### 🔍 官方文档说不清时：做「二进制取证」

桌面应用的安装包里就带着它的实现。**官方文档含糊时，直接从二进制里读真值**，
比猜和试都可靠（TASK-18 的 `AGENTS.override.md` 优先级与技能根清单都是这样确认的）：

```bash
# 1) 数命中，确认这个二进制可以搜
grep -a -c 'skills' <app>/resources/<cli>.exe
# 2) 抓关键字的上下文（前后各若干字符），常能直接读到 Rust 模块路径与文档字符串
grep -a -o -E '.{0,140}AGENTS\.md.{0,140}' <cli>.exe | sort -u
grep -a -o -E '.{0,120}\.agents/skills.{0,120}' <cli>.exe | sort -u
```

要点：

- 用 `grep -a`（把二进制当文本）；`-o -E` 配 `.{0,N}` 抽上下文；`sort -u` 去重
- 命中里出现的 **Rust 模块路径**（如 `codex-home/src/instructions/mod.rs`）极有价值 ——
  能定位到具体的加载逻辑文件名
- 内置的 **Python/JS 脚本字符串**会暴露安装路径与目录常量
- **Electron 的 `app.asar` 也可直接 `grep -a`**（含 JS 源码），别以为它是压缩包

**结论只有两种**：① 有落点 → 继续；② 无落点（如 Qoder 只有空 `memory/`）→ **明确告诉用户接不了**，
不要硬造一个文件出来。

探测结论落 `docs/testing/SPIKE_RESULTS.md`。

---

## 1. 判断该工具的 L0 形态（**最关键的一步**）

三种形态，**互相不可照搬**：

| 形态 | 代表工具 | 写法 | 致命照搬点 |
|---|---|---|---|
| **目录型** | Antigravity IDE | 在 `<base>/rules/` 下**建独立文件** `crossbrain-L0.md` | 别去动工具自己的规则目录 |
| **单文件 + `@` 引用** | Claude Code | 在 `CLAUDE.md` 注入标记块，块内用 `@~/.ai-profile/...` **引用** | 工具不认识 `@` → 全文可见性丢失 |
| **单文件 + 内联全文** | Codex | 在 `AGENTS.md` 注入标记块，块内放**规则全文** | 内联型必须调 `ensure_no_marker_in_content()` 防标记串提前闭合 |

判据：**这个工具认识 `@` / `~` 引用语法吗？** 认识 → 形态二；不认识 → 形态三；
规则目录是「一目录多文件」→ 形态一。

### ⚠️ 还要问一个「遮蔽文件」问题（Codex 的教训，别漏）

**落点正确 ≠ 会被读到。** 有些工具存在**优先级高于我们落点**的文件，
或「同级只取第一个非空文件」的规则，会让我们的写入静默失效。

实例：Codex 读全局规则时**先看 `AGENTS.override.md`，没有才回落 `AGENTS.md`**，
且每层只取一个非空文件 → 用户一旦建了 `~/.codex/AGENTS.override.md`，
我们写进 `AGENTS.md` 的标记块**永远没人读**。

做法：同步前**显式检测**这类遮蔽文件，命中即**中止并报明确错误**
（不要静默写入，也不要自作主张改写进遮蔽文件——会引入「用户删掉它后留下陈旧块」的善后问题）。
本项目的实现见 `CodexAdapter::reject_if_override_present()`。

**每接入一个新工具，都去查一遍它的规则发现优先级，别假设「写进去就会被读」。**

---

## 2. 写 Adapter（`src/adapters/<tool>.rs`）

必须实现的 trait（定义在 `src/adapters/mod.rs`）：

```rust
fn detect(&self) -> Result<bool, AdapterError>;                       // 只看目录存在性
fn sync_l0(&self, rules_content: &str) -> Result<(), AdapterError>;   // 必须幂等
fn sync_l2(&self, slug_hash: &str, content: &str) -> Result<(), AdapterError>; // content 已含 frontmatter
fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError>;
```

- `detect()` 返回 `Ok(false)` 是**正常**情况，调用方静默跳过，不要当错误。
- `sync_l2` 只负责落盘，**不拼 frontmatter**——拼装用共享层 `format_skill_md()`。
- `cleanup_orphans` **只能删 `crossbrain-` 前缀**的目录（ADR-09）。直接调共享的
  `cleanup_crossbrain_orphans(...)`，别自己写遍历。

### 共享层现成的东西（`src/adapters/mod.rs`）——**不要再写一遍**

| 符号 | 用途 |
|---|---|
| `MARKER_START` / `MARKER_END` / `MARKER_NOTE` | 标记块常量（**由它们拼 regex**，不要硬编标志串） |
| `marker_regex()` | 由常量拼接 + `regex::escape`，防硬编码分叉 |
| `has_marker_block(content)` | 检测标记块存在 |
| `ensure_no_marker_in_content(content)` | **内联型必调**：防规则内容里混入 `MARKER_START` |
| `write_breaking_hardlink(path, content)` | 断链三步法写文件（铁律 L-01 + 决策 D-03） |
| `inject_marker_block(target, backup, block)` | 标记块四情况协议的完整实现 |
| `restore_l0_backup(target, backup)` | 还原到首次注入前的内容（`ADR-15`）；自带「现场另存」与断链写入 |
| `remove_marker_block_from_file(target, backup)` | 卸载用：移除标记块、块外内容原样保留（TASK-14）；含「整文件皆我方创建→删文件」特例 |
| `delete_backup_files(backup, pre_restore)` | 卸载用：删两个备份文件；**只能在标记块成功移除之后调用** |
| `skill_cleanup_actions(report)` | 把 `cleanup_orphans(&[])` 的报告转成卸载动作描述 |
| `pre_restore_path(target)` | 算 `*.crossbrain-before-restore` 路径 |
| `SyncL0Result` | `Silent` / `BackupCreated { backup_path }` |
| `L0Backup` | `{ target_file, backup_file }`，由 `Adapter::l0_backup()` 返回 |
| `ensure_crossbrain_slug(slug_hash)` | 校验 slug 在命名空间内且是安全单层目录名 |
| `format_skill_md(prefix, body)` | 拼 SKILL.md（含 frontmatter） |

写 `sync_l0_with_result()` 时，**只做两件事**：拼出本工具特有的 `marker_block`（形态二/三），
然后 `inject_marker_block(&self.<l0_file>(), &self.backup_file(), &block)`。
断链、四情况、备份逻辑**全在共享层**，Adapter 里不许再出现。

**额外要覆盖 `l0_backup()`——仅形态二 / 形态三（标记块注入型）需要**：

```rust
fn l0_backup(&self) -> Option<L0Backup> {
    Some(L0Backup { target_file: self.<l0_file>(), backup_file: self.backup_file() })
}
```

返回**一对**路径而不是单个备份路径：还原同时需要落点与快照，只给备份路径就得到
「把 `.crossbrain-backup` 去掉反推文件名」——又是一份会分叉的命名规则副本。
形态一（目录型，写自己的独立文件）**不要覆盖**，默认的 `None` 会让它自动从
「备份与还原」列表里消失，用户不会看到一条永远「无备份」的空条目。

**额外要覆盖 `uninstall()`——所有形态都要**（TASK-14「一键卸载 / 去痕」，默认实现是空动作）：

```rust
fn uninstall(&self) -> Result<Vec<String>, AdapterError> {
    let mut actions = Vec::new();
    // 形态二/三（标记块注入型）：① 先移标记块，② 后删备份，③ 清技能目录
    if let Some(desc) = remove_marker_block_from_file(&self.<l0_file>(), &self.backup_file())? {
        actions.push(desc);
    }
    actions.extend(delete_backup_files(&self.backup_file(), &pre_restore_path(&self.<l0_file>())));
    actions.extend(skill_cleanup_actions(self.cleanup_orphans(&[])?));
    // 形态一（目录型）：① 删自己的独立规则文件（NotFound 静默跳过），② 清技能目录
    Ok(actions)
}
```

⚠️ **顺序是安全属性**：删备份必须在标记块成功移除**之后**（备份是用户唯一的
还原手段）。返回值是**面向用户的动作描述**（含「你自己的内容原样保留」），
不是调试日志。

⚠️ 本机**存在真实用户数据**：新工具接进来后必须跑安全回归 ——
硬链接组 md5 一致、用户技能完好、真实目录不出现 `*.crossbrain-backup` /
`*.crossbrain-before-restore` / `*.crossbrain-tmp`、`~/.ai-profile` 零改动。
**基线以 `.workbuddy/memory/MEMORY.md` 的「真实数据基线」为准**（真机同步过
一次后 links=5 已变 4+1，别按旧数字判失败）。

---

## 3. 注册点（**四处，漏一处就白干**）

1. **`src/adapters/mod.rs`** —— `pub mod <tool>;`
2. **`src/paths.rs`** —— 新增 `pub fn <tool>_dir() -> PathBuf { home_dir().join(".<tool>") }`，
   并在注释里写清该目录内部结构（哪些**绝不能碰**）
3. **`src/sync.rs`** —— `build_tools()` 追加一个 `ToolDescriptor`（`tool_id` / `display_name`
   / `push_path` / `adapter: Box::new(<Tool>Adapter::new())`）；`use` 补 import
4. **`src-tauri/tests/adapter_consistency.rs`** —— import 补新 Adapter，在三向一致性断言里
   加入新的 base 目录，`t8_trait_objects_share_identical_behaviour` 的对比列表扩展

⚠️ **入口是 `src-tauri/src/lib.rs`，不是 `main.rs`**。新 `mod` 必须在 `lib.rs` 挂载，
否则整个目录**不参与编译**而 `cargo build` 照样成功（验收全空转）。

### ⛔ 写入边界（每次新增 Adapter 都必须守住）

已核查并可作为对用户的承诺：CrossBrain **只写三类目标** ——
① L0 规则文件 ② L2 技能目录（仅 `crossbrain-*` 前缀）③ `*.crossbrain-backup`。

**绝不打开写入**（代码里必须保持零引用）：各工具的 `settings.json` / `config.toml` /
`mcp_config.json`、命令白名单（如 Codex `rules/default.rules`）、内置技能目录
（如 `skills/.system/`）、第三方跨宿主库（`~/.agents/`），以及 backups / cache /
history / sessions 等工具内部数据。**用户偏好配置里任何一项都不是我们的目标。**

自检一句话：**新 Adapter 会写到的每一个路径，是否都能落进上面三类里？**
落不进 → 停下，先问用户，别自作主张扩大写入范围。
（配套决策见 `DECISION_LOG.md` **ADR-15**：改动必须「知情且可逆」。）

---

## 4. 前端（如果引入了新的展示项）

- `src/api.ts` 的 `UPCOMING_TOOLS`：**接入后必须把该工具从「即将支持」里移除**，
  否则向导会同时显示「X 已安装」和「X 即将支持」——同一件事两种说法。
- 界面文案**不得出现内部术语**（L0 / L2 / SSOT / slug / Adapter），由
  `tests/ipc_contract::wizard_copy_has_no_internal_terms` 锁定。该测试用 `template_of()`
  扫一个**文件列表** —— **新增任何面向用户的界面文件（视图 / 弹窗组件）后，
  必须把它加进那个列表**，否则它会成为绕过术语约束的口子。
- 若新工具的 L0 是「标记块注入型」，它会自动出现在「设置 → 备份与还原」里
  （靠 `l0_backup()`，见第 3 节），前端**无需改动**。
- 图标一律 `~icons/lucide/*` 的 SVG（**项目禁止 Emoji**）。图标名以实际图标集为准：
  `node_modules/@iconify-json/lucide/icons.json`。是 `triangle-alert` 而非 `alert-triangle`。
- Tailwind v4 的语义色**只有 `-500` 一档**（`success-500` / `warning-500` / `error-500`），
  要浅色底就配透明度修饰符（`bg-warning-500/10`），别写不存在的档位。

---

## 5. 测试（写新 Adapter 的测试 + 扩展交叉测试）

- `detect()`：目录**不存在**时 `detect()` 为 `false`（必须在真实临时目录里造）
- 标记块**四情况**各一条：无块 / 有块 / 有块但夹在中间 / 有块 + 内容变更
- **防注入**：规则内容里混入 `MARKER_START` 时必须报错（仅形态三）
- **硬链接隔离**：调用 `sync_l0` 后，原 inode 的其它链接**不受影响**
- `sync_l2` 落盘内容与 `format_skill_md` 输出一致
- `cleanup_orphans`：删孤儿、留 active、**不碰用户自建**
- trait 契约一致性（`tests/adapter_consistency.rs`）：新老 Adapter 用同一组用例，
  断言输出**相等**——不是分别验证「A 对」「B 对」

**所有测试跑临时目录**（`with_base_dir(temp)` 注入），
临时目录名 = 进程号 + `static AtomicU32` 计数器，`Drop` 里 `let _ = fs::remove_dir_all(...)`。
**绝不写真实路径**（本机有真实用户数据）。

---

## 6. 干跑 + 真实 Spike

- 扩展 `src-tauri/examples/dryrun_sync.rs`：新增该工具的副本目录、技能目标、
  工具清单条目、落盘效果断言（形态特有的检查点，如「内联全文」「未混入 `@`」）。
- **关键**：干跑全跑在**副本**上，证明不了「向导到底显示几个工具」。
  需要一节**读真实路径**（只 `is_dir()`，不写入）断言检测清单条数与新工具 `installed == true`。
- **真实 Spike（ADAPTER_SPEC §6，接入前的强制步骤）**，三项必答：
  ① 真能读到 L0 落点 ② skills 是否按 `description` 懒加载 ③ **作用域是全局还是项目级**
  （若为项目级，L0 策略要重新设计）。

  #### 🎯 优先找「dump 模型可见输入」的命令 —— 能把 Spike 变成一条命令

  **先翻 `--help`**：`<cli> --help` → `<cli> debug --help` → 找形如
  「render the model-visible prompt input」的子命令。

  Codex 的实例（2026-09-14 实测有效）：

  ```bash
  # 必须在「中立目录」（项目目录之外）执行，否则无法区分全局加载与项目根 AGENTS.md 加载
  mkdir -p /tmp/cbspike && cd /tmp/cbspike
  <app>/resources/codex.exe debug prompt-input > pi.json    # Dump 模型可见输入为 JSON
  grep -c "<探针口令>" pi.json                                # L0 验证
  grep -c "<探针技能名>" pi.json                              # L2 验证
  ```

  同族还有 `codex debug models`（模型目录）。**有这类命令就完全不需要人工开会话**，
  且能直接在 JSON 里看到 `<INSTRUCTIONS>` / `<skills_instructions>` 的真实内容。

  #### 没有这类命令时，才退回人工探针

  往真实 L0 落点写探针串 → 开一次真实工具会话 → 问它是否读到 → **清空**。
  探针前先确认该文件是空的/可恢复的，把风险压到零；探针必须**可整份删除**，
  且在文件里写明「这是临时探针、验证后删除」，避免后续会话误判。

  #### 两个易踩的坑

  - **测试位置**：验证 L0 时若在项目目录内跑，项目根的 `AGENTS.md` 也会被加载，
    结果无法归因。**换到中立目录（如 `%TEMP%`）再跑**。
  - **探针必须清理**：删掉探针技能目录、把探针文件恢复原状，
    再跑一遍第 7 节的安全回归，把「清理记录」写进文档。
  - **「磁盘上有、清单里没有」未必是落点问题**：技能若带
    `disable-model-invocation: true`（或宿主侧 `policy.allow_implicit_invocation: false`），
    会**从清单中消失**。别据此误判落点错误。本项目 `format_skill_md()` 不生成该标记。

---

## 7. 真实数据安全回归（**每轮改动后必做**）

```bash
# 5 个硬链接目标 md5 应全等 47645f603b7bfd2ce5eeba650ed09981
for f in ~/.ai-memory/user_profile.md ~/.cursor/rules/user_profile.md \
         ~/.config/opencode/AGENTS.md ~/.gemini/config/rules/user_global.md \
         ~/.claude/CLAUDE.md; do md5sum "$f"; done
# 外加：links=5、用户自建技能完好、无 crossbrain-* 残留
```

⚠️ 检查「清理/删除类」断言时：① 状态要在动作**之前**采样；② 比对**前后差异清单**，
不要硬编码真实数据的文件名。

---

## 8. 文档同步（**四份，缺一不可**）

1. `docs/impl/TASK_BREAKDOWN.md` —— 新增 `## TASK-NN` 节 + 补「⚠️ 实施修正」小节
   （任务文档的示例代码**经常是错的**，每个偏差都要写「原文怎么写 → 为什么不对 → 改成什么」）
   + 补「✅ 实施记录」小节（交付文件 / 实测命令与结果 / 遗留 warning / 交给后续任务的约束）
2. `docs/product/PRD.md` —— 工具矩阵：把该工具从 V1.5/V2 提到 V1（或标注命名修正）
3. `docs/testing/SPIKE_RESULTS.md` —— 落点探测结论 + 待完成 Spike 项
4. `CURRENT_STATUS.md` —— **三处同步**：阶段表、任务状态（🔲→✅）、本次变更记录
   （含精确时间戳，用 `date +"%Y-%m-%d %H:%M:%S"` 取，**不要自己算**）

外加 `.workbuddy/memory/YYYY-MM-DD.md` 追加记录、必要时更新 `MEMORY.md`。

---

## 收尾自检清单

- [ ] 落点探测结论写明（有/无落点，在不在硬链接组）
- [ ] L0 形态判定正确，没照搬别的形态
- [ ] 共享层已用上，Adapter 里没有重复的断链/标记块/孤儿逻辑
- [ ] 形态二 / 三 已覆盖 `l0_backup()`（形态一**不覆盖**，用默认 `None`）
- [ ] 已覆盖 `uninstall()`（所有形态都要，顺序：移块 → 删备份 → 清技能目录）
- [ ] 若新增了面向用户的界面文件，已加进 `ipc_contract` 的术语检查文件列表
- [ ] 四处注册点全部落地（`grep` 逐项核验，别等编译）
- [ ] 新测试全绿，`cargo test` 总数增加且 0 failed / 0 warning
- [ ] `dryrun_sync` 0 项失败（含新工具的形态特有断言）
- [ ] 真实数据回归：md5 全等、links=5、用户技能完好、无残留
- [ ] 四份文档 + 记忆同步
- [ ] 真实 Spike 已做（优先用工具自带的 `debug` dump 命令做无头验证；
      在**中立目录**执行；探针已清理并复跑安全回归），或**明确列为遗留**并告知用户
- [ ] 新的工具特有机理（遮蔽文件、多技能根、体积大的既有资产）已写入
      `SPIKE_RESULTS.md` 与本技能，供下次接入复用

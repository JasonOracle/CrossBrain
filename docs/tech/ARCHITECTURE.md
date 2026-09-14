# CrossBrain 系统架构文档

> 版本：V3.0（依据 plan-v2.5.md 实测定稿版）  
> 更新时间：2026-09-14

---

## 1. 架构总览：Hub & Spoke 模型

CrossBrain 采用 **单一事实源（SSOT）+ 幂等分发** 的 Hub & Spoke 架构：

- **Hub**：`~/.ai-profile/` — 用户编辑内容的唯一权威来源
- **Spoke**：各 AI 工具的配置目录 — 由分发引擎写入，用户不直接操作

```
┌─────────────────────────────────────────────────┐
│              用户操作层（CrossBrain UI）           │
└────────────────────┬────────────────────────────┘
                     │ 点击【立即同步】
                     ▼
┌─────────────────────────────────────────────────┐
│         SSOT 数据仓库 ~/.ai-profile/              │
│  ├── global/rules.md   （全局规则，用户唯一编辑源） │
│  └── knowledge/*.md    （技能知识，每篇一个主题）   │
└──────┬──────────────────────────┬───────────────┘
       │ Adapter A                │ Adapter B
       ▼                          ▼
┌──────────────────┐   ┌──────────────────────────┐
│ Antigravity IDE  │   │      Claude Code CLI      │
│ ~/.gemini/config/│   │      ~/.claude/           │
└──────────────────┘   └──────────────────────────┘
```

---

## 2. SSOT 本地数据仓库结构

```
~/.ai-profile/
├── .git/                        # 本地版本历史（未装 git 时降级无历史模式，不阻断）
├── .gitignore                   # 固定忽略 AGENTS.md 等动态中间产物
├── global/
│   └── rules.md                 # 全局规则（用户在 UI 编辑的唯一源头）
├── knowledge/
│   ├── debug-rust.md            # 技能知识（每篇 = 一个独立经验主题）
│   ├── vue3-patterns.md
│   └── ...
└── AGENTS.md                    # 动态生成中间产物，包含 @import 或 rules.md 汇总
                                 # 此文件由 CrossBrain 自动维护，用户无需手动编辑
```

**核心约束**：
- `global/rules.md` 是用户编辑的唯一入口，严禁让用户直接编辑各工具的配置目录
- `AGENTS.md` 是中间产物，每次同步前重新生成，`.gitignore` 排除追踪

---

## 3. 幂等分发流程

```
用户在 UI 点击【立即同步】
  │
  ├─ Step 1. 读取 SSOT
  │      ├─ 解析 global/rules.md
  │      └─ 扫描 knowledge/*.md，对每个文件计算 slug-hash
  │
  ├─ Step 2. 执行 Adapter A: Antigravity IDE
  │      ├─ detect()：检查 %USERPROFILE%\.gemini\config\ 目录是否存在
  │      ├─ sync_l0()：写入 ~/.gemini/config/rules/crossbrain-L0.md（直接全量覆盖）
  │      ├─ sync_l2()：写入 ~/.gemini/config/skills/crossbrain-{slug-hash}/SKILL.md
  │      └─ cleanup_orphans()：
  │              遍历 ~/.gemini/config/skills/ 下所有 crossbrain-* 目录，
  │              删除不属于当前 knowledge 列表的历史技能目录
  │
  ├─ Step 3. 执行 Adapter B: Claude Code
  │      ├─ detect()：检查 %USERPROFILE%\.claude\ 目录是否存在
  │      ├─ sync_l0()：向 ~/.claude/CLAUDE.md 注入标记块（四情况协议，见 ADAPTER_SPEC.md）
  │      ├─ sync_l2()：写入 ~/.claude/skills/crossbrain-{slug-hash}/SKILL.md
  │      └─ cleanup_orphans()：
  │              遍历 ~/.claude/skills/ 下所有 crossbrain-* 目录，
  │              删除不属于当前 knowledge 列表的历史技能目录
  │
  └─ Step 4. SSOT 本地 Git 提交
         git add -A && git commit -m "sync: {ISO-8601-Time}"
         静默处理 "nothing to commit" — 视为成功，不弹报错
```

---

## 4. Slug-Hash 生成算法

**目的**：解决中文文件名、特殊字符导致路径解析崩溃，并实现文件内容变更的精准追踪。

```
算法步骤（以 "Vue3 组件性能优化.md" 为例）：

Step 1. 提取英文字符/拼音，转小写 kebab-case，截断至 ≤30 字符
        输出："vue3-perf"

Step 2. 计算文件内容 SHA-256，取前 6 位十六进制
        输出："7c8e2a"

Step 3. 组合最终 slug-hash
        输出："crossbrain-vue3-perf-7c8e2a"
```

**不变性保证**：
- 文件名修改 → slug 前缀变，hash 不变（内容未改）
- 文件内容修改 → hash 变，旧目录被孤儿清理删除，新目录创建
- 同一文件同一内容 → 永远生成相同 slug-hash（幂等）

### 实现说明（2026-09-14 落地，见 TASK-07）

上面的示例是**形态示意**，两处与实现有差异，以本段为准：

1. **Step 1 不做拼音转换**。示例把「组件性能优化」折算成 `perf` 是人工示意；
   V1 不引入拼音词典，中文一律被折叠掉。因此
   `"Vue3 组件性能优化.md"` 的实际输出是 `"crossbrain-vue3-{hash}"`。
   **纯中文文件名**的 kebab 段会退化为兜底值 `item`（如 `crossbrain-item-7a6883`）——
   功能不受影响（内容不同则 hash 不同，不会互相覆盖），仅影响目录名的可读性；
   UI 侧应始终展示「文件名 → 目录名」映射。
2. **Step 1 的字符集严格限定为 `[a-z0-9-]`**：ASCII 字母数字保留并转小写，
   其余所有字符（中文、空格、`#`、`+`、`_`、`.`、路径分隔符…）一律折叠为 `-`。
   `con` / `nul` / `com1` 这类 Windows 保留设备名由 `crossbrain-` 前缀天然规避。

**落地细节**：SHA-256 取前 3 字节（而非 `DefaultHasher` 之类的进程内哈希）——
后者的输出不保证跨 Rust 版本稳定，会导致升级工具链后全量技能目录被判为孤儿而重建。
内容寻址的标识符必须可跨版本、跨机器复现。

---

## 5. 孤儿清理（Orphan Cleanup）机制

```
每次同步结束后，对每个 Adapter 执行：

1. 列出目标目录下所有 crossbrain-* 子目录
   例：~/.gemini/config/skills/ 下的所有 crossbrain-* 目录

2. 计算当前 knowledge/*.md 列表对应的 slug-hash 集合

3. 删除 步骤1 中存在、但 步骤2 中不存在的目录
   （即已从 knowledge/ 删除的技能对应的旧技能目录）

4. 保留 步骤2 中存在的目录（本次已写入的）
```

**目的**：彻底杜绝幽灵规则残留，防止用户删除技能后 AI 工具仍能加载旧版本。

---

## 6. Git 提交封装规范

```rust
// 正确：使用 chrono crate 生成时间戳
use chrono::Local;
let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();
let msg = format!("sync: {}", timestamp);

// 错误：禁止调用 shell date 命令（Windows 完全不兼容）
// Command::new("sh").args(["-c", "date ..."]) ← 严禁
```

**异常处理**：
- `nothing to commit` → 静默处理，视为成功，不弹任何提示
- git 未安装 → 跳过 git 步骤，UI 显示「未检测到 Git，版本历史功能暂不可用」

---

## 7. Windows 路径处理规范

```rust
// 正确：使用 dirs crate 解析 home 目录
use dirs::home_dir;
let home = home_dir().expect("无法获取 home 目录");
let target = home.join(".ai-profile").join("global").join("rules.md");

// 错误：禁止字符串拼接路径
// let path = format!("C:\\Users\\{}\\...", username); ← 严禁
```

**原则**：
- 全程使用 `dirs` crate 解析 home 目录
- 路径拼接使用 Rust `Path` / `PathBuf` API，不用字符串拼接
- 不硬编码任何用户名或系统盘符

---

## 8. 分层命名规范（内部研发用）

> ⚠️ 以下术语**严禁出现在任何 UI 界面**，仅供开发阶段内部使用。

| 内部术语 | UI 展示用语 |
|:---|:---|
| L0 全局规则 | 全局规则 |
| L2 技能知识 | 技能知识 |
| Adapter | （不暴露） |
| SSOT | （不暴露） |
| slug-hash | （不暴露） |
| YAML frontmatter | （不暴露） |

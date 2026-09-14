# CrossBrain Adapter 接口规范文档

> 版本：V3.0（依据 plan-v2.5.md 实测定稿版）  
> 更新时间：2026-09-14

---

## 1. Adapter Trait 接口定义

所有 Adapter 必须实现以下 Rust trait：

```rust
pub trait Adapter {
    /// 检测目标 AI 工具是否已安装（通过检查其配置目录是否存在）
    /// 返回 Ok(true) = 已安装，Ok(false) = 未安装，Err = 检测过程出错
    fn detect(&self) -> Result<bool, AdapterError>;

    /// 同步全局规则（L0）到目标工具
    /// 必须保证幂等：多次调用结果一致
    fn sync_l0(&self, rules_content: &str) -> Result<(), AdapterError>;

    /// 同步技能知识（L2）到目标工具
    /// slug_hash: 由 SlugHasher 生成的唯一标识符
    /// content: 经过 YAML frontmatter 格式化的完整文件内容
    fn sync_l2(&self, slug_hash: &str, content: &str) -> Result<(), AdapterError>;

    /// 清理孤儿目录：删除不在 active_slugs 列表中的历史 crossbrain-* 目录
    /// active_slugs: 当前 knowledge/ 列表对应的 slug-hash 完整集合
    fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError>;
}

pub struct CleanupReport {
    pub deleted_dirs: Vec<String>,  // 被删除的孤儿目录名列表
    pub kept_dirs: Vec<String>,     // 保留的目录名列表
}

pub enum AdapterError {
    NotInstalled,
    PermissionDenied(String),
    DirectoryCreateFailed(String),
    WriteError(String),
    Other(String),
}
```

---

## 2. Antigravity IDE Adapter

### 2.1 安装检测

```rust
// 检查目录是否存在即可，无需检测可执行文件
let config_dir = home_dir().join(".gemini").join("config");
// Windows: %USERPROFILE%\.gemini\config\
// 目录存在 → 已安装
```

### 2.2 L0 全局规则（sync_l0）

- **目标路径**：`%USERPROFILE%\.gemini\config\rules\crossbrain-L0.md`
- **写入策略**：**直接全量覆盖**（此文件完全由 CrossBrain 托管，不含用户自定义内容）
- **自动创建**：若 `rules/` 目录不存在，自动创建，不报错
- **文件格式**：纯 Markdown，无特殊格式要求

```
~/.gemini/config/rules/crossbrain-L0.md
───────────────────────────────────────
# CrossBrain 全局规则

[global/rules.md 的完整内容直接写入]
```

### 2.3 L2 技能知识（sync_l2）

- **目标路径**：`%USERPROFILE%\.gemini\config\skills\crossbrain-{slug-hash}\SKILL.md`
- **写入策略**：直接覆盖（目录不存在则自动创建）
- **文件格式**：必须包含 YAML frontmatter，格式如下：

```markdown
---
name: vue3-perf
description: Vue3 组件性能优化：长列表虚拟滚动与响应式依赖控制
---

[knowledge/*.md 的正文内容]
```

**frontmatter 字段说明**：

| 字段 | 类型 | 说明 | 来源 |
|:---|:---|:---|:---|
| `name` | string | 技能名：文件名折算的 kebab 段（**不含** hash，**不含** `crossbrain-` 命名空间前缀） | `slug::Slug::skill_name()`（TASK-07） |
| `description` | string | 技能的一句话描述，驱动 AI 懒加载判断 | 取 knowledge/*.md 的第一行 `# ` 标题（去掉 `# `） |

> **懒加载核心**：`description` 字段是 Antigravity IDE 决定是否加载该 SKILL.md 的唯一依据。描述必须精准，过于模糊会导致 AI 无法正确触发加载。

### 2.4 孤儿清理（cleanup_orphans）

```rust
// 扫描 ~/.gemini/config/skills/ 下所有 crossbrain-* 目录
// 对比 active_slugs 列表，删除不在其中的目录
let skills_dir = config_dir.join("skills");
for entry in fs::read_dir(&skills_dir)? {
    let name = entry.file_name();
    if name.starts_with("crossbrain-") && !active_slugs.contains(&name) {
        fs::remove_dir_all(entry.path())?;
    }
}
```

### 2.5 Plan B 降级策略（自动触发，用户无感知）

| 场景 | 主路径 | 降级触发条件 | Plan B 行为 |
|:---|:---|:---|:---|
| L0 全局规则 | 写入 `rules/crossbrain-L0.md` | `rules/` 目录不被读取（极端配置） | 将内容写入 `~/.gemini/config/AGENTS.md` 的标记块 |
| L2 技能知识 | `skills/crossbrain-*/SKILL.md` 懒加载 | 懒加载失效（极端情况） | 降级为 `alwaysApply: true`，全量注入，UI 显示上下文增加警告 |

---

## 3. Claude Code Adapter

### 3.1 安装检测

```rust
// 检查 .claude 目录是否存在
let claude_dir = home_dir().join(".claude");
// Windows: %USERPROFILE%\.claude\
// 目录存在 → 已安装
```

### 3.2 L0 全局规则（sync_l0）— 标记块协议

> ⚠️ **核心原则**：CrossBrain **绝不删除、绝不覆盖**用户既有规则内容。标记块永远是**追加**，而非替换整个文件。

**标记块格式**：

```markdown
<!-- CrossBrain:Start -->
<!-- 本区域由 CrossBrain 自动维护，修改请在 CrossBrain App 中进行 -->
@~/.ai-profile/AGENTS.md
<!-- CrossBrain:End -->
```

**CLAUDE.md 处理四情况协议（顺序判断，命中即止）**：

```
情况一：文件不存在
  → 直接创建，内含 CrossBrain:Start 标记块 + @import 引用
  → 静默完成，无任何 UI 提示

情况二：文件存在，且已含 <!-- CrossBrain:Start --> 标记块
  → 用正则精准替换标记块内的内容（幂等操作）
  → 标记块外的用户内容原封不动
  → 静默完成，无任何 UI 提示

情况三：文件存在，不含标记块，且 .crossbrain-backup 不存在
  → 先备份：cp CLAUDE.md CLAUDE.md.crossbrain-backup
  → 将 CrossBrain 标记块追加到文件末尾（不覆盖，不删除用户已有规则）
  → UI 弹出提示：
    「已将 CrossBrain 规则追加到你的 Claude 配置末尾。
     原文件已备份至 CLAUDE.md.crossbrain-backup，如需恢复请查看此文件。」

情况四：文件存在，不含标记块，且 .crossbrain-backup 已存在
  → 跳过备份步骤（防止覆盖已有备份）
  → 同样执行追加标记块到末尾
  → 静默完成，不重复提示
```

**正则匹配模式（Rust）**：

```rust
// 匹配 CrossBrain 标记块（含块内任意内容）
let re = Regex::new(r"<!-- CrossBrain:Start -->[\s\S]*?<!-- CrossBrain:End -->").unwrap();

// 情况二：替换
let new_content = re.replace(&existing, new_block);

// 情况三/四：追加（not matched）
let appended = format!("{}\n\n{}", existing.trim_end(), new_block);
```

### 3.2.1 写入方式：必须断开硬链接（决策 D-03，硬性要求）

> ⚠️ **本条优先于任何「看起来更简单」的写法**。`CLAUDE.md` 的写入**不得**使用
> 就地 `fs::write`，必须走「临时文件 → 删除原路径 → rename」三步。

**背景（本机实测）**：`~/.claude/CLAUDE.md` 与另外 4 个工具路径是**硬链接**，
共享同一个 inode（`links=5`）：

```
~/.ai-memory/user_profile.md            ← 原始
~/.cursor/rules/user_profile.md
~/.config/opencode/AGENTS.md
~/.gemini/config/rules/user_global.md
~/.claude/CLAUDE.md                     ← 本 Adapter 的目标
```

**若就地覆写**：标记块会同时出现在全部 5 个工具读到的规则里；而
`@~/.ai-profile/AGENTS.md` 是 Claude 专有的引用语法，在 Cursor / OpenCode /
Antigravity 中毫无意义——属于**跨工具污染**。

**正确写法**：

```rust
// ① 内容先写入临时文件（失败也不丢数据）
fs::write(&tmp, new_content)?;          // tmp = CLAUDE.md.crossbrain-tmp
// ② 删除原路径 —— Windows 上这一步才真正打断硬链接
match fs::remove_file(&claude_md) {
    Ok(()) => {}
    Err(e) if e.kind() == ErrorKind::NotFound => {}   // 情况一：文件本就不存在
    Err(e) => return Err(/* 明确报错，不要静默 */),
}
// ③ 同卷原子替换
fs::rename(&tmp, &claude_md)?;
```

**为什么第 ② 步不可省**：Windows 的 `rename` **不覆盖已存在的目标**，
目标存在时直接报错；而「先删再 rename」正好也是**断链本身所必需的动作**——一步兼得。
`rename` 是同卷原子操作，不会出现「读到半个文件」的中间态。

**已接受的代价**：用户手工搭的「一处修改、五处生效」机制在 `CLAUDE.md` 这一侧被打破，
该文件从此是独立文件。

> 该策略对**四情况全部适用**，包括 TASK-14 的「一键卸载/去痕」（移除标记块时同样是写入）。

### 3.3 L2 技能知识（sync_l2）

- **目标路径**：`%USERPROFILE%\.claude\skills\crossbrain-{slug-hash}\SKILL.md`
- **文件格式**：与 Antigravity Adapter 100% 一致（相同的 YAML frontmatter 规范）
- **写入策略**：直接覆盖（目录不存在则自动创建）

### 3.4 孤儿清理（cleanup_orphans）

```rust
// 扫描 ~/.claude/skills/ 下所有 crossbrain-* 目录
// 逻辑与 Antigravity Adapter 完全一致
let skills_dir = claude_dir.join("skills");
// ...（同 Antigravity 实现）
```

### 3.5 Plan B 降级策略

| 场景 | 主路径 | 降级触发条件 | Plan B 行为 |
|:---|:---|:---|:---|
| L0 全局规则 | `CLAUDE.md` 注入标记块 + `@~/.ai-profile/AGENTS.md` | `@~/.ai-profile/AGENTS.md` 路径展开失败 | 将 `global/rules.md` 完整内容直接写入标记块，不生成 AGENTS.md |
| L2 技能知识 | `~/.claude/skills/crossbrain-*/SKILL.md` 懒加载 | `skills/` 目录不支持懒加载 | 将所有知识标题追加到 `CLAUDE.md` 标记块内作为目录索引，降级为全量加载 |

---

## 4. SKILL.md 文件格式规范（两个 Adapter 通用）

### 4.1 完整格式

```markdown
---
name: {slug 前缀，不含 hash}
description: {一句话描述，精准反映技能主题，这是 AI 懒加载的触发依据}
---

# {knowledge/*.md 的原始标题}

{knowledge/*.md 的正文内容，原样写入}
```

### 4.2 description 字段写法规范

| 规则 | 示例 |
|:---|:---|
| ✅ 精准描述具体场景 | `Vue3 组件性能优化：长列表虚拟滚动与响应式依赖控制` |
| ✅ 包含核心技术关键词 | `Rust 生命周期与借用检查器常见错误场景与修复模式` |
| ❌ 过于宽泛 | `前端开发经验` |
| ❌ 使用内部术语 | `L2 技能知识 - Vue3` |

**description 自动生成规则**：取 knowledge/*.md 文件的第一行 `# ` 标题，去掉 `# ` 前缀。若文件无标题行，取文件名的可读形式。

### 4.3 超长内容警告

在 UI 编辑器层面（非 Adapter 层）：  
当 knowledge/*.md 内容超过 **2000 字**时，编辑器顶部显示黄色警告条：  
「此技能知识偏长，建议按主题拆分为多篇，有助于 AI 精准加载」

---

## 5. 路径三级检测策略

适用于所有 Adapter 的 `detect()` 实现：

```
Level 1: 配置目录存在 → 正常推送
Level 2: 检测到安装（如注册表/其他标识）但配置目录缺失 → 自动创建目录，静默继续
Level 3: 无法确认是否安装 → 弹出路径填写框，用户手动确认
Level 0: 确认未安装（配置目录不存在且无安装标识） → 跳过，不报错，UI 标记为「未检测到」
```

---

## 6. 新增 Adapter 标准流程（V1.5 扩展指引）

当接入新工具（如 Trae、ChatGPT IDE）时，必须按以下步骤进行：

1. **Spike 验证**：验证目标工具的全局规则路径和懒加载机制（参见 `docs/testing/SPIKE_RESULTS.md` 格式）
2. **实现 Adapter trait**：实现 `detect()` / `sync_l0()` / `sync_l2()` / `cleanup_orphans()` 四个方法
3. **编写 Plan B**：必须为 L0 和 L2 各准备一个降级方案
4. **补充测试用例**：在 `docs/testing/TEST_PLAN.md` 中添加对应测试用例
5. **更新工具矩阵**：在 `docs/product/PRD.md` 的工具支持矩阵中更新状态

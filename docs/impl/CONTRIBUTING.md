# CrossBrain 贡献者指南

> 更新时间：2026-09-14  
> License：Apache-2.0

---

## 项目目录结构

```
crossbrain/
├── README.md
├── package.json                    # pnpm 管理，version 需与 Cargo.toml 同步
├── src/                            # Vue 3 + TypeScript 前端
│   ├── components/                 # UI 组件（每个组件需有同名 .md 文档）
│   ├── assets/
│   │   └── icons/                  # 本地 SVG 图标（禁止使用 Emoji 作为功能图标）
│   └── main.ts
├── src-tauri/                      # Rust / Tauri 后端
│   ├── src/
│   │   ├── adapters/
│   │   │   ├── mod.rs              # Adapter trait 定义
│   │   │   ├── antigravity.rs      # AntigravityAdapter 实现
│   │   │   └── claude_code.rs      # ClaudeCodeAdapter 实现
│   │   ├── slug.rs                 # Slug-Hash 生成算法
│   │   ├── git.rs                  # Git 提交封装
│   │   └── main.rs
│   └── Cargo.toml                  # version 需与 package.json 同步
└── docs/
    ├── product/
    │   ├── PRD.md
    │   └── CHANGELOG.md
    ├── tech/
    │   ├── ARCHITECTURE.md
    │   ├── ADAPTER_SPEC.md
    │   ├── TECH_STACK.md
    │   └── WINDOWS_COMPATIBILITY.md
    ├── testing/
    │   ├── TEST_PLAN.md
    │   └── SPIKE_RESULTS.md
    └── impl/
        ├── DEV_SETUP.md
        ├── RELEASE.md
        └── CONTRIBUTING.md         # 本文件
```

---

## 代码规范

### 前端（TypeScript / Vue 3）

- **禁止** `any`（包括 `as any`）
- 所有组件的 Props / Emits 必须有明确的 Interface 定义
- 深度对象属性访问必须使用可选链 `?.` 和空值合并 `??`
- 事件监听、定时器必须在组件卸载时显式清理（防内存泄漏）
- 图标只能使用 SVG，禁止 Emoji

### Rust 后端

- 路径处理只能使用 `dirs` crate + `Path` / `PathBuf` API，禁止字符串拼接
- 时间戳只能使用 `chrono` crate，禁止调用 shell 命令
- git 操作必须处理 `nothing to commit` 的非零退出码（静默为成功）

### 包管理

- **只能用 pnpm**，禁止 npm / yarn
- 引入新依赖前必须检查 `package.json` / `Cargo.toml` 中是否已有能满足需求的依赖

---

## 如何新增一个 Adapter（V1.5 扩展指引）

新增 Adapter 必须按以下步骤进行，缺一不可：

### Step 1. Spike 验证（先验证，再编码）

在正式开发前，必须完成以下验证：

1. 确认目标工具的**全局规则文件路径**（或等效机制）
2. 确认**技能知识的懒加载机制**是否存在（按 description 按需加载，或全量加载）
3. 在**零上下文全新对话**中验证（防假阳性）
4. 将验证结果按模板记录到 `docs/testing/SPIKE_RESULTS.md`

验证通过后方可开始编码。

### Step 2. 实现 Adapter trait

在 `src-tauri/src/adapters/` 下新建文件（例：`trae.rs`），实现以下四个方法：

```rust
impl Adapter for TraeAdapter {
    fn detect(&self) -> Result<bool, AdapterError> { ... }
    fn sync_l0(&self, rules_content: &str) -> Result<(), AdapterError> { ... }
    fn sync_l2(&self, slug_hash: &str, content: &str) -> Result<(), AdapterError> { ... }
    fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError> { ... }
}
```

详细接口规范见：[`docs/tech/ADAPTER_SPEC.md`](../tech/ADAPTER_SPEC.md)

### Step 3. 实现 Plan B 降级逻辑

每个 Adapter 的 `sync_l0` 和 `sync_l2` 必须各有一个降级方案，在主路径失败时自动触发，用户无感知。  
将 Plan B 方案记录到 `docs/tech/ADAPTER_SPEC.md` 中对应 Adapter 的章节。

### Step 4. 补充测试用例

在 `docs/testing/TEST_PLAN.md` 中为新 Adapter 添加测试用例，至少覆盖：
- 安装检测（已安装 / 未安装）
- L0 写入与内容验证
- L2 写入与懒加载验证（若支持）
- 孤儿清理

### Step 5. 更新工具矩阵

在 `docs/product/PRD.md` 的工具支持矩阵中将该工具状态更新为 ✅，并标注实测通过时间。

---

## 提交规范（Commit Message）

```
feat: 新增 Trae Adapter
fix: 修复 CLAUDE.md 情况三重复追加 bug
refactor: 简化 Slug-Hash 生成逻辑
docs: 更新 ADAPTER_SPEC.md 的 Plan B 章节
chore: 升级 Tauri 至 2.1.0
```

格式：`{type}: {简短描述（中文或英文均可）}`

| type | 适用场景 |
|:---|:---|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `refactor` | 重构（无功能变更） |
| `docs` | 文档变更 |
| `chore` | 构建、依赖、配置变更 |

---

## Pull Request 规范

1. PR 标题格式与 commit 规范一致
2. PR 描述中必须说明：
   - 解决了什么问题
   - 如何验证（测试步骤）
   - 是否有 Breaking Change
3. 新增 Adapter 的 PR 必须附带 `SPIKE_RESULTS.md` 的更新记录
4. 所有 PR 合并前需通过 Rust 编译（`cargo build`）和前端类型检查（`pnpm typecheck`）

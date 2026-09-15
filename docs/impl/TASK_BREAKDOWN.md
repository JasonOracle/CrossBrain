# CrossBrain — 详细任务拆解与验收标准（TASK BREAKDOWN）

> **写给 AI 工具的使用说明**：  
> 1. 先读 `MASTER_CONTEXT.md` 和 `CURRENT_STATUS.md` 确认你的任务编号  
> 2. 在本文件找到对应任务，**逐步骤执行，不要跳步**  
> 3. 完成后必须执行每个任务末尾的「验收检查清单」，全部通过才算完成  
> 4. 完成后更新 `CURRENT_STATUS.md`

---

## TASK-01：初始化 Tauri v2 + Vue 3 + TypeScript 项目骨架

### 前置条件检查（先做这个，再开始）

在执行任何命令前，先逐项验证：

```powershell
# 1. 检查 pnpm 版本（必须是 8.x 以上）
pnpm --version

# 2. 检查 Rust toolchain（MSVC 模式，Tauri v2 必需）
rustc --version
cargo --version

# 3. 检查 Node.js 版本（必须是 18 / 20 / 22 LTS）
node --version

# 4. 检查 Tauri CLI v2
cargo tauri --version

# 5. 检查 WebView2 Runtime（Windows 必需，一般系统自带）
# 注册表路径：HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}
```

**如果任何一项检查失败，停止执行，读 `docs/impl/DEV_SETUP.md` 先安装依赖。**

### 执行步骤

**步骤 1**：在项目根目录 `d:\project\memory1.0` 下，确认当前目录只有文档文件（无 `src/` 或 `src-tauri/`）

```powershell
ls d:\project\memory1.0
# 预期看到：MASTER_CONTEXT.md, CURRENT_STATUS.md, README.md, plan-v2.5.md, docs/, archive/
# 不应看到：src/, src-tauri/, package.json
```

**步骤 2**：创建 Tauri v2 + Vue 3 + TypeScript 骨架

> ⚠️ **参数已修正**：`create-tauri-app` 4.x **没有** `--app-name` 和 `--no-open-browser` 参数，
> 且 `PROJECTNAME` 只接受目录名，传 `./` 会生成错误的包名（`memory1.0`）。
> 正确做法：在临时子目录里用项目名 `crossbrain` 生成，再合并回根目录（保护已有 `README.md`）。

```powershell
cd d:\project\memory1.0
mkdir .scaffold
cd .scaffold
pnpm dlx create-tauri-app@latest crossbrain --manager pnpm --template vue-ts --identifier com.crossbrain.app --tauri-version 2 --yes

# 合并回根目录（不复制 README.md，避免覆盖项目文档）
cd d:\project\memory1.0
cp .scaffold\crossbrain\.gitignore .
cp -r .scaffold\crossbrain\.vscode .
cp .scaffold\crossbrain\index.html .
cp .scaffold\crossbrain\package.json .
cp -r .scaffold\crossbrain\public .
cp -r .scaffold\crossbrain\src .
cp -r .scaffold\crossbrain\src-tauri .
cp .scaffold\crossbrain\tsconfig.json .
cp .scaffold\crossbrain\tsconfig.node.json .
cp .scaffold\crossbrain\vite.config.ts .
rm -rf .scaffold
```

> 生成后需手动补 `package.json` 的 `packageManager` 字段（create-tauri-app 不会自动写入）：
> `"packageManager": "pnpm@9.15.9"`

> ⚠️ 绝不能覆盖已有的文档文件（`README.md`、`MASTER_CONTEXT.md`、`docs/`）。

**步骤 3**：安装 Node 依赖

```powershell
cd d:\project\memory1.0
pnpm install
```

**步骤 4**：验证开发服务器可以启动

```powershell
# 先只跑前端（不编译 Rust，速度快）
pnpm dev
# 或者跑完整的 Tauri 开发模式（首次会慢，需要编译 Rust 约 5 分钟）
pnpm tauri dev
```

### ✅ 验收检查清单（全部通过才算完成）

```
[ ] d:\project\memory1.0\package.json 存在，且 "name" 为 "crossbrain" 或 "cross-brain"
[ ] d:\project\memory1.0\src-tauri\Cargo.toml 存在，且 [package] name 包含 "crossbrain"
[ ] d:\project\memory1.0\src-tauri\tauri.conf.json 存在
[ ] 执行 pnpm dev 后，浏览器（或 Tauri 窗口）能打开默认页面，无报错
[ ] 原有文档文件（MASTER_CONTEXT.md, docs/ 目录）未被覆盖或删除
[ ] package.json 中 packageManager 字段为 "pnpm@x.x.x"（确认没有被改为 npm）
```

### 完成后必须做

1. 将 `CURRENT_STATUS.md` 中 `TASK-01` 改为 ✅
2. 在「本次变更记录」中追加你的操作记录

---

## TASK-02：配置 Tailwind CSS + Naive UI

### 前置条件

- TASK-01 已完成（✅）
- `package.json` 和 `src/` 目录已存在

### 执行步骤

**步骤 1**：检查现有依赖（先看再装，避免重复引入）

```powershell
# 查看当前 package.json 中已有哪些依赖
cat d:\project\memory1.0\package.json
```

**步骤 2**：安装 Tailwind CSS **v4**（如果不存在）

> ⚠️ **v4 与 v3 不兼容，不要照抄 v3 的做法**。本项目已决策采用 **v4 + 官方 Vite 插件**
> （决策记录见 `CURRENT_STATUS.md` 阻塞项 D-01）。v4 的关键变化：
>
> | v3 的做法 | v4 的做法 |
> |:---|:---|
> | `pnpm add -D tailwindcss postcss autoprefixer` | `pnpm add -D tailwindcss @tailwindcss/vite` |
> | `npx tailwindcss init -p` 生成 `tailwind.config.js` | **无此命令**，也**不需要** `tailwind.config.js` |
> | PostCSS 方案（`postcss.config.js`） | Vite 插件方案（`@tailwindcss/vite`） |
> | CSS 写 `@tailwind base/components/utilities` 三行 | CSS 写 `@import "tailwindcss";` 一行 |
> | `theme.extend` 写在 JS 配置里 | 设计令牌写在 CSS 的 `@theme { }` 里 |

```powershell
pnpm add -D tailwindcss @tailwindcss/vite
```

**步骤 3**：在 `vite.config.ts` 中注册 Tailwind 插件（替代 v3 的 PostCSS 方案）

```ts
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(), // 放在 vue() 之后
    // ...其他插件
  ],
})
```

**步骤 4**：新建 `src/style.css`，以 `@import "tailwindcss";` 开头

```css
/* src/style.css */
@import "tailwindcss";

/* 设计令牌（v4 的配置方式，等价于 v3 的 theme.extend）
   统一在此维护，组件里禁止硬编码色值 */
@theme {
  --color-brand-500: #3366ff;
}
```

并在 `src/main.ts` 中引入该样式文件：

```ts
import './style.css'
```

**步骤 5**：安装 Naive UI（如果不存在）

```powershell
pnpm add naive-ui
# 安装 vfonts 提供字体支持（Naive UI 推荐）
pnpm add vfonts
```

**步骤 6**：在 `src/main.ts` 中引入 Naive UI

```ts
// src/main.ts
import { createApp } from 'vue'
import App from './App.vue'
// Naive UI 按需引入，不在 main.ts 全局注册（避免打包体积过大）
// 具体组件在各 .vue 文件中按需 import
import './style.css'

createApp(App).mount('#app')
```

**步骤 7**：安装 SVG 图标支持

```powershell
pnpm add -D unplugin-icons unplugin-vue-components
# 图标集本地安装一份，保证构建不依赖网络（铁律：项目零网络依赖）
pnpm add -D @iconify-json/lucide
```

在 `vite.config.ts` 中配置：

```ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import Icons from 'unplugin-icons/vite'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
    Icons(),
    Components({
      resolvers: [NaiveUiResolver()],
      dts: 'src/components.d.ts', // 生成自动导入组件的类型声明
    }),
  ],
})
```

**步骤 8**：让 TypeScript 认识 `~icons/*` 模块

在 `src/vite-env.d.ts` 中加入类型引用：

```ts
/// <reference types="vite/client" />
/// <reference types="unplugin-icons/types/vue" />
```

> ⚠️ 模板自带的 `declare module "*.vue"` 兜底声明**必须删掉**：
> vue-tsc / Volar 原生解析 `.vue` 文件，保留该声明会把组件类型擦成 `any`，
> 既违反铁律 L-04（禁用 any），也会让组件的 props 类型提示全部失效。

> ⚠️ `src/components.d.ts` 是**构建时自动生成**的（首次 `vite build` 后出现）。
> 它不在 `.gitignore` 里也没关系，建议提交入库，便于 IDE 直接获得类型提示。

### ✅ 验收检查清单

```
[ ] package.json 的 dependencies 中存在 "naive-ui"
[ ] package.json 的 devDependencies 中存在 "tailwindcss"（版本为 4.x）
[ ] package.json 的 devDependencies 中存在 "@tailwindcss/vite"
[ ] package.json 的 devDependencies 中存在 "unplugin-icons" 与 "unplugin-vue-components"
[ ] vite.config.ts 中包含 tailwindcss()、Icons() 和 Components() 三个插件配置
[ ] 项目中【不存在】tailwind.config.js 和 postcss.config.js（v4 不需要，存在反而是配置错误的信号）
[ ] src/style.css 中包含 @import "tailwindcss";（不是 @tailwind base 三行）
[ ] pnpm build（含 vue-tsc --noEmit）零报错通过
[ ] 构建产物 dist/assets/*.css 中包含 .text-blue-500 规则
[ ] 构建产物 dist/assets/*.css 中包含自定义令牌 .text-brand-500 { color: var(--color-brand-500) }
[ ] 执行 pnpm dev，浏览器无报错
[ ] 页面上的 class="text-blue-500" 元素显示为蓝色（视觉确认）
[ ] 页面上 Naive UI 按钮渲染正常，无样式错乱
```

### 完成后必须做

1. 将 `CURRENT_STATUS.md` 中 `TASK-02` 改为 ✅
2. 追加变更记录

---

## TASK-03：封装 Rust 路径模块 + 初始化 `~/.ai-profile/` 目录

### 前置条件

- TASK-01 已完成（✅）
- `src-tauri/src/` 目录已存在

### 背景说明

`~/.ai-profile/` 是 CrossBrain 的核心数据目录（SSOT）。本任务负责在应用启动时自动创建该目录结构，并封装所有路径获取函数。

**所有路径处理必须通过这里封装的函数，绝对不能在其他文件中硬编码路径。**

### 执行步骤

**步骤 1**：在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加依赖（先检查是否已有）

```toml
[dependencies]
dirs = "5"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tauri = { version = "2", features = [] }
```

**步骤 2**：创建 `src-tauri/src/paths.rs`

```rust
//! paths.rs — CrossBrain 所有路径的统一出口
//! 规则：任何文件需要路径时，必须调用本模块的函数，禁止自行拼接路径

use std::path::PathBuf;

/// 获取 ~/.ai-profile/ 根目录
pub fn profile_root() -> PathBuf {
    dirs::home_dir()
        .expect("无法获取系统 home 目录，这是一个致命错误")
        .join(".ai-profile")
}

/// 获取 ~/.ai-profile/global/rules.md
pub fn global_rules_file() -> PathBuf {
    profile_root().join("global").join("rules.md")
}

/// 获取 ~/.ai-profile/knowledge/ 目录
pub fn knowledge_dir() -> PathBuf {
    profile_root().join("knowledge")
}

/// 获取 ~/.ai-profile/AGENTS.md（动态生成的中间产物）
pub fn agents_md_file() -> PathBuf {
    profile_root().join("AGENTS.md")
}

/// 获取 ~/.gemini/config/ 目录（Antigravity Adapter 检测路径）
pub fn gemini_config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("无法获取系统 home 目录")
        .join(".gemini")
        .join("config")
}

/// 获取 ~/.claude/ 目录（Claude Code Adapter 检测路径）
pub fn claude_dir() -> PathBuf {
    dirs::home_dir()
        .expect("无法获取系统 home 目录")
        .join(".claude")
}
```

**步骤 3**：创建 `src-tauri/src/init.rs`，负责初始化目录结构

```rust
//! init.rs — 应用启动时初始化 ~/.ai-profile/ 目录结构

use std::fs;
use crate::paths;

/// 初始化 SSOT 目录结构。幂等操作，已存在不报错。
/// 在 main.rs 的 tauri::Builder 启动前调用。
pub fn ensure_profile_dirs() -> Result<(), String> {
    let dirs_to_create = vec![
        paths::profile_root(),
        paths::profile_root().join("global"),
        paths::knowledge_dir(),
    ];

    for dir in dirs_to_create {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("无法创建目录 {:?}：{}", dir, e))?;
    }

    // 如果 global/rules.md 不存在，创建空文件（首次启动场景）
    let rules_file = paths::global_rules_file();
    if !rules_file.exists() {
        fs::write(&rules_file, "")
            .map_err(|e| format!("无法创建 rules.md：{}", e))?;
    }

    Ok(())
}
```

**步骤 4**：注册模块并调用初始化

> ⚠️ **注意实际入口是 `lib.rs`，不是 `main.rs`**。
> Tauri v2 官方模板把业务逻辑放在 `lib.rs`（`crossbrain_lib::run()`），
> `main.rs` 只是一个两行的薄壳（`fn main() { crossbrain_lib::run() }`）。
> 这是 Tauri 支持移动端入口的固定结构，**不要改动 `main.rs`**。

```rust
// src-tauri/src/lib.rs
mod init;
mod paths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 应用启动时立即初始化 SSOT 目录底座（~/.ai-profile/）。
    // 失败不 panic：目录不可用时应用仍应能打开并给出提示，而不是闪退。
    if let Err(e) = init::ensure_profile_dirs() {
        eprintln!("CrossBrain 初始化失败：{}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### ✅ 验收检查清单

```
[ ] src-tauri/src/paths.rs 文件存在
[ ] src-tauri/src/init.rs 文件存在
[ ] Cargo.toml [dependencies] 中包含 dirs = "5"
[ ] Cargo.toml [dependencies] 中包含 chrono
[ ] 执行 cargo build（在 src-tauri 目录下）编译无 error（warning 可以有）
[ ] 执行 pnpm tauri dev 后，%USERPROFILE%\.ai-profile\ 目录自动创建
[ ] %USERPROFILE%\.ai-profile\global\rules.md 文件自动创建（内容为空）
[ ] %USERPROFILE%\.ai-profile\knowledge\ 目录自动创建
[ ] %USERPROFILE%\.ai-profile\.gitignore 文件自动创建（内容为忽略 AGENTS.md）
[ ] 再次启动应用，上述目录已存在时不报错（幂等验证）
```

> **关于 `.gitignore`**：它不在原始验收清单中，但 `docs/tech/ARCHITECTURE.md` 第 2 节
> 把 SSOT 的 `.gitignore` 定义为目录结构的固定组成部分（用于排除 `AGENTS.md` 这个动态中间产物）。
> 本任务既然负责「初始化目录底座」，就一并创建，避免后续无人认领。
>
> **关于依赖版本**：`dirs` 锁在 `5`（实际最新为 7.x）。`dirs` 6.0 起调整了 API 与
> 平台后端，5.x 的 `home_dir()` 已在 TASK-03 实测定稿；未经验证前**不要**随手升到 7.x。
>
> **关于首次编译失败**：本机存在系统性的 `拒绝访问 (os error 5)`（Windows 文件句柄延迟，
> 与杀软无关）。若 `cargo build` 因此中断，清理 `target/debug/build/<crate>-*` 后重试即可，
> **不要怀疑代码**。详见 `docs/impl/DEV_SETUP.md`「坑 4」。
>
> **首次编译耗时**：484 个 crate 约 12 分钟；后续增量 1~6 秒。

### 完成后必须做

1. 将 `CURRENT_STATUS.md` 中 `TASK-03` 改为 ✅
2. 追加变更记录

---

## TASK-04：定义 Adapter Trait

### 背景说明

所有 AI 工具的适配器都实现同一个 trait，保证接口统一。本任务只定义接口，不实现具体逻辑。

**完整的接口规范在 `docs/tech/ADAPTER_SPEC.md` 第 1 节，实现前必须读一遍。**

### 执行步骤

> ⚠️ **实施修正（2026-09-14，TASK-04 实测定稿）**
>
> 原步骤只写了「创建 `mod.rs`」，但那样**既跑不通、也无法真正验收**。实测补充两条硬要求：
>
> 1. **必须同时创建 `antigravity.rs` 与 `claude_code.rs` 两个占位文件。**
>    `mod.rs` 中的 `pub mod antigravity;` 声明要求对应文件存在，否则 `cargo build`
>    报 `file not found for module`——与验收清单第 7 条「编译通过」直接冲突。
>    占位文件只需模块级 doc comment（写明归属任务与目标路径），正文留待 TASK-05 / TASK-06 填充。
> 2. **必须在 `src-tauri/src/lib.rs` 中挂载 `pub mod adapters;`。**
>    否则 `adapters/` 只是磁盘上的孤儿目录，**根本不参与编译**——`cargo build`
>    照样返回成功，但验收第 1~6 条全是空转。实测已完成挂载。
>
> **另补**：trait 增加了 `Send + Sync` 约束（原文档未写）。Adapter 实例要放进 Tauri 的
> `State` 并在 command handler（可能运行在其他线程）中被调用；无状态实现天然满足此约束，
> 成本为零。若拖到状态管理任务才补，将被迫修改这个已被多处依赖的契约。
>
> **同时明确**：`CleanupReport` 与 `AdapterError` 均派生 `Debug / Clone / PartialEq / Eq`。
> 原文档只写了 `#[derive(Debug)]`，但 TASK-05 / TASK-06 的 `cleanup_orphans()` 测试
> 需要精确比对两个目录列表，缺 `PartialEq` 就无法编写断言。

**步骤 1**：创建 `src-tauri/src/adapters/mod.rs`

```rust
//! adapters/mod.rs — Adapter trait 定义与错误类型

pub mod antigravity;
pub mod claude_code;

/// 孤儿清理报告
pub struct CleanupReport {
    /// 被删除的孤儿目录名（例："crossbrain-vue3-perf-7c8e2a"）
    pub deleted_dirs: Vec<String>,
    /// 保留的目录名
    pub kept_dirs: Vec<String>,
}

/// Adapter 统一错误类型
#[derive(Debug)]
pub enum AdapterError {
    /// 工具未安装（配置目录不存在）
    NotInstalled,
    /// 文件/目录权限不足
    PermissionDenied(String),
    /// 自动创建目录失败
    DirectoryCreateFailed(String),
    /// 文件写入失败
    WriteError(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::NotInstalled => write!(f, "工具未安装"),
            AdapterError::PermissionDenied(msg) => write!(f, "没有写入权限：{}", msg),
            AdapterError::DirectoryCreateFailed(msg) => write!(f, "无法创建配置目录：{}", msg),
            AdapterError::WriteError(msg) => write!(f, "文件写入失败：{}", msg),
            AdapterError::Other(msg) => write!(f, "错误：{}", msg),
        }
    }
}

/// 所有 AI 工具适配器必须实现的统一接口
pub trait Adapter {
    /// 检测目标工具是否已安装
    /// 返回 Ok(true) = 已安装；Ok(false) = 未安装；Err = 检测过程异常
    fn detect(&self) -> Result<bool, AdapterError>;

    /// 同步全局规则到目标工具
    /// rules_content: global/rules.md 的完整文本内容
    /// 必须幂等：多次调用结果一致
    fn sync_l0(&self, rules_content: &str) -> Result<(), AdapterError>;

    /// 同步一个技能知识到目标工具
    /// slug_hash: 由 slug.rs 生成的完整标识符（如 "crossbrain-vue3-perf-7c8e2a"）
    /// content: 已格式化好的 SKILL.md 完整内容（含 YAML frontmatter）
    fn sync_l2(&self, slug_hash: &str, content: &str) -> Result<(), AdapterError>;

    /// 清理孤儿目录
    /// active_slugs: 本次同步中所有有效的 slug-hash 字符串列表
    /// 将删除目标目录下所有 crossbrain-* 前缀但不在 active_slugs 中的目录
    fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError>;

    /// 将 AdapterError 转换为用户友好的提示文字
    /// 所有 UI 展示的错误文本必须经过此函数，不能直接展示系统错误
    fn user_friendly_error(&self, err: &AdapterError) -> String {
        match err {
            AdapterError::NotInstalled => "未检测到此工具，已跳过".to_string(),
            AdapterError::PermissionDenied(_) => "没有写入权限，请检查目录访问权限".to_string(),
            AdapterError::DirectoryCreateFailed(_) => "无法创建配置目录，请手动创建后重试".to_string(),
            AdapterError::WriteError(_) => "文件写入失败，请检查磁盘空间和权限".to_string(),
            AdapterError::Other(msg) => format!("同步出现问题：{}", msg),
        }
    }
}
```

### ✅ 验收检查清单

```
[ ] src-tauri/src/adapters/ 目录存在
[ ] src-tauri/src/adapters/mod.rs 存在
[ ] mod.rs 中包含 CleanupReport 结构体定义
[ ] mod.rs 中包含 AdapterError 枚举定义（5 个变体）
[ ] mod.rs 中包含 Adapter trait 定义（4 个方法 + 1 个默认实现方法）
[ ] mod.rs 中 pub mod antigravity 和 pub mod claude_code 声明存在
[ ] antigravity.rs / claude_code.rs 占位文件存在（否则上述声明会导致编译失败）
[ ] lib.rs 中已挂载 pub mod adapters;（否则本模块不参与编译，前 6 条验收为空转）
[ ] cargo build 编译通过（在 src-tauri 目录下执行）
[ ] cargo test 通过（含 5 个接口契约测试：object-safe、Send+Sync、幂等可重复、报告可比对、错误文案全覆盖）
```

### 完成后必须做

1. 将 `CURRENT_STATUS.md` 中 `TASK-04` 改为 ✅
2. 在 `CURRENT_STATUS.md` 的「本次变更记录」追加一条，时间精确到时分秒
3. `cargo build` 与 `cargo test` **双通过**后方可标记完成

### ✅ 实施记录（2026-09-14 21:38）

- **交付文件**
  - `src-tauri/src/adapters/mod.rs` — 接口契约（`Adapter` trait + `CleanupReport` + `AdapterError`）+ 5 个契约测试
  - `src-tauri/src/adapters/antigravity.rs` / `claude_code.rs` — 占位模块
  - `src-tauri/src/lib.rs` — 挂载 `pub mod adapters;`
- **实测结果**：`cargo build` 退出码 0 / 0 error（产物 `crossbrain.exe` 12.88 MB）；
  `cargo test` 6 passed / 0 failed（5 个新增 + TASK-03 幂等测试）
- **遗留 warning**：3 个 `never used`（`agents_md_file` / `gemini_config_dir` / `claude_dir`），
  均为 TASK-05 / TASK-06 即将使用的预留路径函数，属**预期**，不要为了消 warning 而删除

---

## TASK-05：实现 AntigravityAdapter

### 前置条件

- TASK-04 已完成（✅）
- **必须先读** `docs/tech/ADAPTER_SPEC.md` 第 2 节（Antigravity Adapter 完整规范）

### 重要规则（违反即重做）

- L0 写入策略：**直接全量覆盖**，不用标记块
- L2 写入必须包含 YAML frontmatter（`name` + `description` 两个字段）
- 孤儿清理只删除 `crossbrain-` 前缀的目录

### ⚠️ 实施修正（2026-09-14，TASK-05 实测定稿）

原文档的实现**不能直接照抄**，实测暴露四个必须处理的问题：

**1. 测试绝不能在真实目录上跑（数据安全问题）**

原验收要求「手动测试 `cleanup_orphans` 的删除行为」。但实测本机
`~/.gemini/config/skills/` 下**真实存在**用户自建的 `grill-me`、
`design-taste-frontend`，以及 Spike 遗留的 `crossbrain-spike-test`。
在真实路径上跑删除测试 = 拿用户数据做实验。

→ 因此 `AntigravityAdapter` 增加了 `base_dir` 注入：

```rust
AntigravityAdapter::new()                 // 生产：真实 ~/.gemini/config/
AntigravityAdapter::with_base_dir(temp)   // 测试：重定向到临时目录
```

**所有单元测试一律走 `with_base_dir`，无任何例外。**

**2. `sync_l2` 必须校验 slug 的命名空间（否则会覆盖用户技能）**

`slug_hash` 直接参与目录名拼接。若不校验，传入 `grill-me` 就会
**覆盖用户的技能文件**；传入 `crossbrain-../../x` 会写到 `skills/` 之外。

→ 新增 `ensure_crossbrain_slug()`：必须以 `crossbrain-` 开头，
且不含 `/`、`\`、`..`。这与孤儿清理「只删 `crossbrain-` 前缀」是
**同一条边界的两侧**——读、写、删三个动作都应限定在同一命名空间内。

**3. `cleanup_orphans` 必须跳过非目录项**

`fs::remove_dir_all` 对同名的**普通文件**会直接失败。→ 先用
`file_type().is_dir()` 判定，不是目录就跳过。**顺带解决符号链接**：
`file_type()` 对 symlink 返回的 `is_dir()` 为 `false`，
因此不会顺着链接误删到目标目录。

**4. frontmatter 的拼装此前没有任何任务覆盖**

原文档只说「由调用方负责格式化」，但那个「调用方」在任务拆解里并不存在，
`sync_l2` 的验收（「文件包含 frontmatter」）因此无法在真实链路中达成。

→ 在 `adapters/mod.rs` 补 `format_skill_md()`。放在共享层是因为
`ADAPTER_SPEC.md` 第 4 节标题明写「两个 Adapter 通用」，
放在任一 Adapter 内都会导致另一边复制一份。

### 执行步骤

**步骤 1**：创建 `src-tauri/src/adapters/antigravity.rs`

```rust
//! antigravity.rs — Antigravity IDE (Google Gemini) Adapter

use std::fs;
use std::path::PathBuf;
use super::{Adapter, AdapterError, CleanupReport};
use crate::paths::gemini_config_dir;

pub struct AntigravityAdapter;

impl AntigravityAdapter {
    /// Antigravity 规则目录：~/.gemini/config/rules/
    fn rules_dir(&self) -> PathBuf {
        gemini_config_dir().join("rules")
    }

    /// L0 规则文件完整路径：~/.gemini/config/rules/crossbrain-L0.md
    fn l0_file(&self) -> PathBuf {
        self.rules_dir().join("crossbrain-L0.md")
    }

    /// 技能目录根路径：~/.gemini/config/skills/
    fn skills_dir(&self) -> PathBuf {
        gemini_config_dir().join("skills")
    }

    /// 某个技能的 SKILL.md 完整路径
    fn skill_file(&self, slug_hash: &str) -> PathBuf {
        self.skills_dir().join(slug_hash).join("SKILL.md")
    }
}

impl Adapter for AntigravityAdapter {
    fn detect(&self) -> Result<bool, AdapterError> {
        Ok(gemini_config_dir().exists())
    }

    fn sync_l0(&self, rules_content: &str) -> Result<(), AdapterError> {
        // 自动创建 rules/ 目录（若不存在）
        fs::create_dir_all(self.rules_dir())
            .map_err(|e| AdapterError::DirectoryCreateFailed(e.to_string()))?;

        // 直接全量覆盖（此文件 100% 由 CrossBrain 托管）
        fs::write(self.l0_file(), rules_content)
            .map_err(|e| AdapterError::WriteError(e.to_string()))
    }

    fn sync_l2(&self, slug_hash: &str, content: &str) -> Result<(), AdapterError> {
        let skill_dir = self.skills_dir().join(slug_hash);

        // 自动创建技能目录（若不存在）
        fs::create_dir_all(&skill_dir)
            .map_err(|e| AdapterError::DirectoryCreateFailed(e.to_string()))?;

        // 写入 SKILL.md（content 已包含 YAML frontmatter，由调用方负责格式化）
        fs::write(skill_dir.join("SKILL.md"), content)
            .map_err(|e| AdapterError::WriteError(e.to_string()))
    }

    fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError> {
        let skills_dir = self.skills_dir();
        let mut report = CleanupReport {
            deleted_dirs: vec![],
            kept_dirs: vec![],
        };

        // 如果 skills 目录不存在，直接返回（无需清理）
        if !skills_dir.exists() {
            return Ok(report);
        }

        for entry in fs::read_dir(&skills_dir)
            .map_err(|e| AdapterError::Other(e.to_string()))?
        {
            let entry = entry.map_err(|e| AdapterError::Other(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();

            // 只处理 crossbrain- 前缀的目录，其他目录一律不动
            if !name.starts_with("crossbrain-") {
                continue;
            }

            if active_slugs.contains(&name) {
                report.kept_dirs.push(name);
            } else {
                // 孤儿目录，删除
                fs::remove_dir_all(entry.path())
                    .map_err(|e| AdapterError::Other(format!("删除孤儿目录 {} 失败：{}", name, e)))?;
                report.deleted_dirs.push(name);
            }
        }

        Ok(report)
    }
}
```

### ✅ 验收检查清单

```
[ ] src-tauri/src/adapters/antigravity.rs 存在
[ ] cargo build 编译通过（0 error）
[ ] cargo test 通过（新增 13 个测试：9 个 antigravity + 4 个共享格式工具；全仓共 19 个）
[ ] sync_l0：rules/crossbrain-L0.md 存在，内容与传入 rules_content 完全一致
[ ] sync_l0：重复执行内容不变（幂等）
[ ] sync_l0：**同目录下用户既有的 user_global.md 一字未动**（数据安全回归）
[ ] sync_l2：skills/{slug_hash}/SKILL.md 存在，以 "---\nname:" 开头
[ ] sync_l2：非 crossbrain- 前缀的目标被拒绝写入（数据安全回归）
[ ] cleanup_orphans：孤儿被删、活跃保留、**用户自建目录未被删除**（数据安全回归）
[ ] cleanup_orphans：skills/ 不存在时静默返回空报告，不是报错
[ ] cleanup_orphans：同前缀的普通文件被跳过，而非报错或被删
[ ] detect()：配置目录存在 → true；不存在 → false
[ ] **真实数据干跑**：`cargo run --example dryrun_antigravity` 通过
    （跑在真实目录的副本上，并验证真实目录本身零改动）
```

### 完成后必须做

1. 将 `CURRENT_STATUS.md` 中 `TASK-05` 改为 ✅
2. 在 `CURRENT_STATUS.md` 的「本次变更记录」追加一条，时间精确到时分秒
3. `cargo build` 与 `cargo test` **双通过**后方可标记完成
4. 若改动了读写或清理逻辑，**必须重跑** `cargo run --example dryrun_antigravity`

### ✅ 实施记录（2026-09-14 21:53）

- **交付文件**
  - `src-tauri/src/adapters/antigravity.rs` — 完整实现 + 9 个测试
  - `src-tauri/src/adapters/mod.rs` — 新增共享 `format_skill_md()` + 4 个测试
  - `src-tauri/examples/dryrun_antigravity.rs` — 真实数据安全干跑（开发期工具，不参与发布产物）
- **实测结果**：`cargo test` **19 passed / 0 failed**，0 error
  （TASK-04 的 6 个 + 本次新增 13 个）
- **真实数据干跑结论**：在真实目录副本上执行 `cleanup_orphans(&[])`（最激进场景）后，
  用户的 `design-taste-frontend`、`grill-me` 与 `rules/user_global.md` **全部完好**；
  真实 `~/.gemini/config` 全程**零改动**（目录清单逐一比对一致）。
- **遗留 warning**：2 个 `never used`（`agents_md_file` / `claude_dir`），
  分别由 TASK-06 与 TASK-09 使用，属**预期**。
- **⚠️ 交给 TASK-06 的关键情报（必读）**：本机 `~/.claude/CLAUDE.md` 通过**硬链接**
  与 `~/.gemini/config/rules/user_global.md`、`~/.cursor/rules/user_profile.md`、
  `~/.config/opencode/AGENTS.md`、`~/.ai-memory/user_profile.md` **共享同一个 inode**
  （`fsutil hardlink list` 已验证，links=5）。

  这意味着 TASK-06 的两种写法各有代价：
  - **就地覆写**（`fs::write` 原路径）→ 会**同时改变全部 5 个工具**看到的规则
  - **原子替换**（临时文件 + rename）→ 会**打断硬链接**，用户的「一处改处处生效」失效

  **必须在实现前向用户确认**（见 `CURRENT_STATUS.md` D-03）。

---

## TASK-06：实现 ClaudeCodeAdapter

### 前置条件

- TASK-04 已完成（✅）
- **必须先读** `docs/tech/ADAPTER_SPEC.md` 第 3 节（ClaudeCode Adapter 完整规范，含四情况协议）

### 重要规则（违反即重做）

⚠️ 这是整个项目最复杂的业务逻辑，必须精确实现四情况协议，不能简化。

- **绝对禁止**：用新内容覆盖整个 CLAUDE.md 文件
- **必须实现**：标记块注入（情况一~四）和备份机制
- 标记块格式固定，不能改变任何一个字符

### ⚠️ 实施修正（2026-09-14，TASK-06 实测定稿）

原文档的实现**不能直接照抄**，实测暴露五个必须处理的问题：

**1. 🔴 所有对 `CLAUDE.md` 的写入必须「断开硬链接」（决策 D-03，本任务最重要的一条）**

本机 `~/.claude/CLAUDE.md` 与另外 4 个工具路径是**硬链接共享同一 inode**
（`links=5`，`fsutil hardlink list` 已验证）：

```
~/.ai-memory/user_profile.md            ← 原始
~/.cursor/rules/user_profile.md
~/.config/opencode/AGENTS.md
~/.gemini/config/rules/user_global.md
~/.claude/CLAUDE.md                     ← 本 Adapter 的目标
```

原文档写的 `fs::write(&claude_md, new_content)` 是**就地覆写**——在硬链接下会
**同时改写全部 5 个工具读到的规则**，把 Claude 专有的 `@~/.ai-profile/AGENTS.md`
语法污染到 Cursor / OpenCode / Antigravity。

→ 用户已拍板（`CURRENT_STATUS.md` D-03）：**断开硬链接 / 原子替换**。
所有写入（情况一~四，**无一例外**）一律走 `write_breaking_hardlink()`：

```rust
// ① 内容先落盘（失败也不丢数据）
fs::write(&tmp, content)?;       // tmp = CLAUDE.md.crossbrain-tmp
// ② 删除原路径 —— Windows 上这一步才真正打断硬链接
fs::remove_file(path)?;          // 文件本不存在时 NotFound 视为正常
// ③ 同卷原子替换（Windows 的 rename 不覆盖已存在目标，故 ② 不可省）
fs::rename(&tmp, path)?;
```

**2. 测试绝不能在真实目录上跑**

真实 `~/.claude/CLAUDE.md` 既可能与 4 个工具共享 inode，也可能装着用户写了很久的规则。
`sync_l0` 是真实写入 → 在真实路径上跑测试 = 拿用户数据做实验。

→ 与 `AntigravityAdapter` 一致，增加 `base_dir` 注入：

```rust
ClaudeCodeAdapter::new()                 // 生产：真实 ~/.claude/
ClaudeCodeAdapter::with_base_dir(temp)   // 测试：重定向到临时目录
```

**所有单元测试一律走 `with_base_dir`，无任何例外。**

**3. `std` 的 inode API 是 unstable，验证改用「行为验证」**

`MetadataExt::file_index()` / `number_of_links()` 仍属 unstable
（feature `windows_by_handle`），稳定通道编译不过（`error[E0658]`）。

→ 用行为验证替代，而且**比读 inode 更硬**——它直接证明我们真正要防的后果：

```rust
// 前置：向 a 就地覆写，若 b 内容跟着变 → 两者确实同 inode（否则测试毫无意义）
// 后置：断链后向 a 就地覆写，b 必须纹丝不动
```

**4. 情况二的判定必须与替换用同一个正则**

原文档代码用 `content.contains(START) && content.contains(END)` 判定：
在「只有 Start 没有 End」或标志乱序时**也判真**，于是进了情况二分支却匹配不到、
替换静默失败——文件该改而没改，还返回成功。

→ 改用 `marker_regex().is_match()`，与替换共用同一正则，保证「进分支即替换成功」。
替换时用 `regex::NoExpand`，避免标记块文本里的 `$1` 之类被当作捕获组展开。

**5. Plan B（`@~` 展开失败降级）V1 不实现**

Spike A 已实测 Windows 下 `@~` 引用可用，触发条件不存在。
`sync_l0_with_result` 保留 `rules_content` 参数作为 Plan B 入口，
但**不造「永远触发不到、也测不到」的死代码**。

**6. 附带重构：孤儿清理与命名空间校验提升到 `adapters/mod.rs` 共享**

TASK-08 要求「两个 Adapter 的清理逻辑行为一致」。靠两份各自演进的代码
「保持」一致不可靠；共用 `cleanup_crossbrain_orphans()` / `ensure_crossbrain_slug()`
才是结构上不会分叉。Antigravity 侧原有的 13 个测试重跑通过，确认重构无回归。

### 标记块格式（必须完全一致，包括换行）

```
<!-- CrossBrain:Start -->
<!-- 本区域由 CrossBrain 自动维护，修改请在 CrossBrain App 中进行 -->
@~/.ai-profile/AGENTS.md
<!-- CrossBrain:End -->
```

### 四情况协议

| 情况 | 条件 | 操作 | UI 提示 |
|:---|:---|:---|:---|
| 一 | CLAUDE.md 不存在 | 直接创建含标记块的文件 | 无 |
| 二 | 存在 + 已有标记块 | 正则替换标记块内容 | 无 |
| 三 | 存在 + 无标记块 + 无备份 | 先备份 → 末尾追加标记块 | 弹提示（内容见下文） |
| 四 | 存在 + 无标记块 + 有备份 | 末尾追加标记块（跳过备份） | 无 |

**情况三 UI 提示文字（必须完全一致）**：  
「已将 CrossBrain 规则追加到你的 Claude 配置末尾。原文件已备份至 CLAUDE.md.crossbrain-backup，如需恢复请查看此文件。」

### 执行步骤

创建 `src-tauri/src/adapters/claude_code.rs`：

```rust
//! claude_code.rs — Claude Code CLI Adapter

use std::fs;
use std::path::PathBuf;
use regex::Regex;
use super::{Adapter, AdapterError, CleanupReport};
use crate::paths::claude_dir;

// 标记块的起始和结束标志（不可修改）
const MARKER_START: &str = "<!-- CrossBrain:Start -->";
const MARKER_END: &str = "<!-- CrossBrain:End -->";

pub struct ClaudeCodeAdapter;

/// sync_l0 操作的返回信息（用于通知 UI 是否需要弹提示）
pub enum SyncL0Result {
    /// 情况一/二/四：静默完成
    Silent,
    /// 情况三：已备份，需要弹提示
    BackupCreated { backup_path: String },
}

impl ClaudeCodeAdapter {
    fn claude_md_file(&self) -> PathBuf {
        claude_dir().join("CLAUDE.md")
    }

    fn backup_file(&self) -> PathBuf {
        claude_dir().join("CLAUDE.md.crossbrain-backup")
    }

    fn skills_dir(&self) -> PathBuf {
        claude_dir().join("skills")
    }

    /// 生成标记块内容（每次重新生成，保持格式一致）
    fn build_marker_block(&self) -> String {
        format!(
            "{}\n<!-- 本区域由 CrossBrain 自动维护，修改请在 CrossBrain App 中进行 -->\n@~/.ai-profile/AGENTS.md\n{}",
            MARKER_START, MARKER_END
        )
    }

    /// 检查文件内容是否已含标记块
    fn has_marker_block(&self, content: &str) -> bool {
        content.contains(MARKER_START) && content.contains(MARKER_END)
    }

    /// 执行四情况协议，返回是否需要弹 UI 提示
    pub fn sync_l0_with_result(&self, _rules_content: &str) -> Result<SyncL0Result, AdapterError> {
        let claude_md = self.claude_md_file();
        let marker_block = self.build_marker_block();

        // 情况一：文件不存在
        if !claude_md.exists() {
            fs::create_dir_all(claude_dir())
                .map_err(|e| AdapterError::DirectoryCreateFailed(e.to_string()))?;
            fs::write(&claude_md, &marker_block)
                .map_err(|e| AdapterError::WriteError(e.to_string()))?;
            return Ok(SyncL0Result::Silent);
        }

        // 读取现有文件内容
        let existing = fs::read_to_string(&claude_md)
            .map_err(|e| AdapterError::Other(format!("读取 CLAUDE.md 失败：{}", e)))?;

        // 情况二：文件存在且已有标记块 → 精准替换标记块内容
        if self.has_marker_block(&existing) {
            let re = Regex::new(
                r"<!-- CrossBrain:Start -->[\s\S]*?<!-- CrossBrain:End -->"
            ).unwrap();
            let new_content = re.replace(&existing, marker_block.as_str()).to_string();
            fs::write(&claude_md, new_content)
                .map_err(|e| AdapterError::WriteError(e.to_string()))?;
            return Ok(SyncL0Result::Silent);
        }

        // 情况三：文件存在，无标记块，无备份文件 → 先备份再追加
        if !self.backup_file().exists() {
            fs::copy(&claude_md, self.backup_file())
                .map_err(|e| AdapterError::WriteError(format!("备份 CLAUDE.md 失败：{}", e)))?;

            let new_content = format!("{}\n\n{}", existing.trim_end(), marker_block);
            fs::write(&claude_md, new_content)
                .map_err(|e| AdapterError::WriteError(e.to_string()))?;

            let backup_path = self.backup_file().to_string_lossy().to_string();
            return Ok(SyncL0Result::BackupCreated { backup_path });
        }

        // 情况四：文件存在，无标记块，备份文件已存在 → 直接追加，跳过备份
        let new_content = format!("{}\n\n{}", existing.trim_end(), marker_block);
        fs::write(&claude_md, new_content)
            .map_err(|e| AdapterError::WriteError(e.to_string()))?;

        Ok(SyncL0Result::Silent)
    }
}

impl Adapter for ClaudeCodeAdapter {
    fn detect(&self) -> Result<bool, AdapterError> {
        Ok(claude_dir().exists())
    }

    fn sync_l0(&self, rules_content: &str) -> Result<(), AdapterError> {
        self.sync_l0_with_result(rules_content)?;
        Ok(())
    }

    fn sync_l2(&self, slug_hash: &str, content: &str) -> Result<(), AdapterError> {
        let skill_dir = self.skills_dir().join(slug_hash);
        fs::create_dir_all(&skill_dir)
            .map_err(|e| AdapterError::DirectoryCreateFailed(e.to_string()))?;
        fs::write(skill_dir.join("SKILL.md"), content)
            .map_err(|e| AdapterError::WriteError(e.to_string()))
    }

    fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError> {
        let skills_dir = self.skills_dir();
        let mut report = CleanupReport {
            deleted_dirs: vec![],
            kept_dirs: vec![],
        };

        if !skills_dir.exists() {
            return Ok(report);
        }

        for entry in fs::read_dir(&skills_dir)
            .map_err(|e| AdapterError::Other(e.to_string()))?
        {
            let entry = entry.map_err(|e| AdapterError::Other(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();

            if !name.starts_with("crossbrain-") {
                continue;
            }

            if active_slugs.contains(&name) {
                report.kept_dirs.push(name);
            } else {
                fs::remove_dir_all(entry.path())
                    .map_err(|e| AdapterError::Other(format!("删除孤儿目录失败：{}", e)))?;
                report.deleted_dirs.push(name);
            }
        }

        Ok(report)
    }
}
```

**步骤 2**：在 `Cargo.toml` 中添加 regex 依赖

```toml
regex = "1"
```

### ✅ 验收检查清单

> ⚠️ 原清单要求「手动测试：先删除 ~/.claude/CLAUDE.md，再执行 sync_l0」——
> **不可执行且危险**：该文件与 4 个工具共享 inode，且装着用户真实规则。
> 已全部改为在**临时目录**中通过 `with_base_dir` 自动化验证，验收强度不降反升
> （手工点击无法覆盖「硬链接是否被断开」这类断言）。

```
[ ] src-tauri/src/adapters/claude_code.rs 存在
[ ] Cargo.toml 中包含 regex = "1"
[ ] cargo build 编译通过（0 error）
[ ] cargo test 通过（新增 19 个 claude_code 测试；全仓共 38 个）

[ ] 情况一（CLAUDE.md 不存在）：自动创建文件，内容逐字符等于固定标记块，返回 Silent
[ ] 情况一后重复执行进入情况二路径，结果字节级幂等
[ ] 情况二（已有标记块）：标记块被替换，块前/块后的用户内容原封不动，只存在一个标记块
[ ] 情况二重复执行字节级幂等
[ ] 情况三（无标记块、无备份）：备份内容 == 原始内容，原文保留，标记块追加末尾，返回 BackupCreated
[ ] 情况三：文件存在但为空时，产出不以空行开头的文件，且仍产生备份
[ ] 情况三：备份是**独立文件**（就地覆写原件不影响备份）
[ ] 情况四（无标记块、有备份）：旧备份未被覆盖，标记块追加，返回 Silent

[ ] 🔴 硬链接隔离（决策 D-03）：
    - 前置校验：先证明测试环境**确实**建立了共享 inode（否则测试是假通过）
    - 情况三路径写入后：其余 4 个工具路径内容**一字未变**
    - 情况二路径写入后：其余 4 个工具路径内容**一字未变**（覆盖面最大的路径）
    - 断链已生效：就地覆写 CLAUDE.md 不再牵动兄弟路径
    - 写入过程不残留 .crossbrain-tmp 临时文件

[ ] sync_l2：skills/{slug_hash}/SKILL.md 存在，以 "---\nname:" 开头
[ ] sync_l2：非 crossbrain- 前缀的目标被拒绝写入（数据安全回归）
[ ] cleanup_orphans：孤儿被删、活跃保留、用户自建目录未被删除（数据安全回归）
[ ] cleanup_orphans：skills/ 不存在时静默返回空报告，不是报错
[ ] cleanup_orphans：同前缀的普通文件被跳过，而非报错或被删
[ ] detect()：配置目录存在 → true；不存在 → false
[ ] trait 的 sync_l0 与 sync_l0_with_result 走同一条路径（副作用一致）
[ ] has_marker_block 与正则替换的判定一致（不会出现「判为情况二却替换失败」）
[ ] **真实数据干跑**：`cargo run --example dryrun_claude` 通过
    （跑在真实 5 链接结构的副本上，并验证 5 个真实文件逐字节零改动）
```

### 完成后必须做

1. 将 `CURRENT_STATUS.md` 中 `TASK-06` 改为 ✅
2. 在 `CURRENT_STATUS.md` 的「本次变更记录」追加一条，时间精确到时分秒
3. `cargo build` 与 `cargo test` **双通过**后方可标记完成
4. 若改动了读写或清理逻辑，**必须重跑** `cargo run --example dryrun_claude`
   ——本任务的写入策略直接作用于用户的 5 个真实工具配置，干跑不是可选项

### ✅ 实施记录（2026-09-14 22:03）

- **交付文件**
  - `src-tauri/src/adapters/claude_code.rs` — 四情况协议完整实现 + 19 个测试
  - `src-tauri/src/adapters/mod.rs` — 新增共享 `SLUG_PREFIX` / `ensure_crossbrain_slug()` /
    `cleanup_crossbrain_orphans()`
  - `src-tauri/src/adapters/antigravity.rs` — 改用上述共享实现
  - `src-tauri/examples/dryrun_claude.rs` — 真实 5 链接结构副本干跑（开发期工具）
  - `src-tauri/Cargo.toml` — 新增 `regex = "1"`
- **实测结果**：`cargo test` **38 passed / 0 failed**，0 error
  （TASK-05 的 19 个 + 本次新增 19 个）
- **真实数据干跑结论**：在真实 5 链接结构的副本上跑完情况三 → 二 → 四 + 孤儿清理，
  **其余 4 个工具路径全程一字未变**；真实 5 个文件 md5 与链接数（links=5）
  运行前后完全一致，**零改动**
- **遗留 warning**：1 个 `never used`（`agents_md_file`），由同步流程（AGENTS.md 生成）使用，属**预期**
- **⚠️ 交给后续任务的关键约束**：
  1. `CLAUDE.md` 的**唯一**写入通道是 `write_breaking_hardlink()`。
     TASK-14（一键卸载/去痕）要「从 CLAUDE.md 中移除标记块」，**同样必须走它**——
     否则卸载动作会把用户刚被隔离的文件又写回共享 inode。
  2. `agents_md_file()`（`~/.ai-profile/AGENTS.md`）目前无人使用：标记块只是**引用**它，
     真正生成 AGENTS.md 的职责在同步流程（TASK-13），届时会消耗掉这个 warning。

---

## TASK-07：实现 Slug-Hash 生成算法

### 背景

算法规范在 `docs/tech/ARCHITECTURE.md` 第 4 节。

### ⚠️ 实施修正（2026-09-14，实施时发现，务必先读）

本节步骤 2 的示例代码有 6 处需要修正。**规格**（ARCHITECTURE 第 4 节的
`crossbrain-{kebab}-{hash6}` 形态与三条不变性）不变，修正的是实现与表述：

1. **不做拼音转换**。ARCHITECTURE 第 4 节的示例把「组件性能优化」折算成 `perf`，
   那是**人工示意**而非算法产物。V1 不引入拼音词典（体积与维护成本都不划算），
   中文一律被折叠掉，因此 `Vue3 组件性能优化.md` 的实际输出是
   `crossbrain-vue3-{hash}` 而非 `crossbrain-vue3-perf-{hash}`。
   → 连带修正 `docs/testing/TEST_PLAN.md` T1-1 的「期望输出」表述。
   → 纯中文文件名（`组件性能优化.md`）的 kebab 段会退化为兜底值 `item`，
   区分度由 hash 段承担；**功能不受影响**，但目录名可读性下降（见下方遗留事项）。

2. **`slug[..30]` 是按字节切片，必须改成按字符**。原代码虽因「输出只含 ASCII」
   而暂时安全，但这是**脆弱的不变量**——一旦有人放宽字符集（比如日后允许拼音字母
   或 `_`），立刻变成 panic。已改为 `chars().take(30)`，并补 `strip_md_extension`
   的字符边界守卫（`"优化"` 是 6 字节，`len - 3 = 3` 恰好落在第二个汉字中间，
   直接切片会 panic —— 已加专门用例踩这个点）。

3. **空 kebab 段必须兜底**。原代码在文件名全为中文/符号时产出空 slug，
   目录名变成 `crossbrain--7a6883`（双横线）：合法但难看，且让「`-` 是分隔符」
   这一约定失效。已加 `FALLBACK_KEBAB = "item"`。

4. **`.md` 剥离要大小写不敏感，且只剥一次**。原 `trim_end_matches(".md")` 有两个坑：
   - 它是「反复去除」语义：`a.md.md` → `a`
   - 大小写敏感：`Note.MD` → kebab 段混入 `md`（产出 `note-md`）
   已改为字符边界安全的 `strip_md_extension()`，大小写不敏感。
   **刻意不认 `.markdown`**：知识库扫描规则是 `knowledge/*.md`，
   接受 `.markdown` 会为一批永远扫不到的文件生成 slug，属制造死路径。

5. **hash 用 SHA-256，不用 `DefaultHasher`**。进程内哈希的输出**不保证跨 Rust 版本稳定**，
   会导致升级工具链后全量技能目录被判为孤儿删除重建。内容寻址的标识符必须可复现。

6. **`SLUG_PREFIX` 的单一真实源放在 `slug.rs`，不是 `adapters/mod.rs`**。
   前缀是 slug 的形态本身；Adapter 侧的命名空间校验是**引用**它。
   已加两条契约测试把两侧钉住：`generate()` 的产出必须永远能通过
   `ensure_crossbrain_slug()`，且两侧常量必须相等——否则分叉的表现是
   **运行时** `sync_l2` 全部失败，而不是编译错误。

**另新增 `generate_parts() -> Slug`（文档未要求）**：因为同一个 slug-hash
在两个地方被使用且需要的形态**不同**——

| 用途 | 需要的形态 | `Slug` 取值 |
|:---|:---|:---|
| `sync_l2()` / `cleanup_orphans()` 的目录名 | 含命名空间前缀 | `dir_name()` → `crossbrain-vue3-7a6883` |
| `format_skill_md()` 写入 frontmatter 的 `name` | **不含**前缀 | `skill_name()` → `vue3` |

若只交付一个字符串，上层只能靠 `strip_suffix` / `split('-')` 反推各段，
而 kebab 段自身就含 `-`，切割位置无从判断。`generate()` 保留原签名（返回 `String`），
内部的同步流程应优先用 `generate_parts()`。

### 执行步骤

**步骤 1**：在 `Cargo.toml` 中添加 SHA256 依赖（先检查是否已有 sha2 或类似）

```toml
sha2 = "0.10"
```

**步骤 2**：创建 `src-tauri/src/slug.rs`

```rust
//! slug.rs — Slug-Hash 生成算法
//! 规则：同文件同内容 → 永远相同输出（幂等）

use sha2::{Sha256, Digest};

/// 从知识文件名和内容生成 crossbrain-{slug}-{hash} 格式的唯一标识符
///
/// # 参数
/// - filename: 知识文件名（如 "Vue3 组件性能优化.md"）
/// - content: 文件完整内容
///
/// # 返回
/// 格式为 "crossbrain-{slug}-{6位hash}" 的字符串
pub fn generate(filename: &str, content: &str) -> String {
    let slug = filename_to_slug(filename);
    let hash = content_hash(content);
    format!("crossbrain-{}-{}", slug, hash)
}

/// 将文件名转换为安全的 kebab-case slug
/// - 保留 ASCII 字母和数字，其余字符替换为 -
/// - 转换为小写
/// - 合并连续 - 为单个 -
/// - 去掉首尾 -
/// - 截断至最多 30 字符
fn filename_to_slug(filename: &str) -> String {
    // 去掉 .md 扩展名
    let name = filename.trim_end_matches(".md");

    // 只保留 ASCII 字母数字，其他全替换为 -
    let raw: String = name.chars().map(|c| {
        if c.is_ascii_alphanumeric() {
            c.to_ascii_lowercase()
        } else {
            '-'
        }
    }).collect();

    // 合并连续 -，去首尾 -
    let mut slug = String::new();
    let mut prev_dash = false;
    for c in raw.chars() {
        if c == '-' {
            if !prev_dash && !slug.is_empty() {
                slug.push('-');
            }
            prev_dash = true;
        } else {
            slug.push(c);
            prev_dash = false;
        }
    }
    let slug = slug.trim_end_matches('-').to_string();

    // 截断至 30 字符
    if slug.len() > 30 {
        slug[..30].trim_end_matches('-').to_string()
    } else {
        slug
    }
}

/// 计算文件内容的 SHA256，取前 6 位十六进制
fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    // 取前 3 字节 = 6 位十六进制
    format!("{:02x}{:02x}{:02x}", result[0], result[1], result[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_幂等性() {
        let slug1 = generate("Vue3 组件性能优化.md", "内容A");
        let slug2 = generate("Vue3 组件性能优化.md", "内容A");
        assert_eq!(slug1, slug2);
    }

    #[test]
    fn test_内容变更后hash变化() {
        let slug1 = generate("test.md", "内容A");
        let slug2 = generate("test.md", "内容B");
        assert_ne!(slug1, slug2);
    }

    #[test]
    fn test_格式合法() {
        let slug = generate("Vue3 · 虚拟列表.md", "内容");
        assert!(slug.starts_with("crossbrain-"));
        // 只含 a-z 0-9 和 -
        assert!(slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    }

    #[test]
    fn test_中文特殊字符安全() {
        let slug = generate("Rust · 生命周期与借用检查器.md", "内容");
        assert!(slug.starts_with("crossbrain-"));
        // 不含中文
        assert!(slug.is_ascii());
    }
}
```

### ✅ 验收检查清单（6/6 通过，2026-09-14）

| # | 验收项 | 验证方式 | 结果 |
|:--|:---|:---|:---|
| 1 | `src-tauri/src/slug.rs` 存在 | 文件系统 | ✅ |
| 2 | `Cargo.toml` 中包含 `sha2` | 已加 `sha2 = "0.10"` | ✅ |
| 3 | `cargo test` 全部测试通过 | **63 passed / 0 failed**（lib 56 + 集成 7） | ✅ |
| 4 | `generate("Vue3 组件性能优化.md", "test")` 以 `crossbrain-` 开头 | 单元测试 + 干跑脚本 | ✅ `crossbrain-vue3-9f86d0` |
| 5 | 同输入两次输出完全相同（幂等） | 单元测试（含 `Slug` 逐字段比对） | ✅ |
| 6 | 含中文的文件名输出全为 ASCII | 6 组真实中文/特殊字符文件名 | ✅ 仅含 `[a-z0-9-]` |

**超出清单的加固验证**（`cargo run --example dryrun_slug`，实际输出见交付记录）：

- **外部交叉验证**：hash 段与系统 `sha256sum` 的结果逐位一致
  （`sha256("test")` → `9f86d0`、`sha256("")` → `e3b0c4` 两个标准向量）。
  这证明算的是**标准 SHA-256**，而不是某个自洽的私有实现。
- **跨模块契约**：9 组极端文件名（含 `../../etc/passwd.md`、`a/b\c:d*e.md`、空串）
  的产出**全部**通过 `adapters::ensure_crossbrain_slug()` 校验。
- **三条不变性**（改名 / 改内容 / 都不改）逐条实测。

### ✅ 实施记录（2026-09-14）

- **交付文件**
  - `src-tauri/src/slug.rs` — 算法完整实现 + 18 个单元测试（含 6 条边界与 4 条跨模块契约）
  - `src-tauri/src/adapters/mod.rs` — `SLUG_PREFIX` 改为转出自 `crate::slug`（单一真实源）
  - `src-tauri/src/lib.rs` — 挂载 `pub mod slug;`
  - `src-tauri/Cargo.toml` — 新增 `sha2 = "0.10"`
  - `src-tauri/examples/dryrun_slug.rs` — 验收脚本（开发期工具，不参与发布产物）
- **实测结果**：`cargo test` **63 passed / 0 failed**，0 error
  （TASK-06 的 38 个 + 本次 slug 18 个 + 集成一致性 7 个）
- **遗留 warning**：1 个 `never used`（`agents_md_file`），由同步流程（AGENTS.md 生成）使用，属**预期**
- **⚠️ 已知局限（需产品侧决策，不阻塞验收）**：
  纯中文文件名的 kebab 段退化为 `item`，目录名形如 `crossbrain-item-7a6883`，
  用户看到目录名无法判断对应哪个知识点。**功能无影响**（内容不同则 hash 不同，
  不会互相覆盖），仅影响可读性与人工排障。
  三条改进路径（按成本递增）：
  1. 保持现状，在 UI 侧始终展示「文件名 → 目录名」映射（推荐：零成本且信息完整）
  2. 兜底时混入文件名 hash（如 `crossbrain-item-3f2a1b-7a6883`）——需改规格
  3. 引入拼音词典做真实转写——体积与维护成本高，V1 明确不做
- **⚠️ 交给后续任务的关键约束**：
  1. **同步流程（TASK-13）应使用 `generate_parts()`**，用 `slug.dir_name()`
     喂 `sync_l2` / `cleanup_orphans` 的 `active_slugs`，用 `slug.skill_name()`
     喂 `format_skill_md()` 的 `slug_prefix`。不要对完整标识符做字符串切割。
  2. **本机已存在 2 个 `crossbrain-spike-test` 孤儿目录**
     （`~/.claude/skills/` 与 `~/.gemini/config/skills/` 各一个，Spike 阶段遗留）。
     TASK-13 首次真实同步时它们会被 `cleanup_orphans()` 删除——这是**正确行为**
     （命名空间内、不在 active 列表），但需在 TASK-10 向导里告知用户，
     避免用户误以为程序删了「自己的东西」。

---

## TASK-08：实现孤儿清理逻辑（已集成在 Adapter 中）

> **说明**：孤儿清理逻辑已在 TASK-05 和 TASK-06 的 `cleanup_orphans()` 方法中实现。  
> 本任务是对集成测试的补充验证，确保两个 Adapter 的清理逻辑行为一致。

### ✅ 验收检查清单（4/4 通过，2026-09-14）

| # | 验收项 | 验证方式 | 结果 |
|:--|:---|:---|:---|
| 1 | `AntigravityAdapter.cleanup_orphans()` 经过独立测试 | TASK-05 的 4 个单元测试 | ✅ |
| 2 | `ClaudeCodeAdapter.cleanup_orphans()` 经过独立测试 | TASK-06 的 4 个单元测试 | ✅ |
| 3 | 两者均不误删非 `crossbrain-` 前缀目录 | `t8_both_never_touch_non_crossbrain_dirs`（双 Adapter 同时跑） | ✅ |
| 4 | skills 目录不存在时返回 `Ok` | `t8_both_return_ok_when_skills_dir_missing`（双 Adapter 同时跑） | ✅ |

> **原验收清单的缺口**：第 1、2 条要求「**分别**独立测试」，
> 但「分别测过」并不能推出「行为一致」——两份实现各自符合自己的预期，
> 仍可能在边界上错开。已补第 3~7 项为**同一组输入同时驱动两个 Adapter**，
> 直接断言报告与文件系统结果**逐项相等**。

### ✅ 实施修正（2026-09-14）

**原任务的定位需要调整**：它被列为「补充验证」，但验收方式只写了「分别测试」，
无法证明「两者行为一致」这个真正的命题。修正为**共用一份实现 + 一致性测试双保险**：

1. **结构性保证**（TASK-06 已完成）：清理逻辑抽到
   `adapters::cleanup_crossbrain_orphans()`，两个 Adapter 只是传入不同的 `skills_dir`。
   行为一致不是「对齐出来的」，而是共用同一份代码、**不可能分叉**。
2. **测试性锁定**（本任务新增）：`src-tauri/tests/adapter_consistency.rs`
   以**集成测试**形式（只能访问 `pub` API）把两个真实 Adapter 放进同一组用例，
   对同一组输入断言 `CleanupReport` 相等 + 清理后目录树相等。

### ✅ 实施记录（2026-09-14）

- **交付文件**：`src-tauri/tests/adapter_consistency.rs`（7 个用例）
- **用例设计**（每条都同时驱动两个 Adapter，并断言结果完全一致）：

  | 用例 | 覆盖点 |
  |:---|:---|
  | `t8_both_delete_orphans_and_keep_active` | 正常清理：删孤儿、留 active |
  | `t8_both_never_touch_non_crossbrain_dirs` | 用户自建技能（`grill-me` 等）不被碰，且不进报告 |
  | `t8_both_return_ok_when_skills_dir_missing` | 首次运行场景：目录缺失 → 空报告、不报错、**且不顺手创建该目录** |
  | `t8_both_skip_plain_files_with_matching_prefix` | 同名前缀的普通文件不删（`remove_dir_all` 对文件必失败） |
  | `t8_both_keep_everything_when_all_active` | 全 active → 一个都不删 |
  | `t8_both_report_is_deterministically_sorted` | 报告按名排序（`read_dir` 顺序不稳定，不能透传） |
  | `t8_trait_objects_share_identical_behaviour` | 经 `Box<dyn Adapter>` 统一调度时仍一致（TASK-13 的真实调用形态） |

- **实测结果**：`cargo test` **63 passed / 0 failed**（lib 56 + 集成 7）
- **真实数据安全**：所有用例跑在系统临时目录（`Drop` 自清理），
  实测真实 5 个硬链接目标 md5 与链接数（links=5）**零改动**，
  `~/.gemini/config/skills/` 与 `~/.claude/skills/` 的用户技能全部完好

---

## TASK-09：封装本地 Git 提交

> **⏸️ 暂缓执行（2026-09-14，用户决策）**
>
> 本机尚未为 `~/.ai-profile/` 建立 git 仓库，用户明确表示「先不急」。
> 跳过本任务**不影响后续任务的代码编写**，但有两点必须记住：
>
> 1. `sync.rs` 的同步流程末尾已预留调用位置（见该文件 `run_full_sync_with_progress`
>    的文档注释）。TASK-09 完成后把它接进去即可生效，**不需要再改编排结构**。
> 2. TASK-13 的验收项「触发完整同步流程（… + git commit）」在 TASK-09 完成前无法勾选。
>
> ⚠️ **开工前必须先与用户确认仓库结构**：`~/.ai-profile/` 是**用户的数据目录**，
> 在其中执行 `git init` 属于替用户做决定，且会永久改变该目录的性质。
> 另需注意 `PRD.md` 第 4 节把「本地版本历史」列为零网络依赖功能，
> 未装 git 时必须降级为「无历史模式」而不阻断同步——这一降级路径要一并实现。

### 执行步骤

创建 `src-tauri/src/git.rs`：

```rust
//! git.rs — 本地 git 提交封装
//! 注意：使用 std::process::Command 调用系统 git（V1 方案）
//! V2 将迁移至 git2 crate，消除对系统 git 的依赖

use std::process::Command;
use chrono::Local;
use crate::paths::profile_root;

/// 执行 git commit，提交 ~/.ai-profile/ 下的所有变更
/// 返回 Ok(()) 不论是否有内容提交（nothing to commit 视为成功）
pub fn commit_sync() -> Result<(), String> {
    let profile = profile_root();

    // 先检查 git 是否可用
    if !is_git_available() {
        // git 未安装，跳过，不报错
        return Ok(());
    }

    // git add -A
    let add_output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(&profile)
        .output()
        .map_err(|e| format!("git add 失败：{}", e))?;

    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr);
        return Err(format!("git add 失败：{}", stderr));
    }

    // 生成 ISO-8601 时间戳（使用 chrono，绝不调用 shell date）
    let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();
    let commit_msg = format!("sync: {}", timestamp);

    // git commit
    let commit_output = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .current_dir(&profile)
        .output()
        .map_err(|e| format!("git commit 失败：{}", e))?;

    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        // "nothing to commit" 是正常情况，视为成功
        if stderr.contains("nothing to commit") {
            return Ok(());
        }
        return Err(format!("git commit 失败：{}", stderr));
    }

    Ok(())
}

/// 检查 git 命令是否可用（即是否安装）
pub fn is_git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

### ✅ 验收检查清单

```
[ ] src-tauri/src/git.rs 存在
[ ] 使用了 chrono::Local 生成时间戳，代码中没有 shell date 相关调用
[ ] cargo build 编译通过
[ ] 手动测试：在 ~/.ai-profile/ 中有变更时，commit_sync() 执行后 git log 能看到新提交
[ ] 手动测试：在 ~/.ai-profile/ 中无变更时，commit_sync() 执行后不报错（nothing to commit 静默）
[ ] 手动测试：将 git 从 PATH 中移除，commit_sync() 返回 Ok(())（不 panic，不报错）
[ ] git log 中的 commit message 格式为 "sync: 2026-09-14T19:00:00+0800"（ISO-8601）
```

---

## TASK-10 ~ TASK-14：前端 UI 实现

> **前端任务详细交互规范，请读 `docs/product/PRD.md` 第 5~7 节。**  
> 以下为每个任务的验收标准，实现细节按 PRD 执行。

### TASK-10：首次启动 5 步向导

**触发条件**：应用首次启动时（`~/.ai-profile/global/rules.md` 存在但内容为空）

#### ⚠️ 实施修正（2026-09-14）

原任务描述只写了 UI 交互，以下 8 处是实际动手时必须补上的。
前 4 条属于**设计缺陷**（照原样实现会出真问题），后 4 条属于**范围界定**。

1. **本任务缺一个前置：整个 IPC 层没有任何任务交代过。**
   TASK-10~14 全部直接描述界面，却没人说「前端怎么调用 Rust 能力」——
   而 `adapters/` 与 `sync.rs` 都是普通模块，前端根本看不见。
   实际新增了三个模块，**没有它们 `invoke` 无一可用**：

   | 新增文件 | 职责 |
   |:---|:---|
   | `src-tauri/src/commands.rs` | 6 个 `#[tauri::command]`，前端唯一入口 |
   | `src-tauri/src/sync.rs` | 同步编排（读 SSOT → 逐工具 detect/sync_l0/sync_l2/cleanup） |
   | `src-tauri/src/state.rs` | 跨启动状态（见第 2 条） |
   | `src/api.ts` | 前端侧的类型与调用封装（命令名只在这一处出现） |

2. **「向导完成后不再显示」光看 `rules.md` 判断不出来。**
   原文把触发条件定为「`rules.md` 存在但内容为空」，但用户完全可以在步骤 3
   一个字都不写就点「跳过，直接进入」——此时 `rules.md` 仍为空，
   下次启动**再次**弹出向导，与验收项「向导完成后再次启动，不再显示向导」直接冲突。
   已增加状态文件 `~/.ai-profile/.crossbrain-state.json` 记录 `wizard_completed`；
   同时把「`rules.md` 已有非空白内容」也视为已用过（用户可能中途关掉窗口）。
   该文件也是 TASK-13 状态栏「上次同步时间」的持久化位置，一并建好了。

3. **「逐行实时显示」要求前端主线程不被阻塞。**
   若 `run_sync` 写成普通同步命令，文件 IO 会占住主线程——
   Vue **收得到事件也渲染不出来**，验收项会退化成「最后一次性出现」的假实时。
   已改为 `async fn` + `tauri::async_runtime::spawn_blocking`，
   进度经 `sync://progress` 事件逐个工具推送。

4. **同步必须严格区分「目录是空的」与「目录读不出来」。**
   `cleanup_orphans` 的语义是「删掉不在本次有效列表里的 `crossbrain-*` 目录」，
   因此**空列表等于删光用户全部技能目录**。
   如果 `knowledge/` 因占用或权限问题暂时读不出来，而代码把它当成「没有文件」，
   就会在数据不完整的状态下执行删除。
   已在 `sync.rs` 立下铁律：**任何 SSOT 读取失败一律整体中止同步，一个 Adapter 都不调用**，
   宁可让用户看到「同步失败」也不能带着残缺数据去删。

5. **原步骤未定义「规则为空时怎么办」。** 已定为：规则全为空白时**跳过 L0**——
   往用户的 `CLAUDE.md` 里追加一个空标记块毫无价值，却会白白触发一次
   备份文件创建 + 硬链接断开（两个不可逆的副作用）。

6. **向导步骤 2 的「推送路径可点击修改」V1 不做。**
   该能力属于 `ADAPTER_SPEC.md` 第 5 节 Level 3 / PRD 第 6.3 节设置页，
   不在本任务的验收清单里。向导中只把路径以等宽字体展示，
   **不放看起来可点、实际无效的入口**。

7. **窗口尺寸由 800×600 调整为 1024×720**（并加最小尺寸约束）：
   5 步向导 + 左右分栏的规则编辑区在 800 宽下会明显拥挤。

8. **不引入 vue-router。** V1 只有「向导」与「主工作台」两个顶层视图，
   主工作台内部是 3 个 Tab。为此引入路由库属于过度设计，用一个布尔状态切换即可，
   零依赖且少一层心智负担。（V2 若页面显著增多，应重新评估——见 ADR-13。）

#### ✅ 验收检查清单

> 每项都标明**验证方式**：能自动化的就不靠人眼（`MASTER_CONTEXT.md` 的验收原则）。
> 「自动」指有可重复执行的命令，任何人重跑都能得到同样结论。

| # | 验收项 | 验证方式 |
|:---|:---|:---|
| 1 | 首启显示向导、不显示主工作台 | 半自动：判定逻辑在 `commands::get_startup_state`；端到端需人工首启观察 |
| 2 | 步骤 2 的各工具检测结果准确 | **自动**：`cargo run --example dryrun_sync` 第 [5] 段 |
| 3 | 未安装的工具**不显示** ❌（用中性图标） | 人工核对：未安装走灰色减号 + 「未检测到该工具，本次不参与同步」 |
| 4 | 步骤 3 的 3 个模板点击后填充编辑器 | 人工 |
| 5 | 步骤 4 同步进度**逐行实时**显示 | **自动**：`dryrun_sync` 断言进度回调逐个工具触发（收到 2 条，而非 1 条汇总） |
| 6 | 步骤 4 完成摘要：「已将 N 条全局规则…同步至 X 个工具」 | **自动**：摘要由 `SyncReport` 的 `rulesSynced` / `skillCount` 算出，两者均有断言 |
| 7 | 步骤 5 提示词可一键复制到剪贴板 | 人工（剪贴板需真实交互） |
| 8 | 「跳过，直接进入」直接进主界面 | 人工 |
| 9 | 向导完成后再次启动不再显示 | **自动**：`state.rs` 往返测试覆盖写入与读回；端到端需人工重启确认 |
| 10 | 向导中不出现 L0 / L2 / SSOT / slug / Adapter | **自动**：`cargo test --test ipc_contract` 的 `wizard_copy_has_no_internal_terms` |

**TASK-10 新增的契约验收项**（原清单未覆盖，但缺了会静默失败）：

| # | 验收项 | 验证方式 |
|:---|:---|:---|
| 11 | 前端调用的每个命令都已在 Rust 侧注册 | **自动**：`ipc_contract::every_invoked_command_is_registered` |
| 12 | 同步进度事件名两侧完全一致 | **自动**：`ipc_contract::sync_progress_event_name_matches` |
| 13 | DTO 字段名两侧对齐（camelCase ↔ snake_case） | **自动**：`ipc_contract::dto_fields_are_mirrored_on_both_sides` |
| 14 | 同步全程不动真实用户数据 | **自动**：`dryrun_sync` 第 [8] 段（2885 项快照逐项比对） |

---

#### ✅ 实施记录（2026-09-14）

**交付文件**

| 文件 | 内容 |
|:---|:---|
| `src-tauri/src/commands.rs`（新） | 6 个 `#[tauri::command]`：`get_startup_state` / `detect_tools` / `read_global_rules` / `save_global_rules` / `complete_wizard` / `run_sync` |
| `src-tauri/src/sync.rs`（新） | 编排层 + DTO（`SyncReport` / `ToolSyncResult` / `ToolInfo`）+ 5 个单元测试 |
| `src-tauri/src/state.rs`（新） | 跨启动状态持久化 + 5 个单元测试 |
| `src-tauri/src/paths.rs`（改） | 新增 `STATE_FILE_NAME` 常量与 `state_file()` |
| `src-tauri/src/init.rs`（改） | `.gitignore` 由「写死内容」改为「幂等补齐条目」+ 4 个单元测试 |
| `src-tauri/src/lib.rs`（改） | 注册 6 个命令，移除脚手架遗留的 `greet` |
| `src-tauri/tests/ipc_contract.rs`（新） | 4 个跨语言契约测试 |
| `src-tauri/examples/dryrun_sync.rs`（新） | 真实结构副本干跑（开发期工具，不参与发布产物） |
| `src/api.ts`（新） | 前端类型定义 + 命令封装 + 剪贴板与错误文案兜底 |
| `src/views/WizardView.vue`（新） | 5 步向导 |
| `src/views/MainView.vue`（新） | 主工作台外壳（状态栏 + 立即同步 + 3 个 Tab 占位） |
| `src/App.vue`（改写） | 应用外壳：加载态 → 错误态 → 向导 / 主工作台 |
| `src-tauri/tauri.conf.json`（改） | 窗口 800×600 → 1024×720，加最小尺寸 |

**实测命令与结果**

| 命令 | 结果 |
|:---|:---|
| `cargo test` | **81 passed / 0 failed**（lib 70 + 集成一致性 7 + IPC 契约 4），0 error 0 warning |
| `cargo run --example dryrun_sync` | **20 项检查全绿**，含「真实目录 2885 项快照零改动」 |
| `pnpm build`（含 `vue-tsc --noEmit`） | **零报错**，产物 `dist/assets/index-*.js` 304 KB（gzip 97 KB） |
| 真实数据回归 | 5 个硬链接目标 md5 全等（`47645f60…`）、`CLAUDE.md` 链接数仍为 **5**、用户技能目录全部完好 |

**干跑的关键证据（`dryrun_sync` 第 [7] 段）**

在真实结构的副本上跑完整同步流程后：

- 两个 `crossbrain-spike-test` 孤儿目录（Spike 遗留）**被正确清理**（跑前记录存在性，跑后断言已消失）
- `agnes-scroll-world` / `huashu-design`（Claude Code 侧）与
  `design-taste-frontend` / `grill-me`（Antigravity 侧）**清单逐项一致，零改动**

> 这里刻意**没有**断言写死的技能名清单。最初版本硬编码了 4 个技能名并断言
> 「每个目录里 4 个都在」，结果真实机器上每个目录只装 2 个——断言本身是错的。
> 改为「跑前跑后清单一致」后，既适应任何用户的目录内容，也更贴近真正要保证的性质
> （清理只动 CrossBrain 自己的目录）。

**交给后续任务的约束**

1. **`CLAUDE.md` 的任何写入仍必须走 `write_breaking_hardlink()`**（D-03）。
   TASK-14 的「一键卸载」要移除标记块，同样是一次写入——**不能例外**，
   否则会把刚隔离出来的文件又写回共享 inode。
2. **`state.rs` 的状态文件不要加入版本历史**：`.gitignore` 已忽略它。
   它是本机状态，多机同步时会互相冲突。
3. **TASK-09 接入位置已预留**：`sync.rs` 的 `run_full_sync_with_progress` 文档注释
   写明了 git 提交应插入的位置，且明确「提交失败不影响同步结果」。
4. **TASK-11/12/14 只需替换 `MainView.vue` 里的 Tab 占位块**，
   状态栏与同步按钮已经是可用的。
5. **TASK-13 的「上次同步时间」已经有数据源**：`get_startup_state` 返回
   `lastSyncAt` / `lastSyncOk`，`run_sync` 会写入。TASK-13 只需补离线指示的细化。

### TASK-11：全局规则编辑器

#### ✅ 验收检查清单

```
[ ] 编辑区和预览区同时可见（左右分栏 或 切换 Tab 均可）
[ ] 保存后不自动触发同步（保存和同步是两个独立动作）
[ ] 保存成功后有提示（轻提示 Toast，不是弹窗）
[ ] 文件内容持久化到 ~/.ai-profile/global/rules.md
```

#### ⚠️ 实施修正（2026-09-15 00:55，TASK-11）

原文只有 4 行验收项，以下 10 处是动手时必须补上的。前 3 条是**安全/数据安全**，
中 4 条是**交互正确性**，后 3 条是**实现选型**。

1. **验收项自相矛盾，按主句实现**。「编辑区和预览区同时可见」与「切换 Tab 均可」
   不能并存（切 Tab 就不可能同时可见）。按主句做**左右分栏**。
   预览**只在主工作台做**：PRD 第 5 节步骤 3 没要求向导带预览，
   向导是 `max-w-3xl` 窄栏，硬塞分栏只会挤坏已验收的布局。

2. **安全缺口：预览渲染器 + `csp: null`**。原文没提「预览怎么渲染」，
   而 `tauri.conf.json` 目前 `csp: null`——若把用户内容里的 `<script>` /
   `onerror` 当真标签插进 DOM，脚本会在 webview 里执行，
   而 webview 拥有 `invoke` 权限，**等同本地文件任意读写**。
   选型 **markdown-it**（默认 `html: false`，原始 HTML 被转义成纯文本，已实测
   `<b>` → `&lt;b&gt;`），因此无需再叠 sanitizer；并用
   `ipc_contract::markdown_preview_must_escape_raw_html` 锁死「不许打开 html 开关」。
   选它而不是 `marked` + `dompurify` 的原因：后者是 2 个依赖还得记得叠 sanitizer，
   漏一处就是 XSS；前者 1 个依赖且默认安全。

3. **预览区链接必须禁止真的跳转**。webview 里导航会把整个应用带离界面。
   组件里用 `@click.capture` 拦截点击（`preventDefault`），
   样式上仍显示为链接但不改 `cursor`——预览不是浏览器。

4. **模板复用与覆盖风险**。三条规则模板 60 行字符串，向导与主工作台各写一份必然分叉
   → 抽成 `useRuleEditor.ts` 导出的 `RULE_TEMPLATES` 常量，两处共用。
   **主工作台里模板入口只在内容为空时显示**：模板一点就整篇替换，
   在向导里是「帮你起步」，在主工作台是**静默覆盖用户已保存的规则**（数据丢失）。
   用「空白才显示」从机制上消灭误触，而不是靠确认弹窗兜底。

5. **【用户反馈 BUG 修复，2026-09-15 11:35】`n-spin` 打断 flex 高度链，分栏高度失控**。
   症状（用户截图实测）：初始态看不到输入框下方边框，footnote 被溢出的面板盖住
   （只有中间缝隙露出「推送」两字）；多行输入后面板整体被顶出视口、顶部内容被盖、
   textarea 无法内部滚动（「滚动不上去」）。
   根因：`<n-spin class="flex-1 min-h-0">` 的 class 落在 `.n-spin-container` 上，
   但插槽外层 `.n-spin-content` 高度默认 `auto`——内部 `h-full` 的 grid
   全部解析失败，两窗格高度变成**随内容撑开**，flex 分配的高度形同虚设。
   修复：`:content-style="{ height: '100%' }"`（一行）。
   教训：**Naive UI 的包裹型组件（n-spin / n-card 等）会把 flex/grid 的高度链
   切断在自己的 DOM 层里**——给它们内部传 `h-full` 时必须显式把高度接下去
   （`content-style` / `content-class`）。修复后 CDP 实测：框高固定 364px、
   footnote 完整可见、40 行内容 textarea 内部滚动正常（scrollTop 可设置）、
   文档零溢出。

5. **Naive UI Tab 默认会卸载面板**。`n-tab-pane` 默认 `display-directive="if"`，
   切走 Tab 即卸载——「全局规则」里没保存的修改会**跟着丢**。
   三个面板全部改为 `display-directive="show:lazy"`（首次进入才渲染、之后常驻）。

6. **「未保存」按内容基线判断，不按「敲过键盘」**。`dirty = content !== baseline`
   （baseline = 与磁盘一致的内容）。用户改了又改回原样不算未保存；
   同时天然满足向导「没动过就不写盘」的老约束，比 `rulesTouched` 布尔标记更准确。

7. **读取失败必须禁用保存（读不出来 ≠ 空）**。原向导对读取失败是
   `catch {}` 后当作空内容继续——放行的话，用户会把一份空文件覆盖到自己的规则上，
   不可逆。改为：`baseline` 未建立前编辑与保存都停用，界面给出原因与重试入口。
   与 ADR-14「SSOT 读取失败即中止」同一条原则。

8. **Toast 需要 provider**。`useMessage()` 只在 `<n-message-provider>` 内部可用，
   该 provider 必须包在 `n-config-provider` 下**所有视图**外层——
   放进某个 `v-if` 分支会让另一分支的调用直接报错。

9. **编辑框用原生 `<textarea>` 而非 `n-input`**。分栏要求两个窗格共用固定高度、
   各自滚动；`n-input` 的 textarea 高度由其内部样式决定，要撑满 flex 窗格得和
   组件内部结构较劲。原生元素 + Tailwind 类完全可控，焦点态对齐项目令牌。

10. **保存入口 = 按钮 + Ctrl/Cmd+S**，快捷键绑在 textarea 上（而非 window）：
    组件常驻（见第 5 条）后 window 级监听在设置页也会触发；绑在编辑框上则天然限定作用域，
    并 `preventDefault` 掉浏览器默认的「存储网页」。

#### ✅ 实施记录（2026-09-15 00:55，TASK-11）

**交付文件**

| 文件 | 内容 |
|:---|:---|
| `src/composables/useRuleEditor.ts`（新） | 读写状态机（`content` / `baseline` / `dirty` / `load` / `save`）+ `RULE_TEMPLATES` + markdown-it 单例与 `previewHtml` |
| `src/components/RuleEditor.vue`（新） | 左右分栏编辑器：状态条 / 模板入口（空白时）/ 原生 textarea / 预览窗格 / Ctrl+S |
| `src/views/MainView.vue`（改） | 「全局规则」Tab 接入 `RuleEditor`，Toast 提示；三个 Tab 改 `show:lazy` |
| `src/views/WizardView.vue`（改） | 步骤 3 的内联模板数组改为引用 `RULE_TEMPLATES`（60 行字符串不再复制） |
| `src/App.vue`（改） | 加 `<n-message-provider>` 包住所有视图 |
| `src/style.css`（改） | 新增 `.md-preview` 排版（标题/列表/代码/引用/表格，色值全走令牌；v-html 内容不受 scoped 样式影响，必须放全局） |
| `package.json`（改） | 新增 `markdown-it 15.0.2` + `@types/markdown-it 14.2.0`（devDep） |
| `src-tauri/tests/ipc_contract.rs`（改） | 新增 2 条契约测试；`RuleEditor.vue` 加入界面术语检查清单 |
| `src-tauri/examples/dryrun_sync.rs`（改） | 干跑预状态自适应（见下方「连带修复」） |

**新增契约测试（3 条，全部反向验证过）**

| 测试 | 锁住的约束 |
|:---|:---|
| `rule_editor_saving_never_triggers_sync` | `useRuleEditor.ts` / `RuleEditor.vue` 里**不得出现** `runSync`，且保存链路确实接了 `saveGlobalRules`（防止前一条恒为真） |
| `markdown_preview_must_escape_raw_html` | markdown-it 实例存在，且源码里不得出现 `html: true`（空白归一后比对，`html:true` 也逃不掉） |
| `wizard_copy_has_no_internal_terms`（扩围） | `RuleEditor.vue` 加入面向用户文件清单 |

> 反向验证记录：故意把 `html: true` + 一处 `runSync` 塞回 `useRuleEditor.ts`、
> 把 `slug` 塞进 `RuleEditor.vue` 模板 → 三条测试**全部如期失败**；还原后复跑全绿。
> 这条习惯（TASK-19 起约定）继续保持。

**连带修复（同一轮发现，不属于 TASK-11 本体）**

`dryrun_sync` 第 [6]/[11] 节的两个断言假设「真实 CLAUDE.md 从未同步过」。
2026-09-15 00:18 用户真机同步过一次，假设永久失效 → 干跑出现 2 项假失败。
已改为**预状态自适应**：跑之前先探测副本是否已含标记块与备份，
据此走「首注分支」（备份 == 注入前的原始文件）或「更新分支」
（同步绝不改写既有备份，ADR-15 的真正不变式）。
顺带修了 `copy_codex_subset` 不复制 `AGENTS.md.crossbrain-backup` 的保真度缺口。

**真实机首测（用户 00:18 的真机同步，结果全部符合设计）**

| 检查 | 结果 |
|:---|:---|
| 断链范围 | `~/.claude/CLAUDE.md` **links=5 → 1**；其余 4 路仍共享同一 inode（links=4，md5 全等 `47645f60…`）——只断了 CLAUDE.md 侧（D-03）✅ |
| 用户内容 | CLAUDE.md 的 1908 字节原文（标记块外）完整保留；`.crossbrain-backup` 仍是原始内容 ✅ |
| Codex | `AGENTS.md` 0 字节 → 579 字节（标记块 + 内联全文），备份为 0 字节原始件 ✅ |
| L0 | `~/.gemini/config/rules/crossbrain-L0.md` 444 字节写入 ✅ |
| 快照 | 真实目录 2949 项跑前跑后逐项一致（干跑只碰副本）✅ |

**实测命令与结果（2026-09-15 00:55）**

| 命令 | 结果 |
|:---|:---|
| `pnpm build`（vue-tsc + vite） | 零报错；产物 477 KB（gzip 159 KB，较 TASK-19 +115 KB，来自 markdown-it） |
| `cargo test` | **120 passed / 0 failed / 0 warning**（较 TASK-19 的 118 + 2） |
| `cargo run --example dryrun_sync` | **0 项检查未通过**（含真实目录 2949 项零改动） |
| 真实数据回归 | 5 路硬链接组 md5 一致（`47645f60…`，links=4+1）、`rules.md` md5 `f0b10675…` 零改动 |

**交给后续任务的约束**

1. **每新增一个面向用户的界面文件，都要加进** `wizard_copy_has_no_internal_terms`
   的 `user_facing` 列表（本次已加 `RuleEditor.vue`）。
2. **`csp: null` 仍是隐患**：预览已转义，但收紧 CSP 属于纵深防御，建议独立小任务处理；
   任何「往预览区渲染用户/远端内容」的新功能都必须先过这条。
3. **切换 Tab 不拦截未保存修改**（`show:lazy` 保内容不丢，但用户可能不知道没保存）。
   「离开未保存提醒」可与 TASK-13/14 一并考虑。
4. **`n-message-provider` 必须保持在所有视图外层**——删掉它，`useMessage()` 的调用点
   会在运行时报错且 `pnpm build` 看不出来。
5. **TASK-12 的技能编辑器应直接复用** `useRuleEditor` 的基线/dirty 思路与
   `.md-preview` 样式；2000 字警告条的插入点是 `RuleEditor.vue` 的工具条区域。

### TASK-12：技能知识卡片列表

#### ✅ 验收检查清单

```
[x] 列表展示 ~/.ai-profile/knowledge/ 下所有 .md 文件（每个文件一张卡片）
    —— `list_knowledge_cards_for()` + `KnowledgeManager.vue` 卡片网格；
       目录不存在 = 空列表；单文件读取失败 = 显式报错（「读不出来 ≠ 没有」）
[x] 卡片显示：文件名（去掉 .md）+ 文件第一行作为摘要
    —— 另按 PRD §6.2 补充展示注入后的目录名（mono 小字）与字数/修改时间
[x] 新建技能时，编辑器默认填入 "# [技术/语言] · [具体场景]" 模板
    —— `create_knowledge_for()` 落盘 `NEW_KNOWLEDGE_TEMPLATE`，编辑器直接打开
[x] 内容超过 2000 字时，编辑器顶部出现黄色警告条（条内文字见 PRD §6.2）
    —— `RuleEditor.vue` 的 `lengthWarning` prop，n-alert type=warning，原文照录
[x] 删除技能后，对应的 knowledge/*.md 文件被删除
    —— `delete_knowledge_file_for()`；重复删除按幂等成功处理；
       工具内副本等下次「立即同步」清理，确认弹窗已明示此时差
[x] 文件保存到 ~/.ai-profile/knowledge/{slug_without_hash}.md
    —— 实施口径见下方修正 ①：文件名 = 标题折算 kebab 段 + `.md`
```

#### ⚠️ 实施修正（2026-09-15 09:48，TASK-12）

1. **验收项「保存到 `{slug_without_hash}.md`」含糊，按字面实现会出两处事故**：
   - 带 hash 段 → 内容一变文件名就得跟着变（hash 是内容寻址），「改一个字文件就改名」；
     定为**不含 hash**，kebab 段承担区分、hash 段只在注入目录名里。
   - 带 `crossbrain-` 前缀 → 文件名再折算时前缀会被原样保留，目录名叠成
     `crossbrain-crossbrain-…`。定为**不含前缀**。
   最终口径：**文件名 = 标题折算的 kebab 段 + `.md`**（复用 `slug::kebab_from_title`）。
   纯中文标题退化为 `item`，**按 `-2`、`-3` 递增去重**——同名覆盖用户内容不可逆
   （原文未提同名场景，这是必须补的防线）。
2. **PRD §6.2 要求卡片同时展示「源文件名」与「注入后的目录名」**（验收清单没写）。
   已实现：目录名以 mono 小字常驻卡片，ADR-11 的可读性补偿 + 排障入口。
3. **保存走 `write_breaking_hardlink()` 而不是 `fs::write`**：万一用户把某个知识文件
   硬链进了自己的记忆中心，原地覆盖会顺着链接改掉另一头的文件——与 L0 写入同一防线。
4. **前端传来的文件名必须校验**（`validated_knowledge_path`）：它直接拼进真实路径，
   `../`、反斜杠、绝对路径都能越出 `knowledge/` 目录读写任意文件。
   拒绝：空 / 含 `/`、`\`、`\0` / 含 `..` / 非 `.md` 扩展名。
5. **`useRuleEditor` 泛化**：状态机加 `EditorIO` 注入点（`load/save` 通道）与
   `openFile()` / `close()`；原「`load()` 只在首次真正读」的语义保留。
   `openFile()` 会**有意覆盖**未保存的内存现场，「离开编辑现场」的确认放视图层
   （`KnowledgeManager.vue` 的 `pendingDiscard`），数据层不掺交互。
6. **`RuleEditor.vue` props 化**（`templates` / `footnote` / `placeholder` /
   `lengthWarning`）供两个页面共用。坑：`withDefaults` 里数组/对象类型的默认值
   **必须是工厂函数**（`templates: () => RULE_TEMPLATES`），直接给常量 vue-tsc 报 TS2322。
7. **删除后不触发同步**（与 TASK-11「编辑≠同步」同源）：工具内副本要等下次
   「立即同步」按孤儿清理移除。**这个时差必须由 UI 明示**，否则用户会以为
   工具里的技能已经消失。已写进删除确认弹窗与成功 Toast。
8. **`dryrun_sync` 的快照范围扩大**：把 `~/.ai-profile/global/rules.md` 与
   `~/.ai-profile/knowledge/` 加进 watched——此前 SSOT 侧完全不在回归比对里，
   「真实目录零改动」对这一半是空话。
9. **列表的失败语义**：目录不存在 = 空列表（`scan_knowledge` 同款兜底）；
   单个文件读失败 = 整体报错而不是悄悄少一篇——少一篇会让用户误以为删过了。
10. **流程教训（再次踩中）**：同一条消息里对 `ipc_contract.rs` 并行发起两处 Edit，
    其中一处被静默丢弃——靠「新测试在变异下没变红」的反向验证才暴露。
    此条已在记忆库，但再次证明：**同文件多次编辑必须串行**。

#### ✅ 实施记录（2026-09-15 09:48，TASK-12）

**交付文件**

| 文件 | 内容 |
|:---|:---|
| `src-tauri/src/sync.rs` | `KnowledgeCard` DTO、`NEW_KNOWLEDGE_TEMPLATE`、知识库 CRUD 五组函数（生产 + `_for(dir)` 可注入变体）、`validated_knowledge_path()`；测试 +5（CRUD 回环 / 同名去重 / 空标题拒绝 / 穿越拦截 / 空目录与非 md 过滤） |
| `src-tauri/src/slug.rs` | `kebab_from_title()` 公开入口（复用 `filename_to_kebab`，保证文件名与注入目录名的 kebab 段同源） |
| `src-tauri/src/commands.rs` / `lib.rs` | `list_knowledge` / `read_knowledge` / `create_knowledge` / `save_knowledge` / `delete_knowledge` 五条命令并注册；删除返回面向用户的成功文案（含清理时机） |
| `src/api.ts` | `KnowledgeCard` 接口 + 五个封装函数 |
| `src/composables/useRuleEditor.ts` | `EditorIO` 注入点、`openFile()` / `close()`；全局规则走默认通道 |
| `src/components/RuleEditor.vue` | props 化（templates / footnote / placeholder / lengthWarning）；2000 字黄色警告条（PRD §6.2 原文）；字数显示 |
| `src/components/KnowledgeManager.vue`（新） | 卡片网格（标题/摘要/目录名/字数/时间）、新建弹窗（标题→文件名规则明示）、删除二次确认、未保存修改的离开确认、列表↔编辑视图切换 |
| `src/views/MainView.vue` | 「技能知识」Tab 从占位换成 `KnowledgeManager` |
| `src-tauri/tests/ipc_contract.rs` | DTO 字段对 +4（fileName/dirName/skillName/charCount）、命令参数名两侧比对、`knowledge_manager_never_triggers_sync`、`KnowledgeManager.vue` 加进界面术语检查列表 |

**实测验证**（均为本轮真实运行结果）

| 项 | 结果 |
|:---|:---|
| `cargo test` | **127 passed / 0 failed / 0 warning**（109 lib + 7 + 11 契约，较 TASK-11 的 120 +7） |
| `dryrun_sync`（新增 [12] 节，共 12 节） | **0 项检查未通过**；新建/去重/列表/保存/穿越拒绝/幂等删除全过 |
| `pnpm build` | 干净通过（487.57 kB / gzip 162.09 kB） |
| 反向验证 | 两条新文本断言在变异下**如期变红**（runSync 注入 / 参数名改动），还原后复绿 |
| 真实数据回归 | 4+1 硬链接组 md5 `47645f60…` 不变、`knowledge/` 保持 0 项、`rules.md` md5 `f0b10675…` 零改动 |

**给后续任务的约束**

1. 知识库的读写/删除入口只允许经 `sync.rs` 的 `_for` 函数族，文件名必须过
   `validated_knowledge_path`——任何绕过校验直接 `paths::knowledge_dir().join(前端输入)`
   的写法都是路径穿越漏洞。
2. `KnowledgeManager.vue` 已加入 `wizard_copy_has_no_internal_terms` 的检查列表。
3. 编辑器状态机再扩展时（如 TASK-13 的同步结果页复用预览），保持「保存不触发同步」
   的两条契约测试覆盖面。

### TASK-13：同步状态栏 + 立即同步

#### ✅ 验收检查清单

```
[x] 状态栏常驻在主界面顶部，内容格式：「上次同步：{时间}  ✅ Claude Code  ✅ Antigravity  」
[x] 无网络时，状态栏额外显示 🔴 离线模式
[x] 「立即同步」按钮点击后，触发完整同步流程（sync_l0 + sync_l2 + cleanup_orphans + git commit）
[x] 同步过程中按钮显示加载状态，不可重复点击
[x] 同步完成后，状态栏「上次同步」时间更新
[x] 同步失败时，显示用户友好提示（不显示系统错误码）
```

#### ⚠️ 实施修正（2026-09-15 09:58，TASK-13）

1. **「git commit」一项顺延（依赖 TASK-09）**：验收原文写「触发完整同步流程
   （sync_l0 + sync_l2 + cleanup_orphans + **git commit**）」，但 `sync.rs` 的
   架构注释明确 git `add/commit` 属于 **TASK-09（SSOT 版本化，已暂缓）** 的封装范围。
   本任务实际触发的流程是 sync_l0 + sync_l2 + cleanup_orphans（三者俱全，实测有效）。
   git 提交不往本任务硬塞——它需要 Git-in-PATH 探测与 PRD §8 的翻译条目
   （「未检测到 Git……」），是 TASK-09 的整体设计。与 TASK-18 Spike 阻塞的先例
   同一处理方式：记录顺延，不假完成。
2. **「🔴 离线模式」不用 Emoji**：PRD 示意里的 🔴 落到界面上是
   `lucide/wifi-off` 图标 + `text-error-500`（红色）——本项目铁律禁 Emoji 图标，
   PRD 的红点意图由「红色 + 图标」等价表达，文案补「同步不受影响」说明语义
   （本地同步不依赖网络，PRD 同节原文也是这么说的）。
3. **「点击可查看原因」是新实现，不是已有能力**：TASK-10 的外壳只放了一行
   不可点击的「上次同步未完全成功」。PRD §7 明确要求失败指示可点击查看原因，
   本任务补上：失败指示为 `<button>`（键盘可达），点击展开各失败工具的
   面向用户文案（来自 `ToolSyncResult.message`，天然不含系统错误码）。
4. **失败原因不跨启动持久化**：启动状态只存「是否成功」（`lastSyncOk`），
   不存失败详情。重启后点开失败指示会看到诚实提示
   （「失败原因只保留到关闭应用为止……」），而不是编造或留空白。
5. **成功判定改用报告权威字段**：原实现 `lines.every(l => l.status !== "failed")`
   是前端从进度事件自行拼凑——事件丢失或迟到会误判。改为 `report?.ok === true`
   （`SyncReport.ok` 由后端决定；`report` 为 null = 请求没走通，必然不算成功）。
6. **进度行末尾用报告覆盖一次**：`lines.value = report.tools`，防个别工具的
   进度事件迟到/丢失导致状态栏缺一行。
7. **契约测试的断言写法**：`status_bar_matches_prd_contract` 断言
   `report?.ok || report.ok`（可选链与普通访问都认），避免断言写死某种
   TypeScript 风格而在无语义重构时假红。
8. **离线探测用 `navigator.onLine`**：WebView2 支持该 API 与 online/offline 事件。
   它只反映「有没有网络接口」而非真实连通性，对「离线模式」的显示语义够用；
   真正的连通性检测留给 TASK-16（离线测试）再评估。

#### ✅ 实施记录（2026-09-15 09:58，TASK-13）

- `src/views/MainView.vue`：离线指示改红色（error-500 + wifi-off 图标）；
  失败指示改可点击 `<button>`，展开区为状态栏下方 `n-alert(warning)`，
  列出失败工具与原因；`doSync` 改用 `SyncReport` 权威字段判定成功并覆盖进度行；
  新一次同步开始时收起上一次的失败详情。
- `src-tauri/tests/ipc_contract.rs`：新增 `status_bar_matches_prd_contract`
  （5 条断言：离线指示存在 / 按钮 loading 绑定 / 防重入检查 / 可展开的失败
  指示 / report.ok 权威判定），已按约定**反向验证**（`:loading="false"` 变异
  → 如期失败 → 还原）。
- 状态栏本体（常驻顶部、时间格式、per-tool ✅/❌、「尚未同步」、加载态、
  toUserMessage 无错误码）在 TASK-10 已落地，本任务审计确认合格后未动。
- 验证：`cargo test` **128 passed / 0 failed / 0 warning**；`dryrun_sync` 12 节
  0 项未通过；`pnpm build` 通过；真实数据零改动（4+1 硬链接组、rules.md
  md5 `f0b10675…`、knowledge 0 项）。
- 遗留：① git commit 顺延 TASK-09（见修正 1）；② 失败原因跨启动持久化
  未做（需 state 扩展，价值有限，暂缓）；③ GUI 走查仍并入 TASK-15。

### TASK-14：设置页「一键卸载 / 去痕」

#### ✅ 验收检查清单

```
[x] 设置页中有「卸载 CrossBrain」或「去痕」按钮
[x] 点击后弹出二次确认弹窗
[x] 确认后执行：
    - 删除 ~/.gemini/config/rules/crossbrain-L0.md
    - 删除 ~/.gemini/config/skills/ 下所有 crossbrain-* 目录
    - 从 ~/.claude/CLAUDE.md 中移除 CrossBrain 标记块（标记块外内容保留）
    - 删除 ~/.claude/skills/ 下所有 crossbrain-* 目录
[x] 卸载完成后提示成功
[x] 用户原有 CLAUDE.md 内容（标记块外的部分）完整保留
```

#### ⚠️ 实施修正（2026-09-15 10:30，TASK-14）

1. **验收清单的覆盖面是早期口径，已扩到全部 3 个工具**：原文只列了 Gemini
   （Antigravity）与 Claude Code——那是 PRD 第 3 节只支持两个工具时写的。
   Codex（TASK-18 增补）同样注入标记块、同样有技能目录与备份，卸载必须一并
   清理，否则「去痕」名不副实。三工具各自实现 `Adapter::uninstall()`，
   编排层（`uninstall_all_for`）不分支。
2. **删备份的时机是安全属性，不是实现细节**：`.crossbrain-backup` 是用户
   唯一的「回到最初」手段，必须在标记块**成功移除之后**才允许删除
   （`delete_backup_files` 文档已写死这条调用前提）。顺序反了 = 移除失败时
   用户既丢了块内引用也没了备份。
3. **尾部空白归整为单个换行**：注入时 `{原文.trim_end()}\n\n{块}` 本来就丢过
   原文的尾部空白，移除时对称地 `trim_end()` 后补单个换行。字符内容零变化。
4. **「整个文件都是我们建的」特例**：移除后内容为空时——有备份 → 写回备份
   （原始件本就是空文件）；无备份 → 说明文件由注入的「文件不存在」分支创建，
   **删除整个文件**，否则留一个空文件不算去痕。
5. **用户数据明确不在清理范围**：`~/.ai-profile/` 下的 `global/rules.md` 与
   `knowledge/` 是用户自己的数据，卸载不碰；确认弹窗里明说了这一点。
   验收原文没提，属于必须向用户交代的边界。
6. **状态文件最后删、由命令层负责**：`.crossbrain-state.json` 是本机运行痕迹，
   卸载成功后删除 → 下次启动回退默认值 → 重新走向导。放在 `uninstall_all`
   **成功之后**（命令层执行）：半途失败时保留状态，用户重试续上（卸载幂等）。
   前端 `App.vue` 监听 `uninstalled` 事件重读本机状态，主工作台自然切回向导。
7. **单工具失败即中止**：报错文案说明「已完成的清理不会重复执行，修复后可直接
   重试」。卸载幂等，重试安全。
8. **反向验证抓到一条弱断言**：`uninstall_requires_confirmation_and_wiring_matches`
   原本只查 api.ts 是否包含子串 `uninstall_crossbrain`——变异成
   `uninstall_crossbrain_x` 后测试**照样通过**（子串仍命中）。已改为断言完整
   调用形态 `invoke<UninstallOutcome[]>("uninstall_crossbrain")`。
   **教训：命令名断言必须含收尾的 `")`，裸子串永远防不住改名变异。**
9. **干跑 [13] 的期望值必须对前置状态自适应**：[10] 节的还原已把副本 CLAUDE.md
   的标记块移除，[13] 不能假设「必有标记块」；Codex 的原始件是空文件，移除后
   期望是空串而非单个换行。与 TASK-11 修干跑的教训同源：断言先问
   「这个前提在前面的步骤之后还成立吗」。

#### ✅ 实施记录（2026-09-15 10:30，TASK-14）

- `adapters/mod.rs`：trait 新增 `uninstall()`（默认空实现）；共享层新增
  `remove_marker_block_from_file()`（与注入对称的移除，含空文件特例）、
  `delete_backup_files()`、`skill_cleanup_actions()`；测试 +3。
- `claude_code.rs` / `codex.rs`：`uninstall()` = 移标记块 → 删备份 → 清技能目录；
  `antigravity.rs`：删独立规则文件 → 清技能目录。各 +1 全周期测试
  （含「用户自建技能 / `skills/.system/` 不被误删」断言）。
- `state.rs`：`remove_state_file()`（+ 可注入变体 `_at`，与 `load_from` 同模式）；
  测试 +1（幂等 + 删除后回默认值）。
- `sync.rs`：`UninstallOutcome` DTO + `uninstall_all()` / `uninstall_all_for()`。
- `commands.rs`：`uninstall_crossbrain`（async + spawn_blocking，成功后删状态文件）；
  `lib.rs` 注册。
- 前端：`api.ts` 加 `UninstallOutcome` / `uninstallCrossbrain()`；
  `MainView.vue` 设置页加「卸载 CrossBrain」危险区 + 二次确认弹窗（写明清理
  范围与保留范围）+ 成功 Toast + `emit("uninstalled")`；`App.vue` 监听后重读
  启动状态切回向导。
- 契约测试：DTO 字段对 +3、`uninstall_requires_confirmation_and_wiring_matches`
  （两段式确认 / 命令名完整形态 / 状态文件删除在命令层），已反向验证。
- `dryrun_sync` 新增 [13] 节（放最后，有破坏性）：三工具卸载、标记块外内容
  保留、备份/L0/技能目录清空、幂等重跑，共 13 节。
- 验证：`cargo test` **136 passed / 0 failed / 0 warning**（+8）；
  `dryrun_sync` 13 节 0 项未通过（真实目录 2950 项零改动）；`pnpm build` 通过。
- 遗留：GUI 点击走查并入 TASK-15；卸载不删 `~/.ai-profile` 用户数据（设计使然，
  弹窗已说明）。

---

## TASK-15 ~ TASK-17：系统验证与打包

> **按 `docs/testing/TEST_PLAN.md` 执行，所有测试项打勾后才能打包。**

### TASK-15：端到端测试

执行 TEST_PLAN.md 的 **T9 端到端流程** 和 **T10 一键卸载**。

#### ✅ 实施记录（2026-09-15 11:12 完成）

**执行方式**：`tauri.conf.json` 临时加 `additionalBrowserArgs --remote-debugging-port=9223`
（跑完即还原）→ Playwright `connectOverCDP` 连上真实 WebView2 窗口自动化点击走查。
wry **不读** `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` 环境变量（读源码确认：wry 自己
拼 `additional_browser_args`，环境变量被 options 覆盖），必须走配置项。

**T9-1 全链路通过（含 AI 工具实际触发）**：
UI 新建技能「Vue3 · 虚拟列表优化」（探针词 `cb-e2e-probe-7391`）→ 保存落盘
`knowledge/vue3.md`（158 字，Toast/字数/预览全对）→ 「立即同步」→ 三工具
`crossbrain-vue3-aaee58` 全部落盘且含探针词 → **`codex debug prompt-input` 实证
Codex 模型可见输入的技能清单含 `- vue3: Vue3 · 虚拟列表优化 (file: r0/crossbrain-vue3-aaee58/SKILL.md)`**。

**T9-2 通过**：UI 删除（二次确认）→ 再同步 → 三工具 `crossbrain-vue3-aaee58`
全部清理，知识库恢复空。

**T10**：文件系统语义已由 `dryrun_sync` [13] 节在副本上覆盖（去痕 + 块外内容保留）；
真机验证覆盖「卸载按钮 → 确认弹窗文案 → 取消路径」全程。**真机确认卸载属删除性操作，
留待用户自行执行**（卸载后重新同步即可恢复，无不可逆损失——`~/.ai-profile` 用户数据不动）。

**积压 GUI 走查（TASK-11/12/13/19）全部完成**：
- TASK-11：编辑器渲染、预览、保存状态机 —— **发现并修复真实 BUG**（见实施修正）
- TASK-12：新建/标题弹窗/编辑/字数/删除二次确认 —— 全对
- TASK-13：状态栏同步后显示三工具结果行 + 上次同步时间；失败展开与离线红色化
  需异常/断网条件，归 T7（TASK-16）
- TASK-19：备份列表（Claude 1.9KB / Codex 0B 两条可还原）与还原弹窗渲染正常；
  真机还原会移除当前标记块，留待用户执行

**验证**：cargo test 136 passed / 0 failed；pnpm build 通过；真实数据零改动
（rules.md md5 `f0b10675…` 不变、`~/.ai-memory` 硬链接组 links=4、knowledge 0 项、
备份文件完好、无 crossbrain 残留）。

#### ⚠️ 实施修正

1. **【BUG 修复】MainView 从未调用 `ruleEditor.load()`**——「全局规则」页
   永远停在「未读过磁盘」状态：编辑器显示空占位符、基线为 null、保存按钮禁用，
   且界面上没有任何入口能触发 load()（错误态才显示重试按钮）。修复：
   `onMounted` 里补 `void ruleEditor.load()`（`load()` 自带幂等：有基线直接返回）。
   教训：`pnpm build` 与全部单测都发现不了这种「忘记接线」——只有真实窗口点开才暴露，
   GUI 走查不可省。
2. **Naive UI `n-tabs` 不渲染 `role="tab"`**：自动化选择器用 `.n-tabs-tab` + hasText。
3. **知识卡片 title = 文件名去 `.md`**（如 `vue3`），标题首行进 summary；
   删除按钮 aria-label 是「删除 vue3」——自动化按此定位。
4. **同窗口有两个 `textarea` 并存**（`show:lazy` 常驻）——定位编辑器必须带
   placeholder 约束，`locator("textarea")` 会 strict mode 冲突。
5. **预览 HTML 转义实测通过**：注入 `<b onclick>` + `<img onerror>` 探针，
   预览 DOM 无真实标签、`window.__pwned` 未执行——TASK-11 的安全红线在真实窗口复验。

### TASK-16：离线测试

执行 TEST_PLAN.md 的 **T7 离线模式**。

#### ✅ 实施记录（2026-09-15 11:24 完成）

**执行方式**：CDP `Network.emulateNetworkConditions(offline: true)` 做浏览器级
离线模拟（零风险、可逆）——`navigator.onLine` 变 false、`offline` 事件正常派发，
UI 离线态与真断网一致。不需要真禁网卡。

| 用例 | 结果 | 证据 |
|:---|:---|:---|
| T7-1 断网后本地同步正常 | ✅ | 模拟离线下点「立即同步」，三工具全部 ok、无网络相关报错 |
| T7-2 离线状态指示 | ✅ | 状态栏右上出现红色「离线模式（同步不受影响）」，同步按钮可点；恢复在线后指示自动消失 |
| T7-3 断网后 git 提交处理 | ⏸️ **不适用** | `~/.ai-profile` 尚无 git 仓库——SSOT 版本化是**暂缓的 TASK-09**。TASK-09 落地后需回补此用例 |

顺带覆盖（T8-2）：连续两次同步（第二次无内容变更）第二次静默成功无报错。
T8-1（git 未安装降级）、T8-3（commit 时间戳格式）同样依赖 TASK-09 的 git 化，随其回补。

**验证**：cargo test 136 passed / 0 failed；真实数据零改动（rules.md md5
`f0b10675…`、knowledge 0 项）；`tauri.conf.json` 调试参数已还原。

### TASK-17：生产构建

```powershell
pnpm tauri build
```

#### ✅ 验收检查清单

```
[ ] T9、T10 全部测试用例通过
[ ] T7 全部测试用例通过
[ ] pnpm tauri build 无 error 完成
[ ] src-tauri/target/release/bundle/msi/*.msi 文件存在
[ ] MSI 文件可以在 Windows 上正常安装
[ ] 安装后应用可以正常启动
```

---

## TASK-18：Codex CLI Adapter 接入（2026-09-14 增补）

> **增补任务，优先级高于 TASK-11。** 触发：用户在使用向导时发现本机装了大量 AI 工具
> 却只识别出 2 个，要求接入 Codex CLI。经本机预探测确认落点后立项，用户已拍板先做此项。

### 背景

`PRD.md` 第 3 节原先把 Codex CLI（当时误记为「ChatGPT IDE」）列在 V1.5 待 Spike。
预探测显示它的接入成本显著低于其余候选——目标文件是空的、目录结构与 Antigravity 同构、
技能机制与 Claude Code 一致——故提入 V1。

### 实测落点（2026-09-14 预探测，详见 `docs/testing/SPIKE_RESULTS.md`）

| 用途 | 路径 | 形态 |
|:---|:---|:---|
| 安装检测 | `~/.codex/` | 目录存在 |
| **L0 全局规则** | `~/.codex/AGENTS.md` | **单文件，实测 0 字节** |
| L2 技能知识 | `~/.codex/skills/{slug-hash}/SKILL.md` | 与 Claude Code 同构，YAML frontmatter |
| 孤儿清理 | `~/.codex/skills/` 下的 `crossbrain-*` | — |
| 备份 | `~/.codex/AGENTS.md.crossbrain-backup` | — |

#### ⛔ 严禁触碰的路径（写代码前先读这一段）

| 路径 | 是什么 | 为什么不能碰 |
|:---|:---|:---|
| `~/.codex/skills/.system/` | Codex **内置技能**（`imagegen` / `openai-docs` / `plugin-creator` / `review-agent` / `skill-creator` / `skill-installer`） | 删掉会破坏 Codex 本体功能 |
| `~/.codex/rules/default.rules` | **命令审批白名单**（`prefix_rule(... decision="allow")`），**不是规则文件** | 被误当作 L0 落点会写坏 Codex 的沙箱策略 |
| `~/.codex/memories/` | Codex 自己的记忆机制（`MEMORY.md` 等） | 与 CrossBrain 无关，属工具内部数据 |
| `~/.codex/config.toml` | Codex 主配置 | 同上 |

> 注：`cleanup_crossbrain_orphans()` 只处理 `SLUG_PREFIX` 前缀的目录，
> 所以 `.system` 因前缀不匹配被自动跳过——这是**结构性安全**，不依赖额外判断。

### 关键设计决策

#### ① L0 必须走标记块注入，不能写独立文件

Antigravity 的 `rules/` 是「整个目录下所有 `.md` 都被读取」，所以 CrossBrain 能写独立文件
`crossbrain-L0.md`，与用户文件物理隔离。

**Codex 不是这样**：它只读 `~/.codex/AGENTS.md` 这一个文件。若照搬 Antigravity 写
`crossbrain-L0.md`，该文件**永远不会被 Codex 读到**，同步会静默失效——
无报错、无效果，属最难排查的一类问题。

因此 Codex 必须与 Claude Code 共用**标记块协议**：只在标记块内改动，块外用户内容一字不动。

#### ② 标记块内容用「内联」，不用 `@~` 引用（与 Claude 的关键差异）

Claude 的标记块内容恒为一行 `@~/.ai-profile/AGENTS.md`。`@` 是 **Claude Code 专有**的
路径展开语法（`ADAPTER_SPEC.md` 已明确），Spike A 只验证了它在 Claude 中可用——
**在 Codex 中能否展开从未验证，且没有理由认为可以**。

故 Codex 的标记块**直接内联 `rules_content` 全文**：

```text
<!-- CrossBrain:Start -->
<!-- 本区域由 CrossBrain 自动维护，修改请在 CrossBrain App 中进行 -->
（global/rules.md 的完整内容）
<!-- CrossBrain:End -->
```

代价是规则内容在 Codex 侧各存一份（Claude 侧只有引用）。这个代价可以接受：
CrossBrain 的职责本就是「把 SSOT 分发到各工具」，分发意味着每个工具都拿到**自己能读懂**的形态。

> `sync_l0(rules_content)` 的签名在 Claude 侧被忽略（该参数是留给 Plan B 的入口），
> 在 Codex 侧则是主路径。**同一签名、两种语义**——实现时不要为了「统一」而删掉它。

#### ③ 内联必须防「标记串注入」，否则会损坏文件

内联内容来自用户的 `global/rules.md`。若用户在规则里写出了 `<!-- CrossBrain:End -->`
（哪怕只是举例说明），标记块正则（非贪婪 `[\s\S]*?`）会**在错误的位置提前闭合**，
下一次同步就会重写错误的区间。

处理：构造标记块前检查 `rules_content` 是否含 `MARKER_START` / `MARKER_END`，
含则**返回明确错误**并中止该工具的 L0 同步（按 ADR-14「失败可见」原则），
不得静默写入一个已损坏的文件。

#### ④ 共享层抽取：把四情况协议提到 `adapters/mod.rs`

Claude 与 Codex 的四情况控制流、备份逻辑、标记块正则、断链写入**逐字相同**。
两份实现各自演进必然分叉（与 TASK-08 抽 `cleanup_crossbrain_orphans` 同一条理由）。

| 项 | 处理 |
|:---|:---|
| `MARKER_START` / `MARKER_END` / `MARKER_NOTE` | 移到 `mod.rs`，**唯一定义** |
| `marker_regex()` | 移到 `mod.rs`，并**改为从常量拼接**（原先正则内硬编码了标记串，改常量不会同步正则） |
| `write_breaking_hardlink()` | 移到 `mod.rs` |
| `SyncL0Result` | 移到 `mod.rs`，`claude_code.rs` 以 `pub use` 转出，既有引用不变 |
| 四情况协议本体 | 抽为 `inject_marker_block(target_file, backup_file, marker_block)` |

两个 Adapter 只提供「目标文件路径 + 备份路径 + 标记块内容」三项差异。

### 执行步骤

1. `adapters/mod.rs`：迁入常量、`marker_regex()`、`write_breaking_hardlink()`、`SyncL0Result`，
   新增 `inject_marker_block()`
2. `adapters/claude_code.rs`：改为调用共享 `inject_marker_block()`，删除本地副本
3. 新建 `adapters/codex.rs`：`CodexAdapter` 实现 trait 四方法
4. `sync.rs` 的 `build_tools()` 注册第三个工具
5. `tests/ipc_contract.rs` 与 `tests/adapter_consistency.rs` 纳入 Codex
6. `examples/dryrun_sync.rs` 覆盖 Codex 路径

### ✅ 验收检查清单

```
[ ] src-tauri/src/adapters/codex.rs 存在，CodexAdapter 实现 Adapter trait
[ ] adapters/mod.rs 中 MARKER_START / MARKER_END 只有一处定义
[ ] marker_regex() 由常量拼接生成（不再是硬编码字符串字面量）
[ ] claude_code.rs 不再持有 write_breaking_hardlink 的副本
[ ] cargo test 全量通过，且 claude_code 既有测试无回归
[ ] 检测逻辑：~/.codex 存在 → true；不存在 → false（临时目录验证）

[ ] L0 情况一（AGENTS.md 不存在）→ 创建含标记块的文件，返回 Silent
[ ] L0 情况二（已有标记块）→ 只替换块内，块外原样，返回 Silent
[ ] L0 情况三（无标记块无备份）→ 先备份再追加，返回 BackupCreated
[ ] L0 情况四（无标记块有备份）→ 不覆盖旧备份，返回 Silent
[ ] L0 内联内容正确：标记块内含 rules_content 全文
[ ] L0 防注入：rules_content 含 <!-- CrossBrain:End --> 时返回 Err，原文件未被损坏
[ ] L0 幂等：同内容连续两次同步，文件字节完全一致
[ ] L2：写入 ~/.codex/skills/{slug}/SKILL.md，格式与 Claude 侧逐字一致
[ ] L2 命名空间校验：非 crossbrain- 前缀的 slug 被拒绝
[ ] 孤儿清理：crossbrain-* 孤儿被删，.system 与用户技能完好
[ ] 孤儿清理：skills 目录不存在时返回 Ok（不报错）
[ ] 真实数据零改动：5 个硬链接目标 md5 与 links=5 不变
[ ] 干跑：在真实结构副本上跑通三 Adapter 全流程
```

### 完成后必须做

1. `CURRENT_STATUS.md` 中 TASK-18 改为 ✅，并更新阶段表
2. `PRD.md` 第 3 节工具矩阵：Codex CLI 移入 V1，修正「ChatGPT IDE」命名
3. `SPIKE_RESULTS.md` 补正式 Spike 结果（写入后 Codex 是否真能读到）——
   ⚠️ **本轮只能完成代码与干跑，读取生效需用户配合开一次真实 Codex 会话验证**

---

### ⚠️ 实施修正（2026-09-14，TASK-18）

实现过程中发现的问题与对本文档的订正。**每一条都是原文与事实不符或未预料到的**。

#### 修正 1：`marker_regex()` 的正则是硬编码字面量，与常量重复

**原文怎么写**：只说「把 `marker_regex()` 移到 `mod.rs`」。

**为什么不对**：原实现里正则写作
`Regex::new(r"<!-- CrossBrain:Start -->[\s\S]*?<!-- CrossBrain:End -->")`——
**标志串被抄了第二遍**。这与本项目「单一真实源」的约定直接冲突，且失败方式最恶劣：
改了 `MARKER_START` 常量而正则没跟上时，**判定与替换会同时静默失效**，
没有任何编译期提示，表现为「同步成功但文件没变」。

**改成什么**：正则由常量拼接生成，并加 `regex::escape` 防元字符：

```rust
Regex::new(&format!(
    "{}[\\s\\S]*?{}",
    regex::escape(MARKER_START),
    regex::escape(MARKER_END)
))
```

并补测试 `marker_regex_matches_the_constants` 锁住这个事实。

#### 修正 2：`has_marker_block` 不该是 Adapter 的方法，其测试也必须跟着协议走

**原文怎么写**：只说「四情况协议本体抽为 `inject_marker_block()`」。

**为什么不对**：`has_marker_block` 原先是 `ClaudeCodeAdapter` 的**私有方法**，
依赖它的测试 `has_marker_block_agrees_with_regex_replacement` 也留在
`claude_code.rs` 里。协议一旦共享，把判定函数留在某一个 Adapter 上就意味着
「另一个 Adapter 用的是没被测试覆盖的那份路径」——测试与实现必须同址。

**改成什么**：`has_marker_block()` 提升为共享层的公开函数；
该测试迁到 `adapters/mod.rs` 的测试模块并改名为
`marker_detection_agrees_with_replacement`。`claude_code.rs` 里那段测试整体删除。

#### 修正 3：Codex 的标记块构造必须返回 `Result`（原文未提及防注入的落点）

**原文怎么写**：设计决策 ③ 说要「检查 `rules_content` 是否含标记串，含则返回明确错误」。

**未说清的部分**：没有指明这个检查放在**哪一层**。放进 `inject_marker_block()`
是错的——那是两个 Adapter 共用的函数，而 **Claude 侧根本不内联用户内容**
（它只写固定的一行 `@` 引用），给它加检查等于凭空多一个永远不触发的分支。

**实现落点**：检查放在 `CodexAdapter::build_marker_block()`，即
**构造内容的那一层**。因此该方法签名是
`fn build_marker_block(rules_content: &str) -> Result<String, AdapterError>`，
而 Claude 侧的 `build_marker_block()` 保持**无参无返回**的纯函数形态。
两侧签名不同是刻意的——差异真实存在，不该被抹平。

#### 修正 4：前端 `UPCOMING_TOOLS` 必须同步（原文完全未提）

**原文怎么写**：只讲了 Rust 侧六步，**没有一个字提到前端**。

**为什么是问题**：`src/api.ts` 里那份「即将支持」列表**硬编码了 `"ChatGPT IDE"`**。
它原本就是命名错误的产物，现在 Codex 已进入真实检测列表，
若不改，向导会**同时**显示「Codex CLI 已安装」与「ChatGPT IDE 即将支持」——
同一件事的两种说法并列，用户必然困惑。

**改成什么**：`UPCOMING_TOOLS` 收敛为 `["Trae"]`，并在注释里写明移除原因。

#### 修正 5：`ToolDescriptor` 顺序即 UI 展示顺序，必须与 PRD 示例对齐

**原文怎么写**：只说「`sync.rs` 的 `build_tools()` 注册第三个工具」。

**未说清的**：`build_tools()` 的返回顺序**直接决定向导步骤 2 的展示顺序**
（`detect_tools()` 原样透传）。Codex 若随手追加在末尾倒也无妨，
但若插到 Claude 之前就会与 `PRD.md` 步骤 2 的示例不一致。

**改为**：追加在末尾（Claude → Antigravity → Codex），与 PRD 步骤 2 示例一致。

---

### ✅ 实施记录（2026-09-14，TASK-18）

#### 交付文件

| 文件 | 变更 |
|:---|:---|
| `src-tauri/src/adapters/codex.rs` | **新建**。`CodexAdapter` 完整实现 + 19 个测试 |
| `src-tauri/src/adapters/mod.rs` | 迁入标记块常量、`marker_regex()`、`has_marker_block()`、`ensure_no_marker_in_content()`、`write_breaking_hardlink()`、`SyncL0Result`、`inject_marker_block()`；新增 6 个协议测试 |
| `src-tauri/src/adapters/claude_code.rs` | 删除本地常量/正则/断链写入/`has_marker_block`；`sync_l0_with_result()` 改为调用共享 `inject_marker_block()`；`pub use` 转出 `SyncL0Result` 保持既有路径可用 |
| `src-tauri/src/paths.rs` | 新增 `codex_dir()` |
| `src-tauri/src/sync.rs` | `build_tools()` 注册第三个工具；imports 补 `CodexAdapter` |
| `src-tauri/tests/adapter_consistency.rs` | 对比列表由两个扩为三个 Adapter；遍历式断言替代硬编码索引 |
| `src-tauri/examples/dryrun_sync.rs` | 新增 `copy_codex_subset()`（只复制 AGENTS.md 与 skills/）；覆盖 Codex 全流程与 `.system/` 保护验证 |
| `src/api.ts` | `UPCOMING_TOOLS` 移除 `"ChatGPT IDE"` |
| `docs/product/PRD.md` | 工具矩阵更新（见下） |
| `docs/testing/SPIKE_RESULTS.md` | 新增「待接入工具路径预探测」节 |

#### 实测命令与结果

```bash
cargo test                # 106 passed / 0 failed，0 warning
cargo build --examples    # 通过
cargo run --example dryrun_sync   # 0 项检查未通过
pnpm build                # vue-tsc + vite 零报错
```

`dryrun_sync` 新增的 Codex 检查项全部通过：

- 进度事件收到 **3 条**（Claude / Antigravity / Codex 各一）
- `AGENTS.md` 已注入标记块，且**标记块内联了规则全文**
- **未混入** `@~/.ai-profile/AGENTS.md`（Claude 专有语法）
- `skills/.system/`（Codex 内置技能）**未被触碰**
- 三个工具的技能目录数、SKILL.md 合规性逐项一致
- 真实目录快照 **2948 项前后逐项比对，零改动**

#### 真实数据安全回归

| 检查项 | 结果 |
|:---|:---|
| 5 个硬链接目标 md5 | 全等 `47645f603b7bfd2ce5eeba650ed09981` ✅ |
| `~/.claude/CLAUDE.md` 链接数 | 仍为 **5** ✅ |
| `~/.codex/AGENTS.md` | 仍为 **0 字节**（未被真实写入）✅ |
| 用户技能 | `agnes-scroll-world` / `huashu-design` / `grill-me` / `design-taste-frontend` 全部完好 ✅ |
| crossbrain 残留文件 | 无 ✅ |

#### 遗留事项（交给后续任务）

1. **⛔ 真实 Spike 未做**（本任务的最大遗留）。已验证的是「我们的代码按预期写入副本」，
   **尚未验证**「Codex CLI 真能读到 `~/.codex/AGENTS.md`」。
   按 `ADAPTER_SPEC.md` 第 6 节，这是接入前的强制步骤。
   验证方式（风险为零，因该文件当前是空的）：

   ```powershell
   # 1. 写入一行探针
   Set-Content -Path "$env:USERPROFILE\.codex\AGENTS.md" -Value "CROSSBRAIN_SPIKE_PROBE_18"
   # 2. 新开一个 Codex 会话，问「你当前加载了哪些全局指令？」
   # 3. 看回复是否体现该探针串 → 通过则清空文件
   Clear-Content -Path "$env:USERPROFILE\.codex\AGENTS.md"
   ```

2. **`~/.codex/skills/` 的懒加载未验证**：`SKILL.md` 的 frontmatter 格式与 Claude 侧
   逐字一致（`name` + `description`），但 Codex 是否同样按 `description` 懒加载，
   需在同一个 Spike 里一并验证。

3. **`~/.codex/AGENTS.md` 是「全局」还是「项目级」尚未确认**：
   Codex 同时支持项目根的 `AGENTS.md`（本项目根就有一个）。
   需确认 `~/.codex/AGENTS.md` 的优先级与作用域——若它其实只对某些 project 生效，
   L0 策略需要重新设计。

4. **共享层注释里的「两个 Adapter」表述已过时**（现为三个），已在本轮顺手订正
   主要几处；若后续引入第四个 Adapter，建议统一改为「各 Adapter」。

---

## TASK-18 补充修正（第二次，2026-09-14 深夜）— 探测到 Codex 桌面应用之后

> **触发**：用户在 TASK-18 收尾后给出线索——「codex ide 安装了啊，目录是 `D:\soft\win-x64\app`」。
> 实测确认为 **OpenAI 官方 Codex 桌面应用**，由此产生 **1 项命名修正 + 2 项新增防线**。

### 实测发现（决定性证据）

| 发现 | 证据 |
|:---|:---|
| `D:\soft\win-x64` 是官方 **Codex 桌面应用**（MSIX 包） | `AppxManifest.xml`：`Identity Name="OpenAI.Codex"`、`DisplayName = Codex`、`PublisherDisplayName = OpenAI`、`Executable="app/Codex.exe"`、版本 `26.623.13972.0` |
| 该包自带 CLI 本体 | `app/resources/codex.exe`（297 MB）+ `codex-command-runner.exe` — 不是「CLI 的替代品」，而是同一产品的桌面外壳 |
| 运行时另存 | `%LocalAppData%\OpenAI\Codex\{bin,runtimes}` |
| **App / CLI / 扩展共用 `~/.codex/`** | `~/.codex/` 正被 App 活跃写入（`logs_2.sqlite` 30 MB、`sessions/`、`thread_history_1.sqlite`、`config.toml` 含 `[desktop]` 段）；官方文档确认「桌面 App 配自定义指令 = 写 Codex home 的 `AGENTS.md`」 |
| App 的 UI 状态另存，**不含指令** | `~/.codex/.codex-global-state.json` 共 35 个 Electron 键（窗口位置 / 项目 / 会话 / 远程主机），**没有任何「自定义指令」键** → 指令只能走 `AGENTS.md` |
| 未装 VS Code 扩展 | `~/.vscode/extensions`、`~/.cursor/extensions` 均无 codex / openai 扩展 |
| 无 `CODEX_HOME` | 环境变量未设，走默认 `~/.codex` |

**结论**：它是**一款工具**（Codex），不是「App + CLI 两款」。`~/.codex/` 是唯一落点，
现有 Adapter 的检测路径与 L0/L2 落点**全部正确，无需改动**。

### 修正 6：显示名 `"Codex CLI"` → `"Codex"`

**原文怎么写**：`ToolDescriptor.display_name = "Codex CLI"`；
PRD 工具矩阵写作「**Codex CLI**（OpenAI）」。

**为什么不对**：用户装的是**桌面 App**，且 App / CLI / 扩展**共用同一个 `~/.codex/`**；
官方产品名（`AppxManifest` 的 `DisplayName`）就是 `Codex`。继续写「Codex CLI」
会重演「ChatGPT IDE」那类错误——展示名与本机实况不符。

**改成什么**：
- `sync.rs` 的 `display_name` 改为 **`"Codex"`**（唯一影响 UI 的地方）
- PRD 工具矩阵行改为 **`Codex`（OpenAI）**，命名修正记录改为「两次修正」
  （ChatGPT IDE → Codex CLI → Codex）
- 代码注释、测试标签、`api.ts` 注释一并为「Codex」

**不改的**：`tool_id` 保持 `"codex"`（稳定标识，供前端映射，与显示名解耦——
显示名今天会因产品改名而变，tool_id 不该跟着变）。

### 修正 7：必须新增 `AGENTS.override.md` 遮蔽检测（原文完全没有）

**原文怎么写**：设计决策里完全没有这一条。

**为什么是问题**：Codex 的规则发现顺序是「先 `AGENTS.override.md`，
没有再回落 `AGENTS.md`」，且**每个层级只取第一个非空文件**。
所以 `~/.codex/AGENTS.override.md` 一旦存在，写进 `AGENTS.md` 的标记块
**永远不会被 Codex 读到**——同步会「成功」但零效果，正是本项目最忌讳的**静默失效**。

**改成什么**：`CodexAdapter` 新增 `reject_if_override_present()`，
在 `sync_l0_with_result()` 的**第一步**调用；命中即返回 `AdapterError::Other`
（含完整路径与原因），中止本次 L0 同步。本 Adapter **只读检测**该文件，**永不写入**。

两个有意为之的决定：

- **不区分「空文件」**：Codex 会跳过空文件，理论上 0 字节 override 不遮蔽我们，
  但仍按「存在即中止」处理。理由：① 「跳过空文件」是 Codex 的实现细节，可能变化；
  ② 空 override 恰恰说明用户正打算用它，此刻放行等于埋一颗
  「某天写一行 → 我们的规则无声失效」的雷。规则单一（存在即中止）比精确模拟更安全、更易解释。
- **不用「改写进 override 文件」替代**：那会引入「用户删掉 override 后，
  `AGENTS.md` 里留下陈旧标记块待善后」的复杂度，语义更绕，且失去了「告知用户」的机会。

### 修正 8：`override` 报错必须验证「能走到报告里」（原文未涉及）

**为什么**：Adapter 报错只是第一步。若 `sync_one_tool` 把 `sync_l0` 的错误吞掉，
用户看到的仍是「同步成功」，**这道防线等于不存在**。
「单元测试证明 Adapter 会报错」推不出「用户能看到这个错」。

**改成什么**：`dryrun_sync.rs` 新增第 **[10] 节**——构造带 `AGENTS.override.md` 的副本，
跑真实 `run_for_tools`，断言 `status == Failed`、`message` 含文件名、`report.ok == false`、
且**没有**写出 `AGENTS.md`。这是**端到端**断言，只有它能证明防线真实存在。

### 第二轮实施记录

| 文件 | 变更 |
|:---|:---|
| `src-tauri/src/adapters/codex.rs` | 模块文档补「App/CLI 同源」与 override 风险；新增 `AGENTS_OVERRIDE_NAME` 常量、`agents_override_file()`、`reject_if_override_present()`；`sync_l0_with_result()` 加首步检测；新增 **4 个**测试 |
| `src-tauri/src/sync.rs` | `display_name`：`"Codex CLI"` → `"Codex"` |
| `src-tauri/src/paths.rs` | `codex_dir()` 注释补「App 与 CLI 共用」+ `AGENTS.override.md` |
| `src-tauri/src/adapters/mod.rs`、`claude_code.rs` | 注释中的「Codex CLI」统一为「Codex」 |
| `src-tauri/tests/adapter_consistency.rs` | 注释与断言文案统一为「Codex」 |
| `src-tauri/examples/dryrun_sync.rs` | 标签统一为「Codex」；新增第 **[10] 节**端到端遮蔽断言 |
| `src/api.ts` | 注释更新（「实为 Codex，桌面 App 与 CLI 共用 `~/.codex/`」） |
| `docs/product/PRD.md` | 工具矩阵行改为 `Codex`；命名修正改为「两次」；步骤 2 示例、V1.5 路标同步 |
| `docs/testing/SPIKE_RESULTS.md` | 预探测表 Codex 行更新；新增「官方文档已确认的部分」表；待 Spike 项由 3 项收敛为 1 项 |

```bash
cargo test                        # 110 passed / 0 failed，0 warning（较上轮 +4 条 override 测试）
cargo run --example dryrun_sync   # 10 节 0 项失败
pnpm build                        # vue-tsc + vite 零报错
```

**真实数据安全回归**：5 个硬链接目标 md5 全等 `47645f603b7bfd2ce5eeba650ed09981`、
`~/.codex/AGENTS.md` 仍 **0 字节 / links=1**、`AGENTS.override.md` **不存在**、
用户技能完好、无新增 crossbrain 残留。

### 遗留更新（原 3 项 → 1 项）

原遗留 **2**（skills 懒加载）与 **3**（作用域全局/项目级）**已由官方文档解答**，移出待实测项：

- **作用域 = 全局**（Codex home，`CODEX_HOME` 可覆盖）→ L0 策略成立，**无需重新设计**
- **技能 = 全局目录**：官方明确「skills can be global」，且用户自建的 `grill-me`
  就在 `~/.codex/skills/`，落点确认无误

**仅剩 1 项需用户配合**：往真实 `~/.codex/AGENTS.md` 写探针 → 新开 Codex 会话
问「总结你当前加载的指令」→ 确认探针出现 → 清空。
（风险为零：该文件当前 0 字节。）

> **补充发现（非阻塞）**：`project_doc_max_bytes` 默认 **32 KiB**，
> 全局 + 项目指令**合并**超限会被截断——规则内容过长时需留意。

---

## TASK-18 Spike 完成（第三次，2026-09-14 23:4x）— 实测闭环

> **触发**：用户贴回真实 Codex 会话的回答，确认探针口令「紫色鳄鱼正在吃第七个饺子」
> 被原样复述，并列出其加载的指令来源。任务由此从「代码完成」推进到「实测完成」。

### 关键突破：Spike 不再需要人工开会话

用户贴回的回答里出现了 `~/.agents/skills/` 路径，这提示 Codex 的技能根不止一个。
为查清落点，采用**二进制取证**：直接对 `D:\soft\win-x64\app\resources\codex.exe`
（297 MB）做字符串检索，从官方实现里读出真值。过程中发现 Codex CLI 自带：

```bash
codex debug prompt-input   # Render the model-visible prompt input list as JSON
```

该命令把**模型实际看到的全部输入**（system prompt、`<INSTRUCTIONS>`、`<skills_instructions>`）
以 JSON 打印，于是三项 Spike 全部变成**一条命令 + grep**，`ADAPTER_SPEC.md` 第 6 节
从「需用户配合」降级为「自动化」。**这是本任务最有复用价值的产出**，已写入
`docs/testing/SPIKE_RESULTS.md` 与项目技能 `crossbrain-add-adapter`。

### 实测结果（三项全通过）

| # | 验证项 | 结果 |
|:---|:---|:---|
| 1 | `~/.codex/AGENTS.md` 是否被读入模型上下文 | ✅ 探针口令出现在 `<INSTRUCTIONS>` 块内 |
| 2 | `~/.codex/skills/{slug}/SKILL.md` 是否纳入技能清单 | ✅ 探针技能以 `name + description + (file: …)` 形式出现 |
| 3 | 作用域是全局还是项目级 | ✅ **全局**（在 `%TEMP%\cbspike` 中立目录下执行，仍被加载） |

> 第 1、3 项**必须在项目目录之外**执行才能成立 —— 本仓库根本身就有 `AGENTS.md`，
> 在项目内跑无法区分两种来源。

### 新发现（已同步到 SPIKE_RESULTS）

**① Codex 的技能根共有四类**，我们的 L2 落点是其中之一：

| 技能根 | 归属 |
|:---|:---|
| `~/.codex/skills/.system/` | Codex 内置，严禁触碰 |
| `~/.codex/skills/{name}/` | **CrossBrain 的 L2 落点** ✅（实测可见） |
| `~/.codex/plugins/cache/` | 插件系统，不碰 |
| `~/.agents/skills/{name}/` | 第三方跨宿主技能库（**86 个技能 / 59 MB**），**不是**我们的落点 |

**② `~/.agents/` 是本机一处体量大、易误伤的既有资产。** 含 `.skill-lock.json`（v3）、
按宿主的 `agents/openai.yaml`、以及 3349 字节的 `~/.agents/AGENTS.md`。
二进制证据（`codex-home/src/instructions/mod.rs` 只硬编 `AGENTS.override.md` / `AGENTS.md`
**一对文件名**）表明：Codex **不读** `~/.agents/AGENTS.md`。
结论：任何 Adapter 都**不得**对 `~/.agents/` 做写入或清理；本项目的孤儿清理靠
「只扫 `~/.codex/skills/`」+「只处理 `crossbrain-` 前缀」双重收窄，已是安全的。

**③ `disable-model-invocation: true` 会让技能从清单中消失。** 用户自建的 `grill-me`
两处副本都不在清单里，原因就是这个标记 —— 说明「磁盘上有、清单里没有」未必是落点问题。
本项目 `format_skill_md()` 不生成该标记，技能可被隐式调用（已实测）。

### 探针清理记录（零残留）

| 动作 | 结果 |
|:---|:---|
| 删除 `~/.codex/skills/crossbrain-spike-load/` | ✅ 目录回到 `.system` + `grill-me` |
| 恢复 `~/.codex/AGENTS.md` 为 0 字节 | ✅ 0 bytes / links=1 |
| `~/.agents/` 零改动 | ✅ `AGENTS.md` md5 `cb47d1ce…` 不变、`grill-me` mtime 保持 |
| 5 路硬链接组 | ✅ md5 全等 `47645f60…` |

### 遗留事项（更新：**全部清空**）

TASK-18 的三项 Spike 全部通过，**本任务无未闭环遗留**。

> 归档说明：`~/.claude/skills/crossbrain-spike-test` 与
> `~/.gemini/config/skills/crossbrain-spike-test` 是**更早期 Spike 的遗留孤儿**，
> 按 `cleanup_orphans()` 设计会被首次真实同步正确删除（干跑已在副本上验证过）。
> 它们**不是**本轮残留，也**不应**手动删除 —— 留给首次真实同步处理，
> 正好可以当作孤儿清理功能的真机验收样本。

---

## TASK-19：改动知情与可逆（断链提示 + 备份一键还原）（2026-09-14 增补）

> **依据**：`DECISION_LOG.md` ADR-15。**优先级 P1**，排在 TASK-11 之后（顺序可与用户再确认）。
>
> **触发**：用户提问「我们的软件是否会破坏用户原有编程软件的用户偏好配置」。
> 逐行核对写入边界后确认：**偏好配置类文件（`settings.json` / `config.toml` /
> `mcp_config.json` / `rules/default.rules` / `skills/.system/` / `~/.agents/`）
> 代码里零引用，不存在破坏路径**。但核查同时暴露一处真实缺口与一处未告知的结构改变。

### 目标背后的原则

「不会破坏用户配置」这句话要成立，必须同时满足**可逆**与**知情**：

- 可逆 = 备份能读回来（当前只写不读 → **缺口**）
- 知情 = 改变用户结构前先告知（断链未告知 → **缺口**）

### Part A — 断链知情（首次同步前一次性告知）

**现状**：D-03 断链只写在代码注释与文档里，界面上毫无提示。用户此前靠硬链接实现
「一份内容 5 个工具生效」，我们断掉其中一侧却不说，属实现细节滥用知情权。

**做法（已定）**：

- 在**首次同步前**弹一次性说明（向导内或同步确认处），文案要点：
  「若目标文件与其他工具共享同一份内容（硬链接），本次写入会断开该共享关系 ——
  文件内容一字不变，但此后不再跟随其他位置一起更新。」
- **用持久化标记控制**（存进既有的 `~/.ai-profile/.crossbrain-state.json`），
  展示过一次就不再打扰，不用「每次同步都弹」。

**为什么不做硬链接数检测**（备选方案的取舍）：Windows 上 Rust std **拿不到**
链接数（`std::os::unix::fs::MetadataExt::nlink()` 是 Unix 专有；
Windows 需 `GetFileInformationByHandle().nNumberOfLinks`，即引入 `windows` crate
依赖或 shell 出去调 `fsutil hardlink list`）。而断链**不是 Claude 独有**——
任何走 `write_breaking_hardlink()` 的单文件型工具都会断链。因此判定为：
**用一次性通用告知，比平台特化的精确检测更稳、更省，且不会对没有硬链接的用户说错话**。
（若日后要做精确检测，作为可选优化，不阻塞本任务。）

### Part B — 备份可见性与一键还原

**现状**（核查结论，均有代码依据）：

| 项 | 事实 |
|:---|:---|
| 备份生成 | ✅ `inject_marker_block()` 情况三 `fs::copy`，写出 `{目标文件}.crossbrain-backup` |
| 备份读取 | ❌ **全仓库无任何读取代码** |
| 前端入口 | ❌ `commands.rs` 仅 6 条命令（`get_startup_state` / `detect_tools` / `read_global_rules` / `save_global_rules` / `complete_wizard` / `run_sync`），**无 restore** |

**要做的事**：

1. **新增 IPC 命令 `list_backups()`** → 逐个工具报告备份是否存在 + 大小 + 修改时间。
   数据源：`ToolDescriptor` 的 adapter 需暴露「备份文件路径」或新增
   `backup_path()` 查询方法（参考 `sync.rs::build_tools()` 现有结构，
   注意不要为前端另建一份工具清单）。
2. **新增 IPC 命令 `restore_backup(tool_id)`**：
   - ⚠️ **还原前先把当前目标文件另存为 `{目标文件}.crossbrain-before-restore`**，
     **绝不能覆盖 `.crossbrain-backup`**（那是干净快照，ADR-15 明令）。
   - ⚠️ 还原写入**必须走 `write_breaking_hardlink()`**，不得用 `fs::write` ——
     否则硬链接会在还原路径上重新连回共享 inode，D-03 的隔离出现「三条路对、这条错」的漏洞。
   - 还原**成功**后保留 `.crossbrain-backup` 不删（用户可能想再次还原）；
     删除动作留给用户或未来的「清理备份」功能。
   - 错误文案需转成用户语言（`commands.rs` 只做文案转换，业务在别处）。
3. **前端**：设置页新增「备份与还原」区 —— 列出各工具是否有备份、
   还原按钮带**二次确认**（文案须说明「当前内容会先另存为 …before-restore」）。
4. ⚠️ **新增界面文案必须纳入 ADR-08 的锁定测试**
   （`ipc_contract::wizard_copy_has_no_internal_terms`），
   不得出现 L0 / L2 / SSOT / slug / Adapter 等内部术语。

### 验收清单（可自动化）

- [ ] `list_backups()` 正确反映三个工具的备份存在性（临时目录 `with_base_dir` 测试）
- [ ] `restore_backup()` 后：目标文件内容 **== 备份内容**，且生成了 `*-before-restore`
- [ ] 还原走断链写入：还原后目标文件**不与其他路径共享 inode**（测试断言 links）
- [ ] `.crossbrain-backup` 内容在还原**前后一字不变**（md5 比对）
- [ ] 重复还原**幂等**：第二次还原不会产生 `*-before-restore` 嵌套
- [ ] 首次同步前的断链提示**只出现一次**（state 标记生效，二次同步不再弹）
- [ ] 新增前端文案通过 `wizard_copy_has_no_internal_terms`
- [ ] 真实数据安全回归：5 路硬链接目标 md5 全等、无残留

### 实测命令

```bash
export PATH="$HOME/.cargo/bin:$PATH"; cd src-tauri
cargo test
cargo run --example dryrun_sync      # 建议在干跑里补一节：还原流程端到端
cd .. && pnpm build
```

### 完成后必做

按项目流程收尾三项：① 本文档补「⚠️ 实施修正」② 补「✅ 实施记录」
③ `CURRENT_STATUS.md` 三处同步（时间戳用 `date` 取，别自己算）。

---

## ⚠️ 实施修正（2026-09-15 00:10，TASK-19）

本节列举任务定义里的假设与真实实施之间的偏差。**示例级的假设同样会误导后来者**，
所以逐条写明「原文 → 为什么不对 → 改成什么」。

**1. 「Adapter 需暴露「备份文件路径」或新增 `backup_path()` 查询方法」——只返回一个路径不够。**

- 原文：让 adapter 暴露单个备份路径，还原时由调用方拼出目标文件。
- 为什么不对：还原需要**两个**路径——落点（被注入标记块的文件）与快照（备份）。
  只给出备份路径，调用方还得反向推导目标文件名（把 `.crossbrain-backup` 去掉），
  这就是「两处各写一份命名规则」，与 `marker_regex()` 硬编标志串是同一类隐患。
- 改成：`fn l0_backup(&self) -> Option<L0Backup>`，`L0Backup` 同时含 `target_file`
  与 `backup_file`。**给默认实现返回 `None`**，于是 Antigravity 与全部测试 Mock
  一行都不用改——而不是被迫为「我没有这个概念」写空方法。

**2. 「还原前先把当前文件另存为 `*.crossbrain-before-restore`」——无条件另存是错的。**

- 原文未区分「当前内容是否已与备份相同」。
- 为什么不对：连续点两次还原时，第二次的「当前内容」就是**第一次还原后的内容**，
  无条件另存会把它写进 `*.crossbrain-before-restore`，**覆盖掉第一次留下的真实现场**——
  用户在这期间的手改就此丢失。为「可逆」而做的功能反而造成不可逆的数据丢失。
- 改成：**仅当当前内容与备份内容不同时**才另存。既避免无意义写入，
  又天然满足幂等（第二次还原不再产生新快照）。

**3. 验收项「第二次还原不会产生 `*-before-restore` 嵌套」表述有误。**

- 路径是固定的（`<目标文件>.crossbrain-before-restore`），**永远不会嵌套**——
  这条断言其实恒为真，测不出东西。
- 真正要断言的是第 2 条修正的后果：**第二次还原不得覆盖第一次的现场快照**。
  已改写为可自动化形式（比对文件内容前后是否一致）。

**4. 「断链告知放在向导或同步确认处」——只做一处会漏。**

- 为什么不对：触发「首次同步」的入口有**两个**（向导步骤 4、「立即同步」），
  而且用户完全可能在向导里没点同步就进入主界面。只在向导里实现的话，
  走另一条路径的用户永远看不到这条告知——而这正是 AC 要求的事前告知。
- 改成：闸门逻辑抽到 `src/composables/useLinkNotice.ts`（本项目**首个组合式函数**），
  两个入口共用一份。里面有一条不能写错的顺序约束：**标记「已展示」必须在用户确认之后**，
  否则用户点取消或关窗口后，这条说明就再也不会出现。

**5. `link_notice_shown` 的读取方式未指定——并入 `StartupState` 一次返回。**

- 若单开一条 `should_show_link_notice()` 查询命令，两个视图在挂载时都需各自再发一次 IPC，
  「点同步 → 先弹说明」就多了一个可能出现空档的时机。
- 改成：加进 `state::AppState` + `commands::StartupState`，随启动状态一次性返回。
  字段带 `#[serde(default)]`，老状态文件仍能解析（默认 `false` = 仍要告知，方向正确）。

**6. 术语锁定测试原本只扫 `WizardView.vue`——新页面会成为绕过口。**

- 原文（`tests/ipc_contract.rs::wizard_copy_has_no_internal_terms`）只检查向导模板。
- 为什么不对：本次新增的 `MainView.vue` 与 `LinkNoticeDialog.vue` 都面向用户，
  却不在检查范围内——「不暴露内部概念」的约束会被新文件悄悄绕开。
- 改成：抽出 `template_of()`，对 `WizardView.vue` / `MainView.vue` /
  `LinkNoticeDialog.vue` 三个文件的模板统一检查。

**7. 任务文档未要求、但同类风险必须补的两条契约测试。**

- `restore_backup_argument_name_matches_on_both_sides`：命令**参数名**
  （前端 `toolId` ↔ Rust `tool_id`）同样是字符串寻址。原有用例只覆盖命令名，
  参数名写错的表现是「还原按钮点了没反应」，且 `cargo test` 与 `pnpm build` 双双通过。
- `link_notice_is_marked_only_after_confirm`：第 4 条修正的顺序约束是本任务的
  关键正确性点，但前端组合式函数没有测试框架覆盖。改用源码比对锁定，
  并**做了反向验证**（把调用挪进 `guard()` → 测试如期失败），确认它不是永不失败的空断言。

**8. 已知坑复述：新增 Naive UI 组件（`NModal` / `NTag`）后必须先跑一次 `npx vite build`。**

- `pnpm build` = `vue-tsc --noEmit && vite build`，而 `src/components.d.ts` 要等
  `vite build` 才重新生成。顺序反了 `vue-tsc` 会因找不到组件类型而报错。
  本次按要求先跑 `npx vite build` 再 `pnpm build`，零报错。

---

## ✅ 实施记录（2026-09-15 00:10，TASK-19）

### 交付文件

| 文件 | 变更 |
|:---|:---|
| `src-tauri/src/adapters/mod.rs` | 新增 `MARKER_PRE_RESTORE_SUFFIX`、`L0Backup`、`RestoreOutcome`、trait 方法 `l0_backup()`（默认 `None`）、`pre_restore_path()`、`restore_l0_backup()`；+5 条测试 |
| `src-tauri/src/adapters/claude_code.rs` | 覆盖 `l0_backup()` |
| `src-tauri/src/adapters/codex.rs` | 覆盖 `l0_backup()` |
| `src-tauri/src/sync.rs` | 新增 `BackupInfo` DTO、`list_backups()` / `list_backups_for()`、`restore_backup()` / `restore_backup_for()`、`format_local_time()` |
| `src-tauri/src/state.rs` | `AppState` 新增 `link_notice_shown` + `mark_link_notice_shown()`；+1 条测试 |
| `src-tauri/src/commands.rs` | `StartupState` 新增 `link_notice_shown`；新增 3 条命令 |
| `src-tauri/src/lib.rs` | 注册 `list_backups` / `restore_backup` / `mark_link_notice_shown` |
| `src/api.ts` | `StartupState` 补字段；新增 `BackupInfo` 类型与 3 个封装函数 |
| `src/composables/useLinkNotice.ts` | **新建**（本项目首个组合式函数） |
| `src/components/LinkNoticeDialog.vue` | **新建**（首次同步前的一次性告知） |
| `src/views/WizardView.vue` | 接 `:startup` prop；`startSync` 过闸门（拆出 `doSync`）；挂载告知弹窗 |
| `src/views/MainView.vue` | 「立即同步」过闸门；设置页新增「备份与还原」区 + 二次确认弹窗 |
| `src/App.vue` | 向 `WizardView` 传 `:startup` |
| `src-tauri/tests/ipc_contract.rs` | DTO 字段对 +5；`template_of()` 扩展术语检查到 3 个文件；+2 条契约测试 |
| `src-tauri/examples/dryrun_sync.rs` | `watched` 增加两个 `*-before-restore` 路径；新增第 [11] 节（备份查询与还原端到端） |
| `src/components.d.ts` | 由 `vite build` 重新生成（`NModal` / `NTag` / `LinkNoticeDialog`） |

### 实测命令与结果

```bash
export PATH="$HOME/.cargo/bin:$PATH"; cd src-tauri
cargo test                  # 118 passed / 0 failed / 0 warning（较 TASK-19 前 +9 条）
cargo build                 # Finished，主程序编译通过（命令注册无误）
cargo run --example dryrun_sync   # 11 节，0 项检查未通过
cd .. && pnpm build         # vue-tsc --noEmit + vite build 零报错
```

干跑第 [11] 节的关键输出（跑在真实结构的副本上）：

```
[11] 备份查询与还原（跑在副本上）
   ✓ Claude Code｜备份=true｜…\claude\CLAUDE.md
   ✓ Codex｜备份=true｜…\codex\AGENTS.md
   ✓ 只列出有备份概念的工具（实际 2 个）
   ✓ 写独立文件的工具不该出现在备份列表里（它从不改动用户既有文件）
      还原返回：已把 Claude Code 还原到最初的内容。还原前的版本已另存为 …\CLAUDE.md.crossbrain-before-restore，需要时可以取回。
   ✓ 还原后的内容必须等于备份内容
   ✓ 还原后的内容必须等于同步前的原始内容
   ✓ 备份在还原后必须一字不变（它是唯一干净快照）
   ✓ 还原前应把当时的现场另存为 .crossbrain-before-restore
   ✓ 第二次还原不该再另存现场
   ✓ 第二次还原覆盖了第一次留下的现场快照
   ✓ Antigravity 应解释「没有需要还原的内容」
   ✓ 未知工具的文案应可读
```

另有一项**反向验证**（不属于常规回归，但值得记录）：把 `markLinkNoticeShown()` 从
`confirm()` 挪进 `guard()` 后，`link_notice_is_marked_only_after_confirm` 如期失败，
确认这条契约测试能真的抓到错误实现。

### 真实数据安全回归（跑完立即执行）

| 检查项 | 结果 |
|:---|:---|
| 5 路硬链接目标 md5 | ✅ 全等 `47645f60…` |
| `~/.claude/CLAUDE.md` links | ✅ 5（未断链） |
| 真实目录出现备份/现场/临时文件 | ✅ 无（`CLAUDE.*` / `AGENTS.*` 只有 `.md` 本体） |
| `~/.codex/AGENTS.md` | ✅ 仍 0 字节，无 `AGENTS.override.md` |
| `~/.ai-profile/.crossbrain-state.json` | ✅ **仍不存在**（本轮无任何代码路径写过真实状态） |
| 用户自建技能 | ✅ 完好（仅早期 Spike 遗留孤儿 `crossbrain-spike-test`，按设计留给首次真实同步清理） |
| 构建产物 | ✅ 新文案与新命令名均已进入 `dist/assets/*.js` |

### 遗留与后续

1. **GUI 未做点击穿过验证。** 已完成的验证是 `cargo build` + 118 条测试 +
   干跑 11 节 + `vue-tsc` + 产物内容核对；**没有**真的打开窗口点一遍
   「首次同步弹说明 → 设置页还原」。原因是拉起 `pnpm tauri dev` 会在用户机器上
   弹出一个他未要求的窗口。**建议**：由用户跑一次 `pnpm tauri dev` 目视确认，
   或在 TASK-15 端到端测试时一并覆盖。
2. **还原不做自动重新同步。** 还原后各工具里是用户最初的内容，CrossBrain 推的规则
   随之消失。这是刻意的（用户按还原就是要回到最初），但界面上未提示「还原后如果还想同步，
   请再点一次立即同步」。留待 TASK-13 / TASK-14 的文案完善一并考虑。
3. **`.crossbrain-before-restore` 没有清理入口。** 每次还原都会产生/更新一份，
   目前只能手工删。与「备份清理」一起作为后续可选优化。
4. **断链告知目前是无条件展示**（不做链接数检测）。理由已写在任务定义里：
   Windows 上 Rust std 拿不到 links，且断链不是 Claude 独有。若日后要做精确检测，
   需引入 `windows` crate 或 shell 调 `fsutil hardlink list`。


---

## TASK-20：探针检测——验证工具真的读到了 CrossBrain 的内容（2026-09-15 增补；**⚠️ UI 已于同日被 TASK-21 替换移除**，但其设计要点仍有效）

> **增补任务。** 触发：用户看到应用只支持 Claude Code / Codex（实际是三个，
> Antigravity 在列表中），追问其它工具何时接入。讨论中确认：项目铁律
> 「接入必先 Spike 探针验证」目前只存在于开发流程里，用户自己无法验证
> 「装了这个工具，CrossBrain 推的内容真的被读到了」。把它产品化进设置页。

### 功能定义

设置页新增「工具读取检测」区，逐工具提供「检测」按钮：

1. **写入探针**：走 `Adapter::sync_l2` 往该工具技能目录写 `crossbrain-probe/`
   （SKILL.md 的 frontmatter description 与正文均含唯一标记 `cb-probe-<时间戳>`）。
2. **验证**：
   - **Codex 全自动**：运行 `codex debug prompt-input`（中立目录、30 秒超时、
     stdout/stderr 落临时文件后判 grep）——标记出现 = `Verified`，当场删探针；
     标记没出现 = `Failed`（保留探针供人工排查）；启动失败/超时 = `NeedsManual`。
   - **Claude Code / Antigravity 半自动**：无取证命令，返回 `NeedsManual`，
     文案给出人工验证问句（「列出你可用的技能」）与标记，等用户确认。
3. **移除**：「移除探针」按钮 + 两道既有兜底——下次同步的孤儿清理与卸载
   都会删 `crossbrain-probe`（`crossbrain-` 前缀天然在命名空间内）。

### ⚠️ 实施修正（2026-09-15）

1. **「移除探针」绝不能复用 `cleanup_orphans`**。原任务草案想「清空后重写」——
   错误方向：`cleanup_orphans(白名单)` 的语义是「删掉所有不在白名单里的目录」，
   要删的恰恰是白名单里的那一个。新增共享函数
   `remove_crossbrain_skill_dir(skills_dir, name)`（只删一个、过命名空间校验、
   不存在返回 `Ok(false)` 幂等）。
2. **为此给 `Adapter` trait 加了必选方法 `skills_root()`**（三个真实 Adapter +
   4 个测试 Mock 全部实现）。它必须与 `sync_l2` 实际写入的技能目录同根——
   写与删各写一份路径，日后分叉就是「探针删不掉」。没有给默认实现：
   返回假路径的默认值是「静默装作有」，与 trait 上其它默认方法（语义上的
   真实空操作）性质不同。
3. **Codex 取证的 stdout 必须落文件而非管道**。管道写满会让子进程卡死，
   超时强杀后管道里的数据也拿不全；临时文件随写随有，强杀后仍可用已产出的
   部分判定。stderr 一并收集——若提示实际写在 stderr，stdout-only 会假阴性。
4. **探针的 token 必须进 frontmatter 的 `description`**，只写正文不够——
   模型可见输入里出现的是技能清单行（`- name: description`），正文是懒加载的。
5. **探针写入失败不产生半状态**：`sync_l2` 失败直接返回 `Failed`，
   不触碰其它文件；`detect=false` 时直接 `Failed` 且不创建任何目录。

### ✅ 实施记录（2026-09-15）

- 后端：`adapters/mod.rs`（trait 方法 + `remove_crossbrain_skill_dir`）、
  三个 Adapter 的 `skills_root()`、`sync.rs` 探针模块（`run_tool_probe(_for)` /
  `remove_tool_probe(_for)` / `verify_codex_probe`）、`commands.rs` 两条命令
  （`run_tool_probe` 走 `spawn_blocking`，真实启动 Codex 最长 30 秒）、`lib.rs` 注册。
- 前端：`api.ts`（`ProbeOutcome` + 两个封装）、`MainView.vue` 设置页新区块
  （复用 `detectTools` 出清单，未安装禁用；结果文案按状态着色）。
- 测试：cargo test **141 passed / 0 failed**（新增 4 个探针单测：写入+人工态+移除幂等、
  移除不波及真实技能、未知/未安装失败语义、未安装时移除幂等；新增契约测试
  `probe_wiring_matches` 锁命令名完整形态与 lib.rs 注册）。`pnpm build` 通过。
- **真机端到端**：手工按同一形态写入 `~/.codex/skills/crossbrain-probe/`
  → `codex debug prompt-input` 输出中出现标记 → 删除探针目录，`AGENTS.md`
  与其它技能零改动。
- 边界说明：Codex 的自动取证依赖其命令行在系统 PATH 中；打包版（TASK-17）
  若 PATH 不含 codex，会走 `NeedsManual` 分支并给出人工指引，不算失败。

---

## TASK-21：工具接入——扫描本机工具 + 勾选加入同步范围（2026-09-15 增补）

> **增补任务。** 触发：用户对 TASK-20 的逐个检测提出产品级重构想法——
> 「全部扫描 → 弹窗列出所有可支持的编程工具 → 左侧复选框选中 → 保存加入」。
> 经确认三项决策：①未适配工具全部列出、置灰登记意愿；②保存即同步（弹窗预告）；
> ③探针 UI 移除，由扫描接入取代。

### 功能定义

1. **扫描**（纯只读）：设置页「工具接入」区新增「扫描本机工具」按钮 →
   弹窗按已知特征路径（`SCAN_CATALOG`，10 个工具：已适配 3 + Spike 预探测 7）
   探测安装痕迹，逐行列出名称 / 状态标签 / 落点（`~/.cursor` 形态）。
2. **勾选**：已适配且已安装的可勾选；未适配的勾选只做**意愿记录**
   （存状态文件，作为后续适配优先级依据，绝不同步）；未安装的已适配工具禁用。
   弹窗初值 = 用户上次保存的清单（从未保存过 = 默认三个已适配全选，与真实同步行为一致）。
3. **保存并同步**：先过断链告知闸门（ADR-15 `linkNotice.guard`）→
   `save_enabled_tools` 命令：校验（未知 id 拒绝、去重保序）→ 写状态 →
   **立即按新范围执行一次完整同步**（复用 `run_full_sync_with_progress`，
   它每次现读状态；进度事件与「立即同步」共用同一通道）→ 状态栏/备份区同步刷新。

### ⚠️ 实施修正（2026-09-15）

1. **同步范围必须由状态驱动，而不是把 `build_tools()` 写死在 `run_full_sync_with_progress`**。
   新增 `filter_tools_with(tools, selected)`：`None`（从未做过选择）= 全保留——
   这个默认语义让**旧状态文件零迁移**；`Some(list)` 过滤，未适配 id 自然丢弃
   （`build_tools()` 只造得出已适配 Adapter，意愿与同步范围是两个层面）。
2. **卸载范围刻意保持全量**（三个已适配工具都清，不看启用集合）：只碰
   `crossbrain-` 命名空间与标记块，幂等且对未启用工具是空操作；按启用集合收缩
   反而会在「禁用后忘删」时留下痕迹，违背去痕承诺。
3. **状态文件必须用 `#[serde(default)]` + `Option`**：老用户没做过扫描时
   `selected_tools = None` = 「三个全启用」。若把默认值写成空列表，会把
   「没选过」误判成「一个都没选」，同步直接静默变空。
4. **探针移除是删 UI/命令/测试，保留 `skills_root()` 与
   `remove_crossbrain_skill_dir()`**：它们是语义正确的 trait 方法与工具函数，
   将来「接入后自动验证」复用，删除只会造成无谓的 churn。
5. **弹窗必须走原生页面**（GUI 走查实证）：CDP `newPage()` 建的页面**没有
   `__TAURI_INTERNALS__` 注入**——Tauri 的初始化脚本只注入它自己的 webview。
   在普通 CDP 页里应用代码照常渲染，但 invoke 全部失败，显示「无法启动」。
   走查脚本必须先找带 `__TAURI_INTERNALS__` 的页面再操作。

### ✅ 实施记录（2026-09-15）

- 后端：`state.rs`（`selected_tools: Option<Vec<String>>` + `mark_selected_tools`）、
  `sync.rs`（`SCAN_CATALOG` 10 工具 / `ScannedTool` / `scan_installed_tools(_in)` /
  `normalize_selection` / `filter_tools_with` / `save_enabled_tools_and_sync`）、
  `paths.rs`（`home()` 只读探测用出口）、`commands.rs` + `lib.rs`
  （`scan_installed_tools`、`save_enabled_tools`，后者走 `spawn_blocking` + 进度事件）。
- 前端：`api.ts`（`ScannedTool` + 两个封装）、`MainView.vue` 设置页
  「工具接入」区（当前同步范围 chips）+ 扫描弹窗（复选框 / 状态标签 / 保存预告 /
  重新扫描 / 保存进度）。
- 测试：cargo test **142 passed / 0 failed**（+1 状态往返、+4 扫描/规范化/过滤单测，
  探针的 4 单测 + 1 契约测试移除，新增 `tools_scan_wiring_matches`——含
  **探针零残留反向断言**）；`pnpm build` 通过；干跑 13 节 0 未通过。
- **真机 GUI 走查 11/11 通过**（CDP 驱动真实窗口）：弹窗 10 工具、
  已接入标签与勾选、未适配置灰可登记、勾选/取消交互、取消关闭不保存。
  走查全程真实数据零改动（状态文件 md5 前后一致、rules.md `f0b10675…` 不变、
  硬链接组 links=4）。
- 遗留：保存→自动同步的**真实触发路径**未在真机点击（会写用户文件），
  由单测 + 干跑 + 契约断言覆盖；用户首次使用时即是一次真机验收。

# CrossBrain 技术栈选型说明

> 版本：V1.0（依据 plan-v2.5.md V3.0 实测定稿版）  
> 更新时间：2026-09-14

---

## 技术栈全览

| 分层 | 选型 | 版本要求 | 说明 |
|:---|:---|:---|:---|
| 桌面框架 | **Tauri v2** | v2.x | 轻量高效，Rust 内核保证系统调用安全 |
| 前端语言/框架 | **Vue 3 + TypeScript** | Vue 3.4+，TS 5.x+（实测 6.0.3） | 严格类型约束，禁用 `any` |
| 样式系统 | **Tailwind CSS** | **v4.x**（实测 4.3.3） | 原子化样式，约束设计一致性。**v4 用 `@tailwindcss/vite` 插件，无 `tailwind.config.js`** |
| UI 组件库 | **Naive UI** | 最新稳定版 | 完善的 Vue 3 原生组件支持 |
| 图标方案 | **SVG（unplugin-icons / 本地 SVG）** | unplugin-icons 24.x | **严禁使用 Emoji 作为功能图标**；图标集（lucide）本地安装，构建不联网 |
| 系统路径处理 | **`dirs` crate** | — | 跨平台 home 目录解析，禁止字符串拼接路径 |
| 时间处理 | **`chrono` crate** | — | 生成 ISO-8601 时间戳，**严禁调用 shell `date` 命令** |
| 包管理器 | **pnpm** | v8.x+ | **强制使用，禁止 npm / yarn** |
| Rust Git 集成 | **`std::process::Command`** （V1） → **`git2` crate**（V2） | — | V1 依赖系统已装 git；V2 内嵌 libgit2，消除外部依赖 |

---

## 约束细节说明

### pnpm 强制

项目根目录设置 `packageManager` 字段：

```json
{
  "packageManager": "pnpm@9.15.9"
}
```

CI/CD 和本地开发环境均只允许 `pnpm` 执行安装和脚本，禁止切换为 npm 或 yarn。

### TypeScript 严格模式

```json
// tsconfig.json 必须包含
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "noUncheckedIndexedAccess": true
  }
}
```

**禁止**：
- 滥用 `any`（包括 `as any`）
- 组件 Props / Emits 没有明确的 Interface 或 Type 定义

### Rust 路径处理

```rust
// ✅ 正确
use dirs::home_dir;
let path = home_dir().unwrap().join(".ai-profile").join("global").join("rules.md");

// ❌ 禁止（硬编码、字符串拼接）
let path = format!("C:\\Users\\{}\\...", username);
```

### `chrono` 时间戳规范

```rust
// ✅ 正确
use chrono::Local;
let ts = Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();

// ❌ 禁止（Windows 不兼容 shell date）
std::process::Command::new("sh").args(["-c", "$(date ...)"]);
```

### 图标规范

- 功能性图标必须使用 SVG（通过 `unplugin-icons` 按需引入，或放置于 `src/assets/icons/` 目录下的本地 SVG 文件）
- **禁止**使用 Emoji 作为任何功能性图标
- 允许在纯文案提示语中使用 Emoji（如错误说明文字中的 🔴）

---

## 依赖升级路径（V1 → V2）

| 依赖 | V1 方案 | V2 升级目标 | 升级原因 |
|:---|:---|:---|:---|
| Git 集成 | `std::process::Command` 调用系统 git | `git2` crate（libgit2） | 消除对用户系统已安装 git 的依赖 |
| 文件变动监听 | 无（用户手动点同步） | `tauri-plugin-fs` Watcher | 实现 SSOT 文件变动自动触发重推 |

---

## 新增依赖审查红线

在引入任何新的第三方依赖前，必须先检查：

1. 当前 `package.json` 和 `Cargo.toml` 中是否已有能满足需求的依赖
2. 是否可以用原生 API 替代（例：简单防抖用 `setTimeout`，不引入 `lodash`）
3. 功能重复的同类包**严禁并存**（已有 `axios` 不引入 `ofetch`；已有 `dayjs` 不引入 `moment`）

# CrossBrain 本地开发环境搭建指南

> 更新时间：2026-09-14

---

## 前置依赖清单

在开始前，确保以下工具已安装并在 PATH 中可用：

| 工具 | 版本要求 | 验证命令 | 下载 |
|:---|:---|:---|:---|
| **Rust toolchain** | stable 最新版 | `rustc --version` | https://rustup.rs |
| **Node.js** | 18.x / 20.x / 22.x LTS | `node --version` | https://nodejs.org |
| **pnpm** | 8.x+ | `pnpm --version` | `npm i -g pnpm` |
| **Tauri CLI v2** | 2.x | `cargo tauri --version` | `cargo install tauri-cli --version "^2"` |
| **Git**（可选） | 任意版本 | `git --version` | https://git-scm.com |

> **注意**：Git 未安装时，CrossBrain 本地版本历史功能不可用，但其余所有功能正常。

---

## Rust toolchain 安装（✅ 2026-09-14 已完成，本节为实测记录）

**当前状态（已装好，新终端直接可用）**：

| 组件 | 实测版本 | 验证方式 |
|:---|:---|:---|
| Rust toolchain | **stable-x86_64-pc-windows-msvc 1.98.1** | `rustc --version` / `cargo --version` |
| rustup profile | **minimal**（刻意不含 rust-docs，原因见坑 3） | `rustup show` |
| MSVC 编译器/链接器 | **14.44.35207**（`cl.exe` + `link.exe`） | `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\` |
| Windows SDK | **10.0.26100.0** | `C:\Program Files (x86)\Windows Kits\10\Lib\` |
| Tauri CLI | **tauri-cli 2.11.4**（npm 版，项目实际使用） | `pnpm tauri --version` |
| WebView2 Runtime | 152.0.4191.66 | 见下方章节 |

> ⚠️ **验证标准不是 `--version` 能打印，而是能真的编译并链接。**
> 本次用的验收方式：写一个 `hello.rs`（含 `Vec`，确保用到标准库），
> 用 `rustc hello.rs -o hello.exe` 编译，确认 exe 生成且能运行。
> 只跑 `rustc --version` 会因为找不到 linker 而在真正 `cargo build` 时才炸。

---

### 🕳️ 踩坑复盘（重装 / 首次构建必看，共 5 个坑）

#### 坑 1：VS 安装器报 5008 / 5002 —— 根因是环境变量大小写重复

**症状**：`vs_BuildTools.exe --quiet --wait ...` 秒退，退出码 5008（自更新路径）或 5002（启动 setup.exe 路径）。
日志 `%TEMP%\dd_bootstrapper_*.log` 里能看到：

```
Error 0x80070057: Couldn't launch setup process.
Error: 已添加项。字典中的关键字:"http_proxy"所添加的关键字:"HTTP_PROXY"
```

**根因**：VS 安装器用 `StringDictionaryWithComparer`（忽略大小写的 .NET 字典）装载环境变量。
本机环境里同时存在成对的大小写变体，装载时抛 `ArgumentException`，安装器在启动子进程前就崩了。

实测本机有 **3 组重复**：

| 重复组 | 值是否相同 | 处理 |
|:---|:---|:---|
| `http_proxy` / `HTTP_PROXY` | 相同 | 删掉一份 |
| `https_proxy` / `HTTPS_PROXY` | 相同 | 删掉一份 |
| `Path` / `PATH` | **不同！**(853 / 962 字符) | 保留 `PATH`（是 `Path` 的前缀超集，多了 WorkBuddy shim 目录） |

**解法**：启动安装器之前，在**同一个进程内**先把大小写重复项去重，再启动子进程。
注意 PowerShell 的 `Env:` 驱动器是忽略大小写的，因此 `Remove-Item Env:\xxx` 删哪一个不确定——
正确做法是**循环删除直到该名字只剩一个条目**，然后**显式把想要的值写回去**：

```powershell
function Get-Pairs { $r=@(); foreach ($e in ([System.Environment]::GetEnvironmentVariables()).GetEnumerator()) { $r += [pscustomobject]@{ Key=$e.Key.ToString(); Val=$e.Value.ToString() } }; $r }
$keepPath = ((Get-Pairs) | Where-Object { $_.Key -ceq "PATH" } | Select-Object -First 1).Val
for ($i=0; $i -lt 60; $i++) {
  $p = Get-Pairs
  $dups = $p | Group-Object { $_.Key.ToLowerInvariant() } | Where-Object { $_.Count -gt 1 }
  if (-not $dups) { break }
  foreach ($g in $dups) { Remove-Item "Env:\$(($g.Group | Select-Object -First 1).Key)" -ErrorAction SilentlyContinue }
}
Set-Item -Path "Env:\PATH" -Value $keepPath
# 之后再启动 vs_BuildTools.exe
```

> 这套去重是**进程级**的，不改注册表。但每次新开进程都会从宿主重新继承，
> 所以**去重和启动安装器必须写在同一条命令里**（PowerShell 工具的 shell 状态不跨调用保持）。

#### 坑 2：VS 安装器自身的版本缺陷（Unexpected Installer Version）

**症状**：bootstrapper 下载完新版 installer 后，日志出现

```
Warning: Unexpected Installer Version. Starting version='' Launching version='3.14.2094...' MinVersion='4.10.30...'
Bootstrapper failed with known error.
```

**根因**：bootstrapper 需要先自我更新 installer，但该路径在 `--quiet` 下失败（微软已知缺陷，对应 ExitCode 5008）。

**解法**：先把最新版 installer 单独装进去，再跑真正的安装：

```powershell
# 1. 下载最新 vs_installer.opc（约 48 MB）
curl.exe -sSL -o "$env:TEMP\vs_installer.opc" "https://aka.ms/vs/install/latest/installer"

# 2. 更新 installer 本体（会把 setup.exe 解压到 Installer 目录）
.\vs_BuildTools.exe --quiet --wait --update --offline "$env:TEMP\vs_installer.opc"

# 3. 真正的安装（⚠️ 不要加 --nocache，它会把刚下载的 opc 清掉）
.\vs_BuildTools.exe --quiet --wait --norestart `
  --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended
```

**成功判据**：`EXIT=0`，且
`C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\<版本>\bin\Hostx64\x64\link.exe` 存在。

#### 坑 3：rustup 走代理会卡死 + rust-docs 解压极慢

**症状 A**：`rustup-init.exe` 停在 `info: downloading 6 components`，实测 **60 秒零字节增量**，
进程 CPU 仅 3.8s / 20 分钟 —— 连接挂死，不是慢。

**解法 A**：改用国内镜像 + **直连（绕开代理）**：

```powershell
# 先把 HTTP_PROXY / HTTPS_PROXY / ALL_PROXY / NO_PROXY 全部移除（含大小写变体，走直连）
# 再设置镜像
$env:RUSTUP_DIST_SERVER = "https://mirrors.tuna.tsinghua.edu.cn/rustup"
$env:RUSTUP_UPDATE_ROOT = "https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup"
rustup toolchain install stable
```

实测清华镜像延迟 0.22s、中科大 0.22s，直连即可，不需要代理。

**症状 B**：下载很快，但卡在解压，`.rustup` 目录增速降到 **0.1 MB/s**。
原因是 `default` profile 里的 **`rust-docs`** 有数万个小文件，
Windows 上叠加 Defender 实时扫描会拖到几十分钟。

**解法 B**：**先设 minimal profile 再装**（注意 `--profile minimal` 参数在「恢复半成品」路径下不生效，
必须用持久化的 `set profile`）：

```powershell
rustup set profile minimal          # 持久化设置，只装 rustc + rust-std + cargo
rustup toolchain uninstall stable   # 清掉半成品，否则会按上次的 default profile 恢复
rustup toolchain install stable
rustup default stable
```

> `rust-docs` 对编译毫无用处。需要文档时用 `rustup doc` 或在线文档。
> 后续如果要 `rustfmt` / `clippy`，用 `rustup component add rustfmt clippy` 单独加即可。

#### 附注：`where link.exe` 命中 PortableGit

本机 `where link.exe` 会返回 `...\PortableGit\...\usr\bin\link.exe`（coreutils 的硬链接工具）。
这是无害的名义冲突：rustc **不通过 PATH** 找 linker，而是通过 vswhere / 注册表定位 MSVC 工具链。
已用真实编译链接测试证明不受影响。

#### 坑 4：`cargo build` 报 `拒绝访问 (os error 5)` —— Windows 文件句柄延迟

**症状**：首次 `cargo build` 中途失败，退出码 101，日志为：

```
error: failed to run custom build command for `proc-macro2 v1.0.107`
  --- stderr
  Failed to clean up D:\...\target\debug\build\proc-macro2-xxxx\out\probe: 拒绝访问。 (os error 5)
```

**根因**：`proc-macro2` 的构建脚本会编译一个探测程序、执行它、再删掉临时目录。
Windows 上刚执行过的 exe 句柄释放有延迟，`remove_dir_all` 撞上 `ERROR_ACCESS_DENIED` 便直接 exit 1。

**⚠️ 不要误判为杀毒软件**：本机 `RealTimeProtectionEnabled = False`（Defender 实时保护本来就没开），
也没有任何第三方杀软进程，因此**不需要**去加杀软排除项。这是文件系统的句柄时序问题。

**解法**：删掉该 crate 的构建残留后重试，一次即过：

```bash
cd src-tauri
rm -rf target/debug/build/proc-macro2-*
cargo build
```

**系统性提示**：本机还会以 **warning** 形式出现同类错误——
`error copying object file ... to incremental directory ...: 拒绝访问 (os error 5)`
（仅增量元数据复制失败，编译本身仍成功）。
结论：`os error 5` 在本机是**系统性现象**。遇到构建失败先怀疑它，
清理 `target/debug/build/<crate>-*` 后重试，不要先怀疑自己的代码。

#### 坑 5：cargo 拉依赖慢 —— 配国内镜像

直连 `static.crates.io` 实测约 **170 KB/s**，而首次构建 Tauri 依赖树有 **484 个 crate**，会拖很久。
已配置用户级 `~/.cargo/config.toml`（镜像 + 直连；实测 rustup / cargo 走代理反而会卡死）：

```toml
[source.crates-io]
replace-with = "tuna"

[source.tuna]
registry = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"

[net]
git-fetch-with-cli = true
```

> 恢复官方源：删掉 `~/.cargo/config.toml` 即可。
> 注意 `RUSTUP_DIST_SERVER`（工具链）与 `[source.crates-io]`（依赖）是**两套独立配置**，都要配。

---

### 路线 B：GNU（未采用，仅备查）

```powershell
# 需先安装 MinGW-w64（例如 https://winlibs.com 的 UCRT 版，解压后把 bin 加入 PATH）
# 然后用 stable-x86_64-pc-windows-gnu 工具链
```

本机**未采用**此路线（决策见 `CURRENT_STATUS.md` D-02）。

---

## WebView2 Runtime

Tauri 在 Windows 上依赖 WebView2。Win10 1803+ / Win11 通常已内置。
验证方式（本机实测已安装，版本 152.0.4191.66）：

```powershell
Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" | Select-Object pv
```

---

## 初始化项目骨架

```bash
# 方式一（推荐，已在 TASK-01 实际执行）：在临时目录生成再合并，保护已有 README.md
mkdir -p .scaffold && cd .scaffold
pnpm dlx create-tauri-app@latest crossbrain --manager pnpm --template vue-ts --identifier com.crossbrain.app --tauri-version 2 --yes
# 然后按 docs/impl/TASK_BREAKDOWN.md TASK-01 步骤 2 的清单合并回根目录

# 方式二：空目录直接生成（仅当目标目录为空时可用）
pnpm dlx create-tauri-app@latest crossbrain --manager pnpm --template vue-ts --identifier com.crossbrain.app
```

> 注意：`create-tauri-app` **没有** `--app-name` 参数，项目名由 `PROJECTNAME` 位置参数决定；
> 该参数同时决定 `package.json` 的 `name` 和 `Cargo.toml` 的 `[package] name`。

---

## 安装依赖

```bash
pnpm install
```

---

## 启动开发服务器

```bash
pnpm tauri dev
```

**首次运行**要编译整个 Tauri 依赖树（484 个 crate），本机实测约 **12 分钟**（大部分时间是下载依赖）；
之后增量编译只需 **1~6 秒**，前端热重载秒级响应。

> 若首次编译因 `拒绝访问 (os error 5)` 中断，见上方「坑 4」——清理残留重试即可，不是代码问题。

---

## 生产构建

```bash
# 仅在需要验证打包或发布时执行，日常开发不需要
pnpm tauri build
```

输出目录：`src-tauri/target/release/bundle/`

---

## 目录结构说明

```
crossbrain/
├── src/                    # Vue 3 前端源码
│   ├── components/         # 组件（每个组件必须有同名 .md 文档）
│   ├── assets/
│   │   └── icons/          # 本地 SVG 图标
│   └── main.ts
├── src-tauri/              # Rust / Tauri 后端
│   ├── src/
│   │   ├── main.rs         # 薄壳，仅一行 crossbrain_lib::run()
│   │   ├── lib.rs          # 真正的入口：注册模块 + 启动前初始化 SSOT
│   │   ├── paths.rs        # 所有路径的统一出口（铁律 L-02，禁止别处拼路径）
│   │   ├── init.rs         # 启动时创建 ~/.ai-profile/ 目录底座（幂等）
│   │   ├── adapters/       # Adapter trait 及各工具实现
│   │   ├── slug.rs         # Slug-Hash 生成算法
│   │   └── git.rs          # Git 提交封装
│   └── Cargo.toml
├── docs/                   # 项目文档
├── README.md
└── package.json
```

> ⚠️ **Rust 侧入口是 `lib.rs` 而不是 `main.rs`**（Tauri v2 为支持移动端入口采用的固定结构）。
> `main.rs` 只有 `fn main() { crossbrain_lib::run() }` 一行，`mod` 注册与新逻辑都写在 `lib.rs` 里。

---

## 常见问题

### Q：`pnpm tauri dev` 卡在编译 Rust 阶段？

首次编译需构建 484 个 crate，本机实测约 12 分钟（含依赖下载），属正常现象；
后续增量编译为 1~6 秒。可确认 Rust toolchain 是否正常：

```bash
rustup show
```

### Q：Windows 下 Tauri 构建报 WebView2 错误？

安装 Microsoft Edge WebView2 Runtime：  
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

### Q：pnpm install 报 `ERR_PNPM_RECURSIVE_RUN_FIRST_FAIL`？

确认 Node.js 版本为 18.x / 20.x / 22.x LTS，不支持 Node 17 及以下版本。

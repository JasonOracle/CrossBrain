# CrossBrain Windows 兼容性备忘录

> 版本：V1.0  
> 更新时间：2026-09-14  
> 适用版本：Windows 10 及以上

---

> ⚠️ **本文档记录 Windows 环境下的专项陷阱**，每条均为已识别的高风险点，开发时必须逐条对照。

---

## 1. 路径处理（最高风险）

### 规范

- **必须**使用 `dirs` crate 解析 home 目录，不得假设用户名或盘符
- **必须**使用 Rust `Path` / `PathBuf` API 拼接路径，禁止字符串拼接

```rust
// ✅ 正确
use dirs::home_dir;
let profile_dir = home_dir()
    .expect("无法获取 home 目录")
    .join(".ai-profile");

// ❌ 禁止 — 盘符硬编码，在非 C 盘安装 Windows 的用户处直接崩溃
let profile_dir = PathBuf::from("C:\\Users\\Administrator\\.ai-profile");

// ❌ 禁止 — 字符串格式化拼接，跨平台分隔符不一致
let path = format!("{}/.ai-profile/global/rules.md", username);
```

### 背景

Windows 下 `%USERPROFILE%` 可能是 `C:\Users\xxx`，也可能是 `D:\Users\xxx`（系统盘不同）或自定义路径。`dirs::home_dir()` 会正确读取系统环境变量，是唯一可靠方式。

---

## 2. 时间戳生成（高风险）

### 规范

- **必须**使用 `chrono` crate 生成时间戳
- **严禁**调用 `shell date` 命令（Windows PowerShell 的 `date` 格式完全不同于 Unix shell）

```rust
// ✅ 正确
use chrono::Local;
let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();
// 输出示例：2026-09-14T19:00:00+0800

// ❌ 禁止 — Windows 不存在 /bin/sh，此调用直接 panic
std::process::Command::new("sh")
    .args(["-c", "date -u +%Y-%m-%dT%H:%M:%SZ"])
    .output();

// ❌ 禁止 — PowerShell Get-Date 格式与 Unix date 不兼容
std::process::Command::new("powershell")
    .args(["-Command", "Get-Date"])
    .output();
```

---

## 3. Git 提交健壮性（中风险）

### `nothing to commit` 处理

```rust
// git commit 返回非零退出码时，判断 stderr 是否包含 "nothing to commit"
// 若包含，视为成功，不向 UI 报错
let output = Command::new("git")
    .args(["commit", "-m", &msg])
    .current_dir(&profile_dir)
    .output()?;

if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("nothing to commit") {
        // 静默处理，正常返回
        return Ok(());
    }
    // 真正的错误才向上抛出
    return Err(GitError::CommitFailed(stderr.to_string()));
}
```

### git 未安装的降级

```rust
// 调用 git 前先检测是否可用，不可用时降级而非崩溃
fn is_git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}
```

---

## 4. 文件权限处理（中风险）

### 自动创建目录

```rust
// 目录不存在时自动创建（含所有父级目录），不报错
use std::fs;
fs::create_dir_all(&target_dir)?;
// 若 create_dir_all 失败（权限问题），向 UI 翻译为用户友好提示
```

### 权限错误翻译

| 系统错误 | `io::ErrorKind` | 用户提示 |
|:---|:---|:---|
| 目录不存在 | `NotFound` | 静默自动创建 |
| 创建目录失败 | `PermissionDenied` | 「无法创建配置目录，请手动创建后重试」 |
| 文件写入失败 | `PermissionDenied` | 「没有写入权限，请检查目录访问权限」 |

---

## 5. 代码签名（发布红线）

### 为什么必须签名

Windows 的 **Mark of the Web（MoTW）** 机制：  
从互联网下载的可执行文件会被标记为不受信任，触发 SmartScreen 蓝色警告弹窗（OV 证书）或红色阻止弹窗（无签名）。**便携版 zip 解压后的 .exe 同样受此影响**，不能因为是便携版就跳过签名。

### 要求

- 申请 **OV（组织验证）级** 代码签名证书（EV 更佳，可直接通过 SmartScreen）
- MSI 安装包和便携版 .exe 均必须完成签名
- 签名工具：`signtool.exe`（Windows SDK 自带）或 Tauri 的内置签名配置

### Tauri 签名配置

```json
// tauri.conf.json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

---

## 6. 国内下载渠道

GitHub Releases 在国内访问不稳定，必须同步提供国内镜像：

| 渠道 | 说明 |
|:---|:---|
| GitHub Releases | 主渠道，国际用户 |
| 国内 OSS / CDN 镜像 | 每次发布后自动同步，国内用户主渠道 |

**镜像同步 SOP**：  
每次 GitHub Release 发布后，CI 自动将 MSI 和 zip 上传至 OSS Bucket，并更新下载页链接。

---

## 7. WebView2 依赖

Tauri v2 在 Windows 上依赖 **Microsoft Edge WebView2 Runtime**。

- Windows 11：预装，无需处理
- Windows 10（2004 及以上）：大概率预装
- Windows 10 旧版本：可能需要用户手动安装

**处理策略**：  
Tauri `--bundle` 模式可选 `fixed-version`（内嵌 WebView2，但安装包体积增大约 100MB）或 `evergreen-installer`（安装时联网下载，体积小）。

V1 推荐使用 `evergreen-installer`，在下载页说明最低系统要求，避免增大安装包体积。

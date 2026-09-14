# CrossBrain 发布与打包流程

> 版本：V1.0  
> 更新时间：2026-09-14

---

## 发布产物

| 产物 | 格式 | 目标用户 |
|:---|:---|:---|
| MSI 安装包 | `CrossBrain_x.x.x_x64_en-US.msi` | 需要安装到系统的用户 |
| 便携版 zip | `CrossBrain_x.x.x_portable.zip` | 无需安装、随身携带的用户 |

> **注意**：便携版 .exe 同样需要代码签名（Windows MoTW 机制），不能跳过。

---

## 构建命令

```bash
# 生产构建（生成 MSI + exe）
pnpm tauri build

# 输出目录
src-tauri/target/release/bundle/
├── msi/
│   └── CrossBrain_x.x.x_x64_en-US.msi
└── nsis/
    └── CrossBrain_x.x.x_x64-setup.exe
```

---

## 版本号规范

遵循语义化版本（SemVer）：`MAJOR.MINOR.PATCH`

| 变更类型 | 版本号变化 | 示例 |
|:---|:---|:---|
| 新增 Adapter / 重大功能 | MINOR +1 | 1.0.0 → 1.1.0 |
| Bug 修复 / 小改动 | PATCH +1 | 1.0.0 → 1.0.1 |
| 不兼容变更（重大架构调整） | MAJOR +1 | 1.x.x → 2.0.0 |

版本号需同步修改：
- `package.json` 中的 `version` 字段
- `src-tauri/Cargo.toml` 中的 `version` 字段
- `src-tauri/tauri.conf.json` 中的 `version` 字段

---

## 代码签名配置

### 前置条件

- 已申请 OV（组织验证）级或 EV 级代码签名证书
- 证书已导入到 Windows 证书存储区

### tauri.conf.json 签名配置

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT_HERE",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

### 手动签名验证

```powershell
# 验证签名是否成功
Get-AuthenticodeSignature .\CrossBrain_x.x.x_x64_en-US.msi
# Status 应为 Valid
```

---

## 发布 SOP（标准操作流程）

```
Step 1. 更新版本号
  → 修改 package.json / Cargo.toml / tauri.conf.json 中的版本号
  → git commit -m "chore: bump version to x.x.x"
  → git tag vx.x.x

Step 2. 执行生产构建
  → pnpm tauri build
  → 确认 bundle/ 目录下产物存在

Step 3. 代码签名（如未配置自动签名）
  → signtool sign /fd sha256 /tr http://timestamp.digicert.com /td sha256 .\xxx.msi

Step 4. 制作便携版 zip
  → 将签名后的 .exe 打包为 zip：CrossBrain_x.x.x_portable.zip

Step 5. 发布到 GitHub Releases
  → git push origin vx.x.x
  → 在 GitHub 创建 Release，上传 MSI 和 zip
  → 填写 Release Notes（从 CHANGELOG.md 提取）

Step 6. 同步国内镜像
  → 将 MSI 和 zip 上传至 OSS Bucket
  → 更新下载页的国内镜像链接

Step 7. 验证下载链接
  → 分别验证 GitHub 链接和国内镜像链接可正常下载
  → 下载后验证签名有效
```

---

## 发布前 Checklist

在执行发布 SOP 前，逐项确认以下工程底线：

- [ ] Windows 端到端流程跑通（新建技能 → 同步 → Claude Code / Antigravity 实际触发）
- [ ] 离线断网实测通过（断网后同步无报错）
- [ ] 代码签名证书有效期确认（剩余有效期 > 6 个月）
- [ ] 版本号三处同步更新（package.json / Cargo.toml / tauri.conf.json）
- [ ] CHANGELOG.md 已更新本版本变更内容
- [ ] README.md 下载链接已更新
- [ ] 国内 OSS 镜像渠道已就绪
- [ ] 发布产物均已通过代码签名验证
- [ ] 安全声明文案已在 README 首屏可见

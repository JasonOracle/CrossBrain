# 更新日志（Changelog）

所有对外可见的变更都记录在这里。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [1.0.0] — 2026-09-15

首个正式版本。CrossBrain 让你在一个地方维护「个人 AI 记忆」（全局规则 + 知识库），
一键同步到多个 AI 编程工具，无需逐个工具重复配置。

### 新增

- **四工具同步引擎**
  - Claude Code（`~/.claude/`：CLAUDE.md 标记块 + `@` 引用，`skills/`）
  - Antigravity IDE（`~/.gemini/config/`：rules 目录独立文件，`skills/`）
  - Codex（`~/.codex/`：AGENTS.md 标记块 + 内联全文，`skills/`）
  - OpenCode（`~/.config/opencode/`：AGENTS.md 标记块 + 内联全文，`skills/`）
- **SSOT 知识中枢**：`~/.ai-profile/global/rules.md` 全局规则 + `knowledge/` 知识库，
  每次同步整体推送到所有已接入工具，一处修改处处生效。
- **内置编辑器**：规则与知识条目的分栏 Markdown 编辑 + 实时预览（原始 HTML 强制转义），
  保存与同步严格分离。
- **工具接入管理**：扫描本机 10 类常见 AI 编程工具（纯只读），复选框选择同步范围；
  未接入的工具可登记使用意愿，作为后续适配优先级。
- **本地版本历史**：`~/.ai-profile/` 自动 git 化，每次同步成功自动提交
  （`sync: <ISO-8601>`），规则与知识库可随时回溯；未安装 git 时静默降级，
  提交失败绝不影响同步结果。
- **数据安全**：
  - 首次写入用户既有文件前自动创建 `.crossbrain-backup` 原始件备份，
    并弹窗告知硬链接断链影响，可一键还原；
  - 卸载完整去痕：移除标记块恢复原文、删除备份与同步技能；
  - 只写 CrossBrain 自有落点，工具自身配置（settings/config）零接触。
- **离线可用**：同步全部本地完成，无网络依赖；断网时同步照常，失败原因明确可展开。
- **安装包**：Windows MSI 安装器 / NSIS setup.exe / 免安装便携版 zip。

### 已知限制

- 尚未配置代码签名证书，首次运行可能触发 SmartScreen / 未知发布者提示。
- 暂仅支持 Windows（Tauri 跨平台架构已就绪，其他平台随需求排期）。
- 以下工具已探测到安装痕迹、落点待实测后接入：CodeBuddy、WorkBuddy、Cursor、Trae、ZCode、Qoder。

[1.0.0]: https://github.com/JasonOracle/CrossBrain/releases/tag/v1.0.0

# CrossBrain 版本演进记录

> 更新时间：2026-09-14

---

## V3.0（2026-09-14）— 实测定稿版（当前版本）

**里程碑**：所有核心 Spike 全线贯通，可立即开工。

### 核心变更

1. **Spike 全线贯通**：实测验证 Claude Code 与 Antigravity IDE 的 Skills 懒加载 100% 成立（Spike A~D 全部通过）
2. **V1 核心矩阵修正**：剔除未安装的 Cursor，将 **Claude Code + Antigravity IDE（Gemini）** 确立为 V1 双王牌，Cursor 移入 V2
3. **Antigravity 规则机制固化**：
   - L0 写入 `~/.gemini/config/rules/crossbrain-L0.md`（独立文件，不破坏用户既有规则）
   - L2 写入 `~/.gemini/config/skills/crossbrain-{slug}/SKILL.md`
4. **既有规则保护策略确立**：确认用户已有完善的 `CLAUDE.md` 与 `user_global.md`，适配器必须采用**标记块注入或备份**策略，严禁粗暴清空

---

## V2.5（阶段版本）

### 核心变更

1. **P0 孤儿清理机制**：每次同步后自动清除不在当前 knowledge 列表中的历史 `crossbrain-*` 目录，彻底杜绝幽灵规则残留
2. **安装检测方法明确**：通过检查配置目录是否存在判断工具是否已安装
3. **三个高风险 Spike Plan B**：为 Claude Code `@import` 路径展开、Skills 懒加载、Antigravity Skills 懒加载各准备了降级方案

---

## V2.4

### 核心变更

1. 确立国内工具支持矩阵（Trae / ChatGPT IDE / OpenCode / WorkBuddy）
2. 确定**离线 100% 兜底策略**：V1 核心功能零网络依赖

---

## V2.3

### 核心变更

1. 修复 7 处架构设计冲突
2. 确立严苛 UX 规范：**禁止在 UI 界面泄露内部术语**（L0 / L2 / Adapter / SSOT / slug / frontmatter 等）
3. 系统错误 → 用户提示翻译表全面落地

---

## V2.2

### 核心变更

1. 项目正式更名为 **CrossBrain（跨脑）**
2. 砍掉 V1 文件 Watcher（降低复杂度，V1 改为用户手动点同步）

---

## V2.1

### 核心变更

1. L2 挂原生懒加载机制（按 description 按需触发）
2. Claude Code 桩文件路径定死（`@~/.ai-profile/AGENTS.md`）

---

## V2.0

### 核心变更

1. 废弃 Symlink 方案（跨平台权限问题过多）
2. 废弃 JSON 配置中间层（过度工程，增加复杂度）
3. 确立 **Hub & Spoke 架构**：SSOT（`~/.ai-profile/`）→ 各工具配置目录直接写入

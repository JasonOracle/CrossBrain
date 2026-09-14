> 🤖 **AI 编程工具注意**：你的第一个必读文件是 [`MASTER_CONTEXT.md`](./MASTER_CONTEXT.md)，从那里开始，不要从这里开始。

# CrossBrain（跨脑）

> 写一次 AI 规则，处处可用。全程本机运行，无需联网。

---

## 是什么

CrossBrain 是一个运行在本地的桌面应用，帮助你把 **AI 编码偏好**（编码风格、语言偏好、工作习惯）和 **技能知识**（特定领域的经验库）一次写好，自动同步到你所有的 AI 编程工具中。

---

## 核心特性

- **写一次，处处生效** — 统一管理，同步到 Claude Code、Antigravity IDE 等主流 AI 工具
- **100% 离线可用** — 所有操作为纯本地文件读写，零网络依赖，天然支持企业隔离内网
- **按需加载，不污染上下文** — 技能知识由 AI 按需自动触发，不在每次对话中全量注入
- **保护你的现有配置** — 通过标记块注入协议，严禁删除或覆盖你已有的 AI 工具配置
- **幂等同步** — 多次同步结果一致，孤儿文件自动清理，没有幽灵规则残留
- **本地版本历史** — 基于 git 的本地变更记录（需安装 git，未安装时降级但不阻断）

---

## 支持的 AI 工具

### V1（当前版本）

| 工具 | 状态 |
|:---|:---|
| Claude Code CLI | ✅ 支持 |
| Antigravity IDE（Google Gemini） | ✅ 支持 |
| Gemini CLI | ✅ 顺带支持 |

### V1.5（即将支持）

Trae · ChatGPT IDE · OpenCode · WorkBuddy / Codebuddy

### V2（规划中）

Cursor · Windsurf · Qoder · Zcode

---

## 下载安装

| 渠道 | 链接 |
|:---|:---|
| GitHub Releases | _发布后更新_ |
| 国内镜像（OSS/CDN） | _发布后更新_ |

**支持格式**：MSI 安装包 / 免安装便携版 zip（两者均已签名）

**系统要求**：Windows 10 及以上

---

## 安全声明

- CrossBrain **仅写入用户自己创建的内容**，不从任何外部 URL 拉取规则（防供应链攻击）
- V1 / V2 不支持远程导入规则
- 首次注入 Claude Code 配置时，CrossBrain 会自动备份你的原始 `CLAUDE.md`（备份至同目录 `CLAUDE.md.crossbrain-backup`），**绝不覆盖用户已有规则**

---

## 与 Shell 脚本的本质区别

> 「Shell 脚本不也能干这个？」

不一样。CrossBrain 提供的是 Shell 脚本几乎必定缺失的能力组合：

| 能力 | Shell 脚本 | CrossBrain |
|:---|:---|:---|
| 幂等安全（标记块协议） | ❌ 需自己实现 | ✅ 内置 |
| 保护既有配置（备份机制） | ❌ 极易误删 | ✅ 内置 |
| 孤儿清理（防幽灵规则残留） | ❌ 需自己实现 | ✅ 内置 |
| 跨平台 GUI | ❌ 无 | ✅ Tauri 原生桌面 |
| 100% 离线可用 | ✅ 可以 | ✅ 可以 |
| 懒加载路由（按需精准分发，不污染上下文） | ❌ 无 | ✅ 内置 |

---

## 技术栈

Tauri v2 · Vue 3 · TypeScript · Rust · Tailwind CSS · Naive UI

---

## License

[Apache-2.0](./LICENSE)

Apache-2.0 含专利保护条款，对开源生态友好。

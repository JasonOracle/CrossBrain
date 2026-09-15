/**
 * useRuleEditor.ts — Markdown 文档的读写状态机（TASK-11 引入，TASK-12 泛化）。
 *
 * # 为什么抽成一个组合式函数
 *
 * 「编辑一份本机 Markdown」在**三个地方**出现：向导步骤 3、主工作台「全局规则」页
 * （两者读写 `~/.ai-profile/global/rules.md`）、主工作台「技能知识」页的逐篇编辑
 * （TASK-12，读写 `~/.ai-profile/knowledge/{文件名}`）。状态机只有一份，
 * 读写目标由调用方注入（`EditorIO`），三处共享同一套基线/dirty 语义——
 * 复制成多份必然各自漂移（本项目已为「同一件事写两处」付过两次学费）。
 *
 * # 「读不出来 ≠ 空的」（与 ADR-14 同一条原则）
 *
 * 读取失败时**不允许保存**：磁盘上明明有内容却读不出来，
 * 若放行保存，用户会把一份空文件覆盖到自己的内容上——不可逆。
 * 此时界面显示错误与重试入口，编辑器保持禁用。
 */

import { computed, reactive, ref } from "vue";
import MarkdownIt from "markdown-it";

import { readGlobalRules, saveGlobalRules, toUserMessage } from "../api";

// ============================================================================
// 读写目标（注入点）
// ============================================================================

/**
 * 一份可编辑文档的读写通道。
 *
 * `file` 参数只在技能知识场景用到（逐篇文件）；全局规则读写的是固定文件，
 * 实现里忽略它即可。统一签名是为了让状态机对「编辑的是什么」无感知。
 */
export interface EditorIO {
  load: (file: string | null) => Promise<string>;
  save: (file: string | null, content: string) => Promise<void>;
}

/** 全局规则通道（默认值）：固定读写 `~/.ai-profile/global/rules.md`。 */
const globalRulesIO: EditorIO = {
  load: () => readGlobalRules(),
  save: (_file, content) => saveGlobalRules(content),
};

// ============================================================================
// 模板（PRD 第 5 节步骤 3 / 第 6.1 节共用）
// ============================================================================

export interface RuleTemplate {
  /** 按钮上显示的名字 */
  name: string;
  /** 点击后填入编辑器的正文 */
  body: string;
}

export const RULE_TEMPLATES: readonly RuleTemplate[] = [
  {
    name: "前端工程师",
    body: `# 我的工作偏好

## 沟通
- 用中文回答，先给结论，再给理由
- 改动代码时给出完整可运行的片段，不要只说「在这里改一下」

## 技术栈
- Vue 3 + TypeScript，组合式 API
- Tailwind CSS：优先用工具类，不轻易新增自定义 CSS
- 包管理统一用 pnpm

## 请特别注意
- 不确定的 API 直接说不确定，不要编造
- 改动涉及依赖升级时，先说明影响面再动手
`,
  },
  {
    name: "后端 / 全栈开发者",
    body: `# 我的工作偏好

## 沟通
- 用中文回答，先给结论，再给理由
- 遇到多种可行方案时列出取舍，而不是直接替我决定

## 技术栈
- 服务端：Node.js / Python，接口遵循 REST 约定
- 数据库：关系型优先，写查询时注意索引与 N+1 问题
- 部署：容器化，配置全部走环境变量

## 请特别注意
- 涉及数据迁移或删除的改动，先说明风险再动手
- 报错信息要能定位问题，但不要外泄敏感数据
`,
  },
  {
    name: "通用开发者",
    body: `# 我的工作偏好

## 沟通
- 用中文回答，先给结论，再给理由
- 不确定的地方直接说不确定，不要编造

## 工作方式
- 动手前先说明打算怎么做，完成后说明改了哪些文件
- 优先小步改动，一次只解决一个问题

## 代码质量
- 命名要表达意图，注释解释「为什么」而不是「做了什么」
- 不引入没有必要的依赖
`,
  },
];

// ============================================================================
// Markdown 渲染器（模块级单例，全应用共用一个实例）
// ============================================================================

/**
 * ⚠️ **保持默认 `html: false`，这是安全要求不是默认值随手抄。**
 *
 * `tauri.conf.json` 当前 `csp: null`——预览区若把用户内容里的
 * `<script>` / `onerror` 当真标签插进 DOM，脚本就会在 webview 里执行，
 * 而 webview 拥有 `invoke` 权限，等同本地文件任意读写。
 * markdown-it 默认把原始 HTML **转义成纯文本**（实测 `<b>` → `&lt;b&gt;`），
 * 因此无需再叠一层 sanitizer。若谁把 `html` 打开，必须同时补 CSP 与 sanitizer。
 */
const md = new MarkdownIt({ linkify: true });

// ============================================================================
// 保存结果
// ============================================================================

export interface SaveOutcome {
  ok: boolean;
  /** `ok === false` 时给用户的可读原因；成功时为空串 */
  error: string;
}

/**
 * 创建一个文档编辑状态机。
 *
 * @param io 读写通道。省略时 = 全局规则（TASK-11 的原始用法）；
 *            技能知识传入逐篇文件通道（`useKnowledgeEditor`）。
 */
export function useRuleEditor(io: EditorIO = globalRulesIO) {
  /**
   * 当前编辑的文件（技能知识 = `knowledge/` 下的文件名；全局规则 = `null`）。
   * `null` 且非知识场景时表示「还没读过磁盘」。
   */
  const file = ref<string | null>(null);

  /** 编辑框里的当前内容 */
  const content = ref("");

  /**
   * 与磁盘一致的内容基线。
   *
   * `null` = 还没成功读过磁盘（此时**不能**保存）。
   * 有了它，「未保存」的判断就是内容级的：用户改了又改回原样，不算未保存——
   * 比只记「有没有敲过键盘」准确，也顺带满足向导原本
   * 「没动过就不写盘、避免无谓改动文件时间」的要求。
   */
  const baseline = ref<string | null>(null);

  const loading = ref(false);
  const loadError = ref("");
  const saving = ref(false);
  /** 最近一次成功保存的时间标签（`HH:MM`），用于界面上的「已保存」提示 */
  const lastSavedAt = ref<string | null>(null);

  /** 是否已经拿到磁盘内容（拿到之前编辑与保存都不可用） */
  const loaded = computed(() => baseline.value !== null);

  /** 有未落盘的改动 */
  const dirty = computed(
    () => baseline.value !== null && content.value !== baseline.value,
  );

  /** 内容是否为空白（决定要不要显示模板快捷入口） */
  const isEmpty = computed(() => content.value.trim() === "");

  /** 预览区 HTML（已由 markdown-it 转义原始 HTML，见上方模块级单例的说明） */
  const previewHtml = computed(() => md.render(content.value));

  /** 真正发起读取（`load` 与 `openFile` 共用） */
  async function fetchInto(): Promise<void> {
    loading.value = true;
    loadError.value = "";
    try {
      const text = await io.load(file.value);
      content.value = text;
      baseline.value = text;
    } catch (e) {
      // 读不出来就不能放行保存（见模块文档），界面据此禁用编辑器并给出重试入口
      loadError.value = toUserMessage(e);
    } finally {
      loading.value = false;
    }
  }

  /**
   * 读取磁盘内容（全局规则场景）。只在第一次调用时真正读——
   * 后续再调会直接返回，避免把用户正在编辑的内容覆盖掉。
   */
  async function load(): Promise<void> {
    if (baseline.value !== null) return;
    await fetchInto();
  }

  /**
   * 打开指定的文件（技能知识场景）。
   *
   * 切换目标文件会**清空状态再读**——这里是有意覆盖内存里的未保存内容，
   * 因为「打开另一篇」必然放弃当前这篇的编辑现场；调用方（卡片列表）
   * 负责在切换前确认。同一文件重复调用直接返回，不重读。
   */
  async function openFile(fileName: string): Promise<void> {
    if (file.value === fileName && baseline.value !== null) return;
    file.value = fileName;
    baseline.value = null;
    content.value = "";
    lastSavedAt.value = null;
    await fetchInto();
  }

  /** 保存到磁盘。⚠️ **保存不触发同步**——「编辑」与「同步」是两个独立动作。 */
  async function save(): Promise<SaveOutcome> {
    // 基线还没建立：磁盘内容未知，写了就可能覆盖用户已有的内容
    if (baseline.value === null) {
      return {
        ok: false,
        error: "还没有读取到现有内容，为避免覆盖你的内容，暂不能保存。",
      };
    }

    // 没有改动就不写盘：白改一次文件时间，备份与同步的「变化检测」都会被搅浑
    if (!dirty.value) {
      return { ok: true, error: "" };
    }

    saving.value = true;
    try {
      await io.save(file.value, content.value);
      baseline.value = content.value;
      lastSavedAt.value = currentTimeLabel();
      return { ok: true, error: "" };
    } catch (e) {
      return { ok: false, error: toUserMessage(e) };
    } finally {
      saving.value = false;
    }
  }

  /**
   * 关闭当前文档，把状态机复位到「未打开」。
   *
   * 删除文件后必须调用：否则残留的基线会让「保存」继续对着一个
   * 已不存在的文件写。未保存内容同样被丢弃——调用方负责先确认。
   */
  function close(): void {
    file.value = null;
    baseline.value = null;
    content.value = "";
    loadError.value = "";
    lastSavedAt.value = null;
  }

  /** 套用一条模板（只在内容为空时入口可见，见 RuleEditor 的说明） */
  function applyTemplate(body: string): void {
    content.value = body;
  }

  return reactive({
    file,
    content,
    loading,
    loadError,
    saving,
    lastSavedAt,
    loaded,
    dirty,
    isEmpty,
    previewHtml,
    load,
    openFile,
    close,
    save,
    applyTemplate,
  });
}

export type RuleEditorState = ReturnType<typeof useRuleEditor>;

/** `HH:MM`，与状态栏「上次同步」的展示格式保持一致的粒度 */
function currentTimeLabel(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(now.getHours())}:${pad(now.getMinutes())}`;
}

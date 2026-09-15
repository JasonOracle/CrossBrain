<script setup lang="ts">
/**
 * RuleEditor.vue — Markdown 左右分栏编辑器（TASK-11 引入，TASK-12 泛化）。
 *
 * 左边写 Markdown 源文，右边实时预览。全局规则页与技能知识页共用这一个组件，
 * 差异全部走 props：模板列表、底部说明、占位文案，以及知识页独有的
 * 「超 2000 字」黄色警告条（PRD §6.2）。
 *
 * 验收项写的是「编辑区和预览区同时可见（左右分栏 或 切换 Tab 均可）」——
 * 这两个说法是矛盾的（切 Tab 就不可能同时可见），按**主句**「同时可见」实现分栏，
 * 理由与后果见 `TASK_BREAKDOWN.md` 的 TASK-11 实施修正。
 *
 * # 为什么编辑框是原生 `<textarea>` 而不是 `n-input`
 *
 * 分栏要求两个窗格**各自滚动、共用一个固定高度**。
 * `n-input` 的 textarea 高度由其内部样式决定（autosize 或固定 rows），
 * 要撑满 flex 窗格得去和组件内部结构较劲，组件一升级就碎。
 * 原生元素 + Tailwind 类完全可控，焦点态样式也对齐项目令牌。
 *
 * # 模板快捷入口为什么只在内容为空时出现
 *
 * 打开的是用户**已经保存过的内容**。模板一点就会整篇替换——
 * 在向导里这是「帮你起步」，在这里是**静默覆盖用户的内容**（数据丢失）。
 * 干脆只在空白时给入口，从机制上消灭这种误触，而不是靠一个确认弹窗兜底。
 */
import { computed } from "vue";

import IconFileText from "~icons/lucide/file-text";
import IconRotateCw from "~icons/lucide/rotate-cw";
import IconSave from "~icons/lucide/save";
import IconSparkles from "~icons/lucide/sparkles";

import { RULE_TEMPLATES, type RuleEditorState, type RuleTemplate } from "../composables/useRuleEditor";

const props = withDefaults(
  defineProps<{
    editor: RuleEditorState;
    /** 模板快捷入口（传空数组 = 不显示入口；知识页新建自带模板，无需再给） */
    templates?: readonly RuleTemplate[];
    /** 底部常驻说明 */
    footnote?: string;
    /** 编辑框占位文案 */
    placeholder?: string;
    /** 内容超过 2000 字时是否显示黄色警告条（PRD §6.2，技能知识专用） */
    lengthWarning?: boolean;
  }>(),
  {
    // 数组/对象类型的默认值必须是工厂函数（Vue 的要求），返回共享常量即可
    templates: () => RULE_TEMPLATES,
    footnote:
      "左侧写 Markdown，右侧实时预览。改动只保存在本机，要点右上角的「立即同步」才会推送到各个工具。",
    placeholder:
      "在这里写下你希望所有 AI 工具遵守的规则，例如：用中文回答，先给结论再给理由……",
    lengthWarning: false,
  },
);

const emit = defineEmits<{
  /** 保存成功（含「本来就没有改动」的空保存） */
  saved: [];
  /** 保存失败，载荷是用户可读原因 */
  "save-failed": [message: string];
}>();

/** PRD §6.2：技能知识超过 2000 字提示拆分 */
const KNOWLEDGE_LENGTH_LIMIT = 2000;

/** 内容为空白时才显示模板入口（见文件头的说明） */
const showTemplates = computed(
  () => props.templates.length > 0 && props.editor.loaded && props.editor.isEmpty,
);

/** 超长警告：只看当前内容，保存与否都不影响提示的语义 */
const overLimit = computed(
  () => props.lengthWarning && props.editor.content.length > KNOWLEDGE_LENGTH_LIMIT,
);

async function doSave(): Promise<void> {
  const outcome = await props.editor.save();
  if (outcome.ok) {
    emit("saved");
  } else {
    emit("save-failed", outcome.error);
  }
}

/**
 * 预览区里的链接一律不许真的跳转——
 * 在 webview 里导航会把整个应用带离界面。只拦截点击，不影响选中复制。
 */
function onPreviewClick(event: MouseEvent): void {
  event.preventDefault();
}

/** Ctrl+S / Cmd+S 保存，同时拦掉浏览器默认的「存储网页」行为 */
function onKeydown(event: KeyboardEvent): void {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
    event.preventDefault();
    void doSave();
  }
}
</script>

<template>
  <div class="flex h-[60vh] min-h-[420px] flex-col gap-3">
    <!-- ── 工具条：保存状态 + 保存按钮 ── -->
    <div class="flex shrink-0 items-center gap-3">
      <span class="flex items-center gap-1.5 text-xs text-neutral-500">
        <IconFileText class="h-3.5 w-3.5" aria-hidden="true" />
        <span v-if="editor.saving">正在保存……</span>
        <span v-else-if="editor.dirty" class="text-warning-500">有未保存的修改</span>
        <span v-else-if="editor.lastSavedAt">已保存 · {{ editor.lastSavedAt }}</span>
        <span v-else>没有改动</span>
      </span>

      <span v-if="lengthWarning" class="text-xs text-neutral-400">
        {{ editor.content.length }} 字
      </span>

      <n-button
        class="ml-auto"
        size="small"
        type="primary"
        :loading="editor.saving"
        :disabled="!editor.loaded"
        @click="doSave"
      >
        <template #icon>
          <IconSave v-if="!editor.saving" class="h-4 w-4" aria-hidden="true" />
        </template>
        保存
      </n-button>
    </div>

    <!-- ── 读取出错：禁止编辑与保存，给出重试 ── -->
    <n-alert v-if="editor.loadError" type="error" :bordered="false">
      <div class="flex flex-col gap-2">
        <span>{{ editor.loadError }}</span>
        <span class="text-xs text-neutral-500">
          为避免覆盖你已有的内容，在确认能读到原内容之前，编辑与保存都已停用。
        </span>
        <div>
          <n-button size="small" @click="editor.load()">
            <template #icon>
              <IconRotateCw class="h-4 w-4" aria-hidden="true" />
            </template>
            重试
          </n-button>
        </div>
      </div>
    </n-alert>

    <n-spin
      v-else
      class="min-h-0 flex-1"
      :show="editor.loading"
      :content-style="{ height: '100%' }"
    >
      <!-- ── 超长警告条（PRD §6.2，技能知识专用）── -->
      <n-alert v-if="overLimit" type="warning" class="mb-3" :bordered="false">
        此技能知识偏长，建议按主题拆分为多篇，有助于 AI 精准加载
      </n-alert>

      <!-- ── 模板快捷入口（仅空白内容时） ── -->
      <div v-if="showTemplates" class="mb-3 flex flex-wrap items-center gap-2">
        <span class="flex items-center gap-1 text-xs text-neutral-500">
          <IconSparkles class="h-3.5 w-3.5 text-brand-500" aria-hidden="true" />
          从模板开始：
        </span>
        <button
          v-for="tpl in templates"
          :key="tpl.name"
          type="button"
          class="rounded-lg border border-neutral-200 bg-white px-3 py-1.5 text-xs text-neutral-700 transition hover:border-brand-300 hover:bg-brand-50"
          @click="editor.applyTemplate(tpl.body)"
        >
          {{ tpl.name }}
        </button>
      </div>

      <!-- ── 左右分栏：源文 / 预览，两窗格各自滚动 ── -->
      <div class="grid h-full grid-cols-1 gap-4 lg:grid-cols-2">
        <textarea
          :value="editor.content"
          class="h-full w-full resize-none rounded-lg border border-neutral-200 bg-white p-4 font-mono text-sm leading-6 text-neutral-800 outline-none transition focus:border-brand-400 focus:ring-2 focus:ring-brand-500/20"
          :placeholder="placeholder"
          spellcheck="false"
          @input="editor.content = ($event.target as HTMLTextAreaElement).value"
          @keydown="onKeydown"
        />

        <div
          class="md-preview h-full overflow-auto rounded-lg border border-neutral-200 bg-white p-4"
          aria-label="预览"
          @click.capture="onPreviewClick"
          v-html="editor.previewHtml"
        />
      </div>
    </n-spin>

    <p class="shrink-0 text-xs text-neutral-500">{{ footnote }}</p>
  </div>
</template>

<script setup lang="ts">
/**
 * KnowledgeManager.vue — 技能知识卡片列表 + 逐篇编辑（TASK-12）。
 *
 * 「技能知识」Tab 的全部内容都在这里：列表视图（卡片网格）与编辑视图
 * （复用 `RuleEditor.vue`）之间切换，外面只看得到这个组件。
 *
 * # 卡片必须同时展示「源文件名」与「注入后的目录名」（PRD §6.2 / ADR-11）
 *
 * V1 不做拼音转换，纯中文标题的目录名会退化成 `crossbrain-item-{hash}`，
 * 只看目录名无法对应回源文件；同时让「这篇知识被推送到了哪里」对用户可查。
 *
 * # 打开另一篇 / 返回列表前为什么要确认
 *
 * `useRuleEditor.openFile()` 会覆盖内存里的未保存内容（这是它的职责），
 * 所以「离开当前编辑现场」的确认放在这一层——状态机只管数据，交互兜底在视图。
 */
import { computed, onMounted, ref } from "vue";
import { useMessage } from "naive-ui";

import IconArrowLeft from "~icons/lucide/arrow-left";
import IconBookOpen from "~icons/lucide/book-open";
import IconPlus from "~icons/lucide/plus";
import IconTrash from "~icons/lucide/trash";

import {
  createKnowledge,
  deleteKnowledge,
  listKnowledge,
  readKnowledge,
  saveKnowledge,
  toUserMessage,
  type KnowledgeCard,
} from "../api";
import RuleEditor from "./RuleEditor.vue";
import { useRuleEditor, type EditorIO } from "../composables/useRuleEditor";

const message = useMessage();

/** 技能知识的读写通道：按文件名逐篇读写 `~/.ai-profile/knowledge/`。 */
const knowledgeIO: EditorIO = {
  load: (file) => readKnowledge(file ?? ""),
  save: (file, content) => saveKnowledge(file ?? "", content),
};
const editor = useRuleEditor(knowledgeIO);

// ── 列表状态 ──
const cards = ref<KnowledgeCard[]>([]);
const loading = ref(true);
const loadError = ref("");

/** 列表 / 编辑 两个视图的切换 */
const view = ref<"list" | "edit">("list");

/** 编辑器当前标题（来自卡片；新建后即时补上，不必等列表刷新） */
const currentTitle = computed(() => {
  const name = editor.file;
  if (!name) return "";
  return cards.value.find((c) => c.fileName === name)?.title ?? name.replace(/\.md$/i, "");
});

onMounted(() => {
  void refresh();
});

async function refresh(): Promise<void> {
  loading.value = true;
  loadError.value = "";
  try {
    cards.value = await listKnowledge();
  } catch (e) {
    loadError.value = toUserMessage(e);
  } finally {
    loading.value = false;
  }
}

// ── 未保存现场的确认兜底 ──

/** 非 null 时弹「放弃未保存修改」确认框；确认后执行的动作 */
const pendingDiscard = ref<(() => void) | null>(null);

function confirmDiscard(): void {
  const action = pendingDiscard.value;
  pendingDiscard.value = null;
  action?.();
}

function cancelDiscard(): void {
  pendingDiscard.value = null;
}

/** 带兜底地打开一篇：同篇直开；换篇且有未保存修改时先确认 */
function openCard(card: KnowledgeCard): void {
  if (editor.file === card.fileName && editor.loaded) {
    view.value = "edit";
    return;
  }
  const open = (): void => {
    view.value = "edit";
    void editor.openFile(card.fileName).then(() => {
      if (editor.loadError) message.error(editor.loadError);
    });
  };
  if (editor.dirty) {
    pendingDiscard.value = open;
  } else {
    open();
  }
}

/** 返回列表：有未保存修改时先确认 */
function backToList(): void {
  const go = (): void => {
    view.value = "list";
    void refresh();
  };
  if (editor.dirty) {
    pendingDiscard.value = go;
  } else {
    go();
  }
}

// ── 新建 ──

const showCreate = ref(false);
const newTitle = ref("");
const creating = ref(false);

function askCreate(): void {
  newTitle.value = "";
  showCreate.value = true;
}

async function confirmCreate(): Promise<void> {
  const title = newTitle.value.trim();
  if (!title) {
    message.warning("请先填一个标题，它会决定文件名。");
    return;
  }
  creating.value = true;
  try {
    const fileName = await createKnowledge(title);
    showCreate.value = false;
    await refresh();
    view.value = "edit";
    await editor.openFile(fileName);
    message.success("已创建，写完记得保存。");
  } catch (e) {
    message.error(toUserMessage(e));
  } finally {
    creating.value = false;
  }
}

// ── 保存 / 删除 ──

function onSaved(): void {
  message.success("已保存到本机，要点右上角「立即同步」才会推送到各工具。");
  // 摘要与字数可能已变，列表数据同步刷新
  void refresh();
}

function onSaveFailed(reason: string): void {
  message.error(reason);
}

/** 待确认删除的卡片；非 null 即弹确认框 */
const pendingDelete = ref<KnowledgeCard | null>(null);

async function confirmDelete(): Promise<void> {
  const card = pendingDelete.value;
  if (!card) return;
  pendingDelete.value = null;

  try {
    // 后端返回的是面向用户的成功说明（含「工具内副本何时清理」）
    message.success(await deleteKnowledge(card.fileName));
    if (editor.file === card.fileName) {
      view.value = "list";
      // 文件已不存在：把状态机复位到「未打开」，防止后续保存写向空文件
      editor.close();
    }
    await refresh();
  } catch (e) {
    message.error(toUserMessage(e));
  }
}
</script>

<template>
  <section class="flex flex-col gap-4">
    <!-- ══════════ 列表视图 ══════════ -->
    <template v-if="view === 'list'">
      <div class="flex items-center gap-3">
        <div>
          <h2 class="text-base font-medium text-neutral-900">技能知识</h2>
          <p class="mt-1 text-sm leading-6 text-neutral-500">
            每一篇是一个具体主题，同步时会作为独立技能推送到各个工具。一篇只讲一件事，讲透它。
          </p>
        </div>
        <n-button class="ml-auto" size="small" type="primary" @click="askCreate">
          <template #icon>
            <IconPlus class="h-4 w-4" aria-hidden="true" />
          </template>
          新建技能
        </n-button>
      </div>

      <div v-if="loading" class="text-sm text-neutral-500">正在读取技能知识……</div>

      <n-alert v-else-if="loadError" type="error" :bordered="false">
        <div class="flex flex-col gap-2">
          <span>{{ loadError }}</span>
          <div>
            <n-button size="small" @click="refresh">重试</n-button>
          </div>
        </div>
      </n-alert>

      <div
        v-else-if="cards.length === 0"
        class="flex flex-col items-center gap-3 rounded-xl border border-dashed border-neutral-200 bg-white p-10 text-sm text-neutral-500"
      >
        <IconBookOpen class="h-8 w-8 text-neutral-300" aria-hidden="true" />
        <span>还没有技能知识。写第一篇，比如「Rust · 生命周期常见错误」。</span>
        <n-button size="small" type="primary" @click="askCreate">新建技能</n-button>
      </div>

      <div v-else class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <button
          v-for="card in cards"
          :key="card.fileName"
          type="button"
          class="group rounded-xl border border-neutral-200 bg-white p-4 text-left transition hover:border-brand-300 hover:shadow-sm"
          @click="openCard(card)"
        >
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="truncate text-sm font-medium text-neutral-900 group-hover:text-brand-600">
                {{ card.title }}
              </div>
              <div class="mt-1 truncate text-xs text-neutral-500">
                {{ card.summary || "（空文档）" }}
              </div>
            </div>
            <span
              class="shrink-0 rounded-md p-1.5 text-neutral-400 transition hover:bg-error-500/10 hover:text-error-500"
              role="button"
              :aria-label="`删除 ${card.title}`"
              @click.stop="pendingDelete = card"
            >
              <IconTrash class="h-4 w-4" aria-hidden="true" />
            </span>
          </div>

          <!-- 目录名必须可见：ADR-11 的可读性补偿 + 排障入口（PRD §6.2） -->
          <div class="mt-3 border-t border-neutral-100 pt-2">
            <p class="truncate font-mono text-[11px] text-neutral-400" :title="card.dirName">
              {{ card.dirName }}
            </p>
            <p class="mt-1 text-[11px] text-neutral-400">
              {{ card.charCount }} 字 · 修改于 {{ card.modifiedAt ?? "未知时间" }}
            </p>
          </div>
        </button>
      </div>
    </template>

    <!-- ══════════ 编辑视图 ══════════ -->
    <template v-else>
      <div class="flex items-center gap-3">
        <n-button size="small" quaternary @click="backToList">
          <template #icon>
            <IconArrowLeft class="h-4 w-4" aria-hidden="true" />
          </template>
          返回列表
        </n-button>
        <span class="min-w-0 truncate text-sm font-medium text-neutral-900">
          {{ currentTitle }}
        </span>
        <span class="ml-auto truncate font-mono text-[11px] text-neutral-400" :title="editor.file ?? ''">
          {{ editor.file }}
        </span>
      </div>

      <RuleEditor
        :editor="editor"
        :templates="[]"
        length-warning
        placeholder="用 Markdown 写这篇技能知识。第一行写成标题，例如：# Rust · 生命周期常见错误"
        footnote="改动只保存在本机。要点右上角的「立即同步」，这篇知识才会作为技能推送到各个工具。"
        @saved="onSaved"
        @save-failed="onSaveFailed"
      />
    </template>

    <!-- ── 新建：标题决定文件名，必须让用户知道 ── -->
    <n-modal
      :show="showCreate"
      preset="card"
      style="width: 460px"
      title="新建技能知识"
      :mask-closable="false"
      @update:show="(v: boolean) => (showCreate = v)"
    >
      <div class="flex flex-col gap-2 text-sm leading-6 text-neutral-700">
        <p>给这篇技能知识起个标题：</p>
        <n-input
          v-model:value="newTitle"
          placeholder="例如：Rust · 生命周期与借用检查器常见错误"
          autofocus
          @keydown.enter="confirmCreate"
        />
        <p class="text-xs text-neutral-500">
          文件名会由标题自动生成（中文会被折算，重名自动加序号）。第一行会预填
          <code># [技术/语言] · [具体场景]</code>，改成你自己的主题即可。
        </p>
      </div>
      <template #footer>
        <div class="flex justify-end gap-2">
          <n-button :disabled="creating" @click="showCreate = false">取消</n-button>
          <n-button type="primary" :loading="creating" @click="confirmCreate">创建</n-button>
        </div>
      </template>
    </n-modal>

    <!-- ── 删除的二次确认 ── -->
    <n-modal
      :show="pendingDelete !== null"
      preset="card"
      style="width: 460px"
      title="确认删除这篇技能知识？"
      :mask-closable="false"
      @update:show="(v: boolean) => !v && (pendingDelete = null)"
    >
      <div class="flex flex-col gap-3 text-sm leading-6 text-neutral-700">
        <p>
          将删除 <span class="font-medium text-neutral-900">{{ pendingDelete?.title }}</span>
          （<code class="break-all">{{ pendingDelete?.fileName }}</code>）。
        </p>
        <p class="text-xs text-neutral-500">
          各个工具里已经生成的技能副本不会立刻消失——会在你下次点「立即同步」时自动清理。
          此操作无法从本软件内撤销。
        </p>
      </div>
      <template #footer>
        <div class="flex justify-end gap-2">
          <n-button @click="pendingDelete = null">取消</n-button>
          <n-button type="error" @click="confirmDelete">确认删除</n-button>
        </div>
      </template>
    </n-modal>

    <!-- ── 放弃未保存修改的确认 ── -->
    <n-modal
      :show="pendingDiscard !== null"
      preset="card"
      style="width: 440px"
      title="有未保存的修改"
      :mask-closable="false"
      @update:show="(v: boolean) => !v && cancelDiscard()"
    >
      <p class="text-sm leading-6 text-neutral-700">
        当前这篇还有没保存的修改，继续操作会丢弃它们。
      </p>
      <template #footer>
        <div class="flex justify-end gap-2">
          <n-button @click="cancelDiscard">留下来继续编辑</n-button>
          <n-button type="warning" @click="confirmDiscard">丢弃修改并继续</n-button>
        </div>
      </template>
    </n-modal>
  </section>
</template>

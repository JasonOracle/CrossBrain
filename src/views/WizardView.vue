<script setup lang="ts">
/**
 * WizardView.vue — 首次运行 5 步向导。
 *
 * 依据 `docs/product/PRD.md` 第 5 节。三条设计原则贯穿全文件：
 * 1. **每一步只问一件事，每个按钮只做一件事**
 * 2. **不出现任何内部术语**（不出现 L0 / L2 / SSOT / slug / Adapter 等词）
 * 3. **所有错误文案面向用户**：绝不显示系统错误码或技术路径
 *
 * 图标一律用 lucide 的 SVG（项目规范禁止 Emoji）。
 */
import { computed, onMounted, onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";

import IconArrowRight from "~icons/lucide/arrow-right";
import IconCheck from "~icons/lucide/check";
import IconClock from "~icons/lucide/clock";
import IconCopy from "~icons/lucide/copy";
import IconMinus from "~icons/lucide/minus";
import IconRefresh from "~icons/lucide/refresh-cw";
import IconShield from "~icons/lucide/shield-check";
import IconSparkles from "~icons/lucide/sparkles";

import {
  UPCOMING_TOOLS,
  completeWizard,
  copyToClipboard,
  detectTools,
  onSyncProgress,
  readGlobalRules,
  runSync,
  saveGlobalRules,
  toUserMessage,
  type StartupState,
  type SyncReport,
  type ToolInfo,
  type ToolSyncResult,
} from "../api";
import LinkNoticeDialog from "../components/LinkNoticeDialog.vue";
import { useLinkNotice } from "../composables/useLinkNotice";
import { RULE_TEMPLATES } from "../composables/useRuleEditor";

/** 启动状态：只需其中的「断链说明是否已展示过」 */
const props = defineProps<{ startup: StartupState | null }>();

const emit = defineEmits<{ done: [] }>();

// ============================================================================
// 状态
// ============================================================================

/** 当前步骤（1~5） */
const step = ref(1);

/** 步骤 2：工具检测结果 */
const tools = ref<ToolInfo[]>([]);
const detecting = ref(false);
const detectError = ref("");

/** 步骤 3：全局规则正文 */
const rules = ref("");
/** 用户是否真的动过规则内容——没动过就不写盘，避免无谓地改动文件时间 */
const rulesTouched = ref(false);

/** 步骤 4：同步状态 */
const syncing = ref(false);
const progressLines = ref<ToolSyncResult[]>([]);
const report = ref<SyncReport | null>(null);
const syncError = ref("");
let unlisten: UnlistenFn | null = null;

/**
 * 断链说明闸门（TASK-19 / ADR-15）。
 *
 * 首次同步会断开 `CLAUDE.md` 与其他位置的共享关系，必须事前告知一次。
 * 闸门逻辑在 `useLinkNotice` 里——向导与主工作台两处入口各自实现一遍的话，
 * 「确认之后才标记已展示」这条顺序约束迟早会在某一侧写错。
 */
const linkNotice = useLinkNotice(props.startup?.linkNoticeShown ?? false);
/** 解构出来才有模板自动解包（嵌套在对象里的 ref 在模板中不会自动解包） */
const { open: linkNoticeOpen } = linkNotice;

/** 步骤 5：复制反馈 */
const copied = ref(false);

/** 网络状态（仅作提示，同步完全在本机进行） */
const online = ref(typeof navigator === "undefined" ? true : navigator.onLine);

// ============================================================================
// 规则模板（PRD 第 5 节步骤 3）
//
// 正文不在本文件里：主工作台的规则编辑器（TASK-11）用同一组模板，
// 复制一份迟早分叉，所以统一取自 `useRuleEditor.ts` 的导出常量。
// ============================================================================

const templates = RULE_TEMPLATES;

/** 步骤 5 的验证提示词 */
const VERIFY_PROMPT = "请用一句话概括你当前遵守的全局编码偏好。";

// ============================================================================
// 派生状态
// ============================================================================

/** 同步摘要（PRD 步骤 4：「已将 1 条全局规则同步至 X 个工具」） */
const syncSummary = computed(() => {
  const r = report.value;
  if (!r) return "";
  const okCount = r.tools.filter((t) => t.status === "ok").length;
  if (okCount === 0) {
    return "本次没有同步到任何工具，请确认至少已安装一个受支持的工具。";
  }
  const rulesPart = r.rulesSynced ? "1 条全局规则" : "0 条全局规则";
  const skillPart = r.skillCount > 0 ? `与 ${r.skillCount} 条技能知识` : "";
  return `已将 ${rulesPart}${skillPart}同步至 ${okCount} 个工具`;
});

/** 是否可以从步骤 4 继续（同步已有结果，无论成败） */
const canLeaveStep4 = computed(() => report.value !== null && !syncing.value);

// ============================================================================
// 行为
// ============================================================================

onMounted(async () => {
  window.addEventListener("online", updateOnline);
  window.addEventListener("offline", updateOnline);
  await refreshTools();
});

onUnmounted(() => {
  window.removeEventListener("online", updateOnline);
  window.removeEventListener("offline", updateOnline);
  // 必须取消订阅：否则反复进出向导会累积监听器
  unlisten?.();
  unlisten = null;
});

function updateOnline() {
  online.value = navigator.onLine;
}

async function refreshTools() {
  detecting.value = true;
  detectError.value = "";
  try {
    tools.value = await detectTools();
  } catch (e) {
    detectError.value = toUserMessage(e);
  } finally {
    detecting.value = false;
  }
}

function applyTemplate(body: string) {
  rules.value = body;
  rulesTouched.value = true;
}

function onRulesInput(value: string) {
  rules.value = value;
  rulesTouched.value = true;
}

/** 读取用户此前可能已写过的规则（首次进入步骤 3 时回填） */
async function loadRulesOnce() {
  if (rulesTouched.value) return;
  try {
    rules.value = await readGlobalRules();
  } catch {
    // 读不到就当作空——用户重新写即可，不必打断流程
  }
}

function goNext() {
  if (step.value === 2) {
    void loadRulesOnce();
  }
  step.value += 1;
}

function goBack() {
  if (step.value > 1 && !syncing.value) {
    step.value -= 1;
  }
}

/** 保存规则（仅在用户真的编辑过时写盘） */
async function persistRules() {
  if (!rulesTouched.value) return;
  await saveGlobalRules(rules.value);
}

/**
 * 用户点了「开始同步」/「重新同步」。
 *
 * 先过断链说明闸门：已告知过就直接同步，否则弹一次说明、用户确认后再同步。
 * 门是用户显式点开的，所以这里不做任何「自动继续」的兜底——
 * 他点「先不同步」就该什么都不发生。
 */
function startSync() {
  if (syncing.value) return;
  linkNotice.guard(() => {
    void doSync();
  });
}

async function doSync() {
  syncing.value = true;
  syncError.value = "";
  progressLines.value = [];
  report.value = null;

  try {
    // 先落盘再同步：同步流程是从本机配置目录读取内容的
    await persistRules();

    // 进度来自事件流，而不是 runSync() 的返回值——
    // 这样才能做到「逐行实时显示」而不是最后一次性呈现
    unlisten = await onSyncProgress((progress) => {
      progressLines.value = [...progressLines.value, progress];
    });

    const result = await runSync();
    report.value = result;
    if (result.error) {
      syncError.value = result.error;
    }
  } catch (e) {
    syncError.value = toUserMessage(e);
  } finally {
    unlisten?.();
    unlisten = null;
    syncing.value = false;
  }
}

async function copyPrompt() {
  const ok = await copyToClipboard(VERIFY_PROMPT);
  copied.value = ok;
  if (!ok) {
    // 复制失败不该阻断流程，提示用户手动选中即可
    syncError.value = "";
  }
}

/** 结束向导：保存规则 + 记录「向导已完成」，然后进入主界面 */
async function finish() {
  try {
    await persistRules();
  } catch {
    // 规则保存失败不阻断进入主界面——用户可在编辑器里重新保存
  }
  try {
    await completeWizard();
  } catch {
    // 记录失败的最坏后果是下次再显示一次向导，不该把用户挡在这里
  }
  emit("done");
}

// ============================================================================
// 小工具
// ============================================================================

function lineIcon(status: ToolSyncResult["status"]) {
  if (status === "ok") return IconCheck;
  if (status === "failed") return IconMinus;
  return IconClock;
}

function lineClass(status: ToolSyncResult["status"]) {
  if (status === "ok") return "text-success-500";
  if (status === "failed") return "text-error-500";
  return "text-neutral-400";
}
</script>

<template>
  <div class="flex min-h-screen flex-col bg-neutral-50">
    <!-- ── 顶部：品牌与步骤 ── -->
    <header class="border-b border-neutral-200 bg-white">
      <div class="mx-auto flex max-w-3xl flex-col gap-5 px-8 py-6">
        <div class="flex items-center gap-2 text-lg font-semibold text-neutral-900">
          <IconSparkles class="h-5 w-5 text-brand-500" aria-hidden="true" />
          CrossBrain
        </div>
        <n-steps :current="step" size="small">
          <n-step title="欢迎" />
          <n-step title="检测工具" />
          <n-step title="写规则" />
          <n-step title="同步" />
          <n-step title="验证" />
        </n-steps>
      </div>
    </header>

    <!-- ── 主体 ── -->
    <main class="mx-auto flex w-full max-w-3xl flex-1 flex-col px-8 py-10">
      <!-- 步骤 1：欢迎 -->
      <section v-if="step === 1" class="flex flex-1 flex-col justify-center gap-6">
        <h1 class="text-3xl leading-snug font-semibold text-neutral-900">
          CrossBrain 让你的 AI 配置跨所有工具同步生效
        </h1>
        <p class="text-base leading-7 text-neutral-600">
          写一次，处处可用。全程在本机运行，无需联网。
        </p>
        <ul class="flex flex-col gap-3 text-sm text-neutral-600">
          <li class="flex items-center gap-2">
            <IconShield class="h-4 w-4 shrink-0 text-brand-500" aria-hidden="true" />
            内容只存在你自己的电脑上，不会上传到任何服务器
          </li>
          <li class="flex items-center gap-2">
            <IconCheck class="h-4 w-4 shrink-0 text-brand-500" aria-hidden="true" />
            只会写入 CrossBrain 自己的文件，不覆盖你已有的配置
          </li>
        </ul>
      </section>

      <!-- 步骤 2：检测工具 -->
      <section v-else-if="step === 2" class="flex flex-col gap-6">
        <div>
          <h2 class="text-xl font-semibold text-neutral-900">检测你的 AI 工具</h2>
          <p class="mt-2 text-sm leading-6 text-neutral-500">
            下面列出本机已安装的 AI 编程工具。未安装的不会参与本次同步，也不会报错。
          </p>
        </div>

        <n-spin :show="detecting">
          <div class="flex flex-col gap-3">
            <!-- 受支持且已安装 / 未安装 -->
            <div
              v-for="tool in tools"
              :key="tool.toolId"
              class="flex items-start gap-3 rounded-xl border border-neutral-200 bg-white p-4"
            >
              <IconCheck
                v-if="tool.installed"
                class="mt-0.5 h-5 w-5 shrink-0 text-success-500"
                aria-hidden="true"
              />
              <IconMinus
                v-else
                class="mt-0.5 h-5 w-5 shrink-0 text-neutral-300"
                aria-hidden="true"
              />

              <div class="flex min-w-0 flex-1 flex-col gap-1">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-neutral-900">
                    {{ tool.displayName }}
                  </span>
                  <span
                    class="rounded-md px-2 py-0.5 text-xs"
                    :class="
                      tool.installed
                        ? 'bg-success-500/10 text-success-500'
                        : 'bg-neutral-100 text-neutral-500'
                    "
                  >
                    {{ tool.installed ? "已安装" : "未安装" }}
                  </span>
                </div>

                <div v-if="tool.installed" class="text-xs text-neutral-500">
                  推送位置：<span class="font-mono">{{ tool.pushPath }}</span>
                </div>
                <div v-else class="text-xs text-neutral-400">
                  未检测到该工具，本次不参与同步
                </div>

                <div v-if="tool.error" class="text-xs text-error-500">
                  {{ tool.error }}
                </div>
              </div>
            </div>

            <!-- 即将支持 -->
            <div
              v-for="name in UPCOMING_TOOLS"
              :key="name"
              class="flex items-center gap-3 rounded-xl border border-dashed border-neutral-200 bg-neutral-50 p-4"
            >
              <IconClock class="h-5 w-5 shrink-0 text-neutral-300" aria-hidden="true" />
              <span class="text-sm text-neutral-500">{{ name }}</span>
              <span class="text-xs text-neutral-400">即将支持（不参与本次同步）</span>
            </div>
          </div>
        </n-spin>

        <n-alert v-if="detectError" type="warning" :bordered="false">
          {{ detectError }}
        </n-alert>

        <p v-if="!online" class="text-xs text-neutral-500">
          当前没有网络连接。同步完全在本机进行，不受影响。
        </p>
      </section>

      <!-- 步骤 3：写第一条规则 -->
      <section v-else-if="step === 3" class="flex flex-col gap-6">
        <div>
          <h2 class="text-xl font-semibold text-neutral-900">写你的第一条 AI 规则</h2>
          <p class="mt-2 text-sm leading-6 text-neutral-500">
            可以点左边的模板快速开始，也可以直接写。稍后还能随时修改。
          </p>
        </div>

        <div class="grid gap-4 md:grid-cols-[200px_1fr]">
          <div class="flex flex-col gap-2">
            <button
              v-for="tpl in templates"
              :key="tpl.name"
              type="button"
              class="rounded-lg border border-neutral-200 bg-white px-3 py-2.5 text-left text-sm text-neutral-700 transition hover:border-brand-300 hover:bg-brand-50"
              @click="applyTemplate(tpl.body)"
            >
              {{ tpl.name }}
            </button>
          </div>

          <div class="flex flex-col gap-2">
            <n-input
              :value="rules"
              type="textarea"
              :autosize="{ minRows: 12, maxRows: 18 }"
              placeholder="例如：用中文回答，先给结论再给理由……"
              @update:value="onRulesInput"
            />
            <p class="text-xs text-neutral-500">
              这段规则会在所有 AI 对话中自动生效，用于告知 AI
              你的编码偏好和工作习惯。留空也可以，之后随时补充。
            </p>
          </div>
        </div>
      </section>

      <!-- 步骤 4：一键同步 -->
      <section v-else-if="step === 4" class="flex flex-col gap-6">
        <div>
          <h2 class="text-xl font-semibold text-neutral-900">一键同步</h2>
          <p class="mt-2 text-sm leading-6 text-neutral-500">
            把刚才写的内容推送到上面检测到的工具。每次点击都会重新同步一遍，可以放心重复执行。
          </p>
        </div>

        <!-- 进度区：逐行实时显示 -->
        <div
          v-if="progressLines.length > 0 || syncing"
          class="flex flex-col gap-2 rounded-xl border border-neutral-200 bg-white p-4"
        >
          <div
            v-for="line in progressLines"
            :key="line.toolId"
            class="flex items-center gap-2 text-sm"
          >
            <component
              :is="lineIcon(line.status)"
              class="h-4 w-4 shrink-0"
              :class="lineClass(line.status)"
              aria-hidden="true"
            />
            <span class="text-neutral-800">{{ line.displayName }}</span>
            <span class="text-xs text-neutral-500">{{ line.message }}</span>
          </div>

          <div v-if="syncing" class="flex items-center gap-2 text-sm text-neutral-500">
            <IconRefresh class="h-4 w-4 shrink-0 animate-spin" aria-hidden="true" />
            正在同步……
          </div>
        </div>

        <!-- 完成摘要 -->
        <n-alert v-if="report && !report.error" :type="report.ok ? 'success' : 'warning'" :bordered="false">
          {{ syncSummary }}
        </n-alert>

        <n-alert v-if="syncError" type="error" :bordered="false">
          {{ syncError }}
        </n-alert>

        <div v-if="!syncing && !report">
          <n-button type="primary" size="large" @click="startSync">
            开始同步
          </n-button>
        </div>

        <div v-else-if="!syncing && report && !report.ok">
          <n-button size="large" @click="startSync">重新同步</n-button>
        </div>
      </section>

      <!-- 步骤 5：验证生效 -->
      <section v-else class="flex flex-col gap-6">
        <div>
          <h2 class="text-xl font-semibold text-neutral-900">验证生效（可选）</h2>
          <p class="mt-2 text-sm leading-6 text-neutral-500">
            在你的 AI 工具里新建一个对话，把下面这句话发给它，看看它是否已经读到你的规则。
          </p>
        </div>

        <div class="flex items-start gap-3 rounded-xl border border-neutral-200 bg-white p-4">
          <code class="min-w-0 flex-1 text-sm leading-6 break-words text-neutral-800">
            {{ VERIFY_PROMPT }}
          </code>
          <n-button size="small" @click="copyPrompt">
            <template #icon>
              <IconCopy class="h-4 w-4" aria-hidden="true" />
            </template>
            {{ copied ? "已复制" : "复制" }}
          </n-button>
        </div>

        <p class="text-xs text-neutral-500">
          没看到预期结果也没关系——规则已经写好，下次同步时会再次推送。
        </p>
      </section>
    </main>

    <!-- ── 底部：操作按钮 ── -->
    <footer class="border-t border-neutral-200 bg-white">
      <div class="mx-auto flex max-w-3xl items-center justify-between gap-4 px-8 py-4">
        <n-button v-if="step > 1" quaternary :disabled="syncing" @click="goBack">
          上一步
        </n-button>
        <span v-else />

        <div class="flex items-center gap-3">
          <!-- 步骤 1~3：单纯前进 -->
          <n-button
            v-if="step < 4"
            type="primary"
            size="large"
            @click="goNext"
          >
            <template #icon>
              <IconArrowRight class="h-4 w-4" aria-hidden="true" />
            </template>
            {{ step === 1 ? "开始配置" : "下一步" }}
          </n-button>

          <!-- 步骤 4：同步完成后才能继续 -->
          <n-button
            v-else-if="step === 4"
            type="primary"
            size="large"
            :disabled="!canLeaveStep4"
            @click="goNext"
          >
            <template #icon>
              <IconArrowRight class="h-4 w-4" aria-hidden="true" />
            </template>
            下一步
          </n-button>

          <!-- 步骤 5：两个出口 -->
          <template v-else>
            <n-button size="large" @click="finish">跳过，直接进入</n-button>
            <n-button type="primary" size="large" @click="finish">
              已验证，进入主界面
            </n-button>
          </template>
        </div>
      </div>
    </footer>

    <!-- 首次同步前的一次性告知（TASK-19 / ADR-15） -->
    <LinkNoticeDialog
      :show="linkNoticeOpen"
      @confirm="linkNotice.confirm"
      @cancel="linkNotice.cancel"
    />
  </div>
</template>

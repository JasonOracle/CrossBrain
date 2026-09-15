<script setup lang="ts">
/**
 * MainView.vue — 主工作台外壳。
 *
 * 顶部同步状态栏与「立即同步」来自 TASK-10（当时只交付外壳 + 占位）。
 * 三个功能区的归属：
 *
 * | 区域 | 归属任务 | 现状 |
 * |:---|:---|:---|
 * | 全局规则编辑器 | TASK-11 | ✅ 已实现（`RuleEditor.vue`） |
 * | 技能知识卡片列表 | TASK-12 | ✅ 已实现（`KnowledgeManager.vue`） |
 * | 设置 / 一键卸载 | TASK-14 | 占位中（备份与还原区已由 TASK-19 落地） |
 *
 * 之所以先做状态栏与「立即同步」，是因为向导结束后用户必须**看到同步结果**
 * 并且能再次触发同步——否则向导最后一步会把用户送进一个什么都不显示的空页面。
 */
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useMessage } from "naive-ui";
import type { UnlistenFn } from "@tauri-apps/api/event";

import IconCheck from "~icons/lucide/check";
import IconEraser from "~icons/lucide/eraser";
import IconMinus from "~icons/lucide/minus";
import IconRadar from "~icons/lucide/radar";
import IconRefresh from "~icons/lucide/refresh-cw";
import IconRotateCcw from "~icons/lucide/rotate-ccw";
import IconTrash from "~icons/lucide/trash";
import IconWifiOff from "~icons/lucide/wifi-off";

import {
  detectTools,
  listBackups,
  onSyncProgress,
  removeToolProbe,
  restoreBackup,
  runSync,
  runToolProbe,
  toUserMessage,
  uninstallCrossbrain,
  type BackupInfo,
  type ProbeOutcome,
  type StartupState,
  type SyncReport,
  type ToolInfo,
  type ToolSyncResult,
} from "../api";
import LinkNoticeDialog from "../components/LinkNoticeDialog.vue";
import KnowledgeManager from "../components/KnowledgeManager.vue";
import RuleEditor from "../components/RuleEditor.vue";
import { useLinkNotice } from "../composables/useLinkNotice";
import { useRuleEditor } from "../composables/useRuleEditor";

const props = defineProps<{ startup: StartupState | null }>();

/** 卸载完成 → 通知 App 重读本机状态、切回向导（TASK-14） */
const emit = defineEmits<{ uninstalled: [] }>();

const activeTab = ref("rules");

const lastSyncAt = ref(props.startup?.lastSyncAt ?? null);
const lastSyncOk = ref(props.startup?.lastSyncOk ?? false);

const syncing = ref(false);
const lines = ref<ToolSyncResult[]>([]);
const errorText = ref("");

const online = ref(typeof navigator === "undefined" ? true : navigator.onLine);
let unlisten: UnlistenFn | null = null;

/**
 * 断链说明闸门（TASK-19 / ADR-15）——与向导共用同一份逻辑。
 *
 * 向导里没同步过就直接进入主界面时，断链说明还没展示过，
 * 那么「立即同步」就是它的第一次机会，这里必须补上。
 */
const linkNotice = useLinkNotice(props.startup?.linkNoticeShown ?? false);
const { open: linkNoticeOpen } = linkNotice;

/** ── 「全局规则」页（TASK-11）── */
const message = useMessage();
const ruleEditor = useRuleEditor();

/** 保存成功：轻提示，不用弹窗（验收项明确要求 Toast） */
function onRulesSaved() {
  message.success("已保存到本机，要点右上角「立即同步」才会推送到各工具。");
}

function onRulesSaveFailed(reason: string) {
  message.error(reason);
}

/** ── 「设置 → 卸载」区（TASK-14）── */
/** 二次确认弹窗开关 */
const pendingUninstall = ref(false);
/** 防重复点击 */
const uninstalling = ref(false);
/** 卸载失败的提示（成功会直接切回向导，无需在此展示结果） */
const uninstallError = ref("");

/** 点「卸载」——先弹确认，不直接动手 */
function askUninstall() {
  uninstallError.value = "";
  pendingUninstall.value = true;
}

function cancelUninstall() {
  if (uninstalling.value) return;
  pendingUninstall.value = false;
}

/** 用户确认后才真正执行卸载 */
async function confirmUninstall() {
  if (uninstalling.value) return;
  pendingUninstall.value = false;
  uninstalling.value = true;
  uninstallError.value = "";

  try {
    await uninstallCrossbrain();
    // 成功提示挂在 provider 层，切回向导后依然可见
    message.success("已完成卸载。你的全局规则与技能知识保留在本机，随时可以重新开始。");
    emit("uninstalled");
  } catch (e) {
    uninstallError.value = toUserMessage(e);
  } finally {
    uninstalling.value = false;
  }
}

/** ── 「设置 → 备份与还原」区（TASK-19） ── */
const backups = ref<BackupInfo[]>([]);
/** 初始为 true：首帧就显示「读取中」，而不是先闪一下「没有备份」再变 */
const backupsLoading = ref(true);
const backupError = ref("");
/** 正在还原的工具标识：同时用于按钮 loading 与防重复点击 */
const restoringToolId = ref("");
/** 待确认还原的条目；非 null 即弹确认框 */
const pendingRestore = ref<BackupInfo | null>(null);
const restoreMessage = ref("");
const restoreError = ref("");

/** 状态栏上「上次同步」的展示值 */
const lastSyncLabel = computed(() => lastSyncAt.value ?? "尚未同步");

/**
 * 失败原因是否展开显示（PRD §7：「上次同步失败（点击可查看原因）」）。
 *
 * ⚠️ 原因**只保留在本次会话里**——启动状态只持久化了「是否成功」，
 * 没有持久化失败详情。重启后点开只会看到诚实提示，而不是编一段原因。
 */
const showSyncFailure = ref(false);

/** 各失败工具的面向用户文案（不含系统错误码，PRD §8） */
const syncFailures = computed(() =>
  lines.value.filter((l) => l.status === "failed").map((l) => `${l.displayName}：${l.message}`),
);

/** 各工具的状态图标：来自本次同步结果 */
const toolStates = computed(() => lines.value);

onMounted(() => {
  window.addEventListener("online", updateOnline);
  window.addEventListener("offline", updateOnline);
  void loadBackups();
  // 「全局规则」编辑器的首次读取：没有这一步，编辑器会一直停在
  // 「未读过磁盘」状态——内容显示为空、基线为 null、保存按钮被禁用，
  // 而界面上没有任何入口能触发 load()（TASK-15 走查发现的真实缺陷）
  void ruleEditor.load();
});

// 切到「设置」时重新读一次：用户可能刚在别处同步过，
// 用挂载时那一份旧数据会让他以为没有备份可还原。
watch(activeTab, (tab) => {
  if (tab === "settings") {
    void loadBackups();
    void loadProbeTools();
  }
});

onUnmounted(() => {
  window.removeEventListener("online", updateOnline);
  window.removeEventListener("offline", updateOnline);
  unlisten?.();
  unlisten = null;
});

function updateOnline() {
  online.value = navigator.onLine;
}

/**
 * 用户点了「立即同步」。
 *
 * 先过断链说明闸门：从未告知过就先弹一次说明，用户确认后才真的同步。
 * 用户点「先不同步」就该什么都不发生——不做任何自动继续的兜底。
 */
function syncNow() {
  if (syncing.value) return;
  linkNotice.guard(() => {
    void doSync();
  });
}

async function doSync() {
  syncing.value = true;
  errorText.value = "";
  lines.value = [];

  // 报告提到 try 外面：finally 里要按它判定「上次同步是否成功」
  let report: SyncReport | null = null;

  try {
    unlisten = await onSyncProgress((progress) => {
      lines.value = [...lines.value, progress];
    });

    report = await runSync();
    // 进度事件逐条到达后，用最终报告覆盖一次：报告是权威结果，
    // 免得某个工具的事件迟到或丢失导致状态栏缺一行
    lines.value = report.tools;
    if (report.error) {
      errorText.value = report.error;
    }
  } catch (e) {
    errorText.value = toUserMessage(e);
  } finally {
    unlisten?.();
    unlisten = null;
    syncing.value = false;
    lastSyncAt.value = currentLocalTime();
    // report 为 null = 请求本身没走通（异常分支），必然不算成功
    lastSyncOk.value = !errorText.value && report?.ok === true;
    // 新一次同步开始了：上一次的失败详情收起，避免展示过期原因
    showSyncFailure.value = false;
    // 同步可能刚产生了备份，顺手刷新——否则设置区要等下次切标签才更新
    void loadBackups();
  }
}

// ============================================================================
// 备份与还原（TASK-19 / ADR-15）
// ============================================================================

async function loadBackups() {
  backupsLoading.value = true;
  backupError.value = "";
  try {
    backups.value = await listBackups();
  } catch (e) {
    backupError.value = toUserMessage(e);
  } finally {
    backupsLoading.value = false;
  }
}

/** 点「还原」——先弹确认，不直接动手 */
function askRestore(item: BackupInfo) {
  restoreMessage.value = "";
  restoreError.value = "";
  pendingRestore.value = item;
}

function cancelRestore() {
  pendingRestore.value = null;
}

async function confirmRestore() {
  const item = pendingRestore.value;
  if (!item) return;

  pendingRestore.value = null;
  restoringToolId.value = item.toolId;
  restoreMessage.value = "";
  restoreError.value = "";

  try {
    // 成功时后端会返回一句面向用户的说明（含另存的路径）
    restoreMessage.value = await restoreBackup(item.toolId);
  } catch (e) {
    restoreError.value = toUserMessage(e);
  } finally {
    restoringToolId.value = "";
    // 还原会改变文件内容，备份状态可能随之变化，重新读一次
    void loadBackups();
  }
}

/** 把字节数换成人类可读的大小（备份都是文本，几十 KB 量级） */
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

// ============================================================================
// 工具读取检测（探针）
// ============================================================================

/** 支持的工具清单（复用向导的检测结果，含安装状态） */
const probeTools = ref<ToolInfo[]>([]);
const probeToolsLoaded = ref(false);
/** 各工具的探针结论，键为 toolId；无键 = 还没测过 */
const probeResults = ref<Record<string, ProbeOutcome>>({});
/** 正在检测的工具（同时承担按钮 loading 与全局防重复） */
const probingToolId = ref("");
/** 正在移除探针的工具 */
const removingProbeToolId = ref("");

async function loadProbeTools() {
  try {
    probeTools.value = await detectTools();
  } catch {
    // 清单读不出来不阻断检测区——留空即可，用户重进设置页会再试
    probeTools.value = [];
  } finally {
    probeToolsLoaded.value = true;
  }
}

async function startProbe(toolId: string) {
  if (probingToolId.value) return;
  probingToolId.value = toolId;
  const next = { ...probeResults.value };
  delete next[toolId];
  probeResults.value = next;

  try {
    const outcome = await runToolProbe(toolId);
    probeResults.value = { ...probeResults.value, [toolId]: outcome };
  } catch (e) {
    probeResults.value = {
      ...probeResults.value,
      [toolId]: {
        toolId,
        displayName: "",
        status: "failed",
        token: "",
        message: toUserMessage(e),
      },
    };
  } finally {
    probingToolId.value = "";
  }
}

async function clearProbe(toolId: string) {
  if (removingProbeToolId.value) return;
  removingProbeToolId.value = toolId;
  try {
    message.success(await removeToolProbe(toolId));
    const next = { ...probeResults.value };
    delete next[toolId];
    probeResults.value = next;
  } catch (e) {
    message.error(toUserMessage(e));
  } finally {
    removingProbeToolId.value = "";
  }
}

/**
 * 同步刚结束时展示的时间。
 *
 * ⚠️ 这里刻意用了**两个**时间来源：权威值由 Rust 侧写入并在下次启动时读回
 * （`startup.lastSyncAt`）；这里只是为了「点完按钮立刻就有显示」而本地补一个。
 * 若两边格式不一致，用户会看到同一件事有两种写法——所以格式必须一致
 * （`YYYY-MM-DD HH:MM`，与 `sync::now_display()` 相同）。
 */
function currentLocalTime(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} ${pad(
    now.getHours(),
  )}:${pad(now.getMinutes())}`;
}
</script>

<template>
  <div class="flex min-h-screen flex-col bg-neutral-50">
    <!-- ── 顶部：同步状态栏（PRD 第 7 节，常驻） ── -->
    <header class="border-b border-neutral-200 bg-white">
      <div
        class="mx-auto flex max-w-5xl flex-wrap items-center gap-x-4 gap-y-2 px-8 py-3 text-sm"
      >
        <span class="font-semibold text-neutral-900">CrossBrain</span>

        <span class="text-neutral-500">
          上次同步：<span class="text-neutral-800">{{ lastSyncLabel }}</span>
        </span>

        <!-- 本次同步各工具的结果（未同步过时不显示，避免空占位） -->
        <span
          v-for="line in toolStates"
          :key="line.toolId"
          class="flex items-center gap-1 text-xs"
          :class="line.status === 'failed' ? 'text-error-500' : 'text-neutral-500'"
        >
          <IconCheck
            v-if="line.status === 'ok'"
            class="h-3.5 w-3.5 text-success-500"
            aria-hidden="true"
          />
          <IconMinus
            v-else-if="line.status === 'failed'"
            class="h-3.5 w-3.5 text-error-500"
            aria-hidden="true"
          />
          <span v-else class="h-3.5 w-3.5 text-neutral-300" aria-hidden="true">·</span>
          {{ line.displayName }}
        </span>

        <!--
          失败指示必须可点击（PRD §7：「❌ 红色：上次同步失败（点击可查看原因）」）。
          用 button 而不是 span：键盘用户也要能展开。
        -->
        <button
          v-if="lastSyncAt && !lastSyncOk"
          type="button"
          class="text-xs text-error-500 underline decoration-dotted underline-offset-2"
          @click="showSyncFailure = !showSyncFailure"
        >
          上次同步未完全成功（点击{{ showSyncFailure ? "收起" : "查看原因" }}）
        </button>

        <span class="ml-auto flex items-center gap-3">
          <!-- 离线指示（PRD §7 用红点表达；本地同步不依赖网络，文案如实说明） -->
          <span v-if="!online" class="flex items-center gap-1 text-xs text-error-500">
            <IconWifiOff class="h-3.5 w-3.5" aria-hidden="true" />
            离线模式（同步不受影响）
          </span>

          <n-button size="small" type="primary" :loading="syncing" @click="syncNow">
            <template #icon>
              <IconRefresh v-if="!syncing" class="h-4 w-4" aria-hidden="true" />
            </template>
            立即同步
          </n-button>
        </span>
      </div>

      <n-alert
        v-if="errorText"
        class="mx-auto max-w-5xl rounded-none"
        type="error"
        :bordered="false"
      >
        {{ errorText }}
      </n-alert>

      <!-- 失败原因展开区（本次会话内有效；重启后诚实说明原因不保留） -->
      <n-alert
        v-if="showSyncFailure && lastSyncAt && !lastSyncOk"
        class="mx-auto max-w-5xl rounded-none"
        type="warning"
        :bordered="false"
      >
        <template v-if="syncFailures.length > 0">
          <p v-for="f in syncFailures" :key="f">{{ f }}</p>
        </template>
        <template v-else>
          失败原因只保留到关闭应用为止。再点一次「立即同步」，如果仍然失败，
          这里会显示每个工具的具体原因。
        </template>
      </n-alert>
    </header>

    <!-- ── 主体 ── -->
    <main class="mx-auto w-full max-w-5xl flex-1 px-8 py-6">
      <!--
        display-directive 用 show:lazy 而不是默认的 if：
        默认值会在切走 Tab 时**卸载**面板，「全局规则」里没保存的修改会跟着丢。
        show:lazy = 首次进入才渲染、之后常驻（只是隐藏），未保存内容得以保留。
      -->
      <n-tabs v-model:value="activeTab" type="line" animated>
        <n-tab-pane name="rules" tab="全局规则" display-directive="show:lazy">
          <RuleEditor
            :editor="ruleEditor"
            @saved="onRulesSaved"
            @save-failed="onRulesSaveFailed"
          />
        </n-tab-pane>

        <n-tab-pane name="knowledge" tab="技能知识" display-directive="show:lazy">
          <KnowledgeManager />
        </n-tab-pane>

        <n-tab-pane name="settings" tab="设置" display-directive="show:lazy">
          <!-- ── 备份与还原（TASK-19 / ADR-15：「可逆」） ── -->
          <section class="flex flex-col gap-4">
            <div>
              <h2 class="text-base font-medium text-neutral-900">备份与还原</h2>
              <p class="mt-1 text-sm leading-6 text-neutral-500">
                同步只会改动下面这些文件里的标记区域。它们在第一次被改动前都自动另存了备份，
                你可以随时还原到最初的内容。
              </p>
            </div>

            <div v-if="backupsLoading" class="text-sm text-neutral-500">正在读取备份信息……</div>

            <n-alert v-else-if="backupError" type="error" :bordered="false">
              {{ backupError }}
            </n-alert>

            <div
              v-else-if="backups.length === 0"
              class="rounded-xl border border-dashed border-neutral-200 bg-white p-8 text-sm text-neutral-500"
            >
              还没有需要备份的文件。
            </div>

            <template v-else>
              <div
                v-for="item in backups"
                :key="item.toolId"
                class="rounded-xl border border-neutral-200 bg-white p-4"
              >
                <div class="flex items-start justify-between gap-4">
                  <div class="min-w-0">
                    <div class="flex flex-wrap items-center gap-2">
                      <span class="text-sm font-medium text-neutral-900">
                        {{ item.displayName }}
                      </span>
                      <n-tag v-if="item.exists" size="small" type="success" :bordered="false">
                        可还原
                      </n-tag>
                      <n-tag v-else size="small" :bordered="false">尚未同步</n-tag>
                    </div>

                    <p class="mt-1 break-all text-xs text-neutral-500">{{ item.targetFile }}</p>

                    <p v-if="item.exists" class="mt-1 text-xs text-neutral-500">
                      备份于 {{ item.modifiedAt ?? "未知时间" }} · {{ formatSize(item.sizeBytes) }}
                    </p>
                    <p v-else class="mt-1 text-xs text-neutral-500">
                      这个文件还没有被改动过，暂时没有可还原的内容。
                    </p>
                  </div>

                  <n-button
                    size="small"
                    :disabled="!item.exists || restoringToolId !== ''"
                    :loading="restoringToolId === item.toolId"
                    @click="askRestore(item)"
                  >
                    <template #icon>
                      <IconRotateCcw class="h-4 w-4" aria-hidden="true" />
                    </template>
                    还原
                  </n-button>
                </div>
              </div>
            </template>

            <n-alert v-if="restoreMessage" type="success" :bordered="false">
              {{ restoreMessage }}
            </n-alert>
            <n-alert v-if="restoreError" type="error" :bordered="false">
              {{ restoreError }}
            </n-alert>
          </section>

          <!-- ── 工具读取检测（探针）：验证各工具真的读到了推送的内容 ── -->
          <section class="mt-8 flex flex-col gap-3 rounded-xl border border-neutral-200 bg-white p-4">
            <div>
              <h2 class="text-base font-medium text-neutral-900">工具读取检测</h2>
              <p class="mt-1 text-sm leading-6 text-neutral-500">
                向各工具写入一条临时的「探针」技能，验证它们真的能读到 CrossBrain
                推送的内容。Codex 会自动给出结论；其它工具需要你到工具里问一句确认。
                探针看完即可移除，下次「立即同步」也会自动清理。
              </p>
            </div>

            <div v-if="!probeToolsLoaded" class="text-sm text-neutral-500">
              正在读取工具信息……
            </div>

            <div v-else class="flex flex-col gap-2">
              <div
                v-for="tool in probeTools"
                :key="tool.toolId"
                class="flex items-start justify-between gap-4 rounded-lg border border-neutral-100 bg-neutral-50 px-4 py-3"
              >
                <div class="min-w-0">
                  <div class="flex flex-wrap items-center gap-2">
                    <span class="text-sm font-medium text-neutral-900">
                      {{ tool.displayName }}
                    </span>
                    <n-tag v-if="!tool.installed" size="small" :bordered="false">未安装</n-tag>
                  </div>
                  <p
                    v-if="probeResults[tool.toolId]"
                    class="mt-1 text-xs leading-5"
                    :class="
                      probeResults[tool.toolId].status === 'failed'
                        ? 'text-error-500'
                        : 'text-neutral-600'
                    "
                  >
                    {{ probeResults[tool.toolId].message }}
                  </p>
                </div>

                <div class="flex shrink-0 items-center gap-2">
                  <n-button
                    size="small"
                    :loading="probingToolId === tool.toolId"
                    :disabled="!tool.installed || probingToolId !== ''"
                    @click="startProbe(tool.toolId)"
                  >
                    <template #icon>
                      <IconRadar v-if="probingToolId !== tool.toolId" class="h-4 w-4" aria-hidden="true" />
                    </template>
                    检测
                  </n-button>
                  <n-button
                    v-if="probeResults[tool.toolId] && probeResults[tool.toolId].status !== 'verified'"
                    size="small"
                    quaternary
                    :loading="removingProbeToolId === tool.toolId"
                    @click="clearProbe(tool.toolId)"
                  >
                    <template #icon>
                      <IconEraser class="h-4 w-4" aria-hidden="true" />
                    </template>
                    移除探针
                  </n-button>
                </div>
              </div>
            </div>
          </section>

          <!-- ── 卸载 / 去痕（TASK-14）：删除性操作，必须二次确认 ── -->
          <section class="mt-8 flex flex-col gap-3 rounded-xl border border-neutral-200 bg-white p-4">
            <div>
              <h2 class="text-base font-medium text-error-500">卸载 CrossBrain</h2>
              <p class="mt-1 text-sm leading-6 text-neutral-500">
                从所有支持的编程工具中移除 CrossBrain 写入的内容，并清空它的技能目录与备份。
                你自己的全局规则和技能知识会保留在本机，不会被删除。
              </p>
            </div>

            <n-alert v-if="uninstallError" type="error" :bordered="false">
              {{ uninstallError }}
            </n-alert>

            <div>
              <n-button type="error" secondary :loading="uninstalling" @click="askUninstall">
                <template #icon>
                  <IconTrash class="h-4 w-4" aria-hidden="true" />
                </template>
                卸载 CrossBrain
              </n-button>
            </div>
          </section>
        </n-tab-pane>
      </n-tabs>
    </main>

    <!-- 首次同步前的一次性告知（TASK-19 / ADR-15） -->
    <LinkNoticeDialog
      :show="linkNoticeOpen"
      @confirm="linkNotice.confirm"
      @cancel="linkNotice.cancel"
    />

    <!-- 还原的二次确认：动的是用户自己的文件，不该一点就执行 -->
    <n-modal
      :show="pendingRestore !== null"
      preset="card"
      style="width: 480px"
      title="确认还原？"
      :mask-closable="false"
      @update:show="(v: boolean) => !v && cancelRestore()"
    >
      <div class="flex flex-col gap-3 text-sm leading-6 text-neutral-700">
        <p>
          会把 {{ pendingRestore?.displayName }} 的
          <code class="break-all text-neutral-900">{{ pendingRestore?.targetFile }}</code>
          还原到它第一次被改动之前的内容。
        </p>
        <p class="text-xs text-neutral-500">
          你之后在这个文件里做过的修改不会保留在当前文件里——但还原之前，
          它会先被另存一份，需要时可以取回。最初的备份不会被覆盖。
        </p>
      </div>

      <template #footer>
        <div class="flex justify-end gap-2">
          <n-button @click="cancelRestore">取消</n-button>
          <n-button type="primary" @click="confirmRestore">确认还原</n-button>
        </div>
      </template>
    </n-modal>

    <!-- 卸载的二次确认（TASK-14）：删除性操作，范围必须写清楚 -->
    <n-modal
      :show="pendingUninstall"
      preset="card"
      style="width: 560px"
      title="确认卸载 CrossBrain？"
      :mask-closable="false"
      @update:show="(v: boolean) => !v && cancelUninstall()"
    >
      <div class="flex flex-col gap-2 text-sm leading-6 text-neutral-700">
        <p>确认后将执行：</p>
        <ul class="flex flex-col gap-1 pl-5">
          <li class="list-disc">
            从 Claude Code、Antigravity IDE、Codex 的配置文件中移除 CrossBrain
            写入的内容——<span class="text-neutral-900">你自己的配置原样保留</span>
          </li>
          <li class="list-disc">删除 CrossBrain 放进各工具的技能目录</li>
          <li class="list-disc">删除 CrossBrain 的备份文件与运行记录</li>
        </ul>
        <p class="text-xs text-neutral-500">
          不会删除：保存在本机的全局规则与技能知识（它们是你的数据）。
          卸载后再次打开 CrossBrain，会像第一次使用一样进入引导。
        </p>
      </div>

      <template #footer>
        <div class="flex justify-end gap-2">
          <n-button :disabled="uninstalling" @click="cancelUninstall">取消</n-button>
          <n-button type="error" :loading="uninstalling" @click="confirmUninstall">
            确认卸载
          </n-button>
        </div>
      </template>
    </n-modal>
  </div>
</template>

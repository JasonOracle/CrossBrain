<script setup lang="ts">
/**
 * App.vue — 应用外壳：决定显示「首次运行向导」还是「主工作台」。
 *
 * 判定依据来自 Rust 侧的 `get_startup_state`，**不在前端重复实现**：
 * 「是否首次运行」同时涉及 `rules.md` 内容与状态文件两处，
 * 散落成两份判断必然分叉。
 */
import { onMounted, ref } from "vue";

import { getStartupState, toUserMessage, type StartupState } from "./api";
import MainView from "./views/MainView.vue";
import WizardView from "./views/WizardView.vue";

const loading = ref(true);
const firstRun = ref(false);
const startup = ref<StartupState | null>(null);
const loadError = ref("");

/**
 * Naive UI 主题覆盖，取值与 `src/style.css` 里的 `--color-brand-*` 令牌保持一致。
 * 两处若不一致，组件库与自绘元素会出现两种蓝。
 */
const themeOverrides = {
  common: {
    primaryColor: "#3366ff",
    primaryColorHover: "#598eff",
    primaryColorPressed: "#1f47eb",
    primaryColorSuppl: "#8eb6ff",
    borderRadius: "8px",
  },
};

onMounted(async () => {
  try {
    const state = await getStartupState();
    startup.value = state;
    firstRun.value = state.firstRun;
  } catch (e) {
    loadError.value = toUserMessage(e);
  } finally {
    loading.value = false;
  }
});

/** 向导结束（走完流程或主动跳过）→ 进入主工作台 */
function onWizardDone() {
  firstRun.value = false;
}

/** 重新加载整个窗口（读不到本机状态时的重试入口） */
function reloadApp() {
  window.location.reload();
}
</script>

<template>
  <n-config-provider :theme-overrides="themeOverrides">
    <!--
      消息提示的挂载点（TASK-11）。
      必须包在所有视图外层：`useMessage()` 只在 provider 内部的组件里可用，
      放进某个 v-if 分支会让另一分支里的调用直接报错。
    -->
    <n-message-provider>
      <!-- 启动瞬间：读取本机状态（通常只有几十毫秒） -->
      <div
        v-if="loading"
        class="flex min-h-screen items-center justify-center bg-neutral-50"
      >
        <n-spin size="medium" />
      </div>

      <!-- 连本机状态都读不出来：明确告知，而不是留一片白屏 -->
      <div
        v-else-if="loadError"
        class="flex min-h-screen items-center justify-center bg-neutral-50 p-8"
      >
        <n-result status="warning" title="无法启动" :description="loadError">
          <template #footer>
            <n-button @click="reloadApp">重试</n-button>
          </template>
        </n-result>
      </div>

      <WizardView v-else-if="firstRun" :startup="startup" @done="onWizardDone" />
      <MainView v-else :startup="startup" />
    </n-message-provider>
  </n-config-provider>
</template>

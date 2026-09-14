<script setup lang="ts">
/**
 * LinkNoticeDialog.vue — 首次同步前的一次性告知（TASK-19 / ADR-15）。
 *
 * # 为什么需要它
 *
 * 同步会往用户**既有的**规则文件里写入一段标记区域。其中 `~/.claude/CLAUDE.md`
 * 在本机与另外 4 个路径共享同一份内容（硬链接），也就是说用户目前是
 * 「一处修改、五个工具同时生效」。为了不让 CrossBrain 的标记内容顺着共享关系
 * 渗到别的工具里，首次写入必须断开这一侧的链接——
 * **内容一个字不变，但共享关系没了**。
 *
 * 这属于改变用户既有的工作流，所以必须**事前**告知，而不是只写在代码注释里
 * （`ADR-15` 的「知情」要求）。告知只做一次，由 `useLinkNotice` 记在本机状态里。
 *
 * # 文案约束
 *
 * 不得出现 L0 / L2 / SSOT / slug / Adapter 等内部术语（`ADR-08`），
 * 且新增文案已被 `tests/ipc_contract.rs` 的术语检查覆盖。
 */
import IconLinkOff from "~icons/lucide/link-2-off";
import IconShield from "~icons/lucide/shield-alert";

defineProps<{ show: boolean }>();

const emit = defineEmits<{
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();
</script>

<template>
  <n-modal
    :show="show"
    preset="card"
    style="width: 560px"
    :mask-closable="false"
    :close-on-esc="false"
    :closable="false"
    title="同步前请知悉"
  >
    <div class="flex flex-col gap-4 text-sm leading-6 text-neutral-700">
      <p>
        同步会往你已有的规则文件里写入一段由 CrossBrain 管理的内容，主要是
        Claude Code 的 <code class="text-neutral-900">CLAUDE.md</code> 和
        Codex 的 <code class="text-neutral-900">AGENTS.md</code>。
      </p>

      <div class="flex items-start gap-3 rounded-lg border border-neutral-200 bg-white p-3">
        <IconShield class="mt-0.5 h-4 w-4 shrink-0 text-success-500" aria-hidden="true" />
        <span>
          标记区域<strong class="font-medium text-neutral-900">之外</strong
          >你自己写的内容逐字保留，不会被改动或删除；原文件在写入前还会自动另存一份备份。
        </span>
      </div>

      <div
        class="flex items-start gap-3 rounded-lg border border-warning-500/30 bg-warning-500/10 p-3"
      >
        <IconLinkOff class="mt-0.5 h-4 w-4 shrink-0 text-warning-500" aria-hidden="true" />
        <span>
          如果这些文件与其他位置共享着同一份内容（Windows 的「硬链接」——一处修改、多处同时生效），
          本次写入会<strong class="font-medium text-neutral-900">断开这种共享关系</strong>。
          文件内容一个字都不会变，但它此后不再跟随其他位置一起变化。
        </span>
      </div>

      <p class="text-xs text-neutral-500">
        这样做是为了不让管理内容通过共享关系渗进其他工具。同步之后，随时可以在
        「设置 → 备份与还原」里一键还原到最初的内容。
      </p>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <n-button @click="emit('cancel')">先不同步</n-button>
        <n-button type="primary" @click="emit('confirm')">我知道了，继续同步</n-button>
      </div>
    </template>
  </n-modal>
</template>

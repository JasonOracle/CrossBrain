/**
 * useLinkNotice.ts — 「同步会断开文件共享关系」说明的展示闸门（TASK-19 / ADR-15）。
 *
 * # 为什么抽成一个组合式函数
 *
 * 触发「首次同步」的入口有**两个**（向导步骤 4 的「开始同步」、主工作台的
 * 「立即同步」）。两边都要做同一件事：先拦一下、弹说明、用户确认后再真的同步。
 *
 * 这段逻辑里有一处**不能写错**的顺序约束：标记「已展示」必须发生在用户**确认之后**，
 * 而不是对话框弹出的那一刻。若顺序反了，用户关掉窗口或点了取消，
 * 这条说明就永远不会再出现——而它恰恰是 AC 要求的事前告知。
 * 这种约束靠两份各自演进的代码「保持」一致是不可靠的，所以共用一份。
 *
 * # 为什么用「待执行动作」而不是让调用方自己判断
 *
 * `guard()` 收到的是**真正要执行的那件事**（例如 `doSync`），确认后由这里回调。
 * 若改成让调用方先问 `shouldShow()` 再自己决定，调用方就有机会在
 * 「问了但忘了处理」时静默什么都不做——按钮点了没反应是最难排查的一类故障。
 */

import { ref } from "vue";

import { markLinkNoticeShown } from "../api";

export function useLinkNotice(initiallyShown: boolean) {
  /** 对话框是否可见 */
  const open = ref(false);
  /** 说明是否已展示过（初始值来自上次启动时的本机状态） */
  const shown = ref(initiallyShown);

  /** 用户确认后要执行的动作 */
  let pending: (() => void) | null = null;

  /**
   * 拦截一次同步动作：已告知过就直接执行，否则先弹说明。
   *
   * @param action 用户确认后要真正执行的事（例如发起同步）
   */
  function guard(action: () => void) {
    if (shown.value) {
      action();
      return;
    }
    pending = action;
    open.value = true;
  }

  /** 用户点了「继续同步」 */
  function confirm() {
    open.value = false;
    shown.value = true;

    // 写状态失败不影响本次同步：最坏情况是下次启动再提示一次，无关紧要。
    // 反过来若为了等这次写入而卡住同步，才是真正的倒退。
    void markLinkNoticeShown().catch(() => {});

    const action = pending;
    pending = null;
    action?.();
  }

  /** 用户点了「先不同步」——不标记已展示，下次仍会提示 */
  function cancel() {
    open.value = false;
    pending = null;
  }

  return { open, shown, guard, confirm, cancel };
}

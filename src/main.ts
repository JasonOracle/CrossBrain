import { createApp } from "vue";
import App from "./App.vue";

// 全局样式入口（含 Tailwind 与设计令牌）
import "./style.css";

// Naive UI 组件通过 unplugin-vue-components 按需自动导入，
// 此处不做全局注册，避免打包体积无谓增大。
createApp(App).mount("#app");

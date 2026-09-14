/// <reference types="vite/client" />
/// <reference types="unplugin-icons/types/vue" />

// 说明：
// 1. 原先模板自带的 `declare module "*.vue"` 兜底声明已移除。
//    vue-tsc / Volar 原生解析 .vue 文件，该兜底声明会把真实类型擦成
//    DefineComponent<{}, {}, any>，既违反铁律 L-04（禁用 any），也丢失组件类型提示。
// 2. unplugin-icons 的 `~icons/*` 模块类型由上面的 reference 提供。
// 3. Naive UI 等自动导入组件的类型由构建时生成的 `src/components.d.ts` 提供。

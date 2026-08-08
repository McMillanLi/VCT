/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

// 注入到 window，用于在纯 Web 开发模式下判断是否处于 Tauri 环境
declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export {};

/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module '@tauri-apps/plugin-dialog' {
  export function save(options?: { filters?: { name: string; extensions: string[] }[] }): Promise<string | null>
}

declare module '@tauri-apps/plugin-fs' {
  export function writeTextFile(path: string, contents: string): Promise<void>
}

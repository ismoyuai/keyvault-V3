# Vue 前端审查规格文档

> **审查目标：** 全面审查 KeyVault v3 Vue 3 前端代码的架构、安全性和代码质量
>
> **审查方法：** 自底向上（L1 UI → L2 Business → L3 Bridge → L4 Views → L5 Routing → L6 Styles）
>
> **审查范围：** `src/` 下所有模块（30 个源文件，约 3,771 行）

---

## 1. 审查范围

### 1.1 模块清单

| 层 | 文件 | 职责 |
|---|---|---|
| L1 UI | `components/ui/KvButton.vue` | 按钮组件（4 variants, 3 sizes） |
| L1 UI | `components/ui/KvInput.vue` | 输入组件（密码切换、错误显示） |
| L1 UI | `components/ui/KvModal.vue` | 模态框（Teleport + 过渡动画） |
| L1 UI | `components/ui/KvToast.vue` | Toast 通知显示 |
| L2 Business | `stores/auth.ts` | 认证状态（unlock/lock/setup） |
| L2 Business | `stores/settings.ts` | 持久化用户偏好 |
| L2 Business | `stores/vault.ts` | 密码库条目列表状态 |
| L2 Business | `stores/ui.ts` | UI 面板切换 |
| L2 Business | `composables/useAutoLock.ts` | 空闲超时自动锁定 |
| L2 Business | `composables/useClipboard.ts` | 剪贴板复制 + 自动清除 |
| L2 Business | `composables/useShortcuts.ts` | 键盘快捷键绑定 |
| L2 Business | `composables/useToast.ts` | Toast 通知系统 |
| L2 Business | `composables/usePasswordGen.ts` | 密码生成 + 强度评估 |
| L2 Business | `composables/index.ts` | Barrel 导出 |
| L2 Business | `types/vault.ts` | 共享 TypeScript 接口 |
| L2 Business | `constants/templates.ts` | 条目类型模板定义 |
| L3 Bridge | `bridge/tauri.ts` | IPC 调用层、session token 管理 |
| L4 Views | `views/UnlockView.vue` | 主密码解锁界面 |
| L4 Views | `views/SetupView.vue` | 初始化设置向导 |
| L4 Views | `views/SettingsView.vue` | 设置页面 |
| L4 Views | `views/VaultView.vue` | 主密码库布局（标题栏、侧边栏、列表、详情） |
| L4 Views | `views/components/Placeholder.vue` | 占位组件 |
| L4 Views | `components/vault/VaultItem.vue` | 密码库列表项 |
| L4 Views | `components/vault/ItemDetail.vue` | 详情面板（字段解密） |
| L4 Views | `components/vault/ItemForm.vue` | 创建/编辑条目表单 |
| L4 Views | `components/search/CommandPalette.vue` | Spotlight 风格命令面板 |
| L5 Routing | `router/index.ts` | 路由定义、hash history、守卫 |
| L5 Routing | `App.vue` | 初始化检查、重定向 |
| L5 Routing | `main.ts` | 应用引导、Pinia + Router 安装 |
| L6 Styles | `styles/base.css` | CSS 重置和全局样式 |
| L6 Styles | `design/tokens.css` | 设计令牌 |
| L6 Styles | `design/typography.css` | 排版定义 |
| L6 Styles | `design/animations.css` | 动画定义 |

### 1.2 关联文件

| 文件 | 审查内容 |
|---|---|
| `package.json` | 依赖版本、脚本 |
| `vite.config.ts` | 构建配置 |
| `tsconfig.json` | TypeScript 配置 |
| `src-tauri/src/lib.rs` | Rust IPC 命令注册对照 |

---

## 2. 各层审查标准

### 2.1 L1 UI 组件层

#### KvButton.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| Props 校验 | variant/size 有默认值 | Important |
| 事件处理 | click 事件正确 emit | Important |
| 禁用状态 | disabled 时阻止交互 | Important |
| 加载状态 | loading 时显示指示器并阻止交互 | Minor |

#### KvInput.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| v-model | 正确使用 modelValue/update:modelValue | Critical |
| 密码切换 | 切换按钮可访问（非 emoji） | Important |
| 错误显示 | error prop 正确显示 | Minor |
| 键盘事件 | Enter 键正确处理 | Minor |

#### KvModal.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| Teleport | 正确使用 Teleport to="body" | Important |
| Escape 关闭 | 仅在 open=true 时监听 | Important |
| 点击遮罩关闭 | backdrop click 正确处理 | Minor |
| 过渡动画 | 进入/离开动画正常 | Minor |

#### KvToast.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 自动消失 | 超时后正确移除 | Important |
| 手动关闭 | 有关闭按钮 | Minor |
| 多 toast | 支持堆叠显示 | Minor |

### 2.2 L2 Business 层

#### stores/auth.ts

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| Token 管理 | unlock 后设置 token，lock 后清除 | Critical |
| 错误状态 | 错误在重试时清除 | Important |
| 加载状态 | pending 状态正确管理 | Important |

#### stores/vault.ts

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 乐观更新 | toggleFavorite/deleteEntry 有回滚 | Important |
| 数据一致性 | 不缓存解密数据 | Critical |
| 搜索状态 | searchQuery 正确管理 | Minor |

#### stores/settings.ts

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 持久化 | 变更通过 IPC 写入后端 | Critical |
| 加载时机 | 应用启动时加载设置 | Important |
| 类型安全 | 解析值有校验 | Minor |

#### composables/useAutoLock.ts

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 事件节流 | mousemove 节流到 1 次/秒 | Important |
| 生命周期 | onMounted 注册，onUnmounted 清除 | Important |
| 锁定触发 | 超时后调用 auth.lock() | Critical |

#### composables/useClipboard.ts

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| Store 调用 | useSettingsStore 在 composable body 中调用 | Minor |
| 清除等待 | clipboard.clear() 结果 await | Minor |
| 自动清除 | 超时后自动清除剪贴板 | Important |

#### composables/usePasswordGen.ts

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 随机性 | 无 modulo bias（rejection sampling） | Critical |
| 强度评估 | 评估逻辑合理 | Minor |
| UI 集成 | 在 ItemForm 中可用 | Important |

#### composables/useShortcuts.ts

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| meta 键 | meta 属性在匹配逻辑中检查 | Minor |
| 清理 | onUnmounted 移除监听器 | Important |

### 2.3 L3 Bridge 层

#### bridge/tauri.ts

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 命令覆盖 | 所有 Rust 命令有对应 bridge 调用 | Critical |
| 无功能调用 | 不存在调用未注册命令的代码 | Critical |
| 类型匹配 | TS 参数类型与 Rust 期望一致 | Critical |
| Token 安全 | _sessionToken 仅 bridge 内部访问 | Important |
| 错误传播 | invoke 错误正确传递 | Important |
| 导出/导入 | export_vault/import_vault bridge 调用 | Important |

### 2.4 L4 Views 层

#### UnlockView.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 组件复用 | 使用 KvButton/KvInput | Minor |
| 错误处理 | 解锁失败显示错误 | Important |
| 动画 | 失败抖动动画 | Minor |

#### SetupView.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 密码确认 | 两次密码输入一致 | Critical |
| 密码强度 | 有强度指示器 | Minor |
| 组件复用 | 使用 KvButton/KvInput | Minor |

#### SettingsView.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 设置持久化 | 变更通过 store 方法写入后端 | Critical |
| 修改密码 | 有修改密码 UI | Important |
| 导出/导入 | 有导出/导入 UI | Important |
| 泄露检测 | 有泄露检测入口 | Important |

#### VaultView.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 计算属性 | filteredEntries 是 computed 而非函数 | Important |
| 搜索防抖 | search input 有 200-300ms 防抖 | Important |
| 快捷键 | Ctrl+K 唽醒命令面板 | Minor |
| 布局 | 标题栏、侧边栏、列表、详情正确布局 | Minor |

#### ItemForm.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| Import 位置 | import 在 script setup 顶部 | Critical |
| Props watch | watch editEntry/open 重置表单状态 | Critical |
| 密码生成 | 集成 usePasswordGen | Important |
| 表单重置 | resetForm 正确清除所有字段 | Important |

#### ItemDetail.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 字段解密 | 按需解密，不缓存 | Critical |
| 内存清理 | 关闭时清除解密数据 | Important |
| 复制功能 | 正确调用 clipboard bridge | Important |

#### CommandPalette.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 全局 keydown | 移除 no-op handleGlobalKeydown | Important |
| 搜索防抖 | 命令面板搜索有防抖 | Minor |
| 模式切换 | > 前缀切换命令模式 | Minor |

### 2.5 L5 Routing + 初始化

#### router/index.ts

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 路由守卫 | beforeEach 有真实鉴权逻辑 | Critical |
| 重定向 | 未初始化时重定向到 /setup | Important |
| 路由定义 | 所有路由正确定义 | Minor |

#### App.vue

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 初始化检查 | 启动时检查 is_initialized | Critical |
| 重定向逻辑 | 未初始化→/setup，已初始化→/login | Important |

#### main.ts

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 全局错误处理 | app.config.errorHandler 已设置 | Minor |
| 插件安装 | Pinia + Router 正确安装 | Important |

### 2.6 L6 Styles

| 检查项 | 标准 | 严重程度 |
|---|---|---|
| 设计令牌 | 正确引用 tokens.css 变量 | Minor |
| 暗黑主题 | 默认暗黑主题一致 | Minor |

---

## 3. 预期发现

### 3.1 Critical 级（5 个）

| # | 问题 | 位置 | 影响 |
|---|---|---|---|
| C1 | tray.lock()/tray.show() 调用未注册的 IPC 命令 | `bridge/tauri.ts:142-145` | 运行时 IPC 错误 |
| C2 | Router guard 空实现，无鉴权保护 | `router/index.ts:36-41` | 未解锁可访问 /vault |
| C3 | updateEntry 类型不匹配（Partial vs 完整对象） | `bridge/tauri.ts:65` | Rust 反序列化失败 |
| C4 | ItemForm.vue import 位置错误 | `ItemForm.vue:104` | 可能构建错误 |
| C5 | ItemForm.vue props 未 watch，表单状态过期 | `ItemForm.vue:27-29` | 编辑不同条目显示旧数据 |

### 3.2 Important 级（9 个）

| # | 问题 | 位置 | 影响 |
|---|---|---|---|
| I1 | 设置变更未持久化到后端 | `SettingsView.vue:31,43` | 重启丢失设置 |
| I2 | Session token 存储在普通 JS 变量 | `bridge/tauri.ts:18` | 防御纵深不足 |
| I3 | 前端密码生成器有 modulo bias | `usePasswordGen.ts:77` | 密码分布偏倚 |
| I4 | filteredEntries 应为 computed | `VaultView.vue:82-86` | 每次渲染重新计算 |
| I5 | CommandPalette 全局 keydown 是 no-op | `CommandPalette.vue:149-153` | 无效代码 |
| I6 | KvModal Escape 监听器始终活跃 | `KvModal.vue:20-27` | 多模态框冲突 |
| I7 | 缺少修改密码 UI | `SettingsView.vue` | 功能不可用 |
| I8 | 缺少泄露检测 UI | 无 | 功能不可用 |
| I9 | 缺少导出/导入 UI + bridge 调用 | 无 | 功能不可用 |

### 3.3 Minor 级（10 个）

| # | 问题 | 位置 | 影响 |
|---|---|---|---|
| M1 | useClipboard 在函数体内调用 store | `useClipboard.ts:20` | 反模式 |
| M2 | clipboard.clear() 未 await | `useClipboard.ts:30` | 错误静默 |
| M3 | usePasswordGenerator 未集成到 UI | `ItemForm.vue` | 用户无法生成密码 |
| M4 | 搜索输入未防抖 | `VaultView.vue:163` | 每次按键触发 IPC |
| M5 | mousemove 未节流 | `useAutoLock.ts:43` | 60Hz 事件处理 |
| M6 | useShortcuts meta 属性未检查 | `useShortcuts.ts` | 逻辑不完整 |
| M7 | 无全局错误处理器 | `main.ts` | 错误静默 |
| M8 | Auth 视图未使用 KvButton/KvInput | `UnlockView/SetupView` | 样式不一致 |
| M9 | ItemDetail 未清零解密数据 | `ItemDetail.vue` | 内存安全 |
| M10 | @vueuse/core 已安装但未充分利用 | 多处 | 依赖浪费 |

---

## 4. 安全红线对照

审查过程中必须对照以下 CLAUDE.md 安全红线逐条验证：

1. **主密码零存储** — 前端不存储主密码，只通过 IPC 发送到后端
2. **密钥不落盘** — 前端不接触加密密钥
3. **锁定时清零** — lock 后清除所有解密数据和 session token
4. **字段级加密** — 前端只接收解密后的文本，不接收密文
5. **前端零缓存** — 解密数据不持久化到 localStorage/sessionStorage
6. **IPC 全部验证 session token** — bridge 层正确附加 token

---

## 5. 审查工作流

```
读取源文件 → 逐层审查 → 记录发现 → 分类排序 → 产出报告
     ↓
  L1 UI → L2 Business → L3 Bridge → L4 Views → L5 Routing → L6 Styles
     ↓
  每层完成后立即记录，不等全部结束
```

---

## 6. 交付物

| 文件 | 内容 | 用途 |
|---|---|---|
| `docs/superpowers/specs/2026-06-03-frontend-audit.md` | 本文档 | 审查规格 |
| 审查报告（内嵌于本文档更新） | 详细发现 | 记录所有问题 |
| `docs/superpowers/plans/2026-06-03-frontend-fixes.md` | 修复计划 | 可执行任务清单 |

---

## 7. 审查报告

> **审查完成时间：** 2026-06-03
>
> **审查范围：** `src/` 全部前端模块（共 30 个源文件，约 3,771 行）
>
> **发现汇总：** 5 Critical, 9 Important, 10 Minor

---

### 7.1 L1 UI 组件层审查结果

#### KvButton.vue — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| Props 校验 | **通过** | variant/size 有默认值（'primary'/'md'） |
| 事件处理 | **通过** | @click 正确处理 |
| 禁用状态 | **通过** | disabled prop + :disabled 绑定 |
| 加载状态 | **通过** | loading prop 显示 spinner 并阻止交互 |

#### KvInput.vue — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| v-model | **通过** | 正确使用 modelValue/update:modelValue |
| 密码切换 | **Minor** | 使用 emoji（🙈/👁）而非图标，可访问性不足 |
| 错误显示 | **通过** | error prop 正确显示 |
| 键盘事件 | **通过** | @keyup.enter 正确处理 |

#### KvModal.vue — 1 个 Important 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| Teleport | **通过** | 正确使用 Teleport to="body" |
| Escape 关闭 | **Important** | handleKeydown 在 onMounted 注册，即使 open=false 也活跃 |
| 点击遮罩关闭 | **通过** | @click.self 正确处理 |
| 过渡动画 | **通过** | Transition 组件正确使用 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| I-6 | Important | Escape 监听器始终活跃 | `KvModal.vue:20-27` | 仅在 open=true 时注册监听器，或在 handleKeydown 中检查 open 状态 |

#### KvToast.vue — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| 自动消失 | **通过** | onMounted 设置 setTimeout |
| 手动关闭 | **Minor** | 无手动关闭按钮 |
| 多 toast | **通过** | 通过 toast composable 管理多个 |

---

### 7.2 L2 Business 层审查结果

#### stores/auth.ts — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| Token 管理 | **通过** | unlock 后调用 setSessionToken，lock 后调用 clearSessionToken |
| 错误状态 | **通过** | catch 块设置 error，下次操作前清除 |
| 加载状态 | **通过** | try/finally 正确管理 pending |

#### stores/vault.ts — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| 乐观更新 | **Minor** | toggleFavorite/deleteEntry 乐观更新无回滚（单用户本地应用可接受） |
| 数据一致性 | **通过** | 只存储 EntryMeta，不缓存解密字段 |
| 搜索状态 | **通过** | searchQuery 正确管理 |

#### stores/settings.ts — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| 持久化 | **通过** | setAutoLock/setClipboardClear 调用 settings.set() IPC |
| 加载时机 | **通过** | loadSettings 在 setup 时调用 |
| 类型安全 | **Minor** | parseInt 无校验 |

#### composables/useAutoLock.ts — 1 个 Important 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 事件节流 | **Important** | mousemove 未节流，60+Hz 触发 |
| 生命周期 | **通过** | onMounted 注册，onUnmounted 清除 |
| 锁定触发 | **通过** | 超时后调用 authStore.lock() |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| M-5 | Important | mousemove 未节流 | `useAutoLock.ts:43` | 使用 throttle 或 @vueuse/core 的 useThrottleFn |

#### composables/useClipboard.ts — 2 个 Minor 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| Store 调用 | **Minor** | useSettingsStore() 在 copy() 函数体内调用 |
| 清除等待 | **Minor** | clipboard.clear() 返回 Promise 但未 await |
| 自动清除 | **通过** | setTimeout 正确实现自动清除 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| M-1 | Minor | store 在函数体内调用 | `useClipboard.ts:20` | 移到 composable body 顶部 |
| M-2 | Minor | clipboard.clear() 未 await | `useClipboard.ts:30` | 添加 await 或 .catch() |

#### composables/usePasswordGen.ts — 1 个 Critical 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 随机性 | **Critical** | `randomBytes[i] % chars.length` 有 modulo bias |
| 强度评估 | **通过** | 评估逻辑合理 |
| UI 集成 | **Important** | 未集成到 ItemForm.vue |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| I-3 | Critical | modulo bias | `usePasswordGen.ts:77` | 使用 rejection sampling |
| M-3 | Important | 未集成到 UI | `ItemForm.vue` | 添加密码生成按钮 |

#### composables/useShortcuts.ts — 1 个 Minor 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| meta 键 | **Minor** | meta 属性在接口中定义但匹配逻辑中未检查 |
| 清理 | **通过** | onUnmounted 移除监听器 |

---

### 7.3 L3 Bridge 层审查结果

#### bridge/tauri.ts — 3 个 Critical 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 命令覆盖 | **Important** | export_vault/import_vault 缺少 bridge 调用 |
| 无功能调用 | **Critical** | tray.lock()/tray.show() 调用未注册的 IPC 命令 |
| 类型匹配 | **Critical** | updateEntry 使用 Partial<CreateEntryInput>，Rust 期望完整对象 |
| Token 安全 | **Important** | _sessionToken 是模块级变量 |
| 错误传播 | **通过** | invoke 错误正确传递 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| C-1 | Critical | tray 调用未注册命令 | `bridge/tauri.ts:142-145` | 移除 tray 对象，托盘功能由 Rust 端处理 |
| C-3 | Critical | updateEntry 类型不匹配 | `bridge/tauri.ts:65` | 改为 CreateEntryInput（完整对象） |
| I-2 | Important | token 存储在普通变量 | `bridge/tauri.ts:18` | 可接受（Tauri WebView 隔离），但需文档说明 |
| I-9 | Important | 缺少 export/import bridge | `bridge/tauri.ts` | 添加 exportVault/importVault 调用 |

---

### 7.4 L4 Views 层审查结果

#### UnlockView.vue — 1 个 Minor 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 组件复用 | **Minor** | 使用原生 `<button>` 和 `<input>` 而非 KvButton/KvInput |
| 错误处理 | **通过** | 错误状态正确显示 |
| 动画 | **通过** | 失败抖动动画 |

#### SetupView.vue — 1 个 Minor 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 密码确认 | **通过** | 两次密码一致校验 |
| 密码强度 | **Minor** | 无密码强度指示器 |
| 组件复用 | **Minor** | 使用原生组件 |

#### SettingsView.vue — 2 个 Important 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 设置持久化 | **Critical** | select v-model 直接绑定 store ref，绕过 setter 方法 |
| 修改密码 | **Important** | 缺少修改密码 UI |
| 导出/导入 | **Important** | 缺少导出/导入 UI |
| 泄露检测 | **Important** | 缺少泄露检测入口 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| I-1 | Critical | 设置变更未持久化 | `SettingsView.vue:31,43` | 使用 @change 调用 store 方法 |
| I-7 | Important | 缺少修改密码 UI | `SettingsView.vue` | 添加修改密码表单 |
| I-8 | Important | 缺少泄露检测 UI | `SettingsView.vue` | 添加泄露检测入口 |
| I-9 | Important | 缺少导出/导入 UI | `SettingsView.vue` | 添加导出/导入按钮 |

#### VaultView.vue — 2 个 Important 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 计算属性 | **Important** | filteredEntries 是函数而非 computed |
| 搜索防抖 | **Important** | @input 直接调用 search，无防抖 |
| 快捷键 | **通过** | Ctrl+K 正确处理 |
| 布局 | **通过** | 四区域布局正确 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| I-4 | Important | filteredEntries 应为 computed | `VaultView.vue:82-86` | 改为 computed() |
| M-4 | Important | 搜索未防抖 | `VaultView.vue:163` | 使用 useDebounceFn |

#### ItemForm.vue — 2 个 Critical 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| Import 位置 | **Critical** | import 在 script setup 底部（line 104） |
| Props watch | **Critical** | 未 watch editEntry/open，表单状态过期 |
| 密码生成 | **Important** | 未集成 usePasswordGen |
| 表单重置 | **Important** | resetForm 不完全重置 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| C-4 | Critical | import 位置错误 | `ItemForm.vue:104` | 移到 script setup 顶部 |
| C-5 | Critical | props 未 watch | `ItemForm.vue:27-29` | 添加 watch(editEntry) 重置表单 |
| M-3 | Important | 未集成密码生成 | `ItemForm.vue` | 添加密码生成按钮和 usePasswordGen 调用 |

#### ItemDetail.vue — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| 字段解密 | **通过** | 按需加载，不缓存 |
| 内存清理 | **Minor** | onUnmounted 设置 secrets=null，依赖 GC |
| 复制功能 | **通过** | 正确调用 clipboard bridge |

#### CommandPalette.vue — 1 个 Important 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 全局 keydown | **Important** | handleGlobalKeydown 是 no-op |
| 搜索防抖 | **Minor** | 无防抖 |
| 模式切换 | **通过** | > 前缀正确切换模式 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| I-5 | Important | no-op keydown handler | `CommandPalette.vue:149-153` | 移除 handleGlobalKeydown |

---

### 7.5 L5 Routing + 初始化审查结果

#### router/index.ts — 1 个 Critical 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 路由守卫 | **Critical** | beforeEach 空实现，无鉴权检查 |
| 重定向 | **通过** | / 重定向到 /login |
| 路由定义 | **通过** | 所有路由正确定义 |

**发现：**

| # | 严重程度 | 问题 | 位置 | 修复建议 |
|---|---|---|---|---|
| C-2 | Critical | 路由守卫空实现 | `router/index.ts:36-41` | 检查 authStore.isUnlocked，未解锁重定向到 /login |

#### App.vue — 通过

| 检查项 | 结果 | 说明 |
|---|---|---|
| 初始化检查 | **通过** | onMounted 检查 is_initialized |
| 重定向逻辑 | **通过** | 未初始化→/setup，已初始化→/login |

#### main.ts — 1 个 Minor 问题

| 检查项 | 结果 | 说明 |
|---|---|---|
| 全局错误处理 | **Minor** | 未设置 app.config.errorHandler |
| 插件安装 | **通过** | Pinia + Router 正确安装 |

---

### 7.6 L6 Styles 审查结果

| 检查项 | 结果 | 说明 |
|---|---|---|
| 设计令牌 | **通过** | 正确引用 tokens.css 变量 |
| 暗黑主题 | **通过** | 默认暗黑主题一致 |

---

### 7.7 安全红线对照结果

| # | 红线 | 状态 | 说明 |
|---|---|---|---|
| 1 | 主密码零存储 | **通过** | 前端不存储主密码，只通过 IPC 发送 |
| 2 | 密钥不落盘 | **通过** | 前端不接触加密密钥 |
| 3 | 锁定时清零 | **通过** | authStore.lock() 清除 token 和 vault 数据 |
| 4 | 字段级加密 | **通过** | 前端只接收解密后文本 |
| 5 | 前端零缓存 | **通过** | 无 localStorage/sessionStorage 使用 |
| 6 | IPC 全部验证 session | **通过** | bridge 层正确附加 token |

---

### 7.8 发现汇总

#### Critical 级（5 个）

| # | 问题 | 位置 | 修复建议 |
|---|---|---|---|
| C-1 | tray 调用未注册的 IPC 命令 | `bridge/tauri.ts:142-145` | 移除 tray 对象 |
| C-2 | Router guard 空实现 | `router/index.ts:36-41` | 添加真实鉴权逻辑 |
| C-3 | updateEntry 类型不匹配 | `bridge/tauri.ts:65` | 改为完整 CreateEntryInput |
| C-4 | ItemForm import 位置错误 | `ItemForm.vue:104` | 移到 script setup 顶部 |
| C-5 | ItemForm props 未 watch | `ItemForm.vue:27-29` | 添加 watch 重置表单 |

#### Important 级（9 个）

| # | 问题 | 位置 | 修复建议 |
|---|---|---|---|
| I-1 | 设置变更未持久化 | `SettingsView.vue:31,43` | 使用 @change 调用 store 方法 |
| I-2 | Session token 在普通变量 | `bridge/tauri.ts:18` | 可接受，文档说明 |
| I-3 | 密码生成器 modulo bias | `usePasswordGen.ts:77` | 使用 rejection sampling |
| I-4 | filteredEntries 应为 computed | `VaultView.vue:82-86` | 改为 computed() |
| I-5 | CommandPalette no-op keydown | `CommandPalette.vue:149-153` | 移除无效代码 |
| I-6 | KvModal Escape 始终活跃 | `KvModal.vue:20-27` | 检查 open 状态 |
| I-7 | 缺少修改密码 UI | `SettingsView.vue` | 添加表单 |
| I-8 | 缺少泄露检测 UI | 无 | 添加入口 |
| I-9 | 缺少导出/导入 UI + bridge | 无 | 添加 bridge + UI |

#### Minor 级（10 位）

| # | 问题 | 位置 | 修复建议 |
|---|---|---|---|
| M-1 | store 在函数体内调用 | `useClipboard.ts:20` | 移到 body 顶部 |
| M-2 | clipboard.clear() 未 await | `useClipboard.ts:30` | 添加 await |
| M-3 | 密码生成未集成 UI | `ItemForm.vue` | 添加生成按钮 |
| M-4 | 搜索未防抖 | `VaultView.vue:163` | 使用 useDebounceFn |
| M-5 | mousemove 未节流 | `useAutoLock.ts:43` | 使用 throttle |
| M-6 | shortcuts meta 未检查 | `useShortcuts.ts` | 添加 meta 匹配 |
| M-7 | 无全局错误处理器 | `main.ts` | 添加 errorHandler |
| M-8 | Auth 视图未用共享组件 | `UnlockView/SetupView` | 使用 KvButton/KvInput |
| M-9 | ItemDetail 未清零数据 | `ItemDetail.vue` | 文档说明 GC 依赖 |
| M-10 | @vueuse/core 未充分利用 | 多处 | 使用 useDebounceFn 等 |

---

### 7.9 修复优先级排序

**第一优先级（阻塞应用运行）：**
1. C-1: 移除 tray IPC 调用
2. C-2: 实现路由守卫
3. C-3: 修复 updateEntry 类型
4. C-4: 修复 ItemForm import
5. C-5: 添加 props watch

**第二优先级（功能完整性）：**
6. I-1: 修复设置持久化
7. I-3: 修复密码生成器 bias
8. I-4: filteredEntries 改为 computed
9. I-9: 添加 export/import bridge + UI

**第三优先级（安全加固）：**
10. I-5: 移除 no-op handler
11. I-6: 修复 KvModal Escape
12. I-7: 添加修改密码 UI
13. I-8: 添加泄露检测 UI

**第四优先级（质量改进）：**
14. M-1 ~ M-10: Minor 问题

# KeyVault 需求覆盖矩阵

**版本:** 2026-06-04  
**用途:** 对照产品提示词、UI 原型与 Vue/Rust 实现，标注缺口与优先级。  
**相关:** [UNIFIED-SPEC.md](./UNIFIED-SPEC.md) · [PROTOTYPE-PROMPTS.md](./PROTOTYPE-PROMPTS.md) · [README.md](./README.md)

---

## 图例

| 优先级 | 含义 |
|--------|------|
| **Must** | 阻塞 UI 重构或核心流程 |
| **Should** | 功能完整、体验完整所需 |
| **Future** | 后端未就绪或 v1 明确不做 |
| **Done** | 已有 canonical 原型且基本可用 |

---

## 覆盖矩阵

| 需求域 | 具体需求 | 原型 | 代码 | 优先级 | 缺口 / 行动 |
|--------|----------|------|------|--------|-------------|
| **全局** | 暗色主题 1200×800 | 多套已统一 | `tokens.css` 未完全对齐 | Should | 重构时同步 `_shared/tailwind-extend.json` |
| **全局** | 自定义标题栏 | `keyvault_3` 等 | Tauri 窗口 | Done | TopBar 统一 40px |
| **全局** | 浅色主题 | 归档 `secure_utility_light` | 无 | Future | v1 不做 |
| **账户** | 首次设置（输入+确认） | `_1` 需修订；`setup_2` **待生成** | `SetupView.vue` 2 步 | Must | 用户按提示词生成或后续 HTML |
| **账户** | 恢复密钥 Setup | `_2` 归档 | 无 | Future | 不纳入 v1 |
| **账户** | 解锁 | `keyvault_6` | `UnlockView.vue` | Done | — |
| **账户** | 5 次失败锁定 5 分钟 | `unlock_lockout` **待生成** | `auth.rs` 已实现 | Must | 补锁定态原型 |
| **账户** | 修改主密码 | `modal_change_password` **待生成** | Settings 内联表单 | Should | 独立模态原型 |
| **账户** | 自动锁定 5/15/30/60 | `keyvault_2` | `settings` store | Done | 扩展稿补剪贴板/泄露检测 |
| **账户** | 生物识别 | `keyvault_6` 有入口 | 未实现 | Future | disabled + tooltip |
| **账户** | 紧急擦除 | `keyvault_6` + `modal_emergency_wipe` Future | 无 | Future | 二次确认原型可选 |
| **密码库** | 三栏主页 | `keyvault_3` | `VaultView.vue` | Done | — |
| **密码库** | 10 种条目类型 | `keyvault_1` 仅 4 类 | `templates.ts` 10 类 | Must | 扩展类型选择器原型 |
| **密码库** | 通用详情面板 | `entry_detail_generic` **待生成** | `ItemDetail.vue` | Must | SSH 示例作模板 |
| **密码库** | API 详情 | `api_keyvault` | ItemDetail 变体 | Done | — |
| **密码库** | 笔记详情 | `keyvault_7` | ItemDetail 变体 | Done | — |
| **密码库** | 自定义字段 | 合入新建模态提示词 | `ItemForm.vue` | Should | 表单内「添加字段」 |
| **密码库** | 收藏/标签/搜索 | `keyvault_3` | vault store | Done | — |
| **密码库** | 删除确认 | `modal_delete` **待生成** | delete 逻辑有 | Must | 模态原型 |
| **密码库** | 回收站 | `vault_trash` **待生成** | 需确认后端 | Should | 列表+详情 |
| **密码库** | 空状态 | `empty_states` **待生成** | 部分逻辑 | Should | 三场景合一 |
| **安全工具** | 密码生成器 | `keyvault_4` | `PasswordGenerator.vue` | Done | — |
| **安全工具** | 强度评估 | `_1`/表单 | `PasswordStrength.vue` | Done | — |
| **安全工具** | HIBP 泄露检测 | `keyvault_2` 扩展 **待生成** | `SettingsView.vue` | Should | 设置页区块 |
| **安全工具** | 剪贴板倒计时 | `component_clipboard_timer` **待生成** | `ClipboardTimer.vue` | Should | 全局条原型 |
| **命令** | Ctrl+K 面板 | `keyvault_3` overlay | `CommandPalette.vue` | Done | — |
| **数据** | 导出/导入 JSON | `keyvault_2` | SettingsView | Done | — |
| **快捷键** | Ctrl+K / N / L | 命令面板稿 | `useShortcuts.ts` | Done | — |

---

## 原型完备度评估

| 阶段 | 仅依赖现有 canonical | 补全 Must 原型后 | 补全 Should 后 |
|------|----------------------|------------------|----------------|
| 壳层 + 令牌重构 | 约 70% | 约 85% | 约 95% |
| 完整产品（10 类型+安全工具） | 不足 | 基本可开发 | 推荐上线前完成 |

**结论:** 现有原型 **不能单独** 支撑完整开发；按 [PROTOTYPE-PROMPTS.md](./PROTOTYPE-PROMPTS.md) 补 **Must** 项后即可启动 Vue 重构，**Should** 项可与开发并行。

---

## 待生成原型目录（入库检查表）

生成后在此表更新状态，并放入 `docs/软件界面原型/<目录>/code.html` + `screen.png`。

| 目录 | 优先级 | 状态 | 提示词章节 |
|------|--------|------|------------|
| `setup_1/` 或修订 `_1` | Must | 待生成 | P0-1 |
| `setup_2/` | Must | 待生成 | P0-2 |
| `modal_delete/` | Must | 待生成 | P0-3 |
| `entry_detail_generic/` | Must | 待生成 | P0-4 |
| `unlock_lockout/` | Must | 待生成 | P1-1 |
| `keyvault_1` 扩展 | Must | 待生成 | P1-2 |
| `keyvault_2` 扩展 | Should | 待生成 | P1-3 |
| `modal_change_password/` | Should | 待生成 | P1-4 |
| `vault_trash/` | Should | 待生成 | P2-1 |
| `empty_states/` | Should | 待生成 | P2-2 |
| `component_clipboard_timer/` | Should | 待生成 | P2-3 |
| `modal_emergency_wipe/` | Future | 可选 | Future |

---

## 归档（勿作 v1 canonical）

| 目录 | 原因 |
|------|------|
| `_2` | 恢复密钥；Setup 已改为 2 步，无后端 |
| `keyvault_5` | 与 `keyvault_3` 重复 |
| `keyvault_8` | `keyvault_6` 紧凑变体 |
| `secure_utility_light/` | v1 仅暗色 |

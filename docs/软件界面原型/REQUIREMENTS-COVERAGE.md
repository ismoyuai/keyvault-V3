# KeyVault 需求覆盖矩阵

**版本:** 2026-06-04（Stitch 补充包入库后更新）  
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

## 覆盖矩阵（摘要）

| 需求域 | 具体需求 | 原型 | 代码 | 优先级 | 状态 |
|--------|----------|------|------|--------|------|
| **账户** | 首次设置 2 步 | `setup_1`、 `setup_2` | `SetupView.vue` | Must | **Done** |
| **账户** | 解锁 | `keyvault_6` | `UnlockView.vue` | Done | **Done** |
| **账户** | 5 次失败锁定 | `unlock_lockout` | `auth.rs` | Must | **Done** |
| **账户** | 修改主密码 | `modal_change_password` | Settings 内联 | Should | **Done** |
| **密码库** | 三栏主页 | `keyvault_3` | `VaultView.vue` | Done | **Done** |
| **密码库** | 10 种条目类型 | `keyvault_1` 仅表单 | `templates.ts` | Must | 待扩展类型网格 |
| **密码库** | 通用详情 | `entry_detail_generic` | `ItemDetail.vue` | Must | **Done** |
| **密码库** | 删除确认 | `modal_delete` | delete 逻辑 | Must | **Done** |
| **密码库** | 回收站 | `vault_trash` | 待确认后端 | Should | **Done**（原型） |
| **密码库** | 空状态 | `empty_states` | 部分逻辑 | Should | **Done** |
| **安全工具** | 剪贴板倒计时 | `component_clipboard_timer` | `ClipboardTimer.vue` | Should | **Done** |
| **安全工具** | HIBP 泄露检测 | `keyvault_2` 扩展 | `SettingsView.vue` | Should | 待扩展 |
| **账户** | 紧急擦除 | `modal_emergency_wipe` | 无 | Future | **Done**（原型） |
| **账户** | 恢复密钥 Setup | `_2` 归档 | 无 | Future | 不做 v1 |

---

## 原型入库检查表

| 目录 | 优先级 | 状态 | 来源 |
|------|--------|------|------|
| `setup_1/` · `_1/` | Must | **已入库** | Stitch `1/` |
| `setup_2/` | Must | **已入库** | Stitch `2/` |
| `modal_delete/` | Must | **已入库** | Stitch `keyvault_4/` |
| `entry_detail_generic/` | Must | **已入库** | Stitch `ssh_keyvault/` |
| `unlock_lockout/` | Must | **已入库** | Stitch `keyvault_6/` |
| `modal_new_entry/` · `keyvault_1` | Must | **已入库**（缺 10 类型网格） | Stitch `keyvault_2/` |
| `modal_change_password/` | Should | **已入库** | Stitch `keyvault_1/` |
| `vault_trash/` | Should | **已入库** | Stitch `keyvault_3/` |
| `empty_states/` | Should | **已入库** | Stitch `keyvault_5/` |
| `component_clipboard_timer/` | Should | **已入库** | Stitch `keyvault_7/` |
| `modal_emergency_wipe/` | Future | **已入库** | Stitch `keyvault_8/` |
| `keyvault_2` 扩展 | Should | 待生成 | P1-3 提示词 |
| `keyvault_1` 10 类型 | Must | 待扩展 | P1-2 Step A |

---

## 原型完备度评估

| 阶段 | 状态 |
|------|------|
| Must 原型 | **已齐**（新建条目缺类型选择网格） |
| Should 原型 | **基本齐**（设置页 HIBP/剪贴板下拉待扩展） |
| Vue 重构启动 | **可启动** |

**结论:** Stitch 补充包入库后，桌面端 Must 原型已覆盖；剩余工作为 `keyvault_1` 类型选择器扩展与 `keyvault_2` 设置页安全区块扩展。

---

## 归档（勿作 v1 canonical）

| 目录 | 原因 |
|------|------|
| `_2` | 恢复密钥；Setup 已改为 2 步 |
| `secure_utility_light/` | v1 仅暗色 |
| `stitch_keyvault-软件补充界面/` | 原始导入包；以 canonical 目录为准 |

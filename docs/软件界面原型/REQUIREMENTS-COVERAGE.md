# KeyVault 需求覆盖矩阵

**版本:** 2026-06-04（目录重组后）  
**用途:** 对照产品需求、UI 原型与 Vue/Rust 实现。  
**目录索引:** [PROTOTYPE-INDEX.md](./PROTOTYPE-INDEX.md) · [README.md](./README.md)

---

## 图例

| 优先级 | 含义 |
|--------|------|
| **Must** | 阻塞 UI 重构或核心流程 |
| **Should** | 功能完整、体验完整所需 |
| **Future** | 后端未就绪或 v1 明确不做 |
| **Done** | 已有 canonical 原型且代码已对齐 |

---

## 覆盖矩阵（摘要）

| 需求域 | 具体需求 | Canonical 原型路径 | 代码 | 优先级 | 状态 |
|--------|----------|-------------------|------|--------|------|
| **账户** | 首次设置 2 步 | `screens/account/setup-step1-password/`、`setup-step2-confirm/` | `SetupView.vue` | Must | **Done** |
| **账户** | 解锁 | `screens/account/unlock/` | `UnlockView.vue` | Done | **Done** |
| **账户** | 5 次失败锁定 | `screens/account/unlock-lockout/` | `auth.rs` + `UnlockView` | Must | **Done** |
| **账户** | 修改主密码 | `screens/modals/change-password/` | `ChangePasswordModal.vue` | Should | **Done** |
| **密码库** | 三栏主页 | `screens/vault/main/` | `VaultView.vue` + Shell | Done | **Done** |
| **密码库** | 10 种条目类型 | `screens/modals/new-entry/` | `EntryTypePicker.vue` | Must | **Done** |
| **密码库** | 通用详情 | `screens/vault/entry-detail-ssh/` | `ItemDetail.vue` | Must | **Done** |
| **密码库** | API Key 详情 | `screens/vault/entry-detail-api-key/` | `ItemDetail.vue` | Should | **Done** |
| **密码库** | 删除确认 | `screens/modals/delete-confirm/` | `DeleteConfirmModal.vue` | Must | **Done** |
| **密码库** | 回收站 | `screens/vault/trash/` | `VaultView` + soft-delete | Should | **Done** |
| **密码库** | 空状态 | `screens/vault/empty-states/` | `VaultEmptyState.vue` | Should | **Done** |
| **安全工具** | 剪贴板倒计时 | `screens/components/clipboard-timer/` | `ClipboardTimer.vue` | Should | **Done** |
| **安全工具** | HIBP | `screens/settings/main/`（扩展） | `SettingsView` + `BreachBadge` | Should | **Done** |
| **账户** | 紧急擦除 | `screens/modals/emergency-wipe/` | 无 | Future | 仅原型 |
| **账户** | 恢复密钥 Setup | `_archive/recovery-key-setup/` | 无 | Future | 不做 v1 |
| **密码库** | 笔记变体 B | `screens/vault/note-layout-b/` | 标准三栏 | Should | ⏸ 延后 |

---

## 原型入库检查表

| Canonical 路径 | 优先级 | 状态 |
|----------------|--------|------|
| `screens/account/setup-step1-password/` | Must | **已入库** |
| `screens/account/setup-step2-confirm/` | Must | **已入库** |
| `screens/account/unlock-lockout/` | Must | **已入库** |
| `screens/modals/delete-confirm/` | Must | **已入库** |
| `screens/vault/entry-detail-ssh/` | Must | **已入库** |
| `screens/modals/new-entry/` | Must | **已入库** |
| `screens/modals/change-password/` | Should | **已入库** |
| `screens/vault/trash/` | Should | **已入库** |
| `screens/vault/empty-states/` | Should | **已入库** |
| `screens/components/clipboard-timer/` | Should | **已入库** |
| `screens/modals/emergency-wipe/` | Future | **已入库** |
| `screens/vault/main/` | Done | **已入库**（非 Stitch 包，独立 canonical） |
| `screens/account/unlock/` | Done | **已入库** |
| `screens/modals/password-generator/` | Done | **已入库** |
| `screens/settings/main/` | Done | **已入库** |
| `screens/vault/entry-detail-api-key/` | Should | **已入库** |
| `screens/vault/note-layout-b/` | Should | 延后 |

---

## 结论

- **Canonical 根目录:** 仅使用 `screens/` 下语义化路径；勿引用根目录旧名 `keyvault_*` / `modal_*`。  
- **Stitch 溯源:** `_sources/stitch-2026-06-04/`（见 PROTOTYPE-INDEX §4 易混对照）。  
- **代码:** 桌面 Must/Should 已在 `master` 实现；剩余目视 `screen.png` sign-off 与笔记变体 B。

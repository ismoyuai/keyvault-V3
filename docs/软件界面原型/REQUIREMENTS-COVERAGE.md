# KeyVault 需求覆盖矩阵

**版本:** 2026-06-05（每屏 `code.html` + `screen.png`）  
**用途:** 对照产品需求、UI 原型与 Vue/Rust 实现。  
**入口:** [README.md](./README.md) · **验收:** [SIGNOFF.md](./SIGNOFF.md) · **项目状态:** [PROJECT_STATUS.md](../PROJECT_STATUS.md)

---

## 图例

| 优先级 | 含义 |
|--------|------|
| **Must** | 核心流程 |
| **Should** | 体验完整所需 |
| **Future** | v1 不做 |
| **Done** | 有原型且代码已对齐 |

---

## 覆盖矩阵

| 需求域 | 具体需求 | 原型路径 | 代码 | 状态 |
|--------|----------|----------|------|------|
| 账户 | 首次设置 2 步 | `screens/account/setup-step1-password/`、`setup-step2-confirm/` | `SetupView.vue` | **Done** |
| 账户 | 解锁 | `screens/account/unlock/` | `UnlockView.vue` | **Done** |
| 账户 | 5 次失败锁定 | `screens/account/unlock-lockout/` | `auth.rs` + `UnlockView` | **Done** |
| 账户 | 修改主密码 | `screens/modals/change-password/` | `ChangePasswordModal.vue` | **Done** |
| 密码库 | 三栏主页 | `screens/vault/main/` | `VaultView.vue` + Shell | **Done** |
| 密码库 | 10 种条目类型 | `screens/modals/new-entry/` | `EntryTypePicker.vue` | **Done** |
| 密码库 | 通用详情 | `screens/vault/entry-detail-ssh/` | `ItemDetail.vue` | **Done** |
| 密码库 | API Key 详情 | `screens/vault/entry-detail-api-key/` | `ItemDetail.vue` | **Done** |
| 密码库 | 删除确认 | `screens/modals/delete-confirm/` | `DeleteConfirmModal.vue` | **Done** |
| 密码库 | 回收站 | `screens/vault/trash/` | `VaultView` + soft-delete | **Done** |
| 密码库 | 空状态 | `screens/vault/empty-states/` | `VaultEmptyState.vue` | **Done** |
| 安全工具 | 剪贴板倒计时 | `screens/components/clipboard-timer/` | `ClipboardTimer.vue` | **Done** |
| 安全工具 | HIBP | `screens/settings/main/` | `SettingsView` + `BreachBadge` | **Done** |
| 账户 | 紧急擦除 | `screens/modals/emergency-wipe/` | `EmergencyWipeModal.vue` + `auth.rs` | **Done** |
| 密码库 | 笔记变体 B | `screens/vault/note-layout-b/` | 标准三栏 | ⏸ 延后 |

---

## 结论

- 实现对照 **仅使用** `screens/<分类>/<名>/` 下的 `code.html` 与 `screen.png`。  
- 桌面 Must/Should 已在 `master` 实现；含紧急擦除、导入/导出、WebDAV 同步、Diceware 密码生成。  
- 剩余工作：人工目视 sign-off（[SIGNOFF.md](./SIGNOFF.md)）。  
- **浏览器扩展暂停**（Sprint 4：C-5/C-6/C-7），见 [PROJECT_STATUS.md](../PROJECT_STATUS.md)。

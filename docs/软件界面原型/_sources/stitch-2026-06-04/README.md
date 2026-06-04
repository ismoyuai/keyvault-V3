# Stitch 软件补充界面 · 原始导入包

**来源:** AI 原型工具（Stitch）批量生成  
**入库日期:** 2026-06-04  
**处理脚本:** [`../_shared/ingest-stitch-prototypes.py`](../_shared/ingest-stitch-prototypes.py)

本目录为 **原始导出**，仅作溯源。Canonical 实现请使用 [`../../screens/`](../../screens/)，映射关系见 [PROTOTYPE-INDEX.md §4](../../PROTOTYPE-INDEX.md#4-旧目录名--新路径迁移对照)。

## Stitch → Canonical 映射

| Stitch 子目录 | Canonical `screens/` 路径 |
|---------------|---------------------------|
| `1/` | `account/setup-step1-password/` |
| `2/` | `account/setup-step2-confirm/` |
| `screens/account/unlock/` | `account/unlock-lockout/` |
| `screens/settings/main/` | `modals/new-entry/` |
| `keyvault_1/` | `modals/change-password/` |
| `screens/modals/password-generator/` | `modals/delete-confirm/` |
| `screens/vault/main/` | `vault/trash/` |
| `keyvault_5/` | `vault/empty-states/` |
| `screens/vault/note-layout-b/` | `components/clipboard-timer/` |
| `keyvault_8/` | `modals/emergency-wipe/` |
| `ssh_keyvault/` | `vault/entry-detail-ssh/` |

> **注意:** Stitch 的 `keyvault_3` 是回收站，不是主界面；主界面对应仓库根目录旧名 `screens/vault/main/` → 现 `screens/vault/main/`。

## 重新入库

```bash
python docs/软件界面原型/_shared/ingest-stitch-prototypes.py
```

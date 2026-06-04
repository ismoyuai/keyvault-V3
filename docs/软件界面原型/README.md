# KeyVault UI 原型索引

> **重构必读:** [UNIFIED-SPEC.md](./UNIFIED-SPEC.md) · [REQUIREMENTS-COVERAGE.md](./REQUIREMENTS-COVERAGE.md) · [PROTOTYPE-INDEX.md](./PROTOTYPE-INDEX.md)（目录关系与旧名映射）

## 快速定位

| 需求 | 路径 |
|------|------|
| 所有 canonical 屏幕 | [`screens/`](./screens/) |
| 设计令牌 | [`design-system/DESIGN.md`](./design-system/DESIGN.md) |
| Stitch 原始包 | [`_sources/stitch-2026-06-04/`](./_sources/stitch-2026-06-04/) |
| 归档 / 重复 | [`_archive/`](./_archive/) |
| 补缺口 AI 提示词 | [PROTOTYPE-PROMPTS.md](./PROTOTYPE-PROMPTS.md) |
| 浏览器扩展 | [EXTENSION-PROTOTYPE-PROMPTS.md](./EXTENSION-PROTOTYPE-PROMPTS.md) |

## Canonical 屏幕（`screens/`）

| 路径 | 界面 | Vue 目标 |
|------|------|----------|
| `screens/account/setup-step1-password/` | Setup 第 1 步 | `SetupView.vue` |
| `screens/account/setup-step2-confirm/` | Setup 第 2 步 | `SetupView.vue` |
| `screens/account/unlock/` | 解锁 | `UnlockView.vue` |
| `screens/account/unlock-lockout/` | 解锁锁定 | `UnlockView.vue` |
| `screens/vault/main/` | 主界面 + 命令面板 | `VaultView.vue` |
| `screens/vault/trash/` | 回收站 | `VaultView` |
| `screens/vault/empty-states/` | 空状态 | `VaultEmptyState.vue` |
| `screens/vault/entry-detail-ssh/` | 通用详情 | `ItemDetail.vue` |
| `screens/vault/entry-detail-api-key/` | API Key 详情 | `ItemDetail.vue` |
| `screens/vault/note-layout-b/` | 笔记变体 B | ⏸ 延后 |
| `screens/modals/new-entry/` | 新建条目 | `ItemForm.vue` |
| `screens/modals/delete-confirm/` | 删除确认 | `DeleteConfirmModal.vue` |
| `screens/modals/change-password/` | 修改主密码 | `ChangePasswordModal.vue` |
| `screens/modals/password-generator/` | 密码生成器 | `PasswordGenerator.vue` |
| `screens/modals/emergency-wipe/` | 紧急擦除 | Future |
| `screens/settings/main/` | 设置 | `SettingsView.vue` |
| `screens/components/clipboard-timer/` | 剪贴板倒计时 | `ClipboardTimer.vue` |

完整关系图与 **旧目录名对照** 见 [PROTOTYPE-INDEX.md](./PROTOTYPE-INDEX.md)。

## 工具脚本

| 脚本 | 用途 |
|------|------|
| `_shared/ingest-stitch-prototypes.py` | Stitch 包 → `screens/` |
| `_shared/apply-unification.py` | 批量文案/色值统一 |

```bash
python docs/软件界面原型/_shared/ingest-stitch-prototypes.py
```

## 布局变体（UNIFIED-SPEC）

| 变体 | 说明 |
|------|------|
| A | 标准三栏：侧栏 260 + 列表 flex + 详情 400 |
| B | 笔记：列表 320 + 详情 flex（`note-layout-b`，v1 延后） |
| C | 设置：SettingsNav 192 + 内容区 |
| D/E | Setup / Unlock 居中卡片 |
| F | 模态：新建 560 / 删除 420 / 生成器 480 |

## 入库流程

1. 生成 `code.html` + `screen.png` 到目标 `screens/.../` 目录  
2. HTML 顶部加 `<!-- CANONICAL: ... -->`  
3. 运行统一脚本（若来自 Stitch，先 `ingest-stitch-prototypes.py`）  
4. 更新 [REQUIREMENTS-COVERAGE.md](./REQUIREMENTS-COVERAGE.md) 与 [PROTOTYPE-INDEX.md](./PROTOTYPE-INDEX.md)

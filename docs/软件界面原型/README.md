# KeyVault UI 原型索引

> **重构前必读:** [UNIFIED-SPEC.md](./UNIFIED-SPEC.md) · [REQUIREMENTS-COVERAGE.md](./REQUIREMENTS-COVERAGE.md)  
> **补缺口原型:** [PROTOTYPE-PROMPTS.md](./PROTOTYPE-PROMPTS.md)（桌面）· [EXTENSION-PROTOTYPE-PROMPTS.md](./EXTENSION-PROTOTYPE-PROMPTS.md)（扩展）  
> **Stitch 入库:** [stitch_keyvault-软件补充界面/README.md](./stitch_keyvault-软件补充界面/README.md)  
> **设计令牌:** [keyvault/DESIGN.md](./keyvault/DESIGN.md) · [_shared/tailwind-extend.json](./_shared/tailwind-extend.json)

## 文档

| 文件 | 说明 |
|------|------|
| `keyvault/DESIGN.md` | 主规范（暗色，v1 唯一主题） |
| `UNIFIED-SPEC.md` | 壳层、布局、组件 canonical 决策 |
| `REQUIREMENTS-COVERAGE.md` | 产品需求 ↔ 原型 ↔ 代码覆盖矩阵 |
| `PROTOTYPE-PROMPTS.md` | 桌面缺口界面 AI 补图提示词 |
| `EXTENSION-PROTOTYPE-PROMPTS.md` | 浏览器扩展 AI 提示词 |
| `_shared/ingest-stitch-prototypes.py` | Stitch 补充包入库脚本 |
| `_archive/README.md` | 归档原型说明 |

## Canonical 原型（已入库）

| 目录 | 界面 | Vue 目标 |
|------|------|----------|
| `setup_1/` · `_1/` | Setup 第 1 步 | `SetupView.vue` |
| `setup_2/` | Setup 第 2 步 | `SetupView.vue` |
| `keyvault_6` | 解锁 | `UnlockView.vue` |
| `unlock_lockout/` | 解锁锁定态 | `UnlockView.vue` |
| `keyvault_3` | 主界面 + 命令面板 | `VaultView.vue` |
| `modal_delete/` | 删除确认 | `KvModal` |
| `entry_detail_generic/` | 通用详情（SSH） | `ItemDetail.vue` |
| `api_keyvault` | API Key 详情 | `ItemDetail.vue` |
| `keyvault_7` | 安全笔记 | Note 变体 |
| `keyvault_1/` · `modal_new_entry/` | 新建网站密码 | `ItemForm.vue` |
| `modal_change_password/` | 修改主密码 | `SettingsView.vue` |
| `keyvault_4` | 密码生成器 | `PasswordGenerator.vue` |
| `keyvault_2` | 设置 Bento | `SettingsView.vue` |
| `vault_trash/` | 回收站 | `VaultView` trash 路由 |
| `empty_states/` | 空状态三场景 | 列表/搜索/回收站 |
| `component_clipboard_timer/` | 剪贴板倒计时 | `ClipboardTimer.vue` |
| `modal_emergency_wipe/` | 紧急擦除 Future | — |

## 待生成 / 待扩展

| 目录 | 优先级 | 说明 |
|------|--------|------|
| `keyvault_1` 10 类型网格 | Must | 新建模态需补类型选择 Step A（见 P1-2） |
| `keyvault_2` 扩展 | Should | HIBP 泄露检测、剪贴板设置（见 P1-3） |
| `extension/*` | P0+ | 见 EXTENSION-PROTOTYPE-PROMPTS.md |

## 浏览器扩展原型（待生成）

见 [EXTENSION-PROTOTYPE-PROMPTS.md](./EXTENSION-PROTOTYPE-PROMPTS.md) → `extension/` 子目录。

## 归档（勿作 v1 实现依据）

见 [_archive/README.md](./_archive/README.md)：`_2`（恢复密钥）、`secure_utility_light/`（浅色）。

> 注：原归档的 `keyvault_5`/`keyvault_8` 已由 Stitch 包重新定义为 `empty_states`/`modal_emergency_wipe` canonical。

## 布局变体

```
标准三栏:     TopBar 40px | Side 260px | List flex | Detail 400px
笔记三栏:     TopBar 40px | Side 260px | List 320px | Detail flex
设置双导航:   Side 260px | SettingsNav 192px | Content flex
Setup:        TopBar 40px | 居中卡片（2 步，无侧栏）
解锁:         TopBar 40px | 居中卡片 max ~480px
模态:         新建 560px | 生成器 480px | 删除 420px
```

## 入库流程

1. AI 生成 → `docs/软件界面原型/<目录>/`（`code.html` + `screen.png`）
2. HTML 顶部加 `<!-- CANONICAL: ... -->`
3. 运行 `python _shared/apply-unification.py` 或 `ingest-stitch-prototypes.py`
4. 更新 [REQUIREMENTS-COVERAGE.md](./REQUIREMENTS-COVERAGE.md)

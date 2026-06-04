# KeyVault UI 原型索引

> **重构前必读:** [UNIFIED-SPEC.md](./UNIFIED-SPEC.md) · [REQUIREMENTS-COVERAGE.md](./REQUIREMENTS-COVERAGE.md)  
> **补缺口原型:** [PROTOTYPE-PROMPTS.md](./PROTOTYPE-PROMPTS.md)（AI 工具提示词）  
> **设计令牌:** [keyvault/DESIGN.md](./keyvault/DESIGN.md) · [_shared/tailwind-extend.json](./_shared/tailwind-extend.json)

## 文档

| 文件 | 说明 |
|------|------|
| `keyvault/DESIGN.md` | 主规范（暗色，v1 唯一主题） |
| `UNIFIED-SPEC.md` | 壳层、布局、组件 canonical 决策 |
| `REQUIREMENTS-COVERAGE.md` | 产品需求 ↔ 原型 ↔ 代码覆盖矩阵 |
| `PROTOTYPE-PROMPTS.md` | 缺口界面 AI 补图提示词（12+1 条） |
| `_archive/README.md` | 归档原型说明 |

## 已有 Canonical 原型

| 目录 | 界面 | 说明 |
|------|------|------|
| `_1` | Setup Step 1 | 设主密码（需修订为 STEP 1/2，见 P0-1） |
| `keyvault_6` | 解锁 | 桌面完整版 |
| `keyvault_3` | 主界面 | 三栏 + 命令面板 |
| `api_keyvault` | API Key | 专用详情 |
| `keyvault_7` | 安全笔记 | 窄列表 + 宽阅读区 |
| `keyvault_1` | 新建条目 | 模态（需扩展 10 类型，见 P1-2） |
| `keyvault_4` | 密码生成器 | 模态 |
| `keyvault_2` | 设置 | 双导航 Bento（需扩展，见 P1-3） |

## 待生成原型（用户 + AI 工具）

按 [PROTOTYPE-PROMPTS.md](./PROTOTYPE-PROMPTS.md) 顺序生成，放入下列目录并更新 [REQUIREMENTS-COVERAGE.md](./REQUIREMENTS-COVERAGE.md) 检查表：

| 目录 | 优先级 | 提示词 |
|------|--------|--------|
| `setup_2/` | Must | P0-2 |
| `modal_delete/` | Must | P0-3 |
| `entry_detail_generic/` | Must | P0-4 |
| `unlock_lockout/` | Must | P1-1 |
| `modal_change_password/` | Should | P1-4 |
| `vault_trash/` | Should | P2-1 |
| `empty_states/` | Should | P2-2 |
| `component_clipboard_timer/` | Should | P2-3 |
| `modal_emergency_wipe/` | Future | Future |

修订现有目录（非新建）：`_1`（P0-1）、`keyvault_1`（P1-2）、`keyvault_2`（P1-3）。

## 归档（勿作 v1 实现依据）

见 [_archive/README.md](./_archive/README.md)：`_2`、`keyvault_5`、`keyvault_8`、`secure_utility_light/`。

## 布局变体

```
标准三栏:     TopBar 40px | Side 260px | List flex | Detail 400px
笔记三栏:     TopBar 40px | Side 260px | List 320px | Detail flex
设置双导航:   Side 260px | SettingsNav 192px | Content flex
Setup:        TopBar 40px | 居中卡片（2 步，无侧栏）
解锁:         TopBar 40px | 居中卡片 max ~480px
模态:         新建 560px | 生成器 480px | 命令面板 600px
```

## Vue 映射

| 原型 | 目标 |
|------|------|
| `_1`, `setup_2` | `SetupView.vue` |
| `keyvault_6`, `unlock_lockout` | `UnlockView.vue` |
| `keyvault_3` | `VaultView.vue` |
| `api_keyvault`, `entry_detail_generic` | `ItemDetail.vue` |
| `keyvault_7` | Note 变体 |
| `keyvault_1` | `ItemForm.vue` |
| `keyvault_4` | `PasswordGenerator.vue` |
| `keyvault_2`, `modal_change_password` | `SettingsView.vue` |
| `keyvault_3` overlay | `CommandPalette.vue` |
| `modal_delete` | `KvModal` 删除流 |
| `component_clipboard_timer` | `ClipboardTimer.vue` |

## 入库流程

1. AI 工具生成 → 保存至 `docs/软件界面原型/<目录>/`
2. 文件命名：`code.html`（可选）+ `screen.png`
3. HTML 顶部加 `<!-- CANONICAL: ... -->` 注释
4. 运行 `python _shared/apply-unification.py`（若需批量中文化）
5. 更新 `REQUIREMENTS-COVERAGE.md` 状态 → 本 README 待生成表

# Stitch 软件补充界面 · 入库说明

**来源:** AI 原型工具批量生成（Stitch）  
**入库日期:** 2026-06-04  
**处理脚本:** [`../_shared/ingest-stitch-prototypes.py`](../_shared/ingest-stitch-prototypes.py)

本目录为 **原始导入包**。经统一规范处理后，canonical 版本已迁移至 `docs/软件界面原型/<目录>/`。开发请以 canonical 目录为准，本目录仅作溯源参考。

## 文件映射

| 本目录 | Canonical 目录 | 界面 |
|--------|----------------|------|
| `1/` | `setup_1/`、`_1/` | Setup 第 1 步 |
| `2/` | `setup_2/` | Setup 第 2 步（确认主密码） |
| `keyvault_4/` | `modal_delete/` | 删除确认模态 |
| `ssh_keyvault/` | `entry_detail_generic/` | SSH 通用详情 |
| `keyvault_6/` | `unlock_lockout/` | 解锁锁定态 |
| `keyvault_2/` | `modal_new_entry/`、`keyvault_1/` | 新建网站密码模态 |
| `keyvault_1/` | `modal_change_password/` | 修改主密码模态 |
| `keyvault_3/` | `vault_trash/` | 回收站三栏 |
| `keyvault_5/` | `empty_states/` | 空状态三场景 |
| `keyvault_7/` | `component_clipboard_timer/` | 剪贴板倒计时条 |
| `keyvault_8/` | `modal_emergency_wipe/` | 紧急擦除（Future） |

## 已执行的统一修复

- 添加 `<!-- CANONICAL: ... -->` 注释
- 导航/按钮文案中文化（全部条目、设置、锁定等）
- 色值对齐 `#0D1117` / `#161B22` / `#30363D`（见 UNIFIED-SPEC）
- 侧栏选中态统一 `border-l-2`
- 主应用 TopBar 使用 `shield_lock`；Setup/Unlock 使用 `vpn_key`
- 移除 `KeyVault Pro` 等非规范品牌名

## 重新入库

若更新了本目录 HTML，运行：

```bash
python docs/软件界面原型/_shared/ingest-stitch-prototypes.py
```

然后手动检查 `entry_detail_generic/`、`component_clipboard_timer/` 等需结构性调整的页面，并更新 [REQUIREMENTS-COVERAGE.md](../REQUIREMENTS-COVERAGE.md)。

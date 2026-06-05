# 前端原型重构 — 目视验收清单

> **状态:** 代码侧已完成；本清单供人工 `npm run tauri dev` 签字  
> **日期:** 2026-06-05（自归档计划迁入）  
> **关联:** [REQUIREMENTS-COVERAGE.md](./REQUIREMENTS-COVERAGE.md) · [PROJECT_STATUS.md](../PROJECT_STATUS.md)

## 运行方式

```powershell
cd d:\keyvault-V3
npm run tauri dev
```

对照 `docs/软件界面原型/screens/<分类>/<名>/screen.png` 与 `code.html`。

## UNIFIED-SPEC §6 全局

- [ ] TopBar 40px，`shield_lock` + KeyVault 文案
- [ ] 侧栏 260px，选中项 `border-l-2` + `#58a6ff`
- [ ] 侧栏底部「设置」「锁定」文字链（非仅图标）
- [ ] 无组件内裸 hex（仅 `tokens.css`）
- [ ] 敏感字段 JetBrains Mono
- [ ] 模态 blur + 宽度：新建 560 / 删除 420 / 生成器 480 / 命令 600

## 逐屏

| 原型目录 | 路由/触发 | 验收人 | 日期 | 通过 |
|----------|-----------|--------|------|------|
| `setup-step1` + `setup-step2` | 首次启动 `/setup` | | | [ ] |
| `unlock-lockout` | `/unlock` 连续输错 5 次 | | | [ ] |
| `screens/vault/main/` | `/vault` 三栏 + 列表 | | | [ ] |
| `empty-states` | 空库 / 无搜索结果 / 空回收站 | | | [ ] |
| `modals/new-entry` | Ctrl+N 新建 + 类型选择 | | | [ ] |
| `entry-detail-ssh` | 选中条目详情 | | | [ ] |
| `modals/delete-confirm` | 删除 → 移至回收站 | | | [ ] |
| `modals/password-generator` | 密码生成器模态 | | | [ ] |
| `vault/trash` | 侧栏回收站 + 恢复/永久删除 | | | [ ] |
| `settings/main` | `/settings` 双导航 | | | [ ] |
| `modals/change-password` | 设置 → 修改主密码 | | | [ ] |
| `components/clipboard-timer` | Vault 列表头倒计时胶囊 | | | [ ] |
| 命令面板 overlay | Ctrl+K | | | [ ] |

## 延后（v1 不验收）

- `screens/vault/note-layout-b` 笔记布局变体 B
- 浏览器扩展（Sprint 4，见 [PROJECT_STATUS.md](../PROJECT_STATUS.md)）

## 新增验收项

| 功能 | 路由/触发 | 通过 |
|------|-----------|------|
| 导入 JSON | 设置 → 数据 → 导入 | [ ] |
| WebDAV 同步 | 设置 → 数据 → 同步配置 | [ ] |
| 紧急擦除 | 设置 → 安全 → 紧急擦除 | [ ] |
| Diceware 模式 | 密码生成器 → Diceware 词组 | [ ] |

## 性能与安全（可选）

| 项 | 标准 | 通过 |
|----|------|------|
| 冷启动 | < 2s | [ ] |
| 空闲内存 | < 60MB | [ ] |
| 安装包体积 | < 20MB | [ ] |
| SQLCipher CLI | 锁定后 `sqlite3 keyvault.db` 不可读 | [ ] |

## 签字

- **验收人:** _______________
- **结论:** [ ] 通过发布 v1  /  [ ] 需返工（备注：_______________）

# KeyVault UI 原型统一规范

**版本:** 2026-06-04  
**目的:** 梳理 AI 生成原型中的不一致项，确立 canonical 决策，供 Vue/Tauri 重构直接引用。

---

## 1. 原型审计摘要

AI 原型在视觉方向上一致（GitHub Dark + Inter + JetBrains Mono），但存在以下 **7 类不一致**：

| # | 类别 | 典型问题 | 影响 |
|---|------|----------|------|
| A | 设计令牌 | YAML `#0b141c` vs 正文/HTML `#0D1117`；大量硬编码 hex | 实现时色值漂移 |
| B | 应用壳层 | 标题栏 h-10 / h-12 / h-14；Logo 图标 key / shield_lock / vpn_key | 壳层组件无法复用 |
| C | 侧栏 | 底部 Settings/Lock：纯图标 vs 文字链；选中态 border-l-2 vs border-r-2 vs 全边框 | 导航交互不统一 |
| D | 语言 | lang=en 与 lang=zh-CN 混用；UI 文案中英混杂 | i18n 与产品调性 |
| E | 布局变体 | 笔记视图中间栏 320px vs 标准 flex；设置页双导航 | 需明确为「变体」非错误 |
| F | 重复原型 | keyvault_5 ≈ keyvault_3；keyvault_8 ≈ keyvault_6 | 开发引用混乱 |
| G | 流程缺口 | Setup 已改为 2 步；`setup_2` 等待生成 | 见 REQUIREMENTS-COVERAGE |

---

## 2. Canonical 决策（重构必须遵循）

### 2.1 色彩与分层

**以 GitHub Dark Dimmed 语义层为准**（与 HTML 硬编码对齐，YAML 已同步至 DESIGN.md）：

| 令牌 | Hex | 用途 |
|------|-----|------|
| `background` | `#0D1117` | 主工作区、列表区底色 |
| `surface-container-low` | `#161B22` | 侧栏、详情面板 |
| `surface-container` | `#1C2128` | 命令面板、模态、选中容器 |
| `surface-bright` / 列表 hover | `#21262D` | 列表选中行 |
| `border-hairline` / `outline-variant` | `#30363D` | 所有 1px 结构边框 |
| `primary-container` | `#58A6FF` | 主按钮、焦点、链接 |
| `on-primary-container` | `#0D1117` | 主按钮文字 |
| `secondary` / 强度 Strong | `#3FB950` | 成功、强密码 |
| `tertiary-container` | `#DA9600` | Fair 强度 |
| `error` | `#F85149` | 错误、Weak 强度 |

**禁止**在新代码中使用未命名 hex；原型 HTML 逐步改为 Tailwind 语义类（`bg-background`、`border-outline-variant`）。

### 2.2 字体与图标

| 元素 | 规范 |
|------|------|
| UI 文字 | Inter |
| 密码 / API Key / 恢复密钥 / 代码 | JetBrains Mono (`font-code-md`) |
| 字段标签 | `label-caps`：11px、700、0.05em 字距、大写 |
| 图标库 | **Material Symbols Outlined**（实现阶段替换 Lucide） |

**品牌图标：**

| 场景 | 图标 | Material Symbol |
|------|------|-----------------|
| 主应用 TopBar | 盾牌锁 | `shield_lock` (FILL) |
| 设置/解锁/Setup TopBar | 钥匙 | `vpn_key` |
| 窗口内条目类型 | 见 UNIFIED 组件表 | — |

### 2.3 语言策略

- **v1 界面语言：简体中文（zh-CN）**
- 安全/技术术语保留英文缩写：API Key、JSON、AES-256、Zero-Knowledge、Cmd+K
- 原型 `lang` 统一为 `zh-CN`；遗留英文文案在重构时翻译，不保留双语并列

### 2.4 应用壳层（App Shell）

#### TopAppBar — 所有带窗口控件的界面

```
高度: 40px (h-10)
背景: background (#0D1117)
底边框: 1px outline-variant
左: shield_lock + "KeyVault" (headline-md, primary, bold)
中: 拖拽区 (-webkit-app-region: drag)
右: minimize | maximize | close (no-drag, 18px 图标)
```

**例外：**

| 界面 | TopBar 差异 |
|------|-------------|
| Setup `_1` | 同标准 TopBar + `vpn_key` 可选 |
| Setup `_2` | **已归档 Future**（恢复密钥）；v1 用 `_1` + `setup_2` |
| Unlock `keyvault_6` | h-12 → **统一 h-10**；保留窗口控件 |
| Unlock `keyvault_8` | 无窗口控件（紧凑变体，仅作窄屏参考） |
| Settings `keyvault_2` | 无主 TopBar；窗口控件在内容区顶栏 h-10 |

#### 侧栏（SideNav）— 260px 固定

**结构顺序（自上而下）：**

1. Vault 选择器（头像 KV + 名称 + Local Vault + expand_more）
2. **新建条目** CTA — 全宽主按钮
3. 主导航（带计数可选）
4. Tags 分组（可选，带色点）
5. Trash
6. 底部：**Settings** + **Lock**（图标 + 文字，非纯图标）

**主导航项（顺序固定）：**

| ID | 标签 | 图标 |
|----|------|------|
| all | 全部条目 | inventory_2 |
| favorites | 收藏 | star |
| passwords | 密码 | key |
| api-keys | API 密钥 | code |
| notes | 安全笔记 | description |
| trash | 回收站 | delete |

**选中态（统一）：**

```html
<!-- canonical active nav item -->
class="flex items-center gap-md px-md py-sm rounded-lg
       text-primary font-bold
       bg-surface-container border-l-2 border-primary
       transition-all duration-150"
```

- 使用 **左边框 2px primary**，不用 border-r-2
- 背景 `bg-surface-container` (#1C2128)，非全边框盒子

**CTA 按钮（统一）：**

```html
class="w-full bg-primary-container text-[#0D1117] font-bold
       py-sm px-md rounded flex items-center justify-center gap-sm
       hover:brightness-110 transition-colors"
```

标签：**新建条目**（非 New Entry）

**侧栏底部（统一）：**

```html
<div class="px-sm pt-md border-t border-outline-variant flex flex-col gap-xs">
  <a>settings + 设置</a>
  <a>lock + 锁定</a>
</div>
```

`keyvault_3/5` 的纯图标底部改为上述文字链（已在规范层明确，HTML 重构阶段同步）。

### 2.5 布局变体（均为 intentional）

#### 变体 A — 标准三栏（Password、API Key、All Items）

```
┌──────────────────────────────────────────────────────────┐
│ TopAppBar 40px                                           │
├──────────┬─────────────────────┬─────────────────────────┤
│ Side     │ List (flex-1)       │ Detail 400px fixed      │
│ 260px    │ min-width 300px     │                         │
└──────────┴─────────────────────┴─────────────────────────┘
```

- **Canonical 原型:** `keyvault_3`（含命令面板）, `api_keyvault`
- `keyvault_5` = `_3` 去掉命令面板，**不作为独立设计**

#### 变体 B — 笔记三栏（Secure Notes）

```
Side 260px | List 320px fixed | Detail flex (阅读区 max 800px)
```

- **Canonical 原型:** `keyvault_7`
- 详情区：Markdown 文档排版 + 水印 + 代码块复制

#### 变体 C — 设置双导航

```
Side 260px (全局) | SettingsNav 192px | Content flex (max-w-3xl 居中内容)
```

- **Canonical 原型:** `keyvault_2`
- 设置子项：常规 | 安全 | 数据 | 关于

#### 变体 D — Setup 向导（2 步，v1）

| Step | 布局 | 原型 |
|------|------|------|
| 1 主密码 | 无侧栏，居中卡片 max-w-lg | `_1`（STEP 1 OF 2） |
| 2 确认 | 同 Step 1 壳层 | `setup_2/` **待生成**（见 PROTOTYPE-PROMPTS P0-2） |

~~原 `_2` 恢复密钥流程~~ 已归档至 `_archive/`，Future 功能。

#### 变体 E — 解锁（Transactional）

- 无侧栏；居中卡片 max-w-[480px]
- **Canonical:** `keyvault_6`
- **Compact variant:** `keyvault_8`（footer 外置、无窗口按钮）

#### 变体 F — 模态

| 模态 | 宽度 | 原型 |
|------|------|------|
| 新建条目 | max-w-[560px] | `keyvault_1` |
| 密码生成器 | max-w-[480px] | `keyvault_4` |
| 命令面板 | max-w-[600px] | `keyvault_3` 叠加层 |

遮罩：`bg-background/80 backdrop-blur-md`

---

## 3. 组件级统一

### 3.1 列表行

- 行高：40–48px（`py-sm` + 图标 40px）
- 图标容器：40×40，`rounded`，`border outline-variant`
- 选中：`bg-[#21262D]` + `border outline-variant`
- Hover：`bg-surface-container-low`

### 3.2 详情字段（Secure Field）

- 容器：`bg-background border outline-variant rounded-lg p-sm`
- 敏感值：`font-code-md`，密码 `tracking-widest`
- 操作按钮：hover 行/组时显示 copy / visibility / open_in_new
- Focus：`border-primary-container` + `ring-1 ring-primary/20`

### 3.3 密码强度条

4 段水平条，颜色：Weak `#F85149` → Fair `#DA9600` → Good `#FFBA42` → Strong `#3FB950`

### 3.4 命令面板

- 触发：⌘K / Ctrl+K
- 分组：**最近条目** | **常用命令**
- 命令（v1 最小集）：

| 命令 | 快捷键 |
|------|--------|
| 新建条目 | Ctrl+N |
| 锁定密码库 | Ctrl+L |
| 打开设置 | , |

### 3.5 新建条目类型

| 类型 | v1 实现 | 说明 |
|------|---------|------|
| Login | ✅ | 默认 |
| API Key | ✅ | 字段见 api_keyvault |
| Note | ✅ | 详情用变体 B |
| Card | ⏸ 延后 | 原型有关 UI，**v1 可隐藏或 disabled**；需数据模型支持后再开 |

---

## 4. 各原型修正记录（2026-06-04 已完成）

| 文件 | 状态 | 说明 |
|------|------|------|
| `keyvault_5` | ✅ | 标注 DUPLICATE；导航中文化 |
| `keyvault_8` | ✅ | 标注 VARIANT；解锁文案中文化 |
| `keyvault_6` | ✅ | 解锁文案中文化 |
| `api_keyvault` | ✅ | lang zh-CN；导航中文化 |
| `keyvault_1` | ✅ | lang zh-CN；模态表单中文化 |
| `keyvault_7` | ✅ | lang zh-CN；导航中文化；选中态 border-l-2 |
| `keyvault_2/3/4` | ✅ | CANONICAL 注释 + 导航中文化 |
| `_1/_2` | ✅ | CANONICAL 注释 |
| `keyvault/DESIGN.md` | ✅ | YAML 色值与语义层对齐 |
| `_shared/tailwind-extend.json` | ✅ | 共享令牌 |
| `_shared/apply-unification.py` | ✅ | 批量文案统一脚本（可重复运行） |
| `secure_utility_light/DESIGN.md` | ✅ | 标注 v1 归档 |

**待 Phase 2（非阻塞）：** HTML 内硬编码 hex 改为语义类；按 [PROTOTYPE-PROMPTS.md](./PROTOTYPE-PROMPTS.md) 补缺口原型。

**文档（2026-06-04）：** [REQUIREMENTS-COVERAGE.md](./REQUIREMENTS-COVERAGE.md) · [PROTOTYPE-PROMPTS.md](./PROTOTYPE-PROMPTS.md) · [_archive/README.md](./_archive/README.md)

---

## 5. 产品决策（原型无法自答）

重构前需与后端/产品确认：

| # | 问题 | 建议默认 |
|---|------|----------|
| 1 | 恢复密钥 Setup | **`_2` 已归档**；v1 Setup 为 `_1` + `setup_2`（2 步确认） |
| 2 | 生物识别解锁 | UI 保留入口；无能力则 disabled + tooltip |
| 3 | Emergency Wipe | 保留；需二次确认 + 输入主密码 |
| 4 | 忘记主密码 | 链至恢复密钥说明，非 email reset |
| 5 | Card 条目类型 | v1 隐藏 |
| 6 | Secure Note Markdown | v1 纯文本 + 代码块样式；完整 MD 渲染 Phase 2 |
| 7 | 多 Vault / KeyVault Pro | v1 仅 Local Vault；选择器 UI 保留但不可切换 |
| 8 | 亮色主题 | v1 不做；设置页「已锁定」 |

---

## 6. 重构检查清单

实现每个界面时核对：

- [ ] TopBar 40px + 正确 Logo
- [ ] 侧栏 260px + 统一选中态（border-l-2）
- [ ] 侧栏底部「设置」「锁定」文字链
- [ ] 主色/边框使用 CSS 变量或 tokens.css，无裸 hex
- [ ] 敏感字段 JetBrains Mono
- [ ] 中文文案
- [ ] 布局变体正确（标准 / 笔记 / 设置 / Setup / Unlock）
- [ ] 模态宽度与 blur 遮罩
- [ ] 对照 README 中的 canonical 原型截图

---

## 7. 附录：重复与归档对照

| 保留 | 合并自 / 归档 | 说明 |
|------|----------------|------|
| keyvault_3 | keyvault_5 | 命令面板 overlay |
| keyvault_6 | keyvault_8 | 窗口控件、尺寸 |
| `_1` + setup_2 | `_2` | v1 两步入门 vs 恢复密钥 Future |
| keyvault/DESIGN.md | secure_utility_light | v1 仅暗色 |

完整归档索引：[_archive/README.md](./_archive/README.md)

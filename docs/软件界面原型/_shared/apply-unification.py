# -*- coding: utf-8 -*-
"""One-shot script to finish prototype UI string unification.

After new prototypes are added from AI tools:
1. Add per-file REPLACEMENTS below if needed
2. Run: python docs/软件界面原型/_shared/apply-unification.py
3. Update REQUIREMENTS-COVERAGE.md checklist
4. Add <!-- CANONICAL: ... --> to top of new code.html
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

REPLACEMENTS = {
    "keyvault_1/code.html": [
        ("Website URL", "网站 URL"),
        ("Username / Email", "用户名 / 邮箱"),
        (">Password</label>", ">密码</label>"),
        ("> Generate", "> 生成"),
        (
            'Strength: <span class="text-secondary-fixed">Strong</span>',
            '强度：<span class="text-secondary-fixed">强</span>',
        ),
        (">Tags</label>", ">标签</label>"),
        ('placeholder="Add tag..."', 'placeholder="添加标签..."'),
        ("Secure Notes", "安全笔记"),
        (
            'placeholder="Add encrypted notes here..."',
            'placeholder="在此输入加密笔记..."',
        ),
        ("                    Cancel", "                    取消"),
        ("                    Save Entry", "                    保存条目"),
        (
            'placeholder="e.g. Github Personal"',
            'placeholder="例如：GitHub 个人账号"',
        ),
        ('title="Copy username"', 'title="复制用户名"'),
        ('title="Toggle visibility"', 'title="切换可见性"'),
        ('title="Copy password"', 'title="复制密码"'),
        ('title="Notes are encrypted"', 'title="笔记已加密"'),
        ('aria-label="Close"', 'aria-label="关闭"'),
    ],
    "keyvault_2/code.html": [
        (
            "<!DOCTYPE html>\n\n<html",
            "<!DOCTYPE html>\n<!-- CANONICAL: 设置页（双导航 + Bento 卡片）。见 UNIFIED-SPEC.md §2.5 变体 C -->\n<html",
        ),
        ("                New Entry", "                新建条目"),
        ("                All Items", "                全部条目"),
        ("                Favorites", "                收藏"),
        ("                Passwords", "                密码"),
        ("                API Keys", "                API 密钥"),
        ("                Secure Notes", "                安全笔记"),
        ("                Trash", "                回收站"),
        ("                Settings", "                设置"),
        ("                Lock", "                锁定"),
    ],
    "keyvault_3/code.html": [
        ('<span class="flex-1">All Items</span>', '<span class="flex-1">全部条目</span>'),
        ('<span class="flex-1">Favorites</span>', '<span class="flex-1">收藏</span>'),
        ('<span class="flex-1">Passwords</span>', '<span class="flex-1">密码</span>'),
        ('<span class="flex-1">API Keys</span>', '<span class="flex-1">API 密钥</span>'),
        ('<span class="flex-1">Secure Notes</span>', '<span class="flex-1">安全笔记</span>'),
        ('<span class="flex-1">Trash</span>', '<span class="flex-1">回收站</span>'),
        (
            'placeholder="Search entries (Cmd+K)"',
            'placeholder="搜索条目 (Cmd+K)"',
        ),
    ],
    "keyvault_5/code.html": [
        ('<span class="flex-1">All Items</span>', '<span class="flex-1">全部条目</span>'),
        ('<span class="flex-1">Favorites</span>', '<span class="flex-1">收藏</span>'),
        ('<span class="flex-1">Passwords</span>', '<span class="flex-1">密码</span>'),
        ('<span class="flex-1">API Keys</span>', '<span class="flex-1">API 密钥</span>'),
        ('<span class="flex-1">Secure Notes</span>', '<span class="flex-1">安全笔记</span>'),
        ('<span class="flex-1">Trash</span>', '<span class="flex-1">回收站</span>'),
        (
            'placeholder="Search entries (Cmd+K)"',
            'placeholder="搜索条目 (Cmd+K)"',
        ),
    ],
    "_1/code.html": [
        (
            "<!DOCTYPE html>\n\n<html",
            "<!DOCTYPE html>\n<!-- CANONICAL: 首次设置 Step 1。见 UNIFIED-SPEC.md §2.5 变体 D -->\n<html",
        ),
    ],
    "_2/code.html": [
        (
            "<!DOCTYPE html>\n\n<html",
            "<!DOCTYPE html>\n<!-- CANONICAL: 首次设置 Step 2（恢复密钥）。见 UNIFIED-SPEC.md §2.5 变体 D -->\n<html",
        ),
    ],
    "keyvault_4/code.html": [
        (
            "<!DOCTYPE html>\n\n<html",
            "<!DOCTYPE html>\n<!-- CANONICAL: 密码生成器模态。见 UNIFIED-SPEC.md §2.5 变体 F -->\n<html",
        ),
    ],
}

for rel, pairs in REPLACEMENTS.items():
    path = ROOT / rel
    if not path.exists():
        print(f"SKIP missing: {rel}")
        continue
    text = path.read_text(encoding="utf-8")
    orig = text
    for old, new in pairs:
        text = text.replace(old, new)
    if text != orig:
        path.write_text(text, encoding="utf-8")
        print(f"UPDATED: {rel}")
    else:
        print(f"NO CHANGE: {rel}")

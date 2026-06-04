# -*- coding: utf-8 -*-
"""Ingest _sources/stitch-2026-06-04 → canonical screens/ directories.

Run: python docs/软件界面原型/_shared/ingest-stitch-prototypes.py
"""
from __future__ import annotations

import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STITCH = ROOT / "_sources" / "stitch-2026-06-04"

# stitch relative path → (canonical dir under ROOT, CANONICAL comment)
INGEST_MAP: dict[str, tuple[str, str]] = {
    "1/code.html": ("screens/account/setup-step1-password", "首次设置 Step 1/2。见 UNIFIED-SPEC.md §2.5 变体 D"),
    "2/code.html": ("screens/account/setup-step2-confirm", "首次设置 Step 2/2 确认主密码。见 UNIFIED-SPEC.md §2.5 变体 D"),
    "keyvault_4/code.html": ("screens/modals/delete-confirm", "删除条目确认模态。见 UNIFIED-SPEC.md §2.5 变体 F"),
    "ssh_keyvault/code.html": (
        "screens/vault/entry-detail-ssh",
        "通用条目详情（SSH 示例）。见 UNIFIED-SPEC.md §2.5 变体 A",
    ),
    "keyvault_6/code.html": ("screens/account/unlock-lockout", "解锁页暴力破解锁定态。见 UNIFIED-SPEC.md §2.5 变体 D"),
    "keyvault_2/code.html": (
        "screens/modals/new-entry",
        "新建网站密码模态。见 UNIFIED-SPEC.md §2.5 变体 F / PROTOTYPE-PROMPTS P1-2",
    ),
    "keyvault_1/code.html": ("screens/modals/change-password", "修改主密码模态。见 PROTOTYPE-PROMPTS P1-4"),
    "keyvault_3/code.html": ("screens/vault/trash", "主界面三栏 · 回收站。见 PROTOTYPE-PROMPTS P2-1"),
    "keyvault_5/code.html": ("screens/vault/empty-states", "空状态三场景。见 PROTOTYPE-PROMPTS P2-2"),
    "keyvault_7/code.html": (
        "screens/components/clipboard-timer",
        "剪贴板自动清空倒计时条。见 PROTOTYPE-PROMPTS P2-3",
    ),
    "keyvault_8/code.html": ("screens/modals/emergency-wipe", "紧急擦除二次确认（Future）。见 PROTOTYPE-PROMPTS Future"),
}

GLOBAL_REPLACEMENTS: list[tuple[str, str]] = [
    ("KEYVAULT PRO", "KeyVault"),
    ("KeyVault Pro", "KeyVault"),
    ("New Entry", "新建条目"),
    ("All Items", "全部条目"),
    ("Favorites", "收藏"),
    ("Passwords", "密码"),
    ("API Keys", "API 密钥"),
    ("Secure Notes", "安全笔记"),
    ("Local Vault", "本地库"),
    ("Add Item", "新建条目"),
    ("Vaults", "密码库"),
    ("Generator", "生成器"),
    ("Security Audit", "安全审计"),
    ("Last synced: Just now", "上次同步：刚刚"),
    ("所有项目", "全部条目"),
    ("项目详情", "条目详情"),
    ("添加新项", "新建条目"),
    ("全部项目", "全部条目"),
    ("收藏夹", "收藏"),
    ("安全凭证", "API 密钥"),
    ("本地保管库", "本地库"),
    ("搜索全部项目...", "搜索条目 (Cmd+K)"),
    ("Username / Email", "用户名 / 邮箱"),
    ("Authenticator (TOTP)", "身份验证器 (TOTP)"),
    ("Hidden in Trash", "回收站中隐藏"),
    ("Website", "网站"),
    ("Username", "用户名"),
    ("Notes", "笔记"),
    ("Strong", "强"),
    ("Yesterday", "昨天"),
    ("Today", "今天"),
    ('placeholder="Search (Cmd+K)"', 'placeholder="搜索条目 (Cmd+K)"'),
    ('placeholder="Search entries (Cmd+K)"', 'placeholder="搜索条目 (Cmd+K)"'),
    ('placeholder="搜索..."', 'placeholder="搜索条目 (Cmd+K)"'),
    ("Last updated 2 days ago", "最后修改：2 天前"),
    # ui-* aliases (keyvault_3 trash)
    ("bg-ui-bg", "bg-background"),
    ("bg-ui-surface", "bg-surface-container-low"),
    ("border-ui-border", "border-outline-variant"),
    ("text-ui-primary", "text-primary-container"),
    ("bg-ui-primary", "bg-primary-container"),
    ("text-ui-bg", "text-on-primary-container"),
    ("border-ui-primary", "border-primary-container"),
    ("ring-ui-primary", "ring-primary-container"),
    ("text-ui-error", "text-error"),
    ("bg-ui-error", "bg-error"),
    ("group-focus-within:text-ui-primary", "group-focus-within:text-primary-container"),
    # Tailwind token alignment
    ('"background": "#0b141c"', '"background": "#0D1117"'),
    ('"surface": "#0b141c"', '"surface": "#161B22"'),
    ('"surface-dim": "#0b141c"', '"surface-dim": "#0D1117"'),
    ('"surface-container-low": "#141c24"', '"surface-container-low": "#161B22"'),
    ('"surface-container": "#182028"', '"surface-container": "#1C2128"'),
    ('"outline-variant": "#414752"', '"outline-variant": "#30363D"'),
    # Nav selected: canonical border-l-2
    ("border-r-2 border-primary", "border-l-2 border-primary-container"),
    ("border-l-[3px] border-ui-primary", "border-l-2 border-primary-container"),
    ("border-l-[3px] border-primary-container", "border-l-2 border-primary-container"),
    # Main app TopBar icon on vault screens (not setup/unlock/modals on vpn_key contexts)
]

# Word-specific nav/footer (avoid breaking unrelated "Lock" in "Lockout")
NAV_REPLACEMENTS: list[tuple[str, str]] = [
    ("                    Trash", "                    回收站"),
    ("                    Settings", "                    设置"),
    ("                    Lock", "                    锁定"),
    ("<span>Trash</span>", "<span>回收站</span>"),
    ("<span>Settings</span>", "<span>设置</span>"),
    ("<span>Lock</span>", "<span>锁定</span>"),
    ('<span class="flex-1 text-left font-body-md text-body-md">Settings</span>', '<span class="flex-1 text-left font-body-md text-body-md">设置</span>'),
]

FILE_REPLACEMENTS: dict[str, list[tuple[str, str]]] = {
    "1/code.html": [
        ("KeyVault - Setup Step 1", "KeyVault - 设置主密码"),
        ("STEP 1 OF 2", "第 1 步 / 共 2 步"),
    ],
    "2/code.html": [
        ("STEP 2 OF 2", "第 2 步 / 共 2 步"),
    ],
    "keyvault_4/code.html": [
        ('<span class="material-symbols-outlined text-primary text-[20px]">vpn_key</span>', '<span class="material-symbols-outlined text-primary text-[20px]" style="font-variation-settings: \'FILL\' 1;">shield_lock</span>'),
    ],
    "ssh_keyvault/code.html": [
        ("USERNAME", "用户名"),
        ("PASSPHRASE", "口令"),
        ("PUBLIC KEY", "公钥"),
        ("PRIVATE KEY", "私钥"),
        (
            '<span class="material-symbols-outlined text-[18px]">shield_lock</span>\n                    全部项目',
            '<span class="material-symbols-outlined text-[18px]">inventory_2</span>\n                    全部条目',
        ),
        (
            '<div class="flex items-center gap-sm px-sm py-2 bg-vault-overlay border-l-[2px] border-vault-accent text-on-surface font-bold rounded-r-DEFAULT cursor-pointer active:scale-95 duration-150">',
            '<div class="flex items-center gap-sm px-sm py-2 bg-surface-container border-l-2 border-primary-container text-primary font-bold rounded-r-DEFAULT cursor-pointer">',
        ),
        (
            '<div class="flex items-center gap-sm px-sm py-2 text-on-surface-variant hover:bg-vault-overlay hover:text-on-surface transition-colors cursor-pointer active:scale-95 duration-150 border-l-[2px] border-transparent">\n<span class="material-symbols-outlined text-[18px]">settings</span>\n                    设置\n                </div>',
            "",
        ),
        (
            '<div class="flex items-center gap-sm px-sm py-2 text-on-surface-variant hover:bg-vault-overlay hover:text-on-surface transition-colors cursor-pointer active:scale-95 duration-150 border-l-[2px] border-transparent">\n<span class="material-symbols-outlined text-[18px]">sync</span>\n                    同步\n                </div>',
            '<div class="flex items-center gap-sm px-sm py-2 text-on-surface-variant hover:bg-vault-overlay hover:text-on-surface transition-colors cursor-pointer border-l-2 border-transparent">\n<span class="material-symbols-outlined text-[18px]">settings</span>\n                    设置\n                </div>',
        ),
    ],
    "keyvault_7/code.html": [
        (
            '<span class="material-symbols-outlined text-primary dark:text-primary">vpn_key</span>',
            '<span class="material-symbols-outlined text-primary-container" style="font-variation-settings: \'FILL\' 1;">shield_lock</span>',
        ),
        (
            "Backup codes saved in encrypted document on local drive.",
            "备份码已保存在本地加密文档中。",
        ),
        ("SOCIAL", "社交"),
    ],
    "keyvault_3/code.html": [
        ("KeyVault Pro - Trash", "KeyVault - 回收站"),
        ("10:42 AM", "10:42"),
    ],
}


def ensure_canonical_comment(text: str, comment: str) -> str:
    marker = f"<!-- CANONICAL: {comment} -->"
    if marker in text:
        return text
    if text.startswith("<!DOCTYPE html>\n\n<html"):
        return text.replace("<!DOCTYPE html>\n\n<html", f"<!DOCTYPE html>\n{marker}\n<html", 1)
    if text.startswith("<!DOCTYPE html>\n<html"):
        return text.replace("<!DOCTYPE html>\n<html", f"<!DOCTYPE html>\n{marker}\n<html", 1)
    return text


def normalize_password_label_in_fields(text: str) -> str:
    """Replace label text Password → 密码 without touching type=password."""
    text = re.sub(
        r'(<label[^>]*>)\s*Password\s*(</label>)',
        r"\1密码\2",
        text,
        flags=re.IGNORECASE,
    )
    text = re.sub(
        r'(<label[^>]*uppercase[^>]*>)\s*Password\s*(</label>)',
        r"\1密码\2",
        text,
        flags=re.IGNORECASE,
    )
    return text


def process_html(rel: str, canonical_comment: str) -> str:
    path = STITCH / rel
    text = path.read_text(encoding="utf-8")
    for old, new in GLOBAL_REPLACEMENTS:
        text = text.replace(old, new)
    for old, new in NAV_REPLACEMENTS:
        text = text.replace(old, new)
    for old, new in FILE_REPLACEMENTS.get(rel, []):
        text = text.replace(old, new)
    text = normalize_password_label_in_fields(text)
    if 'lang="en"' in text:
        text = text.replace('lang="en"', 'lang="zh-CN"')
    text = ensure_canonical_comment(text, canonical_comment)
    return text


def main() -> None:
    if not STITCH.is_dir():
        raise SystemExit(f"Missing stitch folder: {STITCH}")

    for rel, (canonical_dir, comment) in INGEST_MAP.items():
        src_html = STITCH / rel
        if not src_html.exists():
            print(f"SKIP missing: {rel}")
            continue

        dest_dir = ROOT / canonical_dir
        dest_dir.mkdir(parents=True, exist_ok=True)

        processed = process_html(rel, comment)
        dest_html = dest_dir / "code.html"
        dest_html.write_text(processed, encoding="utf-8")
        print(f"INGEST HTML: {rel} → {canonical_dir}/code.html")

        # Also fix in-place in stitch folder
        src_html.write_text(processed, encoding="utf-8")
        print(f"UPDATED stitch: {rel}")

        # Copy screen.png if present
        src_png = src_html.parent / "screen.png"
        if src_png.exists():
            shutil.copy2(src_png, dest_dir / "screen.png")
            print(f"INGEST PNG:  {rel.replace('/code.html', '')}/screen.png → {canonical_dir}/")

    print("Done. See PROTOTYPE-INDEX.md for canonical paths.")


if __name__ == "__main__":
    main()

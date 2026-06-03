/**
 * 填充器
 * 在 password input 旁注入 KV 图标，处理点击填充
 */
class FormFiller {
  constructor() {
    this.injectedIcons = new Map();
    this.dropdown = null;
    this.init();
  }

  init() {
    window.addEventListener('message', (e) => {
      if (e.data?.type === 'KV_INJECT_ICONS') {
        this.handleInject(e.data.groups, e.data.entries);
      }
    });

    document.addEventListener('click', (e) => {
      if (this.dropdown && !this.dropdown.contains(e.target)) {
        this.closeDropdown();
      }
    });

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') this.closeDropdown();
    });
  }

  handleInject(groups, entries) {
    for (const group of groups) {
      const pwdField = group.passwordFieldSelector
        ? document.querySelector(group.passwordFieldSelector)
        : document.getElementById(group.passwordFieldId);

      if (!pwdField || this.injectedIcons.has(pwdField)) continue;
      this.injectIcon(pwdField, entries);
    }
  }

  injectIcon(pwdField, entries) {
    const container = pwdField.parentElement;
    if (!container) return;

    const containerPos = getComputedStyle(container).position;
    if (containerPos === 'static') {
      container.style.position = 'relative';
    }

    const icon = document.createElement('div');
    icon.className = 'kv-fill-icon';
    icon.textContent = 'KV';
    icon.title = `KeyVault: 找到 ${entries.length} 个匹配凭据`;

    Object.assign(icon.style, {
      position: 'absolute',
      right: '8px',
      top: '50%',
      transform: 'translateY(-50%)',
      width: '20px',
      height: '20px',
      background: 'rgba(56, 139, 253, 0.9)',
      borderRadius: '4px',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      fontSize: '10px',
      fontWeight: '600',
      color: '#fff',
      cursor: 'pointer',
      zIndex: '2147483647',
      fontFamily: '-apple-system, BlinkMacSystemFont, sans-serif',
      transition: 'all 0.15s ease',
    });

    icon.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();

      if (entries.length === 1) {
        this.fillCredential(pwdField, entries[0].id, entries[0].username);
      } else {
        this.showDropdown(icon, pwdField, entries);
      }
    });

    container.appendChild(icon);
    this.injectedIcons.set(pwdField, icon);

    const currentPadding = parseInt(getComputedStyle(pwdField).paddingRight) || 0;
    pwdField.style.paddingRight = `${currentPadding + 28}px`;
  }

  showDropdown(anchor, pwdField, entries) {
    this.closeDropdown();

    const dropdown = document.createElement('div');
    dropdown.className = 'kv-dropdown';

    Object.assign(dropdown.style, {
      position: 'absolute',
      top: '100%',
      right: '0',
      width: '200px',
      maxHeight: '200px',
      overflowY: 'auto',
      background: '#1c2128',
      border: '1px solid rgba(255,255,255,0.1)',
      borderRadius: '6px',
      boxShadow: '0 8px 24px rgba(0,0,0,0.5)',
      zIndex: '2147483647',
      marginTop: '4px',
    });

    for (const entry of entries) {
      const item = document.createElement('div');
      Object.assign(item.style, {
        padding: '8px 12px',
        cursor: 'pointer',
        borderBottom: '1px solid rgba(255,255,255,0.05)',
        transition: 'background 0.1s',
      });

      item.innerHTML = `
        <div style="font-size:12px;font-weight:500;color:#e6edf3;">${this.escapeHtml(entry.title)}</div>
        <div style="font-size:11px;color:#8b949e;margin-top:2px;">${this.escapeHtml(entry.username || '')}</div>
      `;

      item.addEventListener('mouseenter', () => {
        item.style.background = '#22272e';
      });
      item.addEventListener('mouseleave', () => {
        item.style.background = 'transparent';
      });
      item.addEventListener('click', () => {
        this.fillCredential(pwdField, entry.id, entry.username);
        this.closeDropdown();
      });

      dropdown.appendChild(item);
    }

    anchor.style.position = 'relative';
    anchor.appendChild(dropdown);
    this.dropdown = dropdown;
  }

  closeDropdown() {
    if (this.dropdown) {
      this.dropdown.remove();
      this.dropdown = null;
    }
  }

  async fillCredential(pwdField, entryId, username) {
    try {
      const result = await chrome.runtime.sendMessage({
        action: 'FILL_CREDENTIAL',
        entryId,
      });

      if (result?.error) {
        console.error('KeyVault fill error:', result.error);
        return;
      }

      this.setInputValue(pwdField, result.password);

      const container = pwdField.closest('form') || pwdField.parentElement?.parentElement;
      if (container && result.username) {
        const usernameField =
          container.querySelector('input[type="email"]') ||
          container.querySelector('input[autocomplete="username"]') ||
          container.querySelector('input[name*="user" i],input[name*="email" i]') ||
          container.querySelector('input[type="text"]');

        if (usernameField) {
          this.setInputValue(usernameField, result.username);
        }
      }

      this.showSuccess(pwdField);
    } catch (err) {
      console.error('KeyVault fill error:', err);
    }
  }

  setInputValue(input, value) {
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
      window.HTMLInputElement.prototype, 'value'
    )?.set;

    if (nativeInputValueSetter) {
      nativeInputValueSetter.call(input, value);
    } else {
      input.value = value;
    }

    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.dispatchEvent(new Event('change', { bubbles: true }));
    input.dispatchEvent(new Event('blur', { bubbles: true }));
  }

  showSuccess(pwdField) {
    const icon = this.injectedIcons.get(pwdField);
    if (!icon) return;

    icon.textContent = '✓';
    icon.style.background = 'rgba(63, 185, 80, 0.9)';

    setTimeout(() => {
      icon.textContent = 'KV';
      icon.style.background = 'rgba(56, 139, 253, 0.9)';
    }, 3000);
  }

  escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }
}

new FormFiller();

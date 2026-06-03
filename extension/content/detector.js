/**
 * 表单检测器
 * 检测页面中的登录表单，通知 Service Worker 查询匹配凭据
 */
class FormDetector {
  constructor() {
    this.detectedGroups = new Set();
    this.observer = null;
  }

  getFormGroups() {
    const groups = [];

    document.querySelectorAll('input[type="password"]').forEach(pwdInput => {
      if (pwdInput.dataset.kvHandled) return;
      const key = this.getElementKey(pwdInput);
      if (this.detectedGroups.has(key)) return;
      this.detectedGroups.add(key);
      const usernameField = this.findUsernameField(pwdInput);
      groups.push({ passwordField: pwdInput, usernameField });
    });

    document.querySelectorAll(
      '[autocomplete="current-password"],[autocomplete="new-password"]'
    ).forEach(input => {
      if (input.type === 'password') return;
      const key = this.getElementKey(input);
      if (this.detectedGroups.has(key)) return;
      this.detectedGroups.add(key);
      groups.push({
        passwordField: input,
        usernameField: this.findUsernameField(input),
      });
    });

    return groups;
  }

  findUsernameField(referenceInput) {
    const container =
      referenceInput.closest('form') ||
      referenceInput.closest('[class*="login"],[class*="signin"],[class*="auth"]') ||
      referenceInput.parentElement?.parentElement;

    if (!container) return null;

    return (
      container.querySelector('input[type="email"]') ||
      container.querySelector('input[autocomplete="username"],input[autocomplete="email"]') ||
      container.querySelector('input[name*="user" i],input[name*="email" i],input[name*="login" i]') ||
      container.querySelector('input[type="text"]')
    );
  }

  getElementKey(el) {
    return el.id || el.name || `${el.offsetTop}-${el.offsetLeft}`;
  }

  watchDynamic() {
    this.observer = new MutationObserver(() => {
      const newGroups = this.getFormGroups();
      if (newGroups.length > 0) {
        this.notifyAndInject(newGroups);
      }
    });
    this.observer.observe(document.body, { childList: true, subtree: true });
  }

  async notifyAndInject(groups) {
    try {
      const result = await chrome.runtime.sendMessage({
        action: 'FIND_CREDENTIALS',
        url: location.href,
      });

      if (result?.error) return;
      if (!result?.entries?.length) return;

      window.postMessage({
        type: 'KV_INJECT_ICONS',
        groups: groups.map(g => ({
          passwordFieldId: g.passwordField.id || g.passwordField.name || '',
          passwordFieldSelector: this.buildSelector(g.passwordField),
        })),
        entries: result.entries,
      }, '*');
    } catch {
      // Service Worker 未就绪或连接断开，静默忽略
    }
  }

  buildSelector(el) {
    if (el.id) return `#${el.id}`;
    if (el.name) return `input[name="${el.name}"]`;
    return null;
  }
}

const detector = new FormDetector();

const initialGroups = detector.getFormGroups();
if (initialGroups.length > 0) {
  detector.notifyAndInject(initialGroups);
}

detector.watchDynamic();

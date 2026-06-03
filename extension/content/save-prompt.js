// extension/content/save-prompt.js
// Detects form submissions with credentials and offers to save them.

(function () {
  'use strict';

  let pendingCredentials = null;

  // Listen for form submissions
  document.addEventListener('submit', (e) => {
    const form = e.target;
    if (!(form instanceof HTMLFormElement)) return;

    const passwordInput = form.querySelector('input[type="password"]');
    if (!passwordInput || !passwordInput.value) return;

    // Find username
    const usernameInput =
      form.querySelector('input[type="email"]') ||
      form.querySelector('input[autocomplete="username"]') ||
      form.querySelector('input[name*="user" i]') ||
      form.querySelector('input[name*="email" i]') ||
      form.querySelector('input[type="text"]');

    const username = usernameInput?.value || '';
    const password = passwordInput.value;

    if (!password) return;

    pendingCredentials = { username, password, url: location.href };

    // Ask service worker if we should save
    chrome.runtime.sendMessage(
      { action: 'CHECK_SAVE', url: location.href, username },
      (response) => {
        if (response?.shouldSave) {
          showSavePrompt(pendingCredentials);
        }
      }
    );
  });

  function showSavePrompt(creds) {
    // Remove existing prompt if any
    const existing = document.getElementById('kv-save-prompt');
    if (existing) existing.remove();

    const banner = document.createElement('div');
    banner.id = 'kv-save-prompt';
    banner.style.cssText = `
      position: fixed;
      top: 0;
      left: 50%;
      transform: translateX(-50%);
      z-index: 2147483647;
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 10px 16px;
      background: #1c2128;
      border: 1px solid rgba(255,255,255,0.1);
      border-radius: 0 0 8px 8px;
      box-shadow: 0 4px 12px rgba(0,0,0,0.4);
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      font-size: 13px;
      color: #e6edf3;
      animation: kv-slide-down 200ms ease-out;
    `;

    banner.innerHTML = `
      <span style="font-weight:500;">🔐 KeyVault</span>
      <span style="color:#8b949e;">是否保存此密码？</span>
      <button id="kv-save-yes" style="
        padding: 4px 12px; border: none; border-radius: 4px;
        background: #388bfd; color: white; cursor: pointer; font-size: 12px;
      ">保存</button>
      <button id="kv-save-no" style="
        padding: 4px 12px; border: none; border-radius: 4px;
        background: transparent; color: #8b949e; cursor: pointer; font-size: 12px;
      ">不用</button>
    `;

    // Add animation keyframes
    if (!document.getElementById('kv-save-style')) {
      const style = document.createElement('style');
      style.id = 'kv-save-style';
      style.textContent = `@keyframes kv-slide-down { from { transform: translateX(-50%) translateY(-100%); opacity: 0; } to { transform: translateX(-50%) translateY(0); opacity: 1; } }`;
      document.head.appendChild(style);
    }

    document.body.appendChild(banner);

    document.getElementById('kv-save-yes').addEventListener('click', () => {
      chrome.runtime.sendMessage({
        action: 'SAVE_CREDENTIAL',
        url: creds.url,
        username: creds.username,
        password: creds.password,
        title: document.title || location.hostname,
      });
      banner.remove();
    });

    document.getElementById('kv-save-no').addEventListener('click', () => {
      banner.remove();
    });

    // Auto-dismiss after 10 seconds
    setTimeout(() => {
      if (banner.parentNode) banner.remove();
    }, 10000);
  }
})();

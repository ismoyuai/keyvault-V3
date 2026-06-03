/**
 * Popup 逻辑
 * 状态机：disconnected → locked → connected
 */
const $ = (id) => document.getElementById(id);

let currentTab = null;

async function init() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  currentTab = tab;

  const status = await sendToBackground({ action: 'GET_STATUS' });

  if (status?.status === 'connected') {
    showView('connected');
    loadCurrentPageEntries();
  } else if (status?.status === 'locked') {
    showView('locked');
  } else {
    showView('disconnected');
  }

  $('btn-open-app')?.addEventListener('click', () => chrome.runtime.sendNativeMessage('com.keyvault.app', { action: 'open_app' }));
  $('btn-unlock')?.addEventListener('click', () => chrome.runtime.sendNativeMessage('com.keyvault.app', { action: 'open_app' }));
  $('btn-open-main')?.addEventListener('click', () => chrome.runtime.sendNativeMessage('com.keyvault.app', { action: 'open_app' }));
  $('btn-generate')?.addEventListener('click', () => chrome.runtime.sendNativeMessage('com.keyvault.app', { action: 'open_app' }));

  $('search-input')?.addEventListener('input', debounce(handleSearch, 300));
}

function showView(state) {
  $('view-disconnected').style.display = state === 'disconnected' ? 'block' : 'none';
  $('view-locked').style.display = state === 'locked' ? 'block' : 'none';
  $('view-connected').style.display = state === 'connected' ? 'block' : 'none';

  const statusEl = $('status');
  statusEl.className = 'status';
  if (state === 'connected') {
    statusEl.textContent = '● 已连接';
    statusEl.classList.add('status--connected');
  } else if (state === 'locked') {
    statusEl.textContent = '● 已锁定';
    statusEl.classList.add('status--locked');
  } else {
    statusEl.textContent = '● 未连接';
    statusEl.classList.add('status--disconnected');
  }
}

async function loadCurrentPageEntries() {
  if (!currentTab?.url) return;

  const result = await sendToBackground({
    action: 'FIND_CREDENTIALS',
    url: currentTab.url,
  });

  const container = $('current-entries');
  const noMatches = $('no-matches');

  if (result?.error || !result?.entries?.length) {
    container.innerHTML = '';
    noMatches.style.display = 'block';
    return;
  }

  noMatches.style.display = 'none';
  container.innerHTML = result.entries.map(entry => renderEntryCard(entry)).join('');
  bindEntryActions(container);
}

async function handleSearch() {
  const query = $('search-input').value.trim();
  const section = $('section-search');
  const currentSection = $('section-current');

  if (!query) {
    section.style.display = 'none';
    currentSection.style.display = 'block';
    return;
  }

  section.style.display = 'block';
  currentSection.style.display = 'none';

  const result = await sendToBackground({ action: 'SEARCH', query });
  const container = $('search-results');
  const noResults = $('no-results');

  if (result?.error || !result?.entries?.length) {
    container.innerHTML = '';
    noResults.style.display = 'block';
    return;
  }

  noResults.style.display = 'none';
  container.innerHTML = result.entries.map(entry => renderEntryCard(entry)).join('');
  bindEntryActions(container);
}

function renderEntryCard(entry) {
  return `
    <div class="entry-card" data-entry-id="${entry.id}" data-username="${escapeHtml(entry.username || '')}">
      <div class="entry-title">${escapeHtml(entry.title)}</div>
      <div class="entry-username">${escapeHtml(entry.username || '')}</div>
      <div class="entry-actions">
        <button class="btn btn--small btn--fill" data-action="fill">填充</button>
        <button class="btn btn--small btn--copy" data-action="copy">复制密码</button>
      </div>
    </div>
  `;
}

function bindEntryActions(container) {
  container.querySelectorAll('.entry-card').forEach(card => {
    const entryId = card.dataset.entryId;

    card.querySelector('[data-action="fill"]')?.addEventListener('click', async () => {
      if (currentTab?.id) {
        chrome.tabs.sendMessage(currentTab.id, {
          type: 'KV_FILL_DIRECT',
          entryId,
        });
        window.close();
      }
    });

    card.querySelector('[data-action="copy"]')?.addEventListener('click', async () => {
      const result = await sendToBackground({ action: 'FILL_CREDENTIAL', entryId });
      if (result?.password) {
        await navigator.clipboard.writeText(result.password);
        const btn = card.querySelector('[data-action="copy"]');
        btn.textContent = '已复制';
        setTimeout(() => { btn.textContent = '复制密码'; }, 2000);
      }
    });
  });
}

function sendToBackground(message) {
  return chrome.runtime.sendMessage(message);
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}

init();

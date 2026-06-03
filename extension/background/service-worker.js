// Native Messaging 连接管理
let nativePort = null;
let pendingRequests = new Map(); // requestId -> { resolve, reject, timeout }
let requestCounter = 0;

function getOrCreateNativePort() {
  if (nativePort) return nativePort;

  try {
    nativePort = chrome.runtime.connectNative('com.keyvault.app');

    nativePort.onMessage.addListener((message) => {
      const { requestId, ...data } = message;
      const pending = pendingRequests.get(requestId);
      if (pending) {
        clearTimeout(pending.timeout);
        pendingRequests.delete(requestId);
        if (data.error) {
          pending.reject(new Error(data.error));
        } else {
          pending.resolve(data);
        }
      }
    });

    nativePort.onDisconnect.addListener(() => {
      nativePort = null;
      const error = chrome.runtime.lastError?.message || 'KeyVault 连接断开';
      for (const [id, pending] of pendingRequests) {
        clearTimeout(pending.timeout);
        pending.reject(new Error(error));
      }
      pendingRequests.clear();
      chrome.storage.session.set({ connectionStatus: 'disconnected' });
    });

    chrome.storage.session.set({ connectionStatus: 'connected' });
    return nativePort;
  } catch {
    chrome.storage.session.set({ connectionStatus: 'disconnected' });
    return null;
  }
}

async function sendToNative(action, payload = {}) {
  const port = getOrCreateNativePort();
  if (!port) throw new Error('无法连接 KeyVault，请确保应用已启动');

  return new Promise((resolve, reject) => {
    const requestId = ++requestCounter;
    const timeout = setTimeout(() => {
      pendingRequests.delete(requestId);
      reject(new Error('请求超时'));
    }, 5000);

    pendingRequests.set(requestId, { resolve, reject, timeout });
    port.postMessage({ requestId, action, ...payload });
  });
}

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  handleMessage(message, sender)
    .then(sendResponse)
    .catch(err => sendResponse({ error: err.message }));
  return true;
});

async function handleMessage(message, sender) {
  switch (message.action) {
    case 'GET_STATUS':
      try {
        return await sendToNative('get_status');
      } catch {
        return { status: 'disconnected' };
      }

    case 'FIND_CREDENTIALS':
      return await sendToNative('find_credentials', { url: message.url });

    case 'FILL_CREDENTIAL':
      return await sendToNative('get_entry_for_fill', { entryId: message.entryId });

    case 'SEARCH':
      return await sendToNative('search', { query: message.query });

    case 'CHECK_SAVE':
      // Always offer to save for now (could check if already saved)
      return { shouldSave: true };

    case 'SAVE_CREDENTIAL':
      return await sendToNative('save_credential', {
        url: message.url,
        username: message.username,
        password: message.password,
        title: message.title,
      });

    default:
      throw new Error(`未知操作: ${message.action}`);
  }
}

// Reports the active tab's domain to the FocusTime native messaging host
// whenever the active tab changes or its URL is updated. Minimal by design:
// no popup/options UI, no retry/reconnect logic beyond letting
// chrome.runtime.connectNative reconnect naturally on the next event.

const NATIVE_HOST = "com.focustime.native_host";

let port = null;

function getPort() {
  if (!port) {
    port = chrome.runtime.connectNative(NATIVE_HOST);
    port.onDisconnect.addListener(() => {
      port = null;
    });
  }
  return port;
}

function reportDomain(url) {
  if (!url) return;
  let hostname;
  try {
    hostname = new URL(url).hostname;
  } catch {
    return; // Not a real URL (e.g. chrome://, about:blank) — skip.
  }
  if (!hostname) return;

  try {
    getPort().postMessage({ domain: hostname, timestamp: Date.now() });
  } catch {
    port = null; // Host process not running yet; will retry on next event.
  }
}

chrome.tabs.onActivated.addListener(({ tabId }) => {
  chrome.tabs.get(tabId, (tab) => reportDomain(tab?.url));
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.url && tab.active) {
    reportDomain(changeInfo.url);
  }
});

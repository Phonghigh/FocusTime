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
      if (chrome.runtime.lastError) {
        // Surfaces here on connect failure: bad native-host manifest path,
        // wrong allowed_origins/extension id, or the host process crashing.
        console.error("FocusTime native host disconnected:", chrome.runtime.lastError.message);
      }
      port = null;
    });
  }
  return port;
}

function reportDomain(url) {
  if (!url) return;
  let parsed;
  try {
    parsed = new URL(url);
  } catch {
    return; // Not a parseable URL — skip.
  }
  // Only track real web pages. chrome://, edge://, about:, file:// etc. have
  // a "hostname" too (e.g. chrome://extensions -> "extensions") but aren't
  // domains worth classifying, so exclude them explicitly.
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return;
  const hostname = parsed.hostname;
  if (!hostname) return;

  try {
    getPort().postMessage({ domain: hostname, timestamp: Date.now() });
  } catch (err) {
    console.error("FocusTime: failed to send domain to native host:", err);
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

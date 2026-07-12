const HOST = "com.whats_on_my_desk.agent";
let port;
let tabs = new Set();

function connect() {
  if (port) return;
  try {
    port = chrome.runtime.connectNative(HOST);
    port.onMessage.addListener(message => broadcast(message));
    port.onDisconnect.addListener(() => { port = undefined; broadcast({ type: "agent-status", payload: { connected: false } }); });
    broadcast({ type: "agent-status", payload: { connected: true } });
  } catch { broadcast({ type: "agent-status", payload: { connected: false } }); }
}
function broadcast(message) { for (const tabId of tabs) chrome.tabs.sendMessage(tabId, message).catch(() => tabs.delete(tabId)); }
chrome.runtime.onMessage.addListener((message, sender) => {
  if (sender.tab?.id) tabs.add(sender.tab.id);
  if (!["connect-agent", "get-device-snapshot", "refresh-device-snapshot", "get-diagnostics"].includes(message?.type)) return;
  connect();
  if (port) port.postMessage({ type: message.type === "connect-agent" ? "ping" : message.type });
});

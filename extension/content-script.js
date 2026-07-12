const PAGE = "whats-on-my-desk-page";
const EXT = "whats-on-my-desk-extension";
const allowed = new Set(["CONNECT_AGENT", "GET_DEVICE_SNAPSHOT", "REFRESH_DEVICE_SNAPSHOT", "GET_DIAGNOSTICS"]);
window.addEventListener("message", event => {
  if (event.source !== window || event.data?.source !== PAGE || !allowed.has(event.data?.type)) return;
  chrome.runtime.sendMessage({ type: event.data.type.toLowerCase().replaceAll("_", "-") });
});
chrome.runtime.onMessage.addListener(message => {
  const map = { ready: "AGENT_STATUS", "agent-status": "AGENT_STATUS", "device-snapshot": "DEVICE_SNAPSHOT", "device-snapshot-updated": "DEVICE_SNAPSHOT_UPDATED", "agent-error": "AGENT_ERROR" };
  const type = map[message?.type];
  if (type) window.postMessage({ source: EXT, type, payload: message.payload ?? message }, window.location.origin);
});
window.postMessage({ source: EXT, type: "BRIDGE_READY" }, window.location.origin);

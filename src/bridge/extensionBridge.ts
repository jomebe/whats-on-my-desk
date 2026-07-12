import type { DeviceSnapshot } from "../devices/types";

const PAGE = "whats-on-my-desk-page";
const EXT = "whats-on-my-desk-extension";
export type BridgeMessage = { type: "status"; connected: boolean } | { type: "snapshot"; snapshot: DeviceSnapshot } | { type: "error" };

export function connectExtension(onMessage: (message: BridgeMessage) => void) {
  const handler = (event: MessageEvent) => {
    if (event.source !== window || event.origin !== window.location.origin || event.data?.source !== EXT) return;
    if (event.data.type === "AGENT_STATUS") onMessage({ type: "status", connected: Boolean(event.data.payload?.connected ?? true) });
    if (event.data.type === "DEVICE_SNAPSHOT" || event.data.type === "DEVICE_SNAPSHOT_UPDATED") onMessage({ type: "snapshot", snapshot: event.data.payload as DeviceSnapshot });
    if (event.data.type === "AGENT_ERROR") onMessage({ type: "error" });
  };
  window.addEventListener("message", handler);
  window.postMessage({ source: PAGE, type: "CONNECT_AGENT" }, window.location.origin);
  window.postMessage({ source: PAGE, type: "GET_DEVICE_SNAPSHOT" }, window.location.origin);
  return () => window.removeEventListener("message", handler);
}

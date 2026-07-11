import { useCallback, useEffect, useRef, useState } from "react";
import type { DeviceSnapshot } from "../devices/types";

const empty: DeviceSnapshot = { generatedAt: 0, rawDeviceCount: 0, filteredDeviceCount: 0, mergedPhysicalDeviceCount: 0, devices: [] };
type AgentStatus = "connecting" | "online" | "offline";

export function useDeviceSnapshot() {
  const [snapshot, setSnapshot] = useState<DeviceSnapshot>(empty);
  const [status, setStatus] = useState<AgentStatus>("connecting");
  const retry = useRef(500);
  const refresh = useCallback(async () => {
    const response = await fetch("/api/refresh", { method: "POST" });
    if (!response.ok) throw new Error("agent unavailable");
    const next = await response.json() as DeviceSnapshot;
    setSnapshot(next);
  }, []);
  useEffect(() => {
    let socket: WebSocket | undefined;
    let timer: number | undefined;
    let cancelled = false;
    const connect = async () => {
      try {
        const health = await fetch("/api/health");
        if (!health.ok) throw new Error("agent unavailable");
        const response = await fetch("/api/device-snapshot");
        setSnapshot(await response.json() as DeviceSnapshot);
        setStatus("online");
        retry.current = 500;
        socket = new WebSocket(`${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/ws`);
        socket.onmessage = event => {
          const message = JSON.parse(event.data) as { type: string; payload: DeviceSnapshot };
          if (message.type === "device-snapshot-updated") setSnapshot(message.payload);
        };
        socket.onclose = () => schedule();
      } catch { schedule(); }
    };
    const schedule = () => {
      if (cancelled) return;
      setStatus("offline");
      timer = window.setTimeout(connect, retry.current);
      retry.current = Math.min(retry.current * 2, 10_000);
    };
    void connect();
    return () => { cancelled = true; socket?.close(); if (timer) clearTimeout(timer); };
  }, []);
  return { snapshot, status, refresh };
}

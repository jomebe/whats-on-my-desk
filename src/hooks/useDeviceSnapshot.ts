import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import type { DeviceSnapshot } from "../devices/types";

const empty: DeviceSnapshot = { generatedAt: 0, devices: [] };

export function useDeviceSnapshot(enabled: boolean) {
  const [snapshot, setSnapshot] = useState(empty);
  const [error, setError] = useState(false);
  const refresh = useCallback(async () => {
    if (!enabled) return false;
    try { setSnapshot(await invoke<DeviceSnapshot>("get_device_snapshot")); setError(false); }
    catch { setError(true); return false; }
    return true;
  }, [enabled]);
  useEffect(() => {
    if (!enabled) return;
    let unlisten: (() => void) | undefined;
    refresh().then(async (available) => {
      if (!available) return;
      unlisten = await listen<DeviceSnapshot>("device-snapshot-updated", event => setSnapshot(event.payload));
    });
    return () => unlisten?.();
  }, [enabled, refresh]);
  return { snapshot, refresh: () => { void refresh(); }, error };
}

import { useEffect, useState } from "react";
import type { Settings } from "../devices/types";

const defaults: Settings = { animations: true, showNames: false, showBuiltIn: false, showUnknown: false, showUsbGeneric: false, showPrinters: false, showVirtual: false, theme: "dark", mockMode: false };

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(() => {
    try { return { ...defaults, ...JSON.parse(localStorage.getItem("womd-settings") ?? "{}") }; } catch { return defaults; }
  });
  useEffect(() => {
    let cancelled = false;
    void fetch("/api/settings").then(response => response.ok ? response.json() as Promise<Partial<Settings>> : Promise.reject()).then(saved => {
      if (!cancelled) setSettings(current => ({ ...current, ...saved }));
    }).catch(() => undefined);
    return () => { cancelled = true; };
  }, []);
  useEffect(() => {
    localStorage.setItem("womd-settings", JSON.stringify(settings));
    const { mockMode: _mockMode, ...agentSettings } = settings;
    void fetch("/api/settings", { method: "PUT", headers: { "content-type": "application/json" }, body: JSON.stringify(agentSettings) }).catch(() => undefined);
  }, [settings]);
  return { settings, update: (patch: Partial<Settings>) => setSettings(current => ({ ...current, ...patch })) };
}

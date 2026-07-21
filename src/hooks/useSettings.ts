import { useEffect, useState } from "react";
import type { Settings } from "../devices/types";

const SETTINGS_SCHEMA_VERSION = 2;
const defaults: Settings = { schemaVersion: SETTINGS_SCHEMA_VERSION, animations: true, showNames: false, showBuiltIn: false, showUnknown: false, showUsbGeneric: true, showPrinters: true, showVirtual: false, theme: "dark", mockMode: false };

function savedSettings(value: unknown): Settings {
  if (!value || typeof value !== "object" || (value as Partial<Settings>).schemaVersion !== SETTINGS_SCHEMA_VERSION) return defaults;
  return { ...defaults, ...(value as Partial<Settings>) };
}

export function useSettings() {
  const [serverReady, setServerReady] = useState(false);
  const [settings, setSettings] = useState<Settings>(() => {
    try { return savedSettings(JSON.parse(localStorage.getItem("womd-settings") ?? "{}")); } catch { return defaults; }
  });
  useEffect(() => {
    let cancelled = false;
    void fetch("/api/settings").then(response => response.ok ? response.json() as Promise<Partial<Settings>> : Promise.reject()).then(saved => {
      if (!cancelled && saved.schemaVersion === SETTINGS_SCHEMA_VERSION) setSettings(savedSettings(saved));
    }).catch(() => undefined).finally(() => { if (!cancelled) setServerReady(true); });
    return () => { cancelled = true; };
  }, []);
  useEffect(() => {
    localStorage.setItem("womd-settings", JSON.stringify(settings));
    if (!serverReady) return;
    const { mockMode: _mockMode, ...agentSettings } = settings;
    void fetch("/api/settings", { method: "PUT", headers: { "content-type": "application/json" }, body: JSON.stringify(agentSettings) }).catch(() => undefined);
  }, [serverReady, settings]);
  return { settings, update: (patch: Partial<Settings>) => setSettings(current => ({ ...current, ...patch })) };
}

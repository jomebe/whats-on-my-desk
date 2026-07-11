import { useEffect, useState } from "react";
import type { Settings } from "../devices/types";

const defaults: Settings = { animations: true, showNames: false, showBuiltIn: true, showUnknown: false, showVirtual: false, theme: "system", mockMode: false };

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(() => {
    try { return { ...defaults, ...JSON.parse(localStorage.getItem("womd-settings") ?? "{}") }; } catch { return defaults; }
  });
  useEffect(() => localStorage.setItem("womd-settings", JSON.stringify(settings)), [settings]);
  return { settings, update: (patch: Partial<Settings>) => setSettings(current => ({ ...current, ...patch })) };
}

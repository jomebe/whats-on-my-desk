import type { Settings } from "../devices/types";

export function SettingsPanel({ settings, update, close, refresh }: { settings: Settings; update: (p: Partial<Settings>) => void; close: () => void; refresh: () => void }) {
  const toggle = (key: keyof Settings, label: string) => <label><span>{label}</span><input type="checkbox" checked={Boolean(settings[key])} onChange={e => update({ [key]: e.target.checked })}/></label>;
  return <aside className="settings"><header><strong>Preferences</strong><button onClick={close}>×</button></header>{toggle("animations", "Motion")}{toggle("showNames", "Device names")}{toggle("showBuiltIn", "Built-in devices")}{toggle("showUnknown", "Unknown devices")}{toggle("showUsbGeneric", "General USB")}{toggle("showPrinters", "Printers")}{toggle("showVirtual", "Virtual devices")}{toggle("mockMode", "Mock mode")}<label><span>Theme</span><select value={settings.theme} onChange={e => update({ theme: e.target.value as Settings["theme"] })}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></label><button className="refresh" onClick={refresh}>Refresh devices</button></aside>;
}

import { useMemo, useState } from "react";
import "./App.css";
import { DeviceScene } from "./components/DeviceScene";
import { SettingsPanel } from "./components/SettingsPanel";
import { MockControlPanel } from "./components/MockControlPanel";
import { useSettings } from "./hooks/useSettings";
import { useDeviceSnapshot } from "./hooks/useDeviceSnapshot";
import { mockDevices as initialMocks } from "./devices/mockDevices";

export default function App() {
  const { settings, update } = useSettings();
  const { snapshot, refresh, error } = useDeviceSnapshot(!settings.mockMode);
  const [mocks, setMocks] = useState(initialMocks);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const usesWebDemo = !("__TAURI_INTERNALS__" in window);
  const sourceDevices = settings.mockMode || usesWebDemo ? mocks : snapshot.devices;
  const devices = useMemo(() => sourceDevices.filter(d => (settings.showBuiltIn || d.isExternal !== false) && (settings.showUnknown || d.category !== "unknown") && (settings.showVirtual || !d.isVirtual)), [settings, sourceDevices]);
  return <div className={`app theme-${settings.theme} ${settings.animations ? "with-motion" : "no-motion"}`}>
    <header className="topbar"><span>What’s on My Desk?</span><button className="gear" onClick={() => setSettingsOpen(v => !v)} aria-label="Settings">⌁</button></header>
    <DeviceScene devices={devices} showNames={settings.showNames}/>
    {!devices.length && <div className="empty">{error ? "Device scan unavailable" : "Your desk is quiet"}</div>}
    {settingsOpen && <SettingsPanel settings={settings} update={update} close={() => setSettingsOpen(false)} refresh={refresh}/>} 
    {(settings.mockMode || usesWebDemo) && <MockControlPanel devices={mocks} setDevices={setMocks}/>} 
  </div>;
}

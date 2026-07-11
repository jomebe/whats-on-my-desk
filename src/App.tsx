import { useMemo, useState } from "react";
import "./App.css";
import { AgentOfflineScreen } from "./components/AgentOfflineScreen";
import { DeviceScene } from "./components/DeviceScene";
import { MockControlPanel } from "./components/MockControlPanel";
import { SettingsPanel } from "./components/SettingsPanel";
import { mockDevices as initialMocks } from "./devices/mockDevices";
import { useDeviceSnapshot } from "./hooks/useDeviceSnapshot";
import { useSettings } from "./hooks/useSettings";

export default function App() {
  const { settings, update } = useSettings();
  const { snapshot, status, refresh } = useDeviceSnapshot();
  const [mocks, setMocks] = useState(initialMocks);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const source = settings.mockMode ? mocks : snapshot.devices;
  const devices = useMemo(() => source.filter(device =>
    (settings.showBuiltIn || device.isExternal || device.category === "computer" || device.category === "display") &&
    (settings.showUnknown || device.category !== "unknown") &&
    (settings.showUsbGeneric || device.category !== "usbGeneric") &&
    (settings.showPrinters || device.category !== "printer") &&
    (settings.showVirtual || !device.isVirtual)
  ), [settings, source]);
  if (status === "offline" && !settings.mockMode) return <AgentOfflineScreen retry={() => location.reload()} openDemo={() => update({ mockMode: true })} />;
  return <div className={`app theme-${settings.theme} ${settings.animations ? "with-motion" : "no-motion"}`}>
    <header className="topbar"><span>What’s on My Desk?</span><button className="gear" onClick={() => setSettingsOpen(value => !value)} aria-label="Settings">⌁</button></header>
    <DeviceScene devices={devices} showNames={settings.showNames} />
    {settingsOpen && <SettingsPanel settings={settings} update={update} close={() => setSettingsOpen(false)} refresh={() => void refresh()} />}
    {settings.mockMode && <MockControlPanel devices={mocks} setDevices={setMocks} />}
  </div>;
}

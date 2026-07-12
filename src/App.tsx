import { useEffect, useMemo, useRef, useState } from "react";
import "./App.css";
import { browserMidi } from "./bridge/browserMode";
import { connectExtension } from "./bridge/extensionBridge";
import { DeviceScene } from "./components/DeviceScene";
import { MockControlPanel } from "./components/MockControlPanel";
import { ModeSelector } from "./components/ModeSelector";
import { SettingsPanel } from "./components/SettingsPanel";
import { mockDevices as initialMocks } from "./devices/mockDevices";
import type { DeviceSnapshot } from "./devices/types";
import { useSettings } from "./hooks/useSettings";

type AppMode = "initializing" | "full" | "web" | "setupRequired" | "demo" | "error";
const empty: DeviceSnapshot = { revision: 0, source: "agent", generatedAt: 0, devices: [] };
export default function App() {
  const { settings, update } = useSettings(); const [mode, setMode] = useState<AppMode>("initializing"); const [snapshot, setSnapshot] = useState(empty); const [mocks, setMocks] = useState(initialMocks); const [settingsOpen, setSettingsOpen] = useState(false); const revision = useRef(0);
  useEffect(() => connectExtension(message => { if (message.type === "status") setMode(message.connected ? "full" : "setupRequired"); if (message.type === "snapshot" && message.snapshot.revision > revision.current) { revision.current = message.snapshot.revision; setSnapshot({ ...message.snapshot, devices: [...message.snapshot.devices] }); setMode("full"); } if (message.type === "error") setMode("error"); }), []);
  const source = mode === "demo" ? mocks : snapshot.devices;
  const devices = useMemo(() => source.filter(device => (settings.showBuiltIn || device.isExternal || device.category === "computer" || device.category === "display") && (settings.showUnknown || device.category !== "unknown") && (settings.showUsbGeneric || device.category !== "usbGeneric") && (settings.showPrinters || device.category !== "printer") && (settings.showVirtual || !device.isVirtual)), [settings, source]);
  if (mode === "initializing" || mode === "setupRequired" || mode === "error") return <ModeSelector full={() => setMode("setupRequired")} browser={() => setMode("web")} demo={() => setMode("demo")} />;
  if (mode === "web" && !snapshot.devices.length) return <main className="mode-selector"><div><h1>Browser Mode</h1><p>Only browser-permitted real devices appear.</p><button onClick={() => void browserMidi().then(next => { revision.current = next.revision; setSnapshot(next); })}>Connect MIDI</button><button className="quiet" onClick={() => setMode("setupRequired")}>Back</button></div></main>;
  return <div className={`app theme-${settings.theme} ${settings.animations ? "with-motion" : "no-motion"}`}><header className="topbar"><span>What’s on My Desk?</span><button className="gear" onClick={() => setSettingsOpen(value => !value)} aria-label="Settings">⌁</button></header><DeviceScene devices={devices} showNames={settings.showNames} />{settingsOpen && <SettingsPanel settings={settings} update={update} close={() => setSettingsOpen(false)} refresh={() => window.location.reload()} />}{mode === "demo" && <MockControlPanel devices={mocks} setDevices={setMocks} />}</div>;
}

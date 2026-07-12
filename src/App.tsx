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
type InteractionMode = "wallpaper" | "interactive";
const empty: DeviceSnapshot = { revision: 0, source: "agent", generatedAt: 0, devices: [] };
export default function App() {
  const { settings, update } = useSettings(); const [mode, setMode] = useState<AppMode>("initializing"); const [interactionMode, setInteractionMode] = useState<InteractionMode>("wallpaper"); const [snapshot, setSnapshot] = useState(empty); const [mocks, setMocks] = useState(initialMocks); const [settingsOpen, setSettingsOpen] = useState(false); const [hostStrategy, setHostStrategy] = useState("unknown"); const [lastPointer, setLastPointer] = useState("none"); const [lastKey, setLastKey] = useState("none"); const revision = useRef(0);
  const localApp = location.hostname === "127.0.0.1" || location.hostname === "localhost";
  const debugInteraction = new URLSearchParams(location.search).get("debugInteraction") === "1";
  useEffect(() => {
    if (localApp) {
      let socket: WebSocket | undefined;
      const apply = (next: DeviceSnapshot) => { if (next.revision >= revision.current) { revision.current = next.revision; setSnapshot({ ...next, devices: [...next.devices] }); setMode("full"); } };
      void fetch("/api/device-snapshot").then(response => response.json()).then(apply).catch(() => setMode("error"));
      socket = new WebSocket(`${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/ws`);
      socket.onmessage = event => { const message = JSON.parse(event.data) as { type: string; payload: DeviceSnapshot }; if (message.type === "device-snapshot-updated") apply(message.payload); };
      return () => socket?.close();
    }
    if (!localApp) return;
    return connectExtension(message => { if (message.type === "status") setMode(message.connected ? "full" : "setupRequired"); if (message.type === "snapshot" && message.snapshot.revision >= revision.current) { revision.current = message.snapshot.revision; setSnapshot({ ...message.snapshot, devices: [...message.snapshot.devices] }); setMode("full"); } if (message.type === "error") setMode("error"); });
  }, [localApp]);
  useEffect(() => {
    const onMode = (event: Event) => setInteractionMode((event as CustomEvent<{ mode: InteractionMode }>).detail.mode);
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && interactionMode === "interactive") {
        (window as Window & { ipc?: { postMessage(message: string): void } }).ipc?.postMessage("set-interaction-mode:wallpaper");
      }
    };
    window.addEventListener("womd-interaction-mode", onMode);
    window.addEventListener("keydown", onKeyDown);
    return () => { window.removeEventListener("womd-interaction-mode", onMode); window.removeEventListener("keydown", onKeyDown); };
  }, [interactionMode]);
  useEffect(() => {
    const onPointer = (event: PointerEvent) => setLastPointer(event.type);
    const onKey = (event: KeyboardEvent) => setLastKey(event.key);
    const onHost = (event: Event) => setHostStrategy((event as CustomEvent<{ strategy: string }>).detail.strategy);
    window.addEventListener("pointermove", onPointer); window.addEventListener("keydown", onKey); window.addEventListener("womd-host-debug", onHost);
    return () => { window.removeEventListener("pointermove", onPointer); window.removeEventListener("keydown", onKey); window.removeEventListener("womd-host-debug", onHost); };
  }, []);
  if (!localApp) return <LandingPage />;
  const source = mode === "demo" ? mocks : snapshot.devices;
  const devices = useMemo(() => source.filter(device => (settings.showBuiltIn || device.isExternal || device.category === "computer" || device.category === "display") && (settings.showUnknown || device.category !== "unknown") && (settings.showUsbGeneric || device.category !== "usbGeneric") && (settings.showPrinters || device.category !== "printer") && (settings.showVirtual || !device.isVirtual)), [settings, source]);
  if (mode === "initializing" || mode === "setupRequired" || mode === "error") return <ModeSelector full={() => setMode("setupRequired")} browser={() => setMode("web")} demo={() => setMode("demo")} />;
  if (mode === "web" && !snapshot.devices.length) return <main className="mode-selector"><div><h1>Browser Mode</h1><p>Only browser-permitted real devices appear.</p><button onClick={() => void browserMidi().then(next => { revision.current = next.revision; setSnapshot(next); })}>Connect MIDI</button><button className="quiet" onClick={() => setMode("setupRequired")}>Back</button></div></main>;
  return <div className={`app theme-${settings.theme} mode-${interactionMode} ${settings.animations ? "with-motion" : "no-motion"}`}><header className="topbar"><span>What’s on My Desk?</span>{interactionMode === "interactive" && <small className="interaction-hint">Interactive mode · Press Esc to return</small>}<button className="gear" onClick={() => setSettingsOpen(value => !value)} aria-label="Settings">⌁</button></header><DeviceScene devices={devices} showNames={settings.showNames} interactionMode={interactionMode} />{debugInteraction && <aside className="interaction-debug"><b>{hostStrategy}</b><b>{interactionMode.toUpperCase()}</b><span>Document focus: {String(document.hasFocus())}</span><span>Last pointer: {lastPointer}</span><span>Last key: {lastKey}</span><span>HTTRANSPARENT: {String(interactionMode === "wallpaper")}</span><span>WS_EX_TRANSPARENT: {String(interactionMode === "wallpaper")}</span></aside>}{settingsOpen && <SettingsPanel settings={settings} update={update} close={() => setSettingsOpen(false)} refresh={() => window.location.reload()} />}{mode === "demo" && <MockControlPanel devices={mocks} setDevices={setMocks} />}</div>;
}

function LandingPage() {
  return <main className="landing"><section className="landing-hero"><p className="eyebrow">WINDOWS LIVE WALLPAPER</p><h1>What’s on<br/>My Desk?</h1><p className="landing-copy">A living desktop that changes with devices connected to your PC.</p><div className="landing-actions"><a href="https://github.com/jomebe/whats-on-my-desk/releases/latest/download/WhatsOnMyDeskSetup-x64.exe">Download for Windows</a><a className="quiet-link" href="https://github.com/jomebe/whats-on-my-desk">View on GitHub</a></div></section><section className="landing-scene"><div className="landing-monitor"><i /></div><div className="landing-tower" /><div className="landing-keyboard" /><div className="landing-mouse" /><div className="landing-plant"><i/><i/><i/></div></section><section className="landing-features"><article><b>Your setup, alive</b><span>Devices appear and disappear in real time.</span></article><article><b>Interactive when you need it</b><span>Press Ctrl + Alt + D to explore your desk.</span></article><article><b>Private and local</b><span>Device data stays on your PC. No account, no analytics.</span></article></section></main>;
}

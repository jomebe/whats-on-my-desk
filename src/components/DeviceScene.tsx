import { useEffect, useState } from "react";
import type { DeviceCategory, VisualDevice } from "../devices/types";
import { DeviceIllustration } from "./DeviceIllustration";

export function DeviceScene({ devices, showNames, interactionMode }: { devices: VisualDevice[]; showNames: boolean; interactionMode: "wallpaper" | "interactive" }) {
  const [selected, setSelected] = useState<VisualDevice | null>(null);
  const [rendered, setRendered] = useState(devices);
  useEffect(() => {
    const incoming = devices.map(device => ({ ...device, present: true }));
    setRendered(previous => [...incoming, ...previous.filter(device => device.present && !incoming.some(next => next.id === device.id)).map(device => ({ ...device, present: false }))]);
    const timer = window.setTimeout(() => setRendered(incoming), 220);
    return () => window.clearTimeout(timer);
  }, [devices]);
  const displays = rendered.filter(device => device.category === "display").slice(0, 4);
  const byCategory = (category: DeviceCategory) => rendered.filter(device => device.category === category);
  const camera = byCategory("camera")[0];
  const item = (device: VisualDevice, slot: string, index = 0, attachment = false) => <button key={device.id} className={`scene-device ${device.present ? "" : "leaving"} slot-${slot} slot-${slot}-${index}`} onClick={event => { event.stopPropagation(); setSelected(device); }} aria-label={device.displayName ?? device.category}><DeviceIllustration category={device.category} />{attachment && camera && <span className="camera-attachment"><DeviceIllustration category="camera" /></span>}{device.count > 1 && <span className="count">{device.count}</span>}{showNames && <span className="device-name">{device.displayName}</span>}</button>;
  return <main className={`scene mode-${interactionMode}`} onClick={() => setSelected(null)}>
    <div className="wall-layer"><div className="poster">MAKE<br/>SOMETHING</div><div className="ambient-light" /></div>
    <div className="desk-surface"><div className="desk-grain" /><div className="desk-front" /></div>
    <div className="cable cable-monitor" /><div className="cable cable-keyboard" /><div className="cable cable-midi" />
    <div className="decoration plant"><i/><i/><i/></div><div className="decoration mug" />
    <section className={`displays displays-${displays.length}`}>{displays.map((device, index) => item(device, "display", index, index === 0))}</section>
    {byCategory("computer").slice(0, 1).map(device => item(device, "computer"))}
    {byCategory("keyboard").slice(0, 1).map(device => item(device, "keyboard"))}
    {byCategory("mouse").slice(0, 1).map(device => item(device, "mouse"))}
    {byCategory("headset").slice(0, 1).map(device => item(device, "headset"))}
    {byCategory("speaker").slice(0, 2).map((device, index) => item(device, "speaker", index))}
    {byCategory("microphone").slice(0, 1).map(device => item(device, "microphone"))}
    {byCategory("storage").slice(0, 3).map((device, index) => item(device, "storage", index))}
    {byCategory("phone").slice(0, 1).map(device => item(device, "phone"))}
    {byCategory("gameController").slice(0, 2).map((device, index) => item(device, "controller", index))}
    {byCategory("midiKeyboard").slice(0, 1).map(device => item(device, "midi-keyboard"))}
    {byCategory("midiController").slice(0, 2).map((device, index) => item(device, "midi-controller", index))}
    {byCategory("midiInterface").slice(0, 1).map(device => item(device, "midi-interface"))}
    {byCategory("printer").slice(0, 1).map(device => item(device, "printer"))}
    {byCategory("usbGeneric").slice(0, 1).map(device => item(device, "usb"))}
    {byCategory("unknown").slice(0, 1).map(device => item(device, "unknown"))}
    {selected && <aside className="popover"><strong>{selected.displayName ?? "Connected device"}</strong><span>{selected.category}</span>{selected.manufacturer && <span>{selected.manufacturer}</span>}<span>{selected.connectionType}</span><span>{selected.isExternal ? "External" : "Built-in"}</span></aside>}
  </main>;
}

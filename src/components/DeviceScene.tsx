import { useState } from "react";
import type { DeviceCategory, VisualDevice } from "../devices/types";
import { DeviceIllustration } from "./DeviceIllustration";

export function DeviceScene({ devices, showNames }: { devices: VisualDevice[]; showNames: boolean }) {
  const [selected, setSelected] = useState<VisualDevice | null>(null);
  const displays = devices.filter(device => device.category === "display").slice(0, 4);
  const byCategory = (category: DeviceCategory) => devices.filter(device => device.category === category);
  const item = (device: VisualDevice, slot: string, index = 0) => <button key={device.id} className={`scene-device slot-${slot} slot-${slot}-${index}`} onClick={event => { event.stopPropagation(); setSelected(device); }} aria-label={device.displayName ?? device.category}><DeviceIllustration category={device.category} />{device.count > 1 && <span className="count">{device.count}</span>}{showNames && <span className="device-name">{device.displayName}</span>}</button>;
  return <main className="scene" onClick={() => setSelected(null)}>
    <div className="desk-surface" />
    <section className={`displays displays-${displays.length}`}>{displays.map((device, index) => item(device, "display", index))}</section>
    {byCategory("computer").slice(0, 1).map(device => item(device, "computer"))}
    {byCategory("camera").slice(0, 1).map(device => item(device, "camera"))}
    {byCategory("keyboard").slice(0, 1).map(device => item(device, "keyboard"))}
    {byCategory("mouse").slice(0, 1).map(device => item(device, "mouse"))}
    {byCategory("headset").slice(0, 1).map(device => item(device, "headset"))}
    {byCategory("speaker").slice(0, 2).map((device, index) => item(device, "speaker", index))}
    {byCategory("microphone").slice(0, 1).map(device => item(device, "microphone"))}
    {byCategory("storage").slice(0, 3).map((device, index) => item(device, "storage", index))}
    {byCategory("phone").slice(0, 1).map(device => item(device, "phone"))}
    {byCategory("gameController").slice(0, 2).map((device, index) => item(device, "controller", index))}
    {byCategory("printer").slice(0, 1).map(device => item(device, "printer"))}
    {byCategory("usbGeneric").slice(0, 1).map(device => item(device, "usb"))}
    {byCategory("unknown").slice(0, 1).map(device => item(device, "unknown"))}
    {selected && <aside className="popover"><strong>{selected.displayName ?? "Connected device"}</strong><span>{selected.category}</span>{selected.manufacturer && <span>{selected.manufacturer}</span>}<span>{selected.connectionType}</span><span>{selected.isExternal ? "External" : "Built-in"}</span></aside>}
  </main>;
}

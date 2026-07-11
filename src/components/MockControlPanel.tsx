import type { VisualDevice, DeviceCategory } from "../devices/types";

const options: [DeviceCategory, string][] = [["keyboard","Keyboard"],["mouse","Mouse"],["storage","USB drive"],["headset","Headset"],["camera","Webcam"],["gameController","Controller"],["unknown","Unknown USB"]];
export function MockControlPanel({ devices, setDevices }: { devices: VisualDevice[]; setDevices: (d: VisualDevice[]) => void }) {
  const toggle = ([category, name]: [DeviceCategory, string]) => { const exists = devices.some(d => d.category === category); setDevices(exists ? devices.filter(d => d.category !== category) : [...devices, { id: `mock-${category}`, category, displayName: name, connectionType: category === "mouse" ? "Bluetooth" : "USB", iconKey: category, count: 1, isExternal: true, present: true }]); };
  const monitors = devices.filter(d => d.category === "display")[0]?.count ?? 0;
  return <div className="mock-panel"><span>Demo</span><div>{[1,2,3].map(n => <button key={n} className={monitors === n ? "active" : ""} onClick={() => setDevices([...devices.filter(d => d.category !== "display"), { id: "mock-display", category: "display", displayName: `${n} displays`, connectionType: "DisplayPort", iconKey: "display", count: n, present: true }])}>{n} screens</button>)}</div><div>{options.map(o => <button key={o[0]} className={devices.some(d => d.category === o[0]) ? "active" : ""} onClick={() => toggle(o)}>{o[1]}</button>)}</div></div>;
}

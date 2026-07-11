import { useState } from "react";
import type { VisualDevice } from "../devices/types";
import { DeviceIllustration } from "./DeviceIllustration";

export function DeviceScene({ devices, showNames }: { devices: VisualDevice[]; showNames: boolean }) {
  const [selected, setSelected] = useState<VisualDevice | null>(null);
  return <main className="scene" onClick={() => setSelected(null)}>
    <div className="desk-line" />
    <div className="device-field">
      {devices.map((device, index) => <button key={device.id} className={`device device-${device.category}`} style={{ "--order": index } as React.CSSProperties} onClick={event => { event.stopPropagation(); setSelected(device); }} aria-label={device.displayName ?? device.category}>
        <DeviceIllustration category={device.category}/>
        {device.count > 1 && <span className="count">{device.count}</span>}
        {showNames && <span className="device-name">{device.displayName ?? device.category}</span>}
      </button>)}
    </div>
    {selected && <aside className="popover"><strong>{selected.displayName ?? "Connected device"}</strong><span>{selected.category}</span>{selected.manufacturer && <span>{selected.manufacturer}</span>}<span>{selected.connectionType}</span></aside>}
  </main>;
}

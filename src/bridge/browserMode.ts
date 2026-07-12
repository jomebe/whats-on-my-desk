import type { DeviceSnapshot, VisualDevice } from "../devices/types";

let revision = 0;
const midiDevice = (name: string): VisualDevice => ({ id: `browser-midi-${name.toLowerCase().replace(/[^a-z0-9]/g, "")}`, category: /piano|keyboard|keylab|launchkey|yamaha|roland|korg/i.test(name) ? "midiKeyboard" : "midiController", displayName: name, connectionType: "Browser", iconKey: "midi", count: 1, isExternal: true, isVirtual: false, present: true });
export async function browserMidi(): Promise<DeviceSnapshot> {
  const access = await navigator.requestMIDIAccess({ sysex: false });
  const names = new Set([...access.inputs.values(), ...access.outputs.values()].map(port => port.name ?? "MIDI device"));
  return { revision: ++revision, source: "browser", generatedAt: Date.now(), devices: [...names].map(midiDevice) };
}

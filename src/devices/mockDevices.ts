import type { VisualDevice } from "./types";

export const mockDevices: VisualDevice[] = [
  { id: "mock-display", category: "display", displayName: "Studio Display", connectionType: "DisplayPort", iconKey: "display", count: 1, isExternal: true, isVirtual: false, present: true },
  { id: "mock-computer", category: "computer", displayName: "Desktop PC", connectionType: "BuiltIn", iconKey: "computer", count: 1, isExternal: false, isVirtual: false, present: true },
  { id: "mock-keyboard", category: "keyboard", displayName: "Mechanical Keyboard", connectionType: "USB", iconKey: "keyboard", count: 1, isExternal: true, isVirtual: false, present: true },
  { id: "mock-mouse", category: "mouse", displayName: "Wireless Mouse", connectionType: "Bluetooth", iconKey: "mouse", count: 1, isExternal: true, isVirtual: false, present: true },
];

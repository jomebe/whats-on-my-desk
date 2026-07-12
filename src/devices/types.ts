export type DeviceCategory = "display" | "computer" | "keyboard" | "mouse" | "storage" | "speaker" | "microphone" | "headset" | "camera" | "phone" | "gameController" | "midiKeyboard" | "midiController" | "midiInterface" | "printer" | "usbGeneric" | "unknown";
export type ConnectionType = "USB" | "HDMI" | "DisplayPort" | "Bluetooth" | "BuiltIn" | "Virtual" | "Network" | "Browser" | "Unknown";

export interface VisualDevice {
  id: string;
  category: DeviceCategory;
  displayName?: string;
  manufacturer?: string;
  connectionType: ConnectionType;
  iconKey: string;
  count: number;
  isExternal: boolean;
  isVirtual: boolean;
  present: boolean;
  positionHint?: { x: number; y: number; primary: boolean };
  visualVariant?: string;
  midi?: { hasInput: boolean; hasOutput: boolean; portCount: number };
}

export interface DeviceSnapshot { revision: number; source: "agent" | "browser" | "demo"; generatedAt: number; rawDeviceCount?: number; filteredDeviceCount?: number; mergedPhysicalDeviceCount?: number; devices: VisualDevice[] }

export interface Settings {
  animations: boolean;
  showNames: boolean;
  showBuiltIn: boolean;
  showUnknown: boolean;
  showUsbGeneric: boolean;
  showPrinters: boolean;
  showVirtual: boolean;
  theme: "system" | "light" | "dark";
  mockMode: boolean;
}

export type DeviceCategory = "display" | "keyboard" | "mouse" | "storage" | "audioOutput" | "audioInput" | "headset" | "camera" | "gameController" | "printer" | "bluetooth" | "usbGeneric" | "unknown";
export type ConnectionType = "USB" | "HDMI" | "DisplayPort" | "Bluetooth" | "BuiltIn" | "Virtual" | "Network" | "Unknown";

export interface VisualDevice {
  id: string;
  category: DeviceCategory;
  displayName?: string;
  manufacturer?: string;
  connectionType: ConnectionType;
  iconKey: string;
  count: number;
  isExternal?: boolean;
  isVirtual?: boolean;
  present: boolean;
}

export interface DeviceSnapshot { generatedAt: number; devices: VisualDevice[] }

export interface Settings {
  animations: boolean;
  showNames: boolean;
  showBuiltIn: boolean;
  showUnknown: boolean;
  showVirtual: boolean;
  theme: "system" | "light" | "dark";
  mockMode: boolean;
}

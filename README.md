# What’s on My Desk?

A local-first Windows desktop app that turns the devices currently connected to your PC into a quiet, illustrated desk scene. It uses Tauri 2, Rust, React, TypeScript, Vite, and native Win32 device APIs.

## Features

- Detects present Windows PnP devices through SetupAPI without administrator access.
- Counts active displays through `EnumDisplayMonitors`.
- Classifies keyboards, mice, USB storage, cameras, controllers, printers, audio, Bluetooth, and generic USB devices.
- Refreshes the scene every second, so device changes appear within about two seconds.
- Keeps raw PnP instance IDs in the Rust process; the UI receives a short SHA-256-derived local ID.
- Includes local settings, light/dark/system themes, reduced-motion support, and a UI-only Mock Mode.
- Makes no network requests and includes no analytics or cloud account.

## Requirements

- Windows 10 or Windows 11
- Node.js 20 or newer and npm
- Rust stable with the MSVC target
- Visual Studio Build Tools with Desktop development with C++
- WebView2 Runtime

## Development

```powershell
npm install
npm run tauri dev
```

Frontend-only preview (device scan is unavailable, but Mock Mode works):

```powershell
npm run dev
```

Open the small control in the upper-right corner and enable **Mock mode** to add/remove sample devices and switch between one, two, or three displays.

## Cloudflare Pages demo

The static [Pages demo](https://whats-on-my-desk.pages.dev/) is intentionally a Mock Mode preview. Browsers cannot inspect all Windows-connected devices without explicit per-device permission, so actual detection is available only in the Tauri app. The Pages deployment contains no Functions or Workers.

## Build

```powershell
npm run build
npm run build:desktop
```

`build:desktop` avoids a Windows redirected-Documents path issue and writes `WhatsOnMyDesk.exe` plus installer packages to `release/`.

## Detection notes and limitations

The current MVP uses `DIGCF_PRESENT` to exclude stale device history, then classifies present devices by PnP class and descriptive hardware metadata. Composite devices are grouped conservatively by visual category, friendly name, and manufacturer. Windows exposes some physical products as multiple vague child devices, so deduplication and built-in/external classification remain heuristic. The current implementation uses one-second snapshot polling as the documented fallback; native `CM_Register_Notification` watching is a future improvement.

Monitor count reflects active desktop monitor handles. Connector type and physical-versus-virtual status are not asserted when Windows does not provide reliable evidence.

## Privacy

Device data stays on the machine. Serial numbers are neither collected nor stored, raw PnP instance IDs are not sent to React, and no device data is transmitted externally.

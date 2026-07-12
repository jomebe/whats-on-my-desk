# What’s on My Desk?

Windows 바탕화면 아이콘 뒤에서 실행되는 interactive device-aware live wallpaper.

## Features

- 활성 디스플레이와 현재 연결된 외장 키보드, 마우스, USB 저장장치, 일부 오디오/MIDI 장치를 장면으로 표시
- `Ctrl + Alt + D` interaction mode, `Escape` wallpaper mode 복귀
- Explorer 재시작 뒤 host/WebView2 재생성 및 재부착
- 로컬 전용 장치 탐지. 계정, 분석 SDK, 장치 데이터 전송 없음

## Download

Latest installer와 portable ZIP: [GitHub Releases](https://github.com/jomebe/whats-on-my-desk/releases/latest)

## Installation

`WhatsOnMyDeskSetup-0.1.0-alpha.2-x64.exe`를 실행한다. WebView2 Evergreen Runtime이 필요하다. 설치 프로그램은 현재 사용자 범위에 설치하며 관리자 권한을 요구하지 않는다.

## Interaction Mode

기본 wallpaper mode는 데스크톱 입력을 통과시킨다. `Ctrl + Alt + D`를 누르면 장치를 hover/click할 수 있다. `Escape`로 돌아온다.

## Supported Devices

Displays, external keyboards/mice, USB storage, selected audio devices, cameras, printers, MIDI ports를 탐지한다. Windows driver 구조와 composite device에 따라 분류는 완벽하지 않을 수 있다.

## Privacy

탐지는 PC 내부에서 수행된다. 원본 PnP instance ID와 serial number를 UI 또는 외부 서버로 보내지 않는다. 인터넷 없이 핵심 기능이 동작한다.

## Development

```powershell
npm install
npm run build
Push-Location src-tauri
cargo run --release
```

## Build

```powershell
.\scripts\package-release.ps1
```

출력: `release/WhatsOnMyDeskSetup-0.1.0-alpha.2-x64.exe`, `release/WhatsOnMyDesk-0.1.0-alpha.2-portable-x64.zip`, `release/SHA256SUMS.txt`.

## Known Limitations

물리 unplug 테스트는 모든 드라이버/장치 조합에서 완료되지 않았다. 특히 Bluetooth composite device와 vendor MIDI driver는 현재 상태 분류가 다를 수 있다.

## License

MIT

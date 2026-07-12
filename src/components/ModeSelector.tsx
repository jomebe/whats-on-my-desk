export function ModeSelector({ full, browser, demo }: { full: () => void; browser: () => void; demo: () => void }) {
  return <main className="mode-selector"><div><h1>What’s on My Desk?</h1><p>Download app. Open it. Your real desk appears.</p><a className="download" href="/downloads/WhatsOnMyDeskAgent.exe" download>Download Windows App</a><button onClick={full}>Connect Local App</button><button onClick={browser}>Use Browser Mode</button><button className="quiet" onClick={demo}>Open Demo</button></div></main>;
}

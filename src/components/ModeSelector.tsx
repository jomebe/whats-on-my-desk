export function ModeSelector({ full, browser, demo }: { full: () => void; browser: () => void; demo: () => void }) {
  return <main className="mode-selector"><div><h1>What’s on My Desk?</h1><p>Your connected devices, one quiet space.</p><button onClick={full}>Connect Full Experience</button><button onClick={browser}>Use Browser Mode</button><button className="quiet" onClick={demo}>Open Demo</button></div></main>;
}

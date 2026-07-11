export function AgentOfflineScreen({ retry, openDemo }: { retry: () => void; openDemo: () => void }) {
  return <main className="offline"><p>What’s on My Desk? Agent is not running.</p><div><button onClick={retry}>Retry</button><button onClick={openDemo}>Open Demo Mode</button></div></main>;
}

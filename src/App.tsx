import { useState } from "react";
import { SetupScreen } from "./screens/setup-screen";
import { LiveSessionScreen } from "./screens/live-session-screen";
import { ReportScreen } from "./screens/report-screen";

type Screen = { name: "setup" } | { name: "live" } | { name: "report"; sessionId: number };

function App() {
  const [screen, setScreen] = useState<Screen>({ name: "setup" });

  if (screen.name === "live") {
    return <LiveSessionScreen onSessionStopped={(sessionId) => setScreen({ name: "report", sessionId })} />;
  }

  if (screen.name === "report") {
    return <ReportScreen sessionId={screen.sessionId} onDone={() => setScreen({ name: "setup" })} />;
  }

  return <SetupScreen onSessionStarted={() => setScreen({ name: "live" })} />;
}

export default App;

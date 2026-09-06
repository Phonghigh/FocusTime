import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Profile } from "./types/profile";

function App() {
  const [profiles, setProfiles] = useState<Profile[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<Profile[]>("list_profiles")
      .then(setProfiles)
      .catch((err) => setError(String(err)));
  }, []);

  return (
    <main className="min-h-screen bg-slate-950 text-slate-100 flex flex-col items-center justify-center gap-4 p-8">
      <h1 className="text-2xl font-semibold">FocusTime</h1>
      {error && <p className="text-red-400">Error: {error}</p>}
      {!error && profiles === null && <p className="text-slate-400">Loading profiles…</p>}
      {!error && profiles !== null && profiles.length === 0 && (
        <p className="text-slate-400">No profiles yet</p>
      )}
      {!error && profiles !== null && profiles.length > 0 && (
        <ul className="text-slate-200">
          {profiles.map((p) => (
            <li key={p.id}>{p.name}</li>
          ))}
        </ul>
      )}
    </main>
  );
}

export default App;

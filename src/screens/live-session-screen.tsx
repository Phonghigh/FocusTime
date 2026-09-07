import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { LiveState } from "../types/tracker";

interface Props {
  onSessionStopped: (sessionId: number) => void;
}

const CATEGORY_COLORS: Record<string, string> = {
  work: "bg-emerald-600",
  entertainment: "bg-amber-500",
  distraction: "bg-red-600",
  unclassified: "bg-slate-600",
};

export function LiveSessionScreen({ onSessionStopped }: Props) {
  const [live, setLive] = useState<LiveState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<LiveState>("live-state-update", (event) => {
      setLive(event.payload);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const stopSession = async () => {
    try {
      const sessionId = await invoke<number | null>("stop_session");
      if (sessionId !== null) onSessionStopped(sessionId);
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <main className="min-h-screen bg-slate-950 text-slate-100 p-8 flex flex-col gap-6 max-w-2xl mx-auto">
      <h1 className="text-2xl font-semibold">FocusTime — Live Session</h1>
      {error && <p className="text-red-400">Error: {error}</p>}

      {live ? (
        <>
          <div className="flex items-center justify-between bg-slate-900 rounded px-4 py-3">
            <span className="text-lg">{live.domain ?? live.process_name}</span>
            <span className={`px-3 py-1 rounded text-sm font-semibold ${CATEGORY_COLORS[live.category]}`}>
              {live.category}
            </span>
          </div>
          {live.domain && <p className="text-slate-500 text-sm -mt-4">via {live.process_name}</p>}
          <p className="text-slate-400">Elapsed: {live.elapsed_seconds}s</p>

          <section className="flex flex-col gap-2">
            <TotalRow label="Work" seconds={live.totals.work} color={CATEGORY_COLORS.work} />
            <TotalRow label="Entertainment" seconds={live.totals.entertainment} color={CATEGORY_COLORS.entertainment} />
            <TotalRow label="Distraction" seconds={live.totals.distraction} color={CATEGORY_COLORS.distraction} />
            <TotalRow label="Unclassified" seconds={live.totals.unclassified} color={CATEGORY_COLORS.unclassified} />
          </section>
        </>
      ) : (
        <p className="text-slate-400">Waiting for first update…</p>
      )}

      <button className="bg-red-600 rounded px-4 py-3 font-semibold w-fit" onClick={stopSession}>
        Stop Session
      </button>
    </main>
  );
}

function TotalRow({ label, seconds, color }: { label: string; seconds: number; color: string }) {
  return (
    <div className="flex items-center justify-between bg-slate-900 rounded px-3 py-2">
      <span className="flex items-center gap-2">
        <span className={`w-3 h-3 rounded-full ${color}`} />
        {label}
      </span>
      <span>{seconds}s</span>
    </div>
  );
}

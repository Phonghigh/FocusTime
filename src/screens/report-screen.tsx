import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SessionReport } from "../types/tracker";

interface Props {
  sessionId: number;
  onDone: () => void;
}

const CATEGORY_COLORS: Record<string, string> = {
  work: "bg-emerald-600",
  entertainment: "bg-amber-500",
  distraction: "bg-red-600",
  unclassified: "bg-slate-600",
};

export function ReportScreen({ sessionId, onDone }: Props) {
  const [report, setReport] = useState<SessionReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<SessionReport>("get_session_report", { sessionId })
      .then(setReport)
      .catch((err) => setError(String(err)));
  }, [sessionId]);

  if (error) return <p className="text-red-400 p-8">Error: {error}</p>;
  if (!report) return <p className="text-slate-400 p-8">Loading report…</p>;

  const { totals } = report;
  const max = Math.max(totals.work, totals.entertainment, totals.distraction, totals.unclassified, 1);

  return (
    <main className="min-h-screen bg-slate-950 text-slate-100 p-8 flex flex-col gap-6 max-w-2xl mx-auto">
      <h1 className="text-2xl font-semibold">FocusTime — Session Report</h1>

      <section className="flex flex-col gap-2">
        <Bar label="Work" seconds={totals.work} max={max} color={CATEGORY_COLORS.work} />
        <Bar label="Entertainment" seconds={totals.entertainment} max={max} color={CATEGORY_COLORS.entertainment} />
        <Bar label="Distraction" seconds={totals.distraction} max={max} color={CATEGORY_COLORS.distraction} />
        <Bar label="Unclassified" seconds={totals.unclassified} max={max} color={CATEGORY_COLORS.unclassified} />
      </section>

      <p className="text-lg">
        Interruptions: <span className="font-semibold">{report.interruption_count}</span>
      </p>

      <section className="flex flex-col gap-1">
        <h2 className="text-lg font-semibold">Timeline</h2>
        <ul className="flex flex-col gap-1 max-h-96 overflow-y-auto">
          {report.events.map((e) => {
            const duration = (e.ended_at ?? e.started_at) - e.started_at;
            return (
              <li key={e.id} className="flex items-center justify-between bg-slate-900 rounded px-3 py-2">
                <span>{e.process_name}</span>
                <span className={`px-2 py-0.5 rounded text-xs font-semibold ${CATEGORY_COLORS[e.category]}`}>
                  {e.category}
                </span>
                <span className="text-slate-400">{duration}s</span>
              </li>
            );
          })}
        </ul>
      </section>

      <button className="bg-slate-700 rounded px-4 py-3 font-semibold w-fit" onClick={onDone}>
        Back to Setup
      </button>
    </main>
  );
}

function Bar({ label, seconds, max, color }: { label: string; seconds: number; max: number; color: string }) {
  const widthPct = Math.round((seconds / max) * 100);
  return (
    <div className="flex flex-col gap-1">
      <div className="flex justify-between text-sm text-slate-400">
        <span>{label}</span>
        <span>{seconds}s</span>
      </div>
      <div className="bg-slate-900 rounded h-4 overflow-hidden">
        <div className={`h-full ${color}`} style={{ width: `${widthPct}%` }} />
      </div>
    </div>
  );
}

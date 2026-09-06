import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Profile } from "../types/profile";
import type { AppRuleInput, Category, DomainRuleInput, RunningProcess } from "../types/tracker";

const CATEGORIES: Category[] = ["work", "entertainment", "distraction", "unclassified"];

interface Props {
  onSessionStarted: (sessionId: number) => void;
}

export function SetupScreen({ onSessionStarted }: Props) {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [profileId, setProfileId] = useState<number | null>(null);
  const [newProfileName, setNewProfileName] = useState("");
  const [processes, setProcesses] = useState<RunningProcess[]>([]);
  const [rules, setRules] = useState<Record<string, Category>>({});
  const [domainRules, setDomainRules] = useState<DomainRuleInput[]>([]);
  const [newDomain, setNewDomain] = useState("");
  const [newDomainCategory, setNewDomainCategory] = useState<Category>("work");
  const [error, setError] = useState<string | null>(null);

  const loadProfiles = () => {
    invoke<Profile[]>("list_profiles")
      .then((list) => {
        setProfiles(list);
        if (list.length > 0 && profileId === null) setProfileId(list[0].id);
      })
      .catch((err) => setError(String(err)));
  };

  useEffect(loadProfiles, []);

  const createProfile = async () => {
    if (!newProfileName.trim()) return;
    try {
      const profile = await invoke<Profile>("create_profile", { name: newProfileName.trim() });
      setNewProfileName("");
      setProfiles((prev) => [...prev, profile]);
      setProfileId(profile.id);
    } catch (err) {
      setError(String(err));
    }
  };

  const detectProcesses = async () => {
    try {
      const list = await invoke<RunningProcess[]>("list_running_processes");
      setProcesses(list);
    } catch (err) {
      setError(String(err));
    }
  };

  const setCategory = (processName: string, category: Category) => {
    setRules((prev) => ({ ...prev, [processName]: category }));
  };

  const addDomainRule = () => {
    const domain = newDomain.trim().toLowerCase();
    if (!domain) return;
    setDomainRules((prev) => [...prev.filter((r) => r.domain !== domain), { domain, category: newDomainCategory }]);
    setNewDomain("");
  };

  const removeDomainRule = (domain: string) => {
    setDomainRules((prev) => prev.filter((r) => r.domain !== domain));
  };

  const startSession = async () => {
    if (profileId === null) return;
    try {
      const ruleInputs: AppRuleInput[] = Object.entries(rules)
        .filter(([, category]) => category !== "unclassified")
        .map(([process_name, category]) => ({ process_name, category }));
      await invoke("set_app_rules", { profileId, rules: ruleInputs });
      await invoke("set_domain_rules", { profileId, rules: domainRules });
      const sessionId = await invoke<number>("start_session", { profileId });
      onSessionStarted(sessionId);
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <main className="min-h-screen bg-slate-950 text-slate-100 p-8 flex flex-col gap-6 max-w-2xl mx-auto">
      <h1 className="text-2xl font-semibold">FocusTime — Setup</h1>
      {error && <p className="text-red-400">Error: {error}</p>}

      <section className="flex flex-col gap-2">
        <label className="text-sm text-slate-400">Profile</label>
        <select
          className="bg-slate-800 rounded px-3 py-2"
          value={profileId ?? ""}
          onChange={(e) => setProfileId(Number(e.target.value))}
        >
          {profiles.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
        <div className="flex gap-2">
          <input
            className="bg-slate-800 rounded px-3 py-2 flex-1"
            placeholder="New profile name"
            value={newProfileName}
            onChange={(e) => setNewProfileName(e.target.value)}
          />
          <button className="bg-slate-700 rounded px-3 py-2" onClick={createProfile}>
            Create
          </button>
        </div>
      </section>

      <section className="flex flex-col gap-2">
        <button className="bg-slate-700 rounded px-3 py-2 w-fit" onClick={detectProcesses}>
          Detect running apps
        </button>
        <ul className="flex flex-col gap-1 max-h-80 overflow-y-auto">
          {processes.map((p) => (
            <li key={p.pid} className="flex items-center justify-between bg-slate-900 rounded px-3 py-2">
              <span>{p.process_name}</span>
              <select
                className="bg-slate-800 rounded px-2 py-1"
                value={rules[p.process_name] ?? "unclassified"}
                onChange={(e) => setCategory(p.process_name, e.target.value as Category)}
              >
                {CATEGORIES.map((c) => (
                  <option key={c} value={c}>
                    {c}
                  </option>
                ))}
              </select>
            </li>
          ))}
        </ul>
      </section>

      <section className="flex flex-col gap-2">
        <label className="text-sm text-slate-400">Domain rules (for browser tabs)</label>
        <div className="flex gap-2">
          <input
            className="bg-slate-800 rounded px-3 py-2 flex-1"
            placeholder="e.g. youtube.com"
            value={newDomain}
            onChange={(e) => setNewDomain(e.target.value)}
          />
          <select
            className="bg-slate-800 rounded px-2 py-1"
            value={newDomainCategory}
            onChange={(e) => setNewDomainCategory(e.target.value as Category)}
          >
            {CATEGORIES.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
          <button className="bg-slate-700 rounded px-3 py-2" onClick={addDomainRule}>
            Add
          </button>
        </div>
        <ul className="flex flex-col gap-1 max-h-40 overflow-y-auto">
          {domainRules.map((r) => (
            <li key={r.domain} className="flex items-center justify-between bg-slate-900 rounded px-3 py-2">
              <span>
                {r.domain} — {r.category}
              </span>
              <button className="text-red-400" onClick={() => removeDomainRule(r.domain)}>
                Remove
              </button>
            </li>
          ))}
        </ul>
      </section>

      <button
        className="bg-emerald-600 rounded px-4 py-3 font-semibold disabled:opacity-50"
        disabled={profileId === null}
        onClick={startSession}
      >
        Start Session
      </button>
    </main>
  );
}

export type Category = "work" | "entertainment" | "distraction" | "unclassified";

export interface AppRule {
  id: number;
  profile_id: number;
  process_name: string;
  category: Category;
}

export interface AppRuleInput {
  process_name: string;
  category: Category;
}

export interface RunningProcess {
  pid: number;
  process_name: string;
}

export interface CategoryTotals {
  work: number;
  entertainment: number;
  distraction: number;
  unclassified: number;
}

export interface LiveState {
  process_name: string;
  category: Category;
  elapsed_seconds: number;
  totals: CategoryTotals;
}

export interface UsageEvent {
  id: number;
  session_id: number;
  process_name: string;
  domain: string | null;
  category: Category;
  started_at: number;
  ended_at: number | null;
}

export interface SessionReport {
  session_id: number;
  totals: CategoryTotals;
  events: UsageEvent[];
  interruption_count: number;
}

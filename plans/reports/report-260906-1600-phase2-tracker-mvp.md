# Phase 2 — Active-Window Tracker MVP

Status: completed (builds pass, app launches; full interactive click-through not empirically observed).
Commit: `0269607` feat(tracker): add active-window tracker, session engine, and Tauri commands

## Files Modified
- src-tauri/Cargo.toml — added `sysinfo = "0.32"`, `windows = "0.58"` (Foundation, UI_WindowsAndMessaging, System_Threading)
- src-tauri/src/lib.rs — mod tracker, manages SessionEngine::default(), registers 8 commands
- src-tauri/src/db/category.rs — added `Unclassified` 4th variant
- src-tauri/src/db/mod.rs — re-exports AppRule, AppRuleInput, UsageEvent
- src-tauri/src/db/models.rs — added AppRule, AppRuleInput, UsageEvent structs
- src-tauri/src/commands/mod.rs — added processes, rules, session modules
- src-tauri/src/commands/profiles.rs — added create_profile(name)
- src/App.tsx — rewritten as 3-screen local-state switcher (setup/live/report)

## Files Created
- src-tauri/src/tracker/mod.rs, window_poller.rs (raw `windows` crate: GetForegroundWindow/GetWindowThreadProcessId/QueryFullProcessImageNameW), classifier.rs (app_rules lookup -> Category, fallback Unclassified), session.rs (SessionEngine: start/on_tick/stop/live_state, CategoryTotals, LiveState)
- src-tauri/src/commands/processes.rs (list_running_processes via sysinfo), rules.rs (set_app_rules, get_app_rules), session.rs (start_session spawns 1s poller thread emitting `live-state-update` event, stop_session, get_session_report with sequential-scan interruption_count)
- src/types/tracker.ts
- src/screens/setup-screen.tsx, live-session-screen.tsx, report-screen.tsx

## Deviations from plan
1. Used raw `windows` crate instead of `active-win-pos-rs` (not verified to compile cleanly with GNU toolchain; raw crate is what Phase 1 environment already proved works).
2. `get_live_state` command replaced with a `live-state-update` Tauri event emitted every ~1s from the poller thread — plan explicitly allowed this as the preferred approach.
3. A Rust integration test driving SessionEngine against in-memory DB was written then removed: the test binary crashed with `STATUS_ENTRYPOINT_NOT_FOUND` because `focustime_lib` pulls in the full tauri/WebView2 runtime, which a standalone test exe can't resolve on this machine's DLL search path. Same class of environment issue as Phase 1's cdylib/GNU linker bug — not a logic bug.
4. Did not perform live interactive click-through: screenshotting/synthetic-clicking would have driven the user's real live desktop (real Chrome, real windows), which was judged unsafe/inappropriate. Verified instead via code review + type-signature matching between Rust commands and TS invoke calls + successful compile/launch.

## Manual test steps (not yet executed by agent — for next run)
1. `cd D:\project\FocusTime && npm run tauri dev` (GNU toolchain on PATH per Phase 1 notes)
2. Setup screen: create a profile, "Detect running apps", assign two open apps to different categories (e.g. editor -> work, browser -> entertainment)
3. "Start Session" -> Live screen
4. Alt-tab between the two apps for ~10-15s; watch current-app badge / elapsed seconds update ~1/s, both category totals climbing
5. "Stop Session" -> Report screen should show non-zero work/entertainment bars proportional to time spent, a timeline of switches, interruption_count > 0 if you went work->entertainment at least once

## Verification done
- `cargo build` (src-tauri, GNU toolchain) — pass
- `npm run build` (runs tsc first) — pass
- `npm run tauri dev` — launched, confirmed running via tasklist

## Unresolved questions / next steps
- Consider extracting db/tracker into a tauri-free lib crate later so standalone Rust tests don't hit STATUS_ENTRYPOINT_NOT_FOUND.
- Genuine automated GUI verification would need a disposable VM/CI runner — not available in this environment, not attempted.
- Phase 3 (browser domain tracking, native messaging) and Phase 4 (history/charts) not started.
- **Action needed from user**: please manually run the steps above once to confirm real end-to-end behavior before Phase 3 builds on top of it.

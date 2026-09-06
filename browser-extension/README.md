# FocusTime Domain Tracker (browser extension)

Reports the active tab's domain to the FocusTime desktop app so browser time
can be classified per-domain (e.g. `github.com` -> Work, `youtube.com` ->
Entertainment) instead of just "chrome.exe".

Not published to any web store — load it unpacked and register the native
messaging host manually (one-time setup, per browser, per machine).

## 1. Load the unpacked extension

1. Open `chrome://extensions` (or `edge://extensions` for Edge).
2. Enable "Developer mode" (top right).
3. Click "Load unpacked" and select this `browser-extension/` folder.
4. Copy the generated extension ID shown on the card (looks like
   `abcdefghijklmnopabcdefghijklmnop`).

## 2. Build the native host binary

Chrome always launches the exact executable in the manifest's `"path"` with
no way for us to pass it a custom flag — so the native host is a **separate,
dedicated binary** (`focustime-native-host.exe`), not the main app
(`focustime.exe`). Build it once from `src-tauri/`:

```
cargo build --release --bin focustime-native-host
```

This produces `src-tauri/target/release/focustime-native-host.exe`.

## 3. Register the native messaging host

1. Edit `native-host/com.focustime.native_host.json`:
   - Set `"path"` to the absolute path of `focustime-native-host.exe` from
     step 2 (e.g.
     `D:\\project\\FocusTime\\src-tauri\\target\\release\\focustime-native-host.exe`).
   - Set `"allowed_origins"` to `"chrome-extension://<your-extension-id>/"`
     using the ID from step 1.
2. Register the manifest in the registry (per-user, no admin rights needed).
   Run in an elevated-free PowerShell/cmd, adjusting the path:

   ```
   reg add "HKCU\Software\Google\Chrome\NativeMessagingHosts\com.focustime.native_host" /ve /t REG_SZ /d "D:\project\FocusTime\browser-extension\native-host\com.focustime.native_host.json" /f
   ```

   For Edge (Chromium), also add:

   ```
   reg add "HKCU\Software\Microsoft\Edge\NativeMessagingHosts\com.focustime.native_host" /ve /t REG_SZ /d "D:\project\FocusTime\browser-extension\native-host\com.focustime.native_host.json" /f
   ```

3. Restart the browser (or reload the extension in `chrome://extensions`)
   so it picks up the new native messaging host.

### Firefox

**Caveat:** Firefox's MV3 `background.service_worker` support is limited/
experimental compared to Chrome/Edge; if the temporary add-on fails to load
the background script, switch `background` in `manifest.json` to
`{"scripts": ["background.js"]}` for Firefox specifically (Chrome/Edge would
need `service_worker` instead) — you may need two slightly different
manifests if you want both browser families working at once.

Firefox doesn't use the Windows registry for native messaging — it reads the
manifest from a fixed per-user folder instead, and its `allowed_extensions`
field (not `allowed_origins`) takes the extension's ID from `manifest.json`
rather than a generated Chrome-style ID:

1. Load the extension temporarily via `about:debugging#/runtime/this-firefox`
   -> "Load Temporary Add-on" -> select `browser-extension/manifest.json`
   (Firefox unloads temporary add-ons on restart; a signed/permanent install
   isn't required for local testing).
2. In `native-host/com.focustime.native_host.json`, use `allowed_extensions`
   instead of `allowed_origins`, e.g.:
   ```json
   "allowed_extensions": ["focustime@example.com"]
   ```
   matching the `browser_specific_settings.gecko.id` you set in
   `manifest.json` (add that key if not already present).
3. Copy the manifest to:
   ```
   %APPDATA%\Mozilla\NativeMessagingHosts\com.focustime.native_host.json
   ```
   (no registry entry needed).
4. Restart Firefox.

## 4. Run FocusTime

Start the FocusTime desktop app (`focustime.exe`, the GUI one) normally —
it listens on `127.0.0.1:47821` for domain updates. When the browser is the
tracked foreground app and you switch tabs, Chrome spawns
`focustime-native-host.exe` in the background, which forwards the active
domain to the running app over that port.

**Note:** this manual registry + unpacked-extension setup has not been
exercised end-to-end in the automated build environment (no interactive
browser/registry access there) — please verify it once locally.

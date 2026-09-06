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

## 2. Register the native messaging host

1. Edit `native-host/com.focustime.native_host.json`:
   - Set `"path"` to the absolute path of the built `focustime.exe`
     (e.g. `D:\\project\\FocusTime\\src-tauri\\target\\release\\focustime.exe`).
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

3. Restart the browser so it picks up the new native messaging host.

## 3. Run FocusTime

Start the FocusTime desktop app normally (it listens on `127.0.0.1:47821`
for domain updates). When the browser is the tracked foreground app and you
switch tabs, Chrome spawns `focustime.exe --native-host` in the background,
which forwards the active domain to the running app over that port.

**Note:** this manual registry + unpacked-extension setup has not been
exercised end-to-end in the automated build environment (no interactive
browser/registry access there) — please verify it once locally.

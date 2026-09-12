# TODO

## Done

- Windows-first v1: Collect/chart/close, Reports, mixed Combine, Help,
  accessibility, CI, and unsigned NSIS.
- Exact rngkit-core `23a67aa` is pinned. Mixed Combine: per-input source/fold,
  bits/interval copy, and Help. User reported native `tauri dev` acceptance
  on 2026-09-11.
- User-reported 2026-09-07 Help/Reports/Combine and Device setup folder-open;
  2026-09-10 TrueRNG freshness; 2026-09-11 BitBabbler acquisition, folds and
  disconnect. No separate logs or fold matrix for the 2026-09-11 hardware run.
- Unsigned NSIS rebuilt 2026-09-11 at `a505141`: ten matching kit files, no
  TrueRNG INF/CAT in the payload, offline WebView2, non-elevated `/S /UPDATE`.
  User reported installation and the installed app work as expected.
  Previous-install INF/CAT leftovers remain. Disconnected install untested.
- Tag-only GitHub Release drafts: Windows NSIS + Ubuntu 22.04 amd64 `.deb`.
  CI on push/PR stays `--no-bundle`. Version/lockfile agreement and exact-SHA
  CI (Windows+Ubuntu) are required before packaging. Drafts stay unpublished
  until a tag is pushed and Publish is clicked. User reported the app works
  on Linux without specifying distro, install method, or hardware tests.
  Linux hardware acceptance is still separate.
- Release version tests derive the real repository tag dynamically; fixed
  stable/prerelease scenarios use isolated fixtures so version bumps keep CI valid.

## Next steps

1. Validate installation and Help kit access with network disconnected as a
   standard user; clean-machine WebView2 remains untested. Decide cleanup of old
   TrueRNG INF/CAT retained by updates. Redistribution permission is required
   only before including them again. Setup must not run implicitly.
2. Separately authorize Ubuntu/Debian hardware validation (both devices after
   login/reconnect, repeated setup, unrelated FTDI unchanged) when a host exists.
3. Validate remaining artifact open/folder actions and native scaling acceptance.

## Backlog

- [ ] Optional sample-count limit with automatic collection finalization.
- [ ] Session name and notes for experiment identification in manifests/reports.
  Both features await design and implementation; sample-count limit comes first.
- Additional hardware/device coverage beyond reported TrueRNG and BitBabbler
  acceptance, and Linux physical behavior; opt-in only.
- Native 100k/1M chart interaction, Reports/Combine dialogs, screen-reader sampling
  and symlink inspection privilege coverage.
- NSIS uninstall/session-data preservation, signing, updater, Store, and
  publishing a GitHub Release; separate authorization remains required.
- TrueRNGpro, RngKitPSG v2, multi-source/XOR, reconnect, resume and statistical
  inference remain non-goals or future work.

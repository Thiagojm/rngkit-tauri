# Agent instructions

## Reading order

Read these files before changing the project:

1. `docs/PROJECT_CONTEXT.md`
2. `docs/DECISIONS.md`
3. `TODO.md`
4. `README.md` and the relevant implementation area

## Verified commands (Windows host, through 2026-08-24)

From the repository root. Scaffold used Node.js 24.18.0 / npm 11.16.0; floors are Node `^20.19.0 || >=22.12.0` and npm `>=10`. Stable Rust is 1.97.1 and MSRV `1.85.0` is installed. Do not install a toolchain silently.

```text
npm ci
npm run format:check
npm run check
npm run lint
npm run test:unit -- --run
npm run test:e2e
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --locked --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --locked --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml --doc
cargo +1.85.0 check --locked --manifest-path src-tauri/Cargo.toml --all-targets
cargo +1.85.0 test --locked --manifest-path src-tauri/Cargo.toml --all-targets
npm run tauri -- build --no-bundle -- --locked
npm run tauri -- build --bundles nsis -- --locked
git status --short --branch
git diff --check
```

`.github/workflows/ci.yml` runs the locked suite without the NSIS step. Ubuntu is compile evidence, not Linux desktop support.
Observed remote success for `061f66a` (2026-08-24): https://github.com/Thiagojm/rngkit-tauri/actions/runs/32755861549

Run the native window with `npm run tauri dev`. Opt-in serial physical smokes use `cargo test --manifest-path src-tauri/Cargo.toml --test hardware bitb -- --ignored --test-threads=1 --nocapture`; replace `bitb` with `trng`, `rdseed`, or `discover`.
Unsigned NSIS is local-only. The user reported offline installation, first launch, and basic app functionality on this Windows host; do not claim uninstall/session-data preservation, signing, or native long-session chart render/interaction. Data-only 100k/1M retention was remeasured in Checkpoint 18.

## Repository conventions

- The original app and 2026-08-24 workflow improvements are implemented and
  published through Help. The 2026-08-25 artifact-feedback/report-chart work,
  terminal-outcome correction, and selected-basename/path-display corrections
  are implemented. The app pins `rngkit-core` revision
  `3dc969d983ffa7c981536c46d19afa223f0c490b`.
- Deterministic, browser-integrated, MSRV, no-bundle release, and native
  PseudoRNG Collect/manifest-backed Reports smoke validation passed. Remaining
  native acceptance boundaries are listed in `docs/PROJECT_CONTEXT.md`.
- Preserve the approved design and plan; update their current-state references only when evidence changes.
- Use the locked versions in `package-lock.json` and `src-tauri/Cargo.lock`; do not float dependencies or use prereleases.
- Browser tests use Playwright with production assets and no real Tauri IPC or
  hardware. On Windows they use the installed Edge channel. Vitest component
  tests set `resolve.conditions` to `browser` so Svelte client `mount` is used.
- Final `rngkit-*` dependencies must use the exact reachable Git revision
  `3dc969d983ffa7c981536c46d19afa223f0c490b`. Never a local path.
- Default tests must not enumerate or open hardware. Physical tests are ignored,
  opt-in, and serial.
- Frontend capabilities stay minimal: `core:default` and `dialog:default` only.
  Never grant general filesystem, shell, opener, or logging access. CSP in
  `src-tauri/tauri.conf.json` is restricted. Production IPC is `get_app_state`, `refresh_sources`, `select_source`, `set_sample_bits`,
  `set_interval_seconds`, `set_fold`, `set_theme`, `choose_output_folder`,
  `start_collection`, `stop_collection`, `start_another_session`,
  `open_session_folder`, `open_collection_working_folder`,
  `acknowledge_outcome`, `copy_diagnostics`, `stop_and_exit`,
  `choose_report_input`, `generate_report`, `replace_report`, `open_report`,
  `open_report_folder`, `open_report_working_folder`,
  `choose_csv_inputs`, `remove_combine_input`, `clear_combine_inputs`,
  `create_derived`, `generate_derived`, `open_derived_folder`, and
  `open_combine_working_folder`. Default start
  does not enumerate hardware. Default
  tests inject fake discovery and fake sources and do not call
  `rngkit_sources::discover()` or open hardware. Native startup performs one
  asynchronous discovery after frontend hydration; it still never selects a
  source. Open session folder and report
  artifacts use backend-known paths only. Close policy is captured while
  coordinator state is locked. Active reloads poll until terminal reconciliation.
  Close while collecting confirms Keep collecting or Stop and exit; close while
  stopping waits for finalization.
- Never persist or expose entropy, seeds, serials, OS device paths, or arbitrary
  diagnostic chains.
- Keep statistical Z and `+/-1.96` explicitly descriptive, never inferential.
- Commit, push, release, signing, publication, deployment, and remote deletion
  remain separate approvals unless explicitly authorized.

## Context maintenance

Update `TODO.md` after relevant work, `docs/DECISIONS.md` for durable contracts,
and `docs/PROJECT_CONTEXT.md` when product facts or evidence change. Keep them dense within the `tjm-memoria` soft budgets.

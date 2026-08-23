# Agent instructions

## Reading order

Read these files before changing the project:

1. `docs/PROJECT_CONTEXT.md`
2. `docs/DECISIONS.md`
3. `TODO.md`
4. `docs/specs/2026-08-22-rngkit-tauri-design.md`
5. `docs/plans/2026-08-22-rngkit-tauri-plan.md`
6. `README.md` and the relevant implementation area

## Verified commands (Windows host, through 2026-08-22)

From the repository root. Node.js 24.18.0 / npm 11.16.0 were used at scaffold
time. The floor is Node `^20.19.0 || >=22.12.0` and npm `>=10`. Stable Rust is
1.97.1; MSRV toolchain `1.85.0` is installed. Do not install a missing
toolchain silently.

```text
npm ci
npm run format:check
npm run check
npm run lint
npm run test:unit -- --run
npm run test:e2e
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --doc
cargo +1.85.0 check --manifest-path src-tauri/Cargo.toml --all-targets
cargo +1.85.0 test --manifest-path src-tauri/Cargo.toml --all-targets
git status --short --branch
git diff --check
```

Run the native window with `npm run tauri dev`. That command was not part of
the Checkpoint 3 automated suite.

Do not claim hardware, CI, installer, or long-session chart evidence until the
applicable later checkpoint verifies those commands.

## Repository conventions

- Follow the approved plan one checkpoint at a time and stop for user testing
  and approval before starting the next checkpoint.
- A material contract change pauses implementation and requires renewed design
  approval.
- Preserve the approved design and plan; update their current-state references
  only when evidence changes.
- Use the locked versions in `package-lock.json` and `src-tauri/Cargo.lock`.
  Do not float dependencies. Prereleases are excluded.
- Browser tests use Playwright with production assets and no real Tauri IPC or
  hardware. On Windows they use the installed Edge channel. Vitest component
  tests set `resolve.conditions` to `browser` so Svelte client `mount` is used.
- Final `rngkit-*` dependencies must use the exact reachable Git revision
  `3f327e9e88679c26683323f116cd6d7b3ea64fff`. Never a local path.
- Default tests must not enumerate or open hardware. Physical tests are ignored,
  opt-in, and serial.
- Frontend capabilities stay minimal: `core:default` and `dialog:default` only.
  Never grant general filesystem, shell, opener, or logging access.
- Never persist or expose entropy, seeds, serials, OS device paths, or arbitrary
  diagnostic chains.
- Keep statistical Z and `+/-1.96` explicitly descriptive, never inferential.
- Commit, push, release, signing, publication, deployment, and remote deletion
  remain separate approvals unless explicitly authorized.

## Context maintenance

Update `TODO.md` after relevant work, `docs/DECISIONS.md` when a durable
contract changes, and `docs/PROJECT_CONTEXT.md` when current product facts or
evidence change. Keep these files dense and within the soft budgets defined by
the `maintain-project-context` skill.

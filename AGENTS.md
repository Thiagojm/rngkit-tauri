# Agent instructions

## Reading order

Read these files before changing the project:

1. `docs/PROJECT_CONTEXT.md`
2. `docs/DECISIONS.md`
3. `TODO.md`
4. `docs/specs/2026-08-22-rngkit-tauri-design.md`
5. `docs/plans/2026-08-22-rngkit-tauri-plan.md`
6. `README.md` and the relevant implementation area once scaffolding exists

## Verified commands

The repository is planning-only as of 2026-08-22. From the repository root,
the verified checks are:

```text
git status --short --branch
git diff --check
```

Do not claim npm, Cargo, Tauri, browser, native, hardware, CI, or installer
validation until the applicable checkpoint creates and verifies those commands.

## Repository conventions

- Follow the approved plan one checkpoint at a time and stop for user testing
  and approval before starting the next checkpoint.
- A material contract change pauses implementation and requires renewed design
  approval.
- Preserve the approved design and plan; update their current-state references
  only when evidence changes.
- Use stable releases and lock exact versions at scaffolding time. Prereleases
  are excluded.
- Final `rngkit-*` dependencies must use an exact reachable Git revision, never
  a local path.
- Default tests must not enumerate or open hardware. Physical tests are ignored,
  opt-in, and serial.
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

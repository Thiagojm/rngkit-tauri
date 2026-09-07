# Device setup integration design

Status: Approved by the user on 2026-09-07; implementation planning authorized.

## Context and evidence

RngKit is Windows-first, with an English Help page, backend-owned folder opening,
minimal frontend capabilities and a per-user NSIS installer. Device setup resources
are not currently bundled. Linux compilation is not native hardware acceptance.

Reviewed sources:
- `D:/Projetos/RngKitPSG/README.md` and `2-Installation/` (Windows).
- `D:/Projetos/flet_test/src/assets/installers/` (Windows and Linux).
- Current local `bitb-rs` and `trng3-rs` platform documentation and BitBabbler
  transport code; implementation must verify against the app's pinned revision.

The Flet Linux kit changes directory permissions incorrectly in its manual guide,
allows overly broad device access, includes generic FTDI IDs, unloads ftdi_sio
without an opt-in, and can report completion with unresolved prerequisites.
No setup script was executed during review.

## Goal and non-goals

Provide offline, task-oriented setup guidance and a reviewed support kit for
BitBabbler and TrueRNG3. Integrate access from Help and Collect without changing
collection, discovery or statistical behavior.

Exclude TrueRNGpro, Python dependencies, global driver blacklisting, automatic
privileged execution, Linux app packaging, signing, release publication and updater
work. Do not modify the legacy repositories. Do not copy their entire asset trees.

## User experience

Keep the eight existing Help topics. Under Choosing a source, add Device setup
with four native disclosures: Windows / BitBabbler, Windows / TrueRNG3,
Ubuntu-Debian / BitBabbler, and Ubuntu-Debian / TrueRNG3. Both platforms remain
available without OS detection. Explain that Linux hardware acceptance is pending.

Each guide covers prerequisites, exact device identity, ordered setup steps,
reconnection or fresh login, Refresh sources, and a short explicit collection test.
PseudoRNG needs no setup; RDSEED availability depends on the CPU.

Add a compact Device setup link near Refresh sources in Collect. It activates Help,
focuses the Device setup heading and scrolls it into view, preserving collection
state and chart dimensions. Reuse existing navigation and focus patterns.

One Open device setup folder button in Help opens the bundled support directory.
The app never launches an installer, terminal, script or elevation request.
Missing resources or folder-opening failures produce existing safe error feedback.

## Support kit

Use `src-tauri/resources/device-setup/` as the single packaged source. Include an
English README, Linux rules/helper and verified Windows TrueRNG3 INF/CAT and Zadig.
Record upstream URL, version, SHA-256, verification date and applicable notices.
Do not include the legacy VC++ redistributable without demonstrated need.

Acquire Windows files from official upstream sources. Verify signatures and
redistribution terms before inclusion; preserve required notices/source offers.
If a file cannot be verified or redistributed, that kit phase is blocked for that
file and must report the problem rather than silently substituting another binary.
Do not use an unversioned automatic download at app runtime.

Windows guidance specifies WinUSB for BitBabbler 0403:7840 and the manufacturer
CDC/usbser setup for TrueRNG3 04d8:f5fe. Never recommend replacing an arbitrary FTDI
or TrueRNG serial driver with WinUSB.

## Linux helper contract

Target Ubuntu/Debian with systemd/udev initially. Supply a Bash helper invoked
manually with sudo, an explicit device selector (bitbabbler, truerng3, or both),
and an explicit non-root target user. Validate the user, tools and all input files
before modifying the system. Include --help and a read-only --check mode.

Use app-owned rule filenames and a dedicated rngkit group, mode 0660, matching only
0403:7840 USB device nodes and 04d8:f5fe tty nodes. TrueRNG ModemManager exclusion
must be device-scoped. Do not grant world access or rely on a shared fixed symlink.
Let the application configure serial settings; validate this with real hardware.

Do not unload or blacklist ftdi_sio. The pinned library's interface-local handling
must be checked before release. Diagnose ownership conflicts without changing
other devices or stopping services automatically.

Identical existing rules are a no-op; differing files are preserved and cause an
explicit conflict error. No --force overwrite path. Install regular root-owned
rule files with mode 0644; never chmod the rules directory. Reject symlink targets.
Track successful changes in output and report partial failure truthfully.

Reload rules, then instruct the user to log out/in and reconnect the selected
device. Do not trigger every device on the system. Do not install OS packages
silently. Any missing runtime dependency must be based on the actual Rust package,
not inherited PyUSB requirements. Missing prerequisites produce actionable failure.

Document rollback of the exact app-owned files and membership changes. Preserve
pre-existing groups, memberships and administrator rules. No broad cleanup script.

## Backend and packaging

Add a no-argument open_device_setup_folder command. Resolve only the fixed resource
directory through Tauri's resource resolver, validate directory containment and
reject unsafe targets, then reuse the existing platform folder-opening pattern.
No frontend path, executable or URL arguments. Do not add shell/filesystem/opener
frontend capabilities. Keep errors and diagnostics redacted.

Register the directory as bundle resources. The per-user NSIS installer copies the
kit without installing drivers or changing system permissions. Linux resources are
included for offline reference; this does not establish Linux desktop support.

## Alternatives

Links-only help is smaller but cannot meet offline setup needs. Fully automatic
driver installation introduces unnecessary privilege and driver-selection risks.
A packaged kit with manual execution matches the existing backend authority model.

## Acceptance and validation

- Help guides are readable offline in both themes at 1280x800 and 800x600, including
  enlarged text; disclosures and Collect navigation work with keyboard focus.
- Collect state and chart size remain unchanged after navigating to setup.
- Folder command accepts no path and handles missing/unsafe resources and opener
  failures with safe errors. Tests must never open real folders or hardware.
- Linux syntax/static checks and isolated command-stub tests cover bad arguments,
  missing user/tools/files, identical/conflicting rules and failed operations.
  CI must never modify host udev, groups or kernel modules.
- Native Ubuntu/Debian hardware acceptance checks permissions after reconnect/login,
  both supported devices, repeated setup and unaffected unrelated FTDI devices.
  Only user-authorized hardware tests can establish this evidence.
- Inspect packaged resource contents and verify the folder action in the installed
  Windows app. Actual driver changes and installer execution require explicit user
  authorization; packaging configuration alone does not establish those results.

## Delivery and approval boundaries

Plan three independently testable phases: Help/Collect guidance; support kit and
Linux helper; backend folder access and packaging integration. Each ends with user
validation before the next phase. In phase 1, omit the folder button until resources
and the command exist; no disabled placeholder.

This document authorizes no implementation. After design approval, write a detailed
English implementation plan. After plan approval, produce an artifact-backed
`tjm-handoff` for another agent, naming the authorized phase explicitly. This agent
reviews the resulting implementation only when the user requests it. Commit, push,
installer build/execution and publication remain separately authorized actions.

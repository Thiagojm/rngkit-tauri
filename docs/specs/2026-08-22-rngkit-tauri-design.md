# RngKit Tauri Application Design

**Status:** Approved
**Date:** 2026-08-22
**Application root:** `D:\Projetos\rustie\rngkit-tauri`
**Library workspace:** `D:\Projetos\rustie\libs\rngkit-core`

## 1. Context and problem

RngKitPSG 3.0 is a Windows PySimpleGUI application that collects fixed-size
bit samples from BitBabbler, TrueRNG, or Python's system-backed PseudoRNG,
writes BIN and CSV files, renders a live cumulative Z chart, generates XLSX
analysis, opens the output folder, and concatenates legacy CSV files. Its code
also contains a combined TrueRNG + BitBabbler mode, silently chooses the first
TrueRNG port, duplicates collection logic between ordinary and live-plot
screens, and performs weak validation around file concatenation. Its README is
not authoritative where it differs from the code.

The replacement must retain familiar workflows and terminology without
copying the legacy architecture or widget layout. In particular, combined
live sources conflict with the accepted one-source-per-session contract, and
silent first-device selection is forbidden.

The reusable Rust layer already exists in the clean `rngkit-core` repository at
commit `fe30e5b`. It contains six crates:

- `rngkit-core` for validated domain contracts and `EntropySource`;
- `rngkit-sources` for discovery, explicit source configuration, adapters, and
  `open()`;
- `rngkit-analysis` for incremental and batch descriptive statistics;
- `rngkit-recording` for native sessions and read-only RngKitPSG v3 import;
- `rngkit-engine` for synchronous, caller-owned, cancellable collection;
- `rngkit-xlsx` for separate Excel report generation.

The library has deterministic Windows and Ubuntu CI evidence and serial
physical evidence for the available Windows BitBabbler, TrueRNG3, RDSEED, and
unified discovery checks. That evidence does not establish support for other
devices or operating systems.

The new application is a separate Tauri repository. Tauri owns desktop
lifecycle, worker threads, IPC, dialogs, frontend state, and presentation. The
existing crates remain responsible for source, collection, recording,
statistics, normalized reading, and XLSX behavior.

CSV concatenation is included in application v1, but the safe reusable
capability does not yet exist. The design therefore includes a prerequisite
addition to `rngkit-recording` and corresponding normalized XLSX support. The
legacy function that merely sorts filenames and copies rows is not reproduced.

## 2. Goals

1. Deliver a Windows-first RngKit desktop application using Tauri 2, Svelte 5,
   TypeScript, Vite, and Tailwind CSS 4.
2. Discover currently selectable entropy sources without blocking the UI and
   require explicit selection when multiple physical devices exist.
3. Configure and collect from exactly one BitBabbler, TrueRNG v1/v2/v3,
   RDSEED, or PseudoRNG source per session.
4. Display every committed cumulative descriptive Z point during collection
   while the complete BIN/CSV recording remains authoritative on disk.
5. Stop cooperatively, finalize the manifest, and handle application close
   without silently abandoning an active session.
6. Generate XLSX reports from native session directories, RngKitPSG v3 BIN or
   CSV files, and validated derived concatenation bundles.
7. Combine compatible legacy v3 CSV files into a provenance-bearing derived
   bundle without modifying the inputs.
8. Preserve library path-containment, no-follow, durability, no-overwrite, and
   statistical-interpretation guarantees at the desktop boundary.
9. Separate stable, safe frontend error messages from detailed redacted
   diagnostics.
10. Establish deterministic tests for Rust application logic, IPC contracts,
    state transitions, Svelte UI behavior, and long chart sessions without
    attached hardware.
11. Produce an unsigned, per-user English NSIS installer that can install
    without network access by bundling WebView2.
12. Create repository-native context files during implementation so future
    work can recover verified commands, decisions, and remaining work.

## 3. Non-goals

- Combined TrueRNG + BitBabbler collection or any multi-source session.
- Live XOR combination, source fallback, automatic reconnect, or session
  resume.
- TrueRNGpro, driver installation, USB permission changes, or device setup.
- RngKitPSG v2 import.
- Native-session merging or BIN merging.
- CSV concatenation without strict compatibility and provenance validation.
- Statistical significance, p-values, confidence intervals, entropy
  certification, causal conclusions, or pass/fail randomness decisions.
- Persisting or logging hardware serials, OS device paths, entropy bytes,
  PseudoRNG seeds, or generator state.
- Background tray collection.
- Linux installer or physical Linux support claims in v1.
- Code signing, publishing, remote creation, releases, auto-update, Microsoft
  Store submission, deployment, or distribution during implementation unless
  separately authorized.
- A general plugin ABI, server component, SSR, database, or cloud service.

## 4. Product scope and navigation

The product name is **RngKit**. The v1 interface is English-only, with
user-facing strings centralized for later localization.

The single resizable application window has four primary destinations:

1. **Collect** — source discovery and selection, session configuration,
   start/stop controls, exact current metrics, live cumulative Z, terminal
   summary, and session-folder access.
2. **Reports** — inspection and XLSX generation for native sessions, legacy v3
   BIN/CSV inputs, and derived concatenation bundles.
3. **Combine** — preview, validation, and creation of a derived bundle from
   compatible legacy v3 CSV inputs.
4. **Help** — source and fold terminology, collection semantics, file formats,
   reports, descriptive statistical limitations, troubleshooting codes, and
   application/library versions.

The collection and live-chart workflows are deliberately unified. There is no
second collection implementation or second start/stop surface.

## 5. Application architecture

### 5.1 Tauri coordinator

Tauri manages one `AppCoordinator` protected by a mutex. It owns:

- the authoritative application state;
- the active discovery generation and transient candidate registry;
- the selected opaque candidate token and validated session draft;
- the active session identifier, cancellation token, and worker handle;
- the next collection-event sequence;
- terminal session summary needed for state reconciliation;
- one mutually exclusive file-operation slot for report or concatenation work;
- safe preferences and a bounded in-memory diagnostic history.

The Tauri application is divided into focused modules for commands,
coordinator/state, DTOs, collection, reports, concatenation, preferences,
artifact opening, and safe errors. Command functions translate only between
IPC and application services; they do not reproduce domain logic.

Discovery and finite file jobs run in Tauri's blocking background context.
Collection runs on one application-owned Rust worker thread because
`rngkit-engine` is synchronous, caller-owned, and long-lived. The worker owns
the opened source, engine configuration, channel sink, and the engine call.
The coordinator retains the cancellation token and join handle.

The application does not add a direct Tokio dependency. Finite blocking work
is scheduled only through the runtime already exposed by Tauri.

### 5.2 Reusable crate boundary

The app calls:

- `rngkit_sources::discover()` in a blocking context;
- an application mapper that stores each `SourceCandidate` behind a transient
  opaque token;
- an explicit `SourceConfig` reconstructed in Rust from the selected token and
  source-specific options;
- `rngkit_sources::open()` when starting;
- `rngkit_engine::run_session()` on the worker thread;
- native/legacy/derived normalized readers for inspection;
- `rngkit_xlsx::write_report()` for reports.

The app never calculates popcounts, cumulative Z, recording offsets, session
names, manifest values, or XLSX contents itself.

## 6. State machine and concurrency

The required top-level collection states are:

```text
idle --discover--> discovering
discovering --results/no selection--> idle
idle --valid selection and config--> ready
ready --start accepted--> collecting
collecting --stop or close request--> stopping
collecting --terminal result--> completed | failed
stopping --terminal result--> completed | failed
completed | failed --new session--> ready
completed | failed --refresh--> discovering
```

State rules:

- Only `ready` accepts Start. The coordinator rejects every double-start even
  if frontend controls are stale.
- Discovery completion is accepted only for its current request generation.
  Refresh invalidates all previous candidate tokens and the previous
  selection.
- Discovery, reports, and concatenation cannot start while collecting or
  stopping.
- Only one report or concatenation operation may run at once.
- `stopping` is irreversible for the current session. Repeated Stop requests
  are idempotent and return the existing operation state.
- Cancellation during a successful blocking read commits that complete sample
  before stopping, matching the engine contract.
- A worker updates coordinator terminal state directly even if frontend
  delivery fails.
- Finished worker handles are joined and removed before a new session starts.
- Completed or failed state returns to `ready` only when the user chooses Start
  another session and the selected token's discovery generation is still
  current. Otherwise the app returns to idle or refreshes discovery. A later
  `open()` remains authoritative if the source disappeared.
- Report inspection rejects the currently active native bundle and any bundle
  whose live writer is owned by this process.

An orthogonal file-job state is `idle`, `inspecting`, `generating_report`, or
`combining`. It cannot overlap another file job and is rejected during
collection or stopping.

## 7. Discovery and selection

Each discovery call receives a monotonically increasing generation. The
application stores the non-serializable library candidates in a backend-only
map keyed by random opaque tokens. The frontend DTO contains only:

- the opaque token;
- stable source ID;
- safe family label;
- safe variant where available;
- a display ordinal for otherwise indistinguishable candidates;
- whether the source requires a fold control.

Serials and port paths never cross IPC. Multiple devices remain separate
choices through distinct tokens and ordinals. Discovery issues map to stable
application codes and safe per-family warnings; one issue never hides another
family's candidates.

The selected token is valid only for its generation and process lifetime. It
is never written to preferences. Start reconstructs an explicit
`SourceConfig`. BitBabbler includes the selected fold. RDSEED and PseudoRNG use
the library defaults in v1. Device disappearance between discovery and open is
a safe source error, not an automatic refresh, fallback, or reconnect.

## 8. Collection workflow and events

The user selects a source, positive byte-aligned sample size, integer interval
of at least one second, output root, and BitBabbler fold when applicable. Rust
constructors perform final validation before opening a source or creating
artifacts. A session has no predefined sample count and continues until the
user stops it or a terminal error occurs.

Start allocates a unique session ID, marks the coordinator collecting, moves
the channel and explicit source configuration into the worker, opens the
source, and calls the engine. Engine timing, durability, analysis, overrun, and
cancellation behavior remain unchanged.

A tagged `CollectionEventDto` contains a session ID, monotonic sequence, and
one of:

- session started with safe stem and display metadata;
- sample committed with durable sample metadata and exact cumulative analysis;
- timing overrun with cycle and interval durations;
- clean stop with committed and overrun counts;
- terminal failure with stable code and safe message.

No entropy bytes or source selectors are present. The frontend accepts an
event only when its session ID is current and its sequence is greater than the
last accepted sequence. `get_app_state` returns an authoritative snapshot for
initial load or reconciliation after a missed update.

Channel delivery failure is terminal under the engine event-sink contract. The
worker best-effort finalizes the failed manifest, updates coordinator state,
and records only a redacted diagnostic.

## 9. Frontend state and chart

The frontend is a client-only Svelte 5 + TypeScript + Vite SPA. A typed IPC
service is the sole caller of Tauri commands and channels. Pages consume one
application state module:

- `$state` holds received application and view state;
- `$derived` computes control availability, validation summaries, labels, and
  terminal presentation;
- `$effect` is restricted to imperative external synchronization such as
  channel cleanup, native dialog coordination, theme application, and the
  chart adapter.

Pages never infer backend authority from button state alone.

The live chart uses a small Svelte wrapper around uPlot. It retains every
accepted `(sample_index, cumulative_z)` pair for the active session in aligned
numeric arrays. Exact current totals remain in the latest engine snapshot and
are not calculated from plotted pixels. Entropy bytes never enter frontend
memory.

Chart behavior:

- use sample index on the x-axis and signed cumulative Z on the y-axis;
- show a zero line and dashed `Reference +1.96` and `Reference -1.96` lines
  without allocating three additional point arrays;
- label Z as descriptive and keep a visible non-inferential explanation;
- start fitted to the session extent;
- allow zoom, pan, Reset view, and Return to live;
- do not reset a user's zoom when new points arrive;
- append every point while coalescing redraw requests to at most one animation
  frame;
- retain points until a new session replaces the active view or the app exits.

Unbounded point retention is an explicit product decision. The implementation
must stress-test 100,000 and 1,000,000 synthetic points and record render,
steady append, memory, and interaction evidence on the Windows reference host.
It must not silently introduce downsampling or a recent-only window.

## 10. UI structure and styling

The default window is approximately 1280 by 800 logical pixels and has a
practical minimum size. A persistent navigation rail exposes Collect, Reports,
Combine, and Help. The top bar contains the product name, global operation
status, and light/dark/system theme control.

### 10.1 Collect

On a wide window, a narrower configuration column sits beside a larger
monitoring region. Narrow windows stack these areas.

The configuration column contains discovery refresh, candidate selection,
per-family warnings, sample size, interval, conditional fold, output root, and
the primary start/stop action. Familiar labels include `Sample size (bits)`,
`Sample interval (seconds)`, and `Fold`, with values `0 - Raw` through `4`.

The monitoring region contains a textual state banner, sample count, elapsed
time, observed one proportion, descriptive cumulative Z, overrun count, the
chart, the statistical limitation, and terminal actions. Completed or failed
views include Open session folder and Start another session.

### 10.2 Reports

The page accepts a native directory, legacy v3 BIN/CSV, or derived bundle.
Rust detects and validates the kind. Before generation, the page displays
origin, source, sample bits, interval, fold, status, row count, and warnings.

The report uses the library-defined same-stem destination. An existing XLSX
returns a conflict. The UI presents Cancel and Replace with Cancel as default;
only the explicit second request uses `Overwrite::Replace`. The library still
performs race-safe promotion. Success offers Open report and Open containing
folder.

### 10.3 Combine

The page accepts multiple legacy v3 CSV files. Preview shows basename, source,
sample bits, interval, fold, first/last timestamp, rows, and validation state.
The files appear in derived chronological order. Incompatibility prevents
creation and identifies the relevant inputs. Success shows bundle path, input
count, total rows, Open folder, and Generate XLSX.

### 10.4 Help and accessibility

Help is structured application documentation rather than an embedded legacy
README. Semantic controls, visible focus, keyboard navigation, associated
labels, restrained live-region announcements, reduced-motion support, Windows
scaling, and accessible contrast are required. State is always conveyed by
text and iconography as well as color. Disabled controls remain legible and
explain non-obvious restrictions.

### 10.5 Tailwind policy

Use the latest stable Tailwind CSS 4 available at scaffolding time through the
official Vite plugin. CSS-first `@theme` tokens define surfaces, text, borders,
focus, actions, status colors, chart colors, spacing, radii, typography, and
breakpoints. Repeated patterns become focused Svelte components. Ordinary
layout uses utilities; scoped CSS is limited to uPlot integration or behavior
that utilities cannot express cleanly.

No additional UI component system is included in v1. Native semantic controls
are preferred when they provide the best accessibility.

## 11. Reports

The application uses normalized readers and `rngkit-xlsx`; it does not parse
session data or build spreadsheets in Tauri or Svelte.

- A native report path is contained within its validated session directory.
- A legacy report is a same-stem sibling of the explicitly selected input.
- A derived report is contained within its validated derived directory.
- Actively recording native sessions are rejected.
- Existing outputs use the explicit Cancel/Replace flow.
- Excel row overflow, corrupt input, unsupported version, and filesystem
  failures map to safe typed application errors.

The report retains the accepted `Summary` and `Samples` sheets, descriptive Z
labeling, zero line, and visual `+1.96` and `-1.96` references. No report path
or UI copy turns those references into a significance decision.

## 12. Strict legacy CSV concatenation

### 12.1 Library ownership

`rngkit-recording` gains a reusable derived-concatenation API. The API provides
typed inspection, creation, manifest reading, consistency validation, and
normalized reading. `rngkit-xlsx` consumes the normalized result and does not
parse the derived CSV itself.

The library workspace records the new public contract in its decision and
context files when implementation is authorized. The app ultimately pins an
immutable Git revision containing that contract.

### 12.2 Compatibility

Inputs must be distinct, nonempty, readable RngKitPSG v3 CSV files with:

- the same source ID;
- the same sample size;
- the same interval;
- the same BitBabbler fold, when applicable;
- valid one-counts not exceeding sample bits;
- nondecreasing timestamps within each file.

Inputs are ordered by first row timestamp. Gaps between files are allowed.
Overlapping timestamp ranges, including equal boundary timestamps, are
rejected because their ordering and duplication semantics are ambiguous.
Selecting the same canonical file twice is rejected. Native CSV inputs,
legacy v2, mixed sources, and empty files are rejected.

Preview is advisory. Creation reopens every input, recomputes metadata and
SHA-256, and revalidates the complete set so changes after preview cannot be
silently accepted.

### 12.3 Derived format

The directory grammar is:

```text
YYYYMMDDTHHMMSS_concat_<source>_s<bits>_i<seconds>[_f<fold>]/
```

The timestamp is the local creation time of the derived artifact, not a claim
about the first input sample. The grammar is separate from `SessionStem` and
cannot be mistaken for a collected session.

The directory contains a same-stem CSV and `manifest.json`. The CSV schema is:

```text
sample_index,captured_at_utc,ones,input_index,input_sample_index
```

Output sample indexes are one-based and contiguous. Input and input-sample
indexes preserve provenance. Timestamps and one-counts are copied without
reinterpretation or recalculation.

The manifest is schema version 1 with artifact kind
`legacy_csv_concatenation`. It contains creation UTC time, local offset,
compatibility fields, CSV basename, total rows, and an ordered input list. Each
input entry contains only basename, SHA-256, row count, first/last timestamp,
and output row range. It never stores an absolute input path.

Creation streams inputs and does not retain the combined rows in memory. It
uses a unique contained staging directory, durably writes CSV and manifest,
then promotes the directory without replacing an existing final path. Failure
leaves inputs unchanged and no partial final bundle. Readers validate schema,
kind, exact contained same-stem CSV basename, contiguous rows, input ranges,
compatibility fields, and one-count bounds.

## 13. Paths, capabilities, privacy, and diagnostics

The frontend has only the minimal Tauri capabilities required for application
IPC and native dialogs. It receives no general filesystem, shell, opener, or
logging permission. User-selected paths return from native dialogs and are
opened in Rust. Open report/folder commands accept only backend-known artifact
identifiers, not arbitrary frontend paths.

Path rules:

- Rust revalidates every path supplied after a dialog.
- Native manifest names remain exact contained basenames and native artifact
  opens retain existing no-follow behavior.
- Native and derived report destinations stay inside validated directories.
- Derived output uses a canonical output root and validated generated names.
- Create and promotion operations use no-overwrite semantics unless report
  replacement was explicitly confirmed.
- Legacy inputs and concatenation inputs are opened read-only and never
  renamed, repaired, moved, or rewritten.

Frontend errors contain a stable application code, safe English message,
operation ID where applicable, and optional safe recovery action. Codes cover
invalid configuration, expired selection, unavailable/busy/disconnected/timed
out source, permission failure, existing output, corrupt or unsupported input,
operation conflict, and unexpected failure.

Rust transforms diagnostics instead of serializing arbitrary error chains.
Production retains a bounded in-memory diagnostic history. Copy diagnostics
produces an explicitly sanitized record with application/library versions,
operation ID, code, and redacted detail. Persistent logs are not enabled in
v1. Diagnostics never contain entropy, seeds, serials, device paths, or
absolute legacy-input paths.

## 14. Preferences

A versioned application-owned JSON file in the platform configuration
directory stores only:

- output root;
- sample bits;
- interval;
- BitBabbler fold;
- light, dark, or system theme;
- validated window position and size.

The backend writes preferences through a sibling temporary and atomic replace.
An invalid or unsupported preferences file is ignored with a safe warning and
defaults are used. Window geometry is clamped to a visible display after
monitor changes. Candidate tokens, selected source families, physical
selectors, entropy, seeds, and generator state are never stored.

## 15. Window and process lifecycle

Closing an idle, ready, completed, or failed window exits normally. Closing
while collecting presents two actions:

- **Stop and exit** requests cooperative cancellation, moves to stopping,
  waits for engine terminal finalization and worker completion, and exits only
  after completed or failed state;
- **Keep collecting** cancels the close request.

A second close request during stopping does not create another stop operation
or force termination. The UI shows that finalization is in progress. v1 has no
force-quit button because it would deliberately create an interrupted session.
Operating-system process termination may still leave a recording manifest;
the existing interrupted-session reader contract handles its committed prefix.

## 16. Repository and version policy

The application repository contains frontend source, `src-tauri`, tests,
capabilities, documentation, and lockfiles. During implementation it creates
and maintains:

- `AGENTS.md` for verified commands and repository rules;
- `docs/PROJECT_CONTEXT.md` for architecture and current evidence;
- `docs/DECISIONS.md` for durable accepted contracts;
- `TODO.md` for completed, in-progress, next, and backlog work.

The Rust app uses edition 2024 and MSRV 1.85, matching the library workspace.
At scaffolding time, implementation verifies and selects the latest mutually
compatible stable releases of Tauri 2, Svelte 5, Tailwind CSS 4, Vite, the
official integrations, and test tooling. Exact selected versions are committed
to `Cargo.lock` and `package-lock.json`; CI uses frozen/locked resolution.
Prereleases are excluded.

The implementation also records its Node.js runtime requirement and package
manager version. Dependency upgrades after initial selection require their own
validation rather than floating automatically.

The app's final `rngkit-*` dependencies use an exact reachable Git revision,
not local paths. The revision must include the approved concatenation contract
and pass the complete library validation suite before the app pins it. Commit
and push of that library change remain separate authorization boundaries.

## 17. Windows packaging

The supported v1 package target is Windows 10/11 x64 using
`x86_64-pc-windows-msvc`. Tauri is configured for an English per-user NSIS
installer. Administrator permission is not required for the default install.

The installer includes the WebView2 offline installer so setup does not depend
on network availability and the required modern web platform is present.
Installer validation covers install, first launch, native dialogs, a short
PseudoRNG collection, graceful close, uninstall, and confirmation that user
session data is not deleted during uninstall.

Installer signing, publication, updater configuration, and release automation
are not part of implementation authority and require later decisions and
approval.

## 18. Failure and edge cases

- Empty discovery shows no-source guidance and Refresh; it is not a fatal app
  failure.
- Per-family discovery failures remain warnings while other candidates stay
  selectable.
- Expired candidate tokens, stale discovery results, double-start, and stale
  events are rejected deterministically.
- Device disappearance before open or during read produces a terminal safe
  source error without reconnect or fallback.
- Invalid configuration and unavailable output roots fail before source open
  or session artifact creation where the library contract permits.
- Session-name collisions, report conflicts, and derived-bundle collisions do
  not invent alternate names or overwrite implicitly.
- Repeated Stop and repeated close during stopping remain idempotent.
- Event-channel failure does not leave backend state falsely collecting.
- Completed-manifest failure retains the primary error and best-effort failed
  finalization behavior from the engine.
- Native interrupted sessions remain inspectable when their committed prefix
  is consistent.
- Corrupt native, legacy, or derived inputs fail safely without repair.
- Report row overflow leaves no partial final workbook.
- A report destination created concurrently is not replaced under
  `ErrorIfExists`.
- Concatenation input changes between preview and create are detected by full
  revalidation and hash recomputation.
- Preference corruption affects only restored UI defaults, never session data.
- A chart stress failure is reported as a validation failure; implementation
  may not silently change the approved retention policy.

## 19. Validation strategy

### 19.1 Library deterministic validation

Add focused tests for:

- derived-name generation and parsing, including fold rules and path tokens;
- manifest JSON round trip and schema/kind rejection;
- same-stem contained CSV validation;
- streaming creation from known legacy fixtures;
- exact output rows and input/output provenance ranges;
- source, bit, interval, fold, empty, malformed, duplicate, overlap, and
  chronology rejection;
- one-count bounds and checked row/index arithmetic;
- changed-input detection after preview;
- SHA-256 evidence and absence of absolute input paths;
- staging cleanup, promotion collision, no-overwrite race, and failed writes;
- normalized derived reading and XLSX report contents;
- source inputs remaining byte-for-byte unchanged.

Run the library's verified format, metadata, workspace check/test, Clippy,
doctest, Rust 1.85, feature-matrix, tree, and diff checks. Default tests must
not enumerate or open hardware.

### 19.2 Tauri backend deterministic validation

Inject fake source, discovery, clock, channel, dialog, and filesystem services.
Test every state transition and prohibited transition, including:

- empty and partial discovery;
- generation invalidation and opaque-token lookup;
- safe mapping to every `SourceConfig` variant;
- double-start, repeated-stop, stop during read, stale event, and close races;
- worker success, source failure, recording failure, channel failure, and
  completed-manifest failure;
- report/concatenation mutual exclusion and active-session rejection;
- report Cancel/Replace behavior and concurrent destination creation;
- safe error/diagnostic redaction;
- preference atomicity, schema fallback, and geometry clamping;
- serialized snapshots containing no selector, seed, entropy, or arbitrary
  diagnostic chain.

### 19.3 Frontend deterministic validation

Use current Svelte testing guidance and browser-level tests with mocked typed
IPC. Cover:

- state reconciliation and sequence/session filtering;
- enabled and disabled controls in every state;
- Collect, Reports, Combine, and Help success and failure flows;
- validation preview, report conflict, close confirmation, and stopping UI;
- keyboard navigation, focus restoration, semantic labels, contrast, and
  restrained live announcements;
- light/dark/system themes and responsive layouts;
- chart references, descriptive labels, zoom persistence, Reset view, Return
  to live, and complete point retention.

Long-session tests construct 100,000 and 1,000,000 aligned points and record
initial render, steady one-point append, viewport interaction, and memory on a
reference Windows host. The test report distinguishes measured evidence from
library marketing claims.

### 19.4 Evidence tiers

Validation claims remain separated:

1. deterministic Rust/frontend checks without hardware;
2. Windows and Ubuntu CI compilation/tests with no hardware enumeration;
3. native Windows desktop smoke with mocked IPC plus a separately identified
   real PseudoRNG session;
4. ignored, opt-in, serial physical validation for the available BitBabbler,
   TrueRNG3, RDSEED, and unified discovery paths;
5. unsigned Windows NSIS install/launch/smoke/uninstall validation.

Physical-device tests remain explicit and serial. They may skip only absent
devices, never permission, busy, protocol, timeout, or USB/serial failures.
Windows evidence is not generalized to other devices or operating systems.

### 19.5 Staged implementation and user checkpoints

The implementation plan must be executable as a sequence of independently
reviewable checkpoints. An implementer is authorized to perform only the
current checkpoint, not later checkpoints in the same batch. At the end of
each checkpoint, the implementer must stop and provide:

- the completed scope and exact files changed;
- automated commands run and their observed results;
- any validation that was not run and why;
- concise manual test instructions for the user;
- the current runnable entry point or artifact;
- known limitations that belong to later checkpoints;
- a request for explicit approval before continuing.

Each application checkpoint must leave the current slice runnable and
testable. Library-only checkpoints must leave the complete library workspace
deterministically green. A failing checkpoint is repaired or explicitly
replanned before later work begins.

The implementation plan must use this progression:

1. implement and validate the reusable strict-concatenation prerequisite in
   `rngkit-core`;
2. scaffold the locked Tauri/Svelte/Tailwind shell and repository context;
3. deliver the responsive application shell and mocked state-machine UI;
4. connect real discovery and explicit transient selection;
5. complete a real PseudoRNG collection vertical slice with native recording;
6. add full live-chart retention, stop, terminal, and close behavior;
7. validate physical source integration through separately identified serial
   checkpoints;
8. add native and legacy report inspection/generation;
9. add strict Combine preview, bundle creation, derived reading, and XLSX;
10. complete security, diagnostics, preferences, accessibility, and long-session
    hardening;
11. complete deterministic CI, native Windows smoke, and unsigned NSIS
    installer validation.

The exact implementation plan may split these checkpoints further when a
smaller runnable slice improves reviewability. It must not merge them into one
large implementation pass. A material contract change pauses implementation,
returns to design review, and requires renewed approval.

Commit, push, remote, release, signing, publication, and deployment authority
remain separate from checkpoint approval unless the user explicitly grants
them.

## 20. Acceptance criteria

1. RngKit is a separate Tauri repository at the approved location and uses a
   client-only Svelte 5 + TypeScript + Vite frontend styled with Tailwind CSS 4.
2. The app pins exact stable dependency versions and an exact reachable
   `rngkit-core` revision containing the concatenation API.
3. Discovery runs off the UI thread, preserves per-family partial success, and
   shows every physical device as a separate opaque candidate.
4. No serial or OS path crosses IPC, and no selector, entropy, seed, or PRNG
   state is persisted or logged.
5. Only ready state can start one worker; double-start, stale discovery, stale
   events, and concurrent file jobs are rejected by Rust authority.
6. One session selects exactly one source and preserves the accepted engine
   timing, durability, cancellation, overrun, and terminal-error contracts.
7. The frontend renders every committed cumulative Z point and exact current
   statistics without receiving raw entropy.
8. The live chart labels Z as descriptive, renders zero and `+/-1.96` only as
   visual references, and presents no inferential conclusion.
9. Closing during collection offers Stop and exit or Keep collecting; Stop and
   exit waits for terminal finalization before process exit.
10. Reports inspect and generate XLSX for validated native, legacy v3, and
    derived inputs while preserving contained paths and explicit replacement.
11. Strict concatenation rejects incompatible or ambiguous inputs and creates
    a complete no-overwrite CSV/manifest bundle with hashes and no absolute
    input paths.
12. Source files remain unchanged after report or concatenation success and
    failure.
13. Safe frontend errors are stable application DTOs; detailed copied
    diagnostics are explicit, bounded, and redacted.
14. Preferences contain only the approved safe fields and recover safely from
    corruption.
15. Deterministic Rust and frontend tests run without hardware and cover the
    complete state machine, IPC filtering, file conflicts, and path safety.
16. Chart stress evidence is recorded at 100,000 and 1,000,000 retained points
    without silently changing retention semantics.
17. Windows and Ubuntu CI evidence is reported separately from Windows native,
    physical-device, and installer evidence.
18. A per-user English NSIS installer works offline, preserves user session
    data on uninstall, and is not signed or published without separate
    authorization.
19. Repository context files document verified commands, durable decisions,
    current evidence, and remaining work.
20. The implementation plan enforces one user-testable checkpoint at a time,
    and the implementer stops for explicit approval with evidence and manual
    test instructions before starting the next checkpoint.

## 21. Alternatives considered

### Modernized legacy tabs

Rejected because separate collection and live-plot screens duplicate
configuration, start/stop controls, and state synchronization.

### Guided collection wizard

Rejected because repeated desktop collection benefits from a persistent
instrument workspace; a wizard adds friction without improving domain safety.

### Global Tauri events

Rejected for the collection stream because a per-session channel provides
ordered, scoped delivery. Global events remain unnecessary.

### Poll-only frontend

Rejected because it adds latency and repeated snapshot traffic for a naturally
streamed one-sample-at-a-time workflow.

### Bounded or recent-only chart history

Rejected by user decision. Every statistical point is retained for the active
session despite growing frontend memory and render cost.

### Additional Svelte component framework

Rejected for v1. Tailwind tokens plus focused semantic components are
sufficient and avoid overlapping design systems.

### Simple flat CSV concatenation

Rejected because it cannot preserve durable provenance or distinguish derived
data safely from a collected session.

### Flat CSV plus sidecar

Rejected because the files can be separated or mismatched. A contained bundle
has one validation and path boundary.

### Combined XLSX without a derived CSV

Rejected because the selected workflow explicitly requires a reusable
concatenated dataset as well as report generation.

### MSI or portable-executable-only delivery

Rejected for v1. NSIS gives a familiar per-user installer with fewer initial
environment requirements than MSI and better setup behavior than a bare
executable.

## 22. Decisions and assumptions

- Product name: RngKit.
- Repository: `D:\Projetos\rustie\rngkit-tauri`.
- Platform: Windows-first, Windows 10/11 x64 v1; portable architecture and
  Ubuntu CI do not constitute Linux desktop support.
- Frontend: latest stable Svelte 5, TypeScript, Vite, and Tailwind CSS 4 at
  scaffolding time, with prereleases excluded and exact versions locked.
- UI: modernized familiarity, unified session workspace, English-only v1.
- Main navigation: Collect, Reports, Combine, Help.
- v1 includes native collection, live descriptive Z, native/legacy/derived
  reports, strict legacy v3 CSV concatenation, and in-app help.
- Every chart point is retained for the active session.
- Close during collection confirms and then cooperatively stops before exit.
- Only safe preferences are persisted; fresh discovery and selection are
  required after launch.
- Existing XLSX requires explicit replacement confirmation.
- Strict concatenation produces a CSV/manifest derived bundle and requires a
  reusable library extension before the app pins its library revision.
- The app uses a Rust-owned coordinator, one collection worker, typed commands,
  and a per-session channel with session/sequence filtering.
- Tauri and library logic remain thinly separated; the frontend never becomes
  authoritative for collection state or filesystem safety.
- v1 packaging is an unsigned per-user English NSIS installer with offline
  WebView2; signing, publishing, release, and updater work are separate.
- Implementation is delivered as independently runnable checkpoints with a
  user approval gate after each checkpoint; later checkpoints are not batched
  into the current one.

## 23. Current primary references

- Tauri frontend configuration: <https://v2.tauri.app/start/frontend/>
- Tauri calling Rust: <https://v2.tauri.app/develop/calling-rust/>
- Tauri calling the frontend and channels:
  <https://v2.tauri.app/develop/calling-frontend/>
- Tauri Windows installers:
  <https://v2.tauri.app/distribute/windows-installer/>
- Tauri Windows signing: <https://v2.tauri.app/distribute/sign/windows/>
- Svelte documentation: <https://svelte.dev/docs/svelte/overview>
- Svelte `$state`: <https://svelte.dev/docs/svelte/$state>
- Svelte `$derived`: <https://svelte.dev/docs/svelte/$derived>
- Svelte `$effect`: <https://svelte.dev/docs/svelte/$effect>
- Tailwind Vite installation: <https://tailwindcss.com/docs/installation>
- Tailwind compatibility: <https://tailwindcss.com/docs/compatibility>
- uPlot documentation: <https://github.com/leeoniya/uPlot/blob/master/docs/README.md>
- Approved library design:
  `D:\Projetos\rustie\libs\rngkit-core\docs\specs\2026-08-21-rngkit-core-design.md`
- Approved discovery design:
  `D:\Projetos\rustie\libs\rngkit-core\docs\specs\2026-08-21-source-discovery-design.md`

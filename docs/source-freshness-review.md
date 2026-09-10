# Source freshness review — 2026-09-10

## Scope and evidence

Reviewed the exact integrated revisions: bitb-rs `18e586e5e2cf3e14742d1cd86f593a8ad1adbe3d`,
intel_seed `182740952a44304302420a5589f134964d5e792a`, and rngkit-core `56c0c84`.
Their local working trees were clean. No device acquisition or product-code edits
were performed. TrueRNG acceptance was reported by the user separately.

`cargo test --locked --lib` passed: BitBabbler 57 tests; RDSEED 29 tests.
Three additional characterization tests in an isolated temporary BitBabbler copy
confirmed the error cases below using the existing USB mock. These establish
software behavior under scripted responses, not occurrence on physical hardware.

## Normal acquisition

- Core adapters directly call `get_bits_with_fold` / `get_bits`; neither caches
  entropy. The engine acquires, commits, then waits for the remaining interval.
  A source error terminates the session before committing that failed sample
  (`rngkit-engine/src/runner.rs:141`).
- BitBabbler sends a new MPSSE byte-in command for the exact requested length
  before reading each chunk (`src/protocol.rs:257`). Successful completion rejects
  excess payload and requires the final line status. The temporary USB chunk is
  consumed, not retained for future samples. A serial-style per-sample purge is
  not justified for this normal command/response flow.
- RDSEED stores only retry policy (`src/rdseed.rs:33`). Each acquisition invokes
  the CPU instruction, checks success, bounds retries, and discards unused bytes
  from the final word. Failure returns no partial sample and uses no fallback.
  No software freshness correction was identified.
- Fresh acquisition means requesting data during the call. Neither interface
  establishes the physical generation timestamp of every bit. RDSEED exposes
  no serial-style flush; discarding an arbitrary number of words would not
  establish a generation-time guarantee.

## Findings and proposed corrections

1. **BitBabbler: stale reply after reuse following a failed read (P2).**
   `read_exact_raw_into` propagates transfer errors immediately; outer cleanup
   clears only the Rust chunk. The next call sends another command without
   invalidating or resynchronizing the transport. A late reply from the failed
   command can therefore satisfy the next request. Reproduction: request two
   bytes, inject a read timeout, queue the old two-byte reply then a new reply,
   call again; the old reply is returned successfully. This affects direct
   consumers that retry the same handle; RngKit exits on the first source error.
   Prefer marking the handle unusable after an acquisition I/O/protocol error
   and requiring reopen. Invalid arguments should not invalidate the handle.
   If transparent recovery is required instead, reset/synchronize before reuse.

2. **BitBabbler: synchronization and purge lack a total bound (P2).**
   `check_sync` (`src/protocol.rs:185`) and `purge_read` (`:520`) reset their empty
   counter whenever payload arrives. Continuous unexpected data can prevent
   termination despite per-transfer timeouts and outer initialization retries.
   Purge is also called by Drop. Characterization tests fed 1,000 unexpected
   packets: sync read all 1,000 before accepting an echo; purge read all 1,000
   before its ten empty reads. With an endless stream neither loop terminates.
   Add a total deadline or transfer budget that payload cannot reset; use a
   bounded best-effort cleanup in Drop. Test the exhausted bound explicitly.

## Validation boundary and next action

Implement the two BitBabbler corrections only after approval, with regressions
for reuse after timeout and continuous unexpected payload. Then integrate exact
revisions through core/app and repeat native acquisition, folds and disconnect
acceptance. No RDSEED product change is currently justified. Its standalone
`tests/hardware.rs` is not ignored: use `--lib` for deterministic-only runs.

Protocol references: [FTDI MPSSE command definitions](https://www.ftdichip.com/Support/Documents/AppNotes/AN2232C-01_MPSSE_Cmnd.pdf)
(byte-in length and Send Immediate), and [Intel DRNG implementation guide](https://www.intel.com/content/www/us/en/developer/articles/guide/intel-digital-random-number-generator-drng-software-implementation-guide.html)
(RDSEED success indication, entropy availability and retry behavior).

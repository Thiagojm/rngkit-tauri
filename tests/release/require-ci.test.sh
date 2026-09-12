#!/usr/bin/env bash
# Isolated tests for scripts/release/require-ci.py. No live GitHub API.
set -eu

ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd -P)
SCRIPT=$ROOT/scripts/release/require-ci.py
PYTHON=$(command -v python3 || command -v python)
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
FAILS=0
SHA_A=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
SHA_B=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb

pass() { printf 'ok %s\n' "$1"; }
fail_test() {
  printf 'not ok %s\n' "$1" >&2
  printf '  %s\n' "$2" >&2
  FAILS=$((FAILS + 1))
}

cat >"$WORK/gh-api" <<EOF
#!/usr/bin/env bash
set -eu
path=\$1
case "\$path" in
  *"/actions/workflows/ci.yml/runs?"*)
    cat "\$RNGKIT_CI_FIXTURE/runs.json"
    ;;
  *"/actions/runs/"*"/jobs")
    cat "\$RNGKIT_CI_FIXTURE/jobs.json"
    ;;
  *)
    printf 'unexpected API path %s\n' "\$path" >&2
    exit 1
    ;;
esac
EOF
chmod +x "$WORK/gh-api"

write_json() {
  local dest=$1
  mkdir -p "$(dirname "$dest")"
  cat >"$dest"
}

run_ci() {
  RNGKIT_GH_API=$WORK/gh-api RNGKIT_CI_FIXTURE=$1 \
    "$PYTHON" "$SCRIPT" --sha "$2" --repo Thiagojm/rngkit-tauri
}

expect_ok() {
  local name=$1
  local fixture=$2
  local sha=$3
  local out
  if out=$(run_ci "$fixture" "$sha" 2>"$WORK/err"); then
    pass "$name"
    printf '%s\n' "$out" >"$WORK/out"
  else
    fail_test "$name" "expected success, got $(cat "$WORK/err")"
  fi
}

expect_fail() {
  local name=$1
  local needle=$2
  local fixture=$3
  local sha=$4
  local err
  if err=$(run_ci "$fixture" "$sha" 2>&1); then
    fail_test "$name" "expected failure, got: $err"
    return
  fi
  case "$err" in
    *"$needle"*) pass "$name" ;;
    *) fail_test "$name" "expected '$needle' in: $err" ;;
  esac
}

write_json "$WORK/success/runs.json" <<EOF
{"workflow_runs":[{"id":11,"head_sha":"$SHA_A","status":"completed","conclusion":"success","created_at":"2026-09-11T18:55:33Z","html_url":"https://example.test/11"}]}
EOF
write_json "$WORK/success/jobs.json" <<EOF
{"jobs":[{"name":"windows-latest","status":"completed","conclusion":"success"},{"name":"ubuntu-22.04","status":"completed","conclusion":"success"}]}
EOF
expect_ok "matching SHA success" "$WORK/success" "$SHA_A"
case "$(cat "$WORK/out")" in
  *"CI passed for commit $SHA_A"*) pass "success message names SHA" ;;
  *) fail_test "success message names SHA" "got $(cat "$WORK/out")" ;;
esac

write_json "$WORK/absent/runs.json" <<'EOF'
{"workflow_runs":[]}
EOF
write_json "$WORK/absent/jobs.json" <<'EOF'
{"jobs":[]}
EOF
expect_fail "absent CI" "CI absent for commit" "$WORK/absent" "$SHA_A"

write_json "$WORK/pending/runs.json" <<EOF
{"workflow_runs":[{"id":12,"head_sha":"$SHA_A","status":"in_progress","conclusion":null,"created_at":"2026-09-11T18:55:33Z","html_url":"https://example.test/12"}]}
EOF
write_json "$WORK/pending/jobs.json" <<'EOF'
{"jobs":[]}
EOF
expect_fail "pending CI" "CI pending for commit" "$WORK/pending" "$SHA_A"

write_json "$WORK/failed/runs.json" <<EOF
{"workflow_runs":[{"id":13,"head_sha":"$SHA_A","status":"completed","conclusion":"failure","created_at":"2026-09-11T18:55:33Z","html_url":"https://example.test/13"}]}
EOF
write_json "$WORK/failed/jobs.json" <<'EOF'
{"jobs":[]}
EOF
expect_fail "failed CI" "CI failed for commit" "$WORK/failed" "$SHA_A"

write_json "$WORK/other/runs.json" <<EOF
{"workflow_runs":[{"id":14,"head_sha":"$SHA_B","status":"completed","conclusion":"success","created_at":"2026-09-11T18:55:33Z","html_url":"https://example.test/14"}]}
EOF
write_json "$WORK/other/jobs.json" <<EOF
{"jobs":[{"name":"windows-latest","status":"completed","conclusion":"success"},{"name":"ubuntu-22.04","status":"completed","conclusion":"success"}]}
EOF
expect_fail "CI for a different SHA" "different commit" "$WORK/other" "$SHA_A"

write_json "$WORK/latest-fail/runs.json" <<EOF
{"workflow_runs":[
  {"id":16,"head_sha":"$SHA_A","status":"completed","conclusion":"failure","created_at":"2026-09-12T00:00:00Z","html_url":"https://example.test/16"},
  {"id":15,"head_sha":"$SHA_A","status":"completed","conclusion":"success","created_at":"2026-09-11T00:00:00Z","html_url":"https://example.test/15"}
]}
EOF
write_json "$WORK/latest-fail/jobs.json" <<EOF
{"jobs":[{"name":"windows-latest","status":"completed","conclusion":"success"},{"name":"ubuntu-22.04","status":"completed","conclusion":"success"}]}
EOF
expect_fail "latest SHA run failed" "CI failed for commit" "$WORK/latest-fail" "$SHA_A"

write_json "$WORK/job-fail/runs.json" <<EOF
{"workflow_runs":[{"id":17,"head_sha":"$SHA_A","status":"completed","conclusion":"success","created_at":"2026-09-11T18:55:33Z","html_url":"https://example.test/17"}]}
EOF
write_json "$WORK/job-fail/jobs.json" <<EOF
{"jobs":[{"name":"windows-latest","status":"completed","conclusion":"success"},{"name":"ubuntu-22.04","status":"completed","conclusion":"failure"}]}
EOF
expect_fail "Ubuntu job failed" "required jobs did not succeed" "$WORK/job-fail" "$SHA_A"

write_json "$WORK/job-missing/runs.json" <<EOF
{"workflow_runs":[{"id":18,"head_sha":"$SHA_A","status":"completed","conclusion":"success","created_at":"2026-09-11T18:55:33Z","html_url":"https://example.test/18"}]}
EOF
write_json "$WORK/job-missing/jobs.json" <<EOF
{"jobs":[{"name":"windows-latest","status":"completed","conclusion":"success"}]}
EOF
expect_fail "missing Ubuntu job" "missing required jobs" "$WORK/job-missing" "$SHA_A"

if [ "$FAILS" -ne 0 ]; then
  printf '%s test(s) failed\n' "$FAILS" >&2
  exit 1
fi
printf 'all tests passed\n'

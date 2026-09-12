#!/usr/bin/env bash
# Isolated tests for scripts/release/validate-version.py.
set -eu

ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd -P)
SCRIPT=$ROOT/scripts/release/validate-version.py
PYTHON=$(command -v python3 || command -v python)
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
FAILS=0

pass() { printf 'ok %s\n' "$1"; }
fail_test() {
  printf 'not ok %s\n' "$1" >&2
  printf '  %s\n' "$2" >&2
  FAILS=$((FAILS + 1))
}

write_tree() {
  local dest=$1
  local version=$2
  local lock_pkg=${3-$version}
  local lock_root=${4-$version}
  local cargo_lock=${5-$version}
  mkdir -p "$dest/src-tauri"
  cat >"$dest/package.json" <<EOF
{"name":"rngkit","version":"$version"}
EOF
  cat >"$dest/package-lock.json" <<EOF
{"name":"rngkit","version":"$lock_pkg","lockfileVersion":3,"packages":{"":{"name":"rngkit","version":"$lock_root"}}}
EOF
  cat >"$dest/src-tauri/tauri.conf.json" <<EOF
{"productName":"RngKit","version":"$version"}
EOF
  cat >"$dest/src-tauri/Cargo.toml" <<EOF
[package]
name = "rngkit"
version = "$version"
EOF
  cat >"$dest/src-tauri/Cargo.lock" <<EOF
[[package]]
name = "rngkit"
version = "$cargo_lock"
EOF
}

expect_ok() {
  local name=$1
  shift
  local out
  if out=$("$PYTHON" "$SCRIPT" "$@" 2>"$WORK/err"); then
    pass "$name"
    printf '%s\n' "$out" >"$WORK/out"
  else
    fail_test "$name" "expected success, got $(cat "$WORK/err")"
  fi
}

expect_fail() {
  local name=$1
  local needle=$2
  shift 2
  local err
  if err=$("$PYTHON" "$SCRIPT" "$@" 2>&1); then
    fail_test "$name" "expected failure, got: $err"
    return
  fi
  case "$err" in
    *"$needle"*) pass "$name" ;;
    *) fail_test "$name" "expected '$needle' in: $err" ;;
  esac
}

expect_ok "repo versions agree" --root "$ROOT"
REPO_VERSION=$("$PYTHON" -c 'import json, sys; print(json.load(open(sys.argv[1], encoding="utf-8"))["version"])' "$ROOT/package.json")
expect_ok "matching repo tag" --root "$ROOT" --tag "v$REPO_VERSION"

write_tree "$WORK/ok" "0.1.0"
expect_ok "fixture agrees without tag" --root "$WORK/ok"
case "$(cat "$WORK/out")" in
  *'version=0.1.0'*'prerelease=false'*) pass "stable fixture output" ;;
  *) fail_test "stable fixture output" "got $(cat "$WORK/out")" ;;
esac
expect_ok "matching tag" --root "$WORK/ok" --tag v0.1.0
expect_fail "divergent tag" "does not match package version 0.1.0" --root "$WORK/ok" --tag v0.2.0
expect_fail "invalid tag" "invalid tag" --root "$WORK/ok" --tag v1
expect_fail "prerelease tag against stable files" "does not match package version 0.1.0" --root "$WORK/ok" --tag v0.1.0-rc.1

write_tree "$WORK/rc" "0.1.0-rc.1"
expect_ok "prerelease files without tag" --root "$WORK/rc"
case "$(cat "$WORK/out")" in
  *'version=0.1.0-rc.1'*'prerelease=true'*) pass "prerelease output true" ;;
  *) fail_test "prerelease output true" "got $(cat "$WORK/out")" ;;
esac
expect_ok "matching prerelease tag" --root "$WORK/rc" --tag v0.1.0-rc.1
expect_fail "stable tag against prerelease files" "does not match package version 0.1.0-rc.1" --root "$WORK/rc" --tag v0.1.0

write_tree "$WORK/tauri" "0.1.0"
TREE=$WORK/tauri "$PYTHON" - <<'PY'
from pathlib import Path
import os
p = Path(os.environ["TREE"]) / "src-tauri" / "tauri.conf.json"
p.write_text('{"version":"0.1.1"}\n', encoding="utf-8")
PY
expect_fail "tauri.conf divergence" "version mismatch" --root "$WORK/tauri"

write_tree "$WORK/lockpkg" "0.1.0" "0.1.1" "0.1.0" "0.1.0"
expect_fail "package-lock top-level divergence" "version mismatch" --root "$WORK/lockpkg"

write_tree "$WORK/lockroot" "0.1.0" "0.1.0" "0.1.1" "0.1.0"
expect_fail "package-lock packages root divergence" "version mismatch" --root "$WORK/lockroot"

write_tree "$WORK/cargolock" "0.1.0" "0.1.0" "0.1.0" "0.1.1"
expect_fail "Cargo.lock divergence" "version mismatch" --root "$WORK/cargolock"

if [ "$FAILS" -ne 0 ]; then
  printf '%s test(s) failed\n' "$FAILS" >&2
  exit 1
fi
printf 'all tests passed\n'

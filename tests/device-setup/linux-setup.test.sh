#!/usr/bin/env bash
# Isolated, unprivileged tests for setup-rng-devices.sh.
# Never touches host /etc, groups, or kernel modules.
set -eu

HELPER=$(CDPATH='' cd -- "$(dirname -- "$0")/../../src-tauri/resources/device-setup/linux" && pwd -P)/setup-rng-devices.sh
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
FAILS=0

pass() { printf 'ok %s\n' "$1"; }
fail_test() {
  printf 'not ok %s\n' "$1" >&2
  printf '  %s\n' "$2" >&2
  FAILS=$((FAILS + 1))
}

write_stubs() {
  local stubs=$1
  mkdir -p "$stubs"
  cat >"$stubs/getent" <<'EOF'
#!/usr/bin/env bash
set -eu
root=${RNGKIT_SETUP_TEST_ROOT:?}
case "${1-}" in
  passwd)
    grep "^${2-}:" "$root/etc/passwd" && exit 0
    exit 2
    ;;
  group)
    grep "^${2-}:" "$root/etc/group" && exit 0
    exit 2
    ;;
  *)
    exit 1
    ;;
esac
EOF
  cat >"$stubs/groupadd" <<'EOF'
#!/usr/bin/env bash
set -eu
root=${RNGKIT_SETUP_TEST_ROOT:?}
name=${!#}
mkdir -p "$root/var/log"
printf 'groupadd %s\n' "$*" >>"$root/var/log/commands"
if grep -q "^${name}:" "$root/etc/group"; then
  printf 'group already exists\n' >&2
  exit 1
fi
printf '%s:x:142:\n' "$name" >>"$root/etc/group"
EOF
  cat >"$stubs/usermod" <<'EOF'
#!/usr/bin/env bash
set -eu
root=${RNGKIT_SETUP_TEST_ROOT:?}
mkdir -p "$root/var/log"
printf 'usermod %s\n' "$*" >>"$root/var/log/commands"
user=
group=
while [ $# -gt 0 ]; do
  case "$1" in
    -aG)
      group=$2
      shift 2
      ;;
    *)
      user=$1
      shift
      ;;
  esac
done
ent=$(grep "^${group}:" "$root/etc/group") || exit 1
members=${ent##*:}
case ",$members," in
  *",$user,"*) exit 0 ;;
esac
if [ -n "$members" ]; then
  sed -i "s/^${group}:.*/${group}:x:142:${members},${user}/" "$root/etc/group"
else
  sed -i "s/^${group}:.*/${group}:x:142:${user}/" "$root/etc/group"
fi
EOF
  cat >"$stubs/udevadm" <<'EOF'
#!/usr/bin/env bash
set -eu
root=${RNGKIT_SETUP_TEST_ROOT:?}
mkdir -p "$root/var/log"
printf 'udevadm %s\n' "$*" >>"$root/var/log/commands"
if [ -f "$root/fail-udevadm" ]; then
  exit 1
fi
if [ "${1-}" = control ] && [ "${2-}" = --reload-rules ]; then
  exit 0
fi
printf 'unexpected udevadm invocation: %s\n' "$*" >&2
exit 1
EOF
  chmod +x "$stubs/getent" "$stubs/groupadd" "$stubs/usermod" "$stubs/udevadm"
}

prepare_root() {
  local root=$1
  mkdir -p "$root/etc/udev/rules.d" "$root/run/systemd/system" "$root/usr/lib" "$root/var/log"
  printf 'ID=ubuntu\nID_LIKE=debian\n' >"$root/etc/os-release"
  printf 'alice:x:1000:1000::/home/alice:/bin/bash\nroot:x:0:0:root:/root:/bin/bash\n' >"$root/etc/passwd"
  : >"$root/etc/group"
  : >"$root/usr/lib/libusb-1.0.so.0"
}

run_helper() {
  local root=$1
  shift
  PATH="$root/stubs:$PATH" RNGKIT_SETUP_TEST_ROOT="$root" bash "$HELPER" "$@"
}

expect_ok() {
  local name=$1
  local root=$2
  shift 2
  local out
  if out=$(run_helper "$root" "$@" 2>&1); then
    pass "$name"
    printf '%s\n' "$out"
  else
    fail_test "$name" "$out"
  fi
}

expect_fail() {
  local name=$1
  local needle=$2
  local root=$3
  shift 3
  local out rc
  rc=0
  out=$(run_helper "$root" "$@" 2>&1) || rc=$?
  if [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -q "$needle"; then
    pass "$name"
  else
    fail_test "$name" "rc=$rc out=$out"
  fi
}

# --help does not require device/user and writes nothing.
HELP_OUT=$(bash "$HELPER" --help)
if printf '%s' "$HELP_OUT" | grep -q 'setup-rng-devices.sh --check'; then
  pass 'help text'
else
  fail_test 'help text' "$HELP_OUT"
fi

ROOT=$WORK/missing
prepare_root "$ROOT"
write_stubs "$ROOT/stubs"
expect_fail 'missing args' 'require --device' "$ROOT"
expect_fail 'unknown option' 'unknown option' "$ROOT" --device bitbabbler --user alice --force
expect_fail 'unknown device' 'unknown --device' "$ROOT" --device ftdi --user alice
expect_fail 'invalid user' 'invalid user name' "$ROOT" --check --device bitbabbler --user 'alice;rm'
expect_fail 'missing user' 'user not found' "$ROOT" --check --device bitbabbler --user bob
expect_fail 'root user' 'refusing to configure the root user' "$ROOT" --check --device bitbabbler --user root

# --check must not mutate.
expect_ok 'check bitbabbler' "$ROOT" --check --device bitbabbler --user alice
if [ -s "$ROOT/var/log/commands" ]; then
  fail_test 'check is read-only' "commands were logged: $(cat "$ROOT/var/log/commands")"
elif [ -e "$ROOT/etc/udev/rules.d/60-rngkit-bitbabbler.rules" ]; then
  fail_test 'check is read-only' 'rule file created'
else
  pass 'check is read-only'
fi

# Apply bitbabbler only.
expect_ok 'apply bitbabbler' "$ROOT" --device bitbabbler --user alice
if [ -f "$ROOT/etc/udev/rules.d/60-rngkit-bitbabbler.rules" ] &&
  [ ! -e "$ROOT/etc/udev/rules.d/60-rngkit-truerng3.rules" ] &&
  grep -q '^rngkit:' "$ROOT/etc/group" &&
  grep -q 'alice' "$ROOT/etc/group" &&
  grep -q 'udevadm control --reload-rules' "$ROOT/var/log/commands" &&
  ! grep -q trigger "$ROOT/var/log/commands"; then
  pass 'bitbabbler scope and reload'
else
  fail_test 'bitbabbler scope and reload' "$(ls -l "$ROOT/etc/udev/rules.d"; cat "$ROOT/etc/group"; cat "$ROOT/var/log/commands")"
fi

if cmp -s "$(dirname "$HELPER")/60-rngkit-bitbabbler.rules" "$ROOT/etc/udev/rules.d/60-rngkit-bitbabbler.rules"; then
  pass 'installed bitbabbler rule matches source'
else
  fail_test 'installed bitbabbler rule matches source' 'content mismatch'
fi

# Second apply preserves identical rules and membership, but reloads udev.
: >"$ROOT/var/log/commands"
expect_ok 'repeat apply identical' "$ROOT" --device bitbabbler --user alice
if grep -Eq 'groupadd|usermod' "$ROOT/var/log/commands" ||
  ! grep -q 'udevadm control --reload-rules' "$ROOT/var/log/commands"; then
  fail_test 'repeat apply identical' "unexpected commands: $(cat "$ROOT/var/log/commands")"
else
  pass 'repeat apply identical'
fi

# TrueRNG3-only should not write the BitBabbler rule on a fresh root.
ROOT2=$WORK/trng
prepare_root "$ROOT2"
write_stubs "$ROOT2/stubs"
expect_ok 'apply truerng3' "$ROOT2" --device truerng3 --user alice
if [ -f "$ROOT2/etc/udev/rules.d/60-rngkit-truerng3.rules" ] &&
  [ ! -e "$ROOT2/etc/udev/rules.d/60-rngkit-bitbabbler.rules" ]; then
  pass 'truerng3 scope'
else
  fail_test 'truerng3 scope' "$(ls -l "$ROOT2/etc/udev/rules.d")"
fi
if grep -q 'ID_MM_DEVICE_IGNORE' "$ROOT2/etc/udev/rules.d/60-rngkit-truerng3.rules" &&
  grep -q 'SUBSYSTEM=="tty"' "$ROOT2/etc/udev/rules.d/60-rngkit-truerng3.rules" &&
  ! grep -qi truerngpro "$ROOT2/etc/udev/rules.d/60-rngkit-truerng3.rules" &&
  ! grep -q SYMLINK "$ROOT2/etc/udev/rules.d/60-rngkit-truerng3.rules"; then
  pass 'truerng3 rule contract'
else
  fail_test 'truerng3 rule contract' "$(cat "$ROOT2/etc/udev/rules.d/60-rngkit-truerng3.rules")"
fi

# both installs both files; conflict on one dest fails before writing the other.
ROOT3=$WORK/both
prepare_root "$ROOT3"
write_stubs "$ROOT3/stubs"
printf 'admin-rule\n' >"$ROOT3/etc/udev/rules.d/60-rngkit-bitbabbler.rules"
expect_fail 'conflict preserves other dest' 'different rule already exists' "$ROOT3" --device both --user alice
if [ ! -e "$ROOT3/etc/udev/rules.d/60-rngkit-truerng3.rules" ] &&
  [ "$(cat "$ROOT3/etc/udev/rules.d/60-rngkit-bitbabbler.rules")" = "admin-rule" ]; then
  pass 'conflict is preflight'
else
  fail_test 'conflict is preflight' "$(ls -l "$ROOT3/etc/udev/rules.d"; cat "$ROOT3/etc/group")"
fi

# Non-regular and symlink targets rejected.
ROOT4=$WORK/nonregular
prepare_root "$ROOT4"
write_stubs "$ROOT4/stubs"
mkdir "$ROOT4/etc/udev/rules.d/60-rngkit-bitbabbler.rules"
expect_fail 'nonregular target' 'not a regular file' "$ROOT4" --device bitbabbler --user alice

ROOT4S=$WORK/symlink
prepare_root "$ROOT4S"
write_stubs "$ROOT4S/stubs"
touch "$ROOT4S/somewhere"
if ln -s "$ROOT4S/somewhere" "$ROOT4S/etc/udev/rules.d/60-rngkit-bitbabbler.rules" 2>/dev/null &&
  [ -L "$ROOT4S/etc/udev/rules.d/60-rngkit-bitbabbler.rules" ]; then
  expect_fail 'symlink target' 'symlink rule target' "$ROOT4S" --device bitbabbler --user alice
else
  pass 'symlink target skipped (this environment cannot create a symlink dest)'
fi

# Identical content with unsafe mode.
ROOT5=$WORK/mode
prepare_root "$ROOT5"
write_stubs "$ROOT5/stubs"
cp "$(dirname "$HELPER")/60-rngkit-bitbabbler.rules" "$ROOT5/etc/udev/rules.d/60-rngkit-bitbabbler.rules"
chmod 666 "$ROOT5/etc/udev/rules.d/60-rngkit-bitbabbler.rules"
if [ "$(stat -c '%a' "$ROOT5/etc/udev/rules.d/60-rngkit-bitbabbler.rules")" = "666" ]; then
  expect_fail 'unsafe identical mode' 'unsafe mode' "$ROOT5" --device bitbabbler --user alice
else
  pass 'unsafe identical mode skipped (this filesystem does not retain mode 666)'
fi

# udevadm failure reports completed rule install.
ROOT6=$WORK/udevfail
prepare_root "$ROOT6"
write_stubs "$ROOT6/stubs"
: >"$ROOT6/fail-udevadm"
out=$(run_helper "$ROOT6" --device bitbabbler --user alice 2>&1 || true)
if printf '%s' "$out" | grep -q 'completed before failure' &&
  printf '%s' "$out" | grep -q 'installed ' &&
  [ -f "$ROOT6/etc/udev/rules.d/60-rngkit-bitbabbler.rules" ]; then
  pass 'partial failure is reported'
else
  fail_test 'partial failure is reported' "$out"
fi

# Retrying a failed reload must fail again until udev recovers, then reload.
expect_fail 'repeat failed reload' 'udevadm control --reload-rules failed' "$ROOT6" --device bitbabbler --user alice
rm -f "$ROOT6/fail-udevadm"
: >"$ROOT6/var/log/commands"
expect_ok 'retry reload after recovery' "$ROOT6" --device bitbabbler --user alice
if grep -q 'udevadm control --reload-rules' "$ROOT6/var/log/commands" &&
  ! grep -Eq 'groupadd|usermod' "$ROOT6/var/log/commands"; then
  pass 'retry reload preserves existing setup'
else
  fail_test 'retry reload preserves existing setup' "$(cat "$ROOT6/var/log/commands")"
fi

# Missing libusb fails BitBabbler, not TrueRNG3.
ROOT7=$WORK/nolibusb
prepare_root "$ROOT7"
write_stubs "$ROOT7/stubs"
rm -f "$ROOT7/usr/lib/libusb-1.0.so.0"
expect_fail 'bitbabbler needs libusb' 'libusb-1.0' "$ROOT7" --check --device bitbabbler --user alice
expect_ok 'truerng3 without libusb' "$ROOT7" --check --device truerng3 --user alice

# Production apply without test root refuses non-root.
rc=0
out=$(bash "$HELPER" --device bitbabbler --user alice 2>&1) || rc=$?
if [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -q 'apply requires root'; then
  pass 'unprivileged apply without test root'
else
  fail_test 'unprivileged apply without test root' "rc=$rc out=$out"
fi

if [ "$FAILS" -ne 0 ]; then
  printf '%s test(s) failed\n' "$FAILS" >&2
  exit 1
fi
printf 'all device-setup helper tests passed\n'

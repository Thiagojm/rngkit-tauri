#!/usr/bin/env bash
# RngKit Ubuntu/Debian permission helper for BitBabbler and TrueRNG3.
# Invoke manually. The application never starts this script.
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
GROUP_NAME=rngkit
RULE_MODE=644
BITB_RULE_NAME=60-rngkit-bitbabbler.rules
TRNG_RULE_NAME=60-rngkit-truerng3.rules
BITB_SOURCE="${SCRIPT_DIR}/${BITB_RULE_NAME}"
TRNG_SOURCE="${SCRIPT_DIR}/${TRNG_RULE_NAME}"

TEST_ROOT=${RNGKIT_SETUP_TEST_ROOT-}
COMPLETED=()
PLANNED_RULES=()
CREATED_GROUP=0
ADDED_MEMBERSHIP=0
INSTALLED_RULES=()

fail() {
  printf 'error: %s\n' "$1" >&2
  if [ "${#COMPLETED[@]}" -gt 0 ]; then
    printf 'completed before failure:\n' >&2
    local item
    for item in "${COMPLETED[@]}"; do
      printf '  %s\n' "$item" >&2
    done
  fi
  exit 1
}

usage() {
  cat <<'EOF'
Usage:
  setup-rng-devices.sh --help
  setup-rng-devices.sh --check --device bitbabbler|truerng3|both --user USER
  setup-rng-devices.sh --device bitbabbler|truerng3|both --user USER

Configure local USB/tty permissions for RngKit BitBabbler (0403:7840) and/or
TrueRNG3 (04d8:f5fe). Apply requires root (sudo). --help and --check do not.

The application never launches this script, an installer, or a terminal.

Options:
  --help    Show this help and exit
  --check   Validate without changing the system
  --device  bitbabbler, truerng3, or both (required for check and apply)
  --user    existing non-root login name (required for check and apply)

This helper does not install OS packages, unload kernel modules, stop
services, overwrite different administrator rule files, chmod the rules
directory, or trigger every udev device. Permission setup cannot certify
hardware I/O.
EOF
}

root_path() {
  if [ -n "$TEST_ROOT" ]; then
    printf '%s%s' "$TEST_ROOT" "$1"
  else
    printf '%s' "$1"
  fi
}

current_uid() {
  id -u
}

if [ -n "$TEST_ROOT" ] && [ "$(current_uid)" -eq 0 ]; then
  fail 'RNGKIT_SETUP_TEST_ROOT is refused while running as root'
fi

CHECK=0
DEVICE=
USER_NAME=

while [ $# -gt 0 ]; do
  case "$1" in
    --help)
      usage
      exit 0
      ;;
    --check)
      CHECK=1
      shift
      ;;
    --device)
      if [ $# -lt 2 ]; then
        fail 'missing value for --device'
      fi
      DEVICE=$2
      shift 2
      ;;
    --user)
      if [ $# -lt 2 ]; then
        fail 'missing value for --user'
      fi
      USER_NAME=$2
      shift 2
      ;;
    --*)
      fail "unknown option: $1"
      ;;
    *)
      fail "unexpected argument: $1"
      ;;
  esac
done

if [ -z "$DEVICE" ] || [ -z "$USER_NAME" ]; then
  fail 'require --device bitbabbler|truerng3|both and --user USER'
fi

case "$DEVICE" in
  bitbabbler|truerng3|both) ;;
  *) fail 'unknown --device (use bitbabbler, truerng3, or both)' ;;
esac

if ! printf '%s' "$USER_NAME" | grep -Eq '^[a-z_][a-z0-9_-]*$'; then
  fail 'invalid user name'
fi

if [ "$CHECK" -eq 0 ] && [ -z "$TEST_ROOT" ] && [ "$(current_uid)" -ne 0 ]; then
  fail 'apply requires root; re-run with sudo after --check succeeds'
fi

require_cmd() {
  local cmd
  for cmd in "$@"; do
    command -v "$cmd" >/dev/null 2>&1 || fail "missing required command: $cmd"
  done
}

require_cmd getent groupadd usermod udevadm mktemp cmp mv chmod cat mkdir

OS_RELEASE=$(root_path /etc/os-release)
if [ ! -f "$OS_RELEASE" ]; then
  fail 'unsupported OS: /etc/os-release is missing'
fi
# shellcheck disable=SC1090
. "$OS_RELEASE"
OS_OK=0
case "${ID-}" in
  ubuntu|debian) OS_OK=1 ;;
esac
case " ${ID_LIKE-} " in
  *" debian "*|*" ubuntu "*) OS_OK=1 ;;
esac
if [ "$OS_OK" -ne 1 ]; then
  fail 'unsupported OS: Ubuntu or Debian with systemd/udev is required'
fi

SYSTEMD_DIR=$(root_path /run/systemd/system)
if [ ! -d "$SYSTEMD_DIR" ]; then
  fail 'systemd is not available; Ubuntu/Debian with systemd/udev is required'
fi

PASSWD_ENT=$(getent passwd "$USER_NAME" || true)
if [ -z "$PASSWD_ENT" ]; then
  fail "user not found: $USER_NAME"
fi
USER_UID=$(printf '%s' "$PASSWD_ENT" | cut -d: -f3)
if [ "$USER_UID" = "0" ]; then
  fail 'refusing to configure the root user'
fi

source_ok() {
  local path=$1
  if [ ! -e "$path" ]; then
    fail "missing source rule: $path"
  fi
  if [ -L "$path" ]; then
    fail "source rule must not be a symlink: $path"
  fi
  if [ ! -f "$path" ]; then
    fail "source rule is not a regular file: $path"
  fi
}

RULES_DIR=$(root_path /etc/udev/rules.d)
if [ ! -d "$RULES_DIR" ]; then
  fail "udev rules directory is missing: $RULES_DIR"
fi

if [ "$DEVICE" = "bitbabbler" ] || [ "$DEVICE" = "both" ]; then
  source_ok "$BITB_SOURCE"
  PLANNED_RULES+=("${RULES_DIR}/${BITB_RULE_NAME}|${BITB_SOURCE}|bitbabbler")
fi
if [ "$DEVICE" = "truerng3" ] || [ "$DEVICE" = "both" ]; then
  source_ok "$TRNG_SOURCE"
  PLANNED_RULES+=("${RULES_DIR}/${TRNG_RULE_NAME}|${TRNG_SOURCE}|truerng3")
fi

if [ "$DEVICE" = "bitbabbler" ] || [ "$DEVICE" = "both" ]; then
  LIBUSB_OK=0
  if [ -n "$TEST_ROOT" ]; then
    if [ -e "$(root_path /usr/lib/libusb-1.0.so.0)" ]; then
      LIBUSB_OK=1
    fi
  else
    if ldconfig -p 2>/dev/null | grep -q 'libusb-1.0.so.0'; then
      LIBUSB_OK=1
    elif ls /usr/lib/*/libusb-1.0.so.0 /lib/*/libusb-1.0.so.0 >/dev/null 2>&1; then
      LIBUSB_OK=1
    fi
  fi
  if [ "$LIBUSB_OK" -ne 1 ]; then
    fail 'BitBabbler needs libusb-1.0 (package libusb-1.0-0). RngKit does not use PyUSB.'
  fi
fi

file_mode() {
  stat -c '%a' "$1"
}

file_uid_gid() {
  stat -c '%u:%g' "$1"
}

dest_conflict() {
  local dest=$1
  local src=$2
  if [ -L "$dest" ]; then
    fail "refusing symlink rule target: $dest"
  fi
  if [ -e "$dest" ] && [ ! -f "$dest" ]; then
    fail "rule target is not a regular file: $dest"
  fi
  if [ -f "$dest" ]; then
    if cmp -s "$src" "$dest"; then
      if [ "$(file_mode "$dest")" != "$RULE_MODE" ]; then
        fail "identical rule exists with unsafe mode at $dest"
      fi
      if [ -z "$TEST_ROOT" ] && [ "$(file_uid_gid "$dest")" != "0:0" ]; then
        fail "identical rule exists with unsafe ownership at $dest"
      fi
    else
      fail "different rule already exists; leaving $dest unchanged"
    fi
  fi
}

spec=
dest=
src=
for spec in "${PLANNED_RULES[@]}"; do
  dest=${spec%%|*}
  src=${spec#*|}
  src=${src%%|*}
  dest_conflict "$dest" "$src"
done

GROUP_ENT=$(getent group "$GROUP_NAME" || true)
IN_GROUP=0
if [ -n "$GROUP_ENT" ]; then
  MEMBERS=$(printf '%s' "$GROUP_ENT" | cut -d: -f4)
  case ",$MEMBERS," in
    *",$USER_NAME,"*) IN_GROUP=1 ;;
  esac
fi

if [ "$CHECK" -eq 1 ]; then
  printf 'check passed for device=%s user=%s\n' "$DEVICE" "$USER_NAME"
  printf 'no files, groups, or memberships were changed\n'
  printf 'permission setup cannot certify hardware I/O\n'
  exit 0
fi

if [ -z "$GROUP_ENT" ]; then
  groupadd --system "$GROUP_NAME" || fail "groupadd $GROUP_NAME failed"
  CREATED_GROUP=1
  COMPLETED+=("created group $GROUP_NAME")
fi

if [ "$IN_GROUP" -eq 0 ]; then
  usermod -aG "$GROUP_NAME" "$USER_NAME" || fail "usermod -aG $GROUP_NAME $USER_NAME failed"
  ADDED_MEMBERSHIP=1
  COMPLETED+=("added $USER_NAME to group $GROUP_NAME")
fi

install_rule() {
  local dest=$1
  local src=$2
  if [ -f "$dest" ]; then
    COMPLETED+=("kept existing identical rule $dest")
    return 0
  fi
  local tmp
  tmp=$(mktemp "${dest}.tmp.XXXXXX") || fail "could not create temporary file for $dest"
  cat "$src" >"$tmp" || {
    rm -f "$tmp"
    fail "could not write temporary rule for $dest"
  }
  chmod "$RULE_MODE" "$tmp" || {
    rm -f "$tmp"
    fail "could not set mode on temporary rule for $dest"
  }
  if [ -z "$TEST_ROOT" ]; then
    chown root:root "$tmp" || {
      rm -f "$tmp"
      fail "could not set ownership on temporary rule for $dest"
    }
  fi
  mv -f "$tmp" "$dest" || {
    rm -f "$tmp"
    fail "could not install rule $dest"
  }
  INSTALLED_RULES+=("$dest")
  COMPLETED+=("installed $dest")
}

for spec in "${PLANNED_RULES[@]}"; do
  dest=${spec%%|*}
  src=${spec#*|}
  src=${src%%|*}
  install_rule "$dest" "$src"
done

# Reload on every apply so retrying a failed reload also works with identical files.
udevadm control --reload-rules || fail 'udevadm control --reload-rules failed; retry this command after resolving the udev error'
COMPLETED+=("reloaded udev rules")

printf 'setup finished for device=%s user=%s\n' "$DEVICE" "$USER_NAME"
if [ "$CREATED_GROUP" -eq 1 ]; then
  printf 'created group %s\n' "$GROUP_NAME"
fi
if [ "$ADDED_MEMBERSHIP" -eq 1 ]; then
  printf 'added %s to group %s\n' "$USER_NAME" "$GROUP_NAME"
fi
if [ "${#INSTALLED_RULES[@]}" -gt 0 ]; then
  for dest in "${INSTALLED_RULES[@]}"; do
    printf 'installed %s\n' "$dest"
  done
fi
printf 'reloaded udev rules (did not trigger all devices)\n'
printf 'log out and back in so group membership applies, then reconnect the selected device\n'
printf 'in RngKit, select Refresh sources and run a short Start/Stop collection\n'
printf 'permission setup cannot certify hardware I/O\n'
printf 'rollback: remove only the rule files listed above if this helper installed them; if it added group membership, run: gpasswd -d %s %s\n' "$USER_NAME" "$GROUP_NAME"
if [ "$CREATED_GROUP" -eq 1 ]; then
  printf 'rollback: this helper created group %s; delete it only if it has no remaining members you want to keep\n' "$GROUP_NAME"
fi
exit 0

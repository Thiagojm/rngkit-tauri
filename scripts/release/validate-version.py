#!/usr/bin/env python3
"""Check installer version files agree, and optionally match a git tag."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

VERSION_RE = re.compile(
    r"^(?P<core>[0-9]+\.[0-9]+\.[0-9]+)(?P<pre>-[0-9A-Za-z][0-9A-Za-z.]*)?$"
)
TAG_RE = re.compile(
    r"^v(?P<version>[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.]*)?)$"
)


def fail(message: str, code: int = 1) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(code)


def read_text(path: Path) -> str:
    if not path.is_file():
        fail(f"missing version file: {path}")
    return path.read_text(encoding="utf-8")


def json_object(path: Path) -> dict:
    try:
        data = json.loads(read_text(path))
    except json.JSONDecodeError as exc:
        fail(f"{path} is not valid JSON: {exc}")
    if not isinstance(data, dict):
        fail(f"{path} is not a JSON object")
    return data


def parse_json_version(path: Path) -> str:
    value = json_object(path).get("version")
    if not isinstance(value, str) or not value:
        fail(f"{path} has no string version")
    return value


def parse_package_lock_root_version(path: Path) -> str:
    packages = json_object(path).get("packages")
    if not isinstance(packages, dict) or "" not in packages:
        fail(f"{path} is missing packages['']")
    root = packages[""]
    if not isinstance(root, dict):
        fail(f"{path} packages[''] is not an object")
    value = root.get("version")
    if not isinstance(value, str) or not value:
        fail(f"{path} packages[''] has no string version")
    return value


def cargo_toml_package_version(path: Path) -> str:
    in_package = False
    for raw in read_text(path).splitlines():
        line = raw.strip()
        if line == "[package]":
            in_package = True
            continue
        if in_package and line.startswith("[") and line.endswith("]"):
            break
        if in_package:
            match = re.match(r'^version\s*=\s*"([^"]+)"', line)
            if match:
                return match.group(1)
    fail(f"{path} [package] has no version")
    raise AssertionError


def cargo_lock_package_version(path: Path, name: str) -> str:
    found: list[str] = []
    blocks = read_text(path).split("[[package]]")
    for block in blocks[1:]:
        pkg_name = None
        pkg_version = None
        for raw in block.splitlines():
            line = raw.strip()
            if line.startswith("name = "):
                pkg_name = line.split("=", 1)[1].strip().strip('"')
            elif line.startswith("version = ") and pkg_version is None:
                pkg_version = line.split("=", 1)[1].strip().strip('"')
            if pkg_name is not None and pkg_version is not None:
                if pkg_name == name:
                    found.append(pkg_version)
                break
    if not found:
        fail(f"{path} has no package named {name}")
    if len(set(found)) != 1:
        fail(f"{path} has conflicting {name} versions: {', '.join(found)}")
    return found[0]


def require_semver(label: str, version: str) -> None:
    if not VERSION_RE.fullmatch(version):
        fail(
            f"{label} version {version!r} is not a supported semver "
            "(expected MAJOR.MINOR.PATCH with optional -prerelease)"
        )


def is_prerelease(version: str) -> bool:
    match = VERSION_RE.fullmatch(version)
    return bool(match and match.group("pre"))


def collect_versions(root: Path) -> dict[str, str]:
    package_json = root / "package.json"
    package_lock = root / "package-lock.json"
    tauri_conf = root / "src-tauri" / "tauri.conf.json"
    cargo_toml = root / "src-tauri" / "Cargo.toml"
    cargo_lock = root / "src-tauri" / "Cargo.lock"
    versions = {
        "package.json": parse_json_version(package_json),
        "package-lock.json": parse_json_version(package_lock),
        "package-lock.json packages['']": parse_package_lock_root_version(package_lock),
        "src-tauri/tauri.conf.json": parse_json_version(tauri_conf),
        "src-tauri/Cargo.toml": cargo_toml_package_version(cargo_toml),
        "src-tauri/Cargo.lock rngkit": cargo_lock_package_version(cargo_lock, "rngkit"),
    }
    for label, version in versions.items():
        require_semver(label, version)
    unique = set(versions.values())
    if len(unique) != 1:
        details = ", ".join(f"{label}={value}" for label, value in versions.items())
        fail(f"version mismatch across package files and lockfiles: {details}")
    return versions


def write_github_output(version: str, prerelease: bool) -> None:
    output = os.environ.get("GITHUB_OUTPUT")
    lines = f"version={version}\nprerelease={'true' if prerelease else 'false'}\n"
    sys.stdout.write(lines)
    if output:
        with Path(output).open("a", encoding="utf-8") as handle:
            handle.write(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument(
        "--tag",
        default="",
        help="git tag name; required correspondence is v<version>",
    )
    args = parser.parse_args()
    root = args.root.resolve()
    versions = collect_versions(root)
    version = next(iter(set(versions.values())))
    tag = args.tag.strip()
    if tag:
        match = TAG_RE.fullmatch(tag)
        if match is None:
            fail(
                f"invalid tag {tag!r}; expected v<MAJOR.MINOR.PATCH> "
                "with optional -prerelease matching the package version"
            )
        tagged = match.group("version")
        if tagged != version:
            fail(
                f"tag {tag} does not match package version {version} "
                "(package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml, lockfiles)"
            )
    write_github_output(version, is_prerelease(version))


if __name__ == "__main__":
    main()

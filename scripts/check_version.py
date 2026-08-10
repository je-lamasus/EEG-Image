#!/usr/bin/env python3
"""Validate the application version and, in CI, require a version bump."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import NoReturn


VERSION_PATTERN = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")
ZERO_SHA = "0" * 40


def fail(message: str) -> NoReturn:
    print(f"version check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def parse_version(value: object, source: str) -> tuple[int, int, int]:
    if not isinstance(value, str):
        fail(f"{source} does not contain a string package version")

    match = VERSION_PATTERN.fullmatch(value)
    if match is None:
        fail(f"{source} version {value!r} must use MAJOR.MINOR.PATCH format")

    return tuple(int(part) for part in match.groups())


def load_toml(path: Path) -> dict:
    try:
        with path.open("rb") as file:
            return tomllib.load(file)
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"cannot read {path}: {error}")


def current_manifest_version() -> tuple[str, tuple[int, int, int]]:
    manifest = load_toml(Path("Cargo.toml"))
    value = manifest.get("package", {}).get("version")
    parsed = parse_version(value, "Cargo.toml")
    return str(value), parsed


def check_lockfile(manifest_version: str) -> None:
    lockfile = load_toml(Path("Cargo.lock"))
    matching_packages = [
        package
        for package in lockfile.get("package", [])
        if package.get("name") == "eeg-image"
    ]

    if len(matching_packages) != 1:
        fail("Cargo.lock must contain exactly one eeg-image package")

    lock_version = matching_packages[0].get("version")
    if lock_version != manifest_version:
        fail(
            f"Cargo.lock contains eeg-image {lock_version!r}, "
            f"but Cargo.toml contains {manifest_version!r}; run cargo check"
        )


def previous_manifest_version(ref: str) -> tuple[str, tuple[int, int, int]] | None:
    if not ref or ref == ZERO_SHA:
        return None

    result = subprocess.run(
        ["git", "show", f"{ref}:Cargo.toml"],
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        fail(
            f"cannot read Cargo.toml at {ref}: "
            f"{result.stderr.decode(errors='replace').strip()}"
        )

    try:
        manifest = tomllib.loads(result.stdout.decode())
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        fail(f"cannot parse Cargo.toml at {ref}: {error}")

    value = manifest.get("package", {}).get("version")
    return str(value), parse_version(value, f"Cargo.toml at {ref}")


def write_dotenv(path: Path, version: str) -> None:
    try:
        path.write_text(f"APP_VERSION={version}\n", encoding="utf-8")
    except OSError as error:
        fail(f"cannot write {path}: {error}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--previous-ref",
        help="Git ref whose Cargo.toml version must be lower than the current version",
    )
    parser.add_argument(
        "--dotenv",
        type=Path,
        help="write APP_VERSION to a CI-compatible dotenv file",
    )
    args = parser.parse_args()

    version, parsed_version = current_manifest_version()
    check_lockfile(version)

    if args.previous_ref:
        previous = previous_manifest_version(args.previous_ref)
        if previous is not None:
            previous_text, previous_parsed = previous
            if parsed_version <= previous_parsed:
                fail(
                    f"version must increase on main: {previous_text} -> {version}; "
                    "update [package].version and Cargo.lock"
                )

    if args.dotenv:
        write_dotenv(args.dotenv, version)

    print(f"version check passed: {version}")


if __name__ == "__main__":
    main()

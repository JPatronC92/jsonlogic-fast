#!/usr/bin/env python3
import json
import subprocess
import sys
import urllib.request

def get_local_version():
    import tomllib
    with open("Cargo.toml", "rb") as f:
        data = tomllib.load(f)
    return data["workspace"]["package"]["version"]

def get_crates_io_version():
    url = "https://crates.io/api/v1/crates/jsonlogic-fast"
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "jsonlogic-fast-ci-check (contact@example.com)"}
    )
    try:
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode())
            return data["crate"]["max_version"]
    except Exception as e:
        print(f"Warning: Failed to fetch crates.io version: {e}")
        return None

def parse_version(version_str):
    return tuple(map(int, version_str.split(".")))

def get_modified_files():
    # Try different strategies to find modified files against main branch
    for cmd in [
        ["git", "diff", "--name-only", "origin/main...HEAD"],
        ["git", "diff", "--name-only", "origin/main"],
        ["git", "diff", "--name-only", "main...HEAD"],
        ["git", "diff", "--name-only", "main"]
    ]:
        try:
            res = subprocess.run(cmd, capture_output=True, text=True, check=True)
            files = [line.strip() for line in res.stdout.strip().split("\n") if line.strip()]
            if files:
                return files
        except subprocess.CalledProcessError:
            continue

    # Fallback to status porcelain
    try:
        res = subprocess.run(["git", "status", "--porcelain"], capture_output=True, text=True, check=True)
        files = []
        for line in res.stdout.strip().split("\n"):
            line = line.strip()
            if line:
                parts = line.split(None, 1)
                if len(parts) > 1:
                    files.append(parts[1])
        return files
    except subprocess.CalledProcessError:
        return []

def main():
    local_version = get_local_version()
    print(f"Local version (Cargo.toml): {local_version}")

    crates_version = get_crates_io_version()
    if not crates_version:
        print("Could not retrieve crates.io version. Skipping guardrail check.")
        sys.exit(0)

    print(f"Crates.io max version: {crates_version}")

    local_t = parse_version(local_version)
    crates_t = parse_version(crates_version)

    if local_t > crates_t:
        print(f"Success: Local version {local_version} is a bump over crates.io version {crates_version}.")
        sys.exit(0)

    print(f"Local version {local_version} is <= crates.io version {crates_version}.")
    print("Checking for modified files in core, python, or wasm directories...")

    modified_files = get_modified_files()
    print(f"Modified files: {modified_files}")

    critical_prefixes = ("core/", "python/", "wasm/")
    violating_files = []
    for f in modified_files:
        if f.startswith(critical_prefixes):
            violating_files.append(f)

    if violating_files:
        print("\nERROR: Changes detected in critical codebase files, but the version in Cargo.toml has not been bumped!")
        print(f"Violating files:\n" + "\n".join(f"- {f}" for f in violating_files))
        print(f"\nPlease bump the workspace version in Cargo.toml (and update references in Python/WASM Cargo.toml/pyproject.toml) to a version > {crates_version}.")
        sys.exit(1)

    print("Success: No critical file changes detected. No version bump required.")
    sys.exit(0)

if __name__ == "__main__":
    main()

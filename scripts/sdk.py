#!/usr/bin/env python3
"""Check the workspace release policy or generate a Cargo local override."""
import argparse
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SOURCE = "https://github.com/OctopusGarage/mesh-foundation.git"


PACKAGES = {"mesh-foundation-" + name for name in
            ("identity", "storage", "e2ee", "connect", "replication", "blob", "mailbox")}
VERSION = re.compile(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)(?:-(?:alpha|beta|rc)\.(?:0|[1-9][0-9]*))?")


def validate(packages, tag=None):
    if {p["name"] for p in packages} != PACKAGES or len(packages) != 7:
        raise ValueError("expected exactly the seven mesh-foundation packages")
    versions = {p["version"] for p in packages}
    if len(versions) != 1 or any(p["publish"] != ["crates-io"] for p in packages):
        raise ValueError("all packages must share a version and publish only to crates-io")
    version, = versions
    if not VERSION.fullmatch(version) or (tag is not None and tag != f"v{version}"):
        raise ValueError(f"expected release tag v{version}, got {tag}")
    for package in packages:
        for dependency in package["dependencies"]:
            if dependency["name"] in PACKAGES and dependency["req"] != f"={version}":
                raise ValueError("internal dependencies must pin the exact workspace version")
    return version


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    check = commands.add_parser("check")
    check.add_argument("--tag")
    patch = commands.add_parser("patch")
    patch.add_argument("--source", choices=["git", "crates-io"], default="git")
    snapshot = commands.add_parser("snapshot", help="export or verify a committed SDK snapshot")
    snapshot.add_argument("action", choices=["export", "verify"])
    snapshot.add_argument("destination", type=Path)
    snapshot.add_argument("--source", type=Path)
    args = parser.parse_args()
    if args.command == "snapshot":
        import sdk_snapshot
        try:
            revision = (sdk_snapshot.export_snapshot(args.source or ROOT, args.destination)
                        if args.action == "export" else sdk_snapshot.verify_snapshot(args.destination, args.source))
        except (ValueError, OSError, subprocess.CalledProcessError) as error:
            parser.error(str(error))
        print(f"Verified SDK snapshot {revision}")
        return
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        cwd=ROOT,
    ))
    packages = sorted(metadata["packages"], key=lambda package: package["name"])
    if args.command == "patch":
        print("[patch.crates-io]" if args.source == "crates-io" else f"[patch.{json.dumps(SOURCE)}]")
        for package in packages:
            path = Path(package["manifest_path"]).parent.as_posix()
            print(f'{package["name"]} = {{ path = {json.dumps(path, ensure_ascii=False)} }}')
        return
    try:
        version = validate(packages, args.tag)
    except ValueError as error:
        parser.error(str(error))
    print(f"Mesh Foundation {version}, {len(packages)} crates")


if __name__ == "__main__":
    main()

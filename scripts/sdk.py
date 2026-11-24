#!/usr/bin/env python3
"""Check the workspace release policy or generate a Cargo local override."""
import argparse
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SOURCE = "https://github.com/OctopusGarage/mesh-foundation.git"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    check = commands.add_parser("check")
    check.add_argument("--tag")
    commands.add_parser("patch")
    args = parser.parse_args()
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        cwd=ROOT,
    ))
    packages = sorted(metadata["packages"], key=lambda package: package["name"])
    if args.command == "patch":
        print(f"[patch.{json.dumps(SOURCE)}]")
        for package in packages:
            path = Path(package["manifest_path"]).parent.as_posix()
            print(f'{package["name"]} = {{ path = {json.dumps(path, ensure_ascii=False)} }}')
        return
    versions = {package["version"] for package in packages}
    if len(versions) != 1 or any(package["publish"] != [] for package in packages):
        parser.error("all workspace crates must share one version and disable registry publishing")
    version, = versions
    if args.tag and (
        not re.fullmatch(r"v(?:0|[1-9][0-9]*)(?:\.(?:0|[1-9][0-9]*)){2}", args.tag)
        or args.tag != f"v{version}"
    ):
        parser.error(f"expected stable release tag v{version}, got {args.tag}")
    print(f"Mesh Foundation {version}, {len(packages)} crates")


if __name__ == "__main__":
    main()

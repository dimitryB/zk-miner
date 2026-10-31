#!/usr/bin/env python3
"""Check cargo metadata proves the intended engine and CPU-only feature closure."""
import argparse
import json
from pathlib import Path

PIN = "f8dcbdf8124a3f1528146bb01d539c55545cbb3d"

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("metadata", type=Path)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--engine", type=Path)
    group.add_argument("--unchanged-verifier", action="store_true")
    args = parser.parse_args()
    metadata = json.loads(args.metadata.read_text())
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    count = 0
    for package in metadata["packages"]:
        if package["name"].startswith("risc0-") or package["name"] == "rzup":
            count += 1
            if args.engine:
                if package["source"] is not None or not Path(package["manifest_path"]).is_relative_to(args.engine.resolve()):
                    parser.error(f"non-local engine crate: {package}")
            elif package["source"] != f"git+https://github.com/hemilabs/risc0?rev=f8dcbdf8#{PIN}":
                parser.error(f"verifier source is not the unchanged pinned Git revision: {package}")
            features = set(nodes[package["id"]]["features"])
            prohibited = {"cuda", "rocm", "intel", "metal", "bonsai"}
            if args.unchanged_verifier:
                prohibited |= {"prove", "client"}
            if features & prohibited:
                parser.error(f"unexpected features in {package['name']}: {features & prohibited}")
            print(package["name"], package["version"], sorted(features))
    if not count:
        parser.error("no engine packages found")
    print(f"verified source and feature closure of {count} engine packages")

if __name__ == "__main__":
    main()

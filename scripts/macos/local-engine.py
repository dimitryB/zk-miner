#!/usr/bin/env python3
"""Generate Cargo overrides for the *entire* pinned Hemi source, never upstream HEAD."""
import argparse
import json
from pathlib import Path
import subprocess
import tomllib

PIN = "f8dcbdf8124a3f1528146bb01d539c55545cbb3d"

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("engine", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    root = args.engine.resolve()
    revision = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
    if revision != PIN:
        parser.error(f"expected pinned Hemi {PIN}, found {revision}")
    origin = subprocess.check_output(["git", "-C", str(root), "remote", "get-url", "origin"], text=True).strip()
    if origin.removesuffix(".git").rstrip("/") not in ("https://github.com/hemilabs/risc0", "git@github.com:hemilabs/risc0"):
        parser.error("engine origin is not hemilabs/risc0")
    packages = {}
    for path in root.rglob("Cargo.toml"):
        if "target" in path.relative_to(root).parts:
            continue
        data = tomllib.loads(path.read_text())
        name = data.get("package", {}).get("name", "")
        if name.startswith("risc0-") or name == "rzup":
            if name in packages:
                parser.error(f"ambiguous engine package: {name}")
            packages[name] = str(path.parent)
    # Git source overrides are distinct from crates.io patches. Guest workspaces
    # receive this same config explicitly; they do not inherit a host root patch.
    text = "# Generated for Hemi " + PIN + "\n"
    for source in ("crates-io", '"https://github.com/hemilabs/risc0"'):
        text += f"\n[patch.{source}]\n"
        text += "".join(f"{name} = {{ path = {json.dumps(path)} }}\n" for name, path in sorted(packages.items()))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(text)
    print(f"wrote {args.output}: {len(packages)} packages, source {revision}")

if __name__ == "__main__":
    main()

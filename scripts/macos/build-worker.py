#!/usr/bin/env python3
"""Build the Mac worker with explicit local overrides; preserve the root Git lock."""
import argparse
import os
from pathlib import Path
import subprocess
import tomllib

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", required=True, type=Path)
    parser.add_argument("--engine", required=True, type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    output = root / "target/macos"
    lock = root / "Cargo.lock"
    original = lock.read_bytes()
    (output / "miner-git-Cargo.lock").write_bytes(original)
    env = os.environ.copy()
    env.pop("RUSTFLAGS", None)
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env["ZKMINER_RISC0_GUEST_DIR"] = str(output / "guest-build/riscv32im-risc0-zkvm-elf/release")
    try:
        subprocess.run(["cargo", "+1.93.0", "build", "-p", "zkminer-prove-risc0", "--bin", "zkminer-prove-risc0",
            "--features", "macos", "--release", "--config", str(args.config.resolve()), "--target-dir", str(output / "worker")],
            cwd=root, env=env, check=True)
        with (output / "worker-metadata.json").open("w") as file:
            subprocess.run(["cargo", "+1.93.0", "metadata", "--locked", "--filter-platform", "aarch64-apple-darwin",
                "--features", "zkminer-prove-risc0/macos", "--config", str(args.config.resolve()), "--format-version", "1"],
                cwd=root, env=env, stdout=file, check=True)
        subprocess.run(["python3", str(root / "scripts/macos/check-source.py"), str(output / "worker-metadata.json"),
                        "--engine", str(args.engine.resolve())], check=True)
        with (output / "worker-tree.txt").open("w") as file:
            subprocess.run(["cargo", "+1.93.0", "tree", "--locked", "-p", "zkminer-prove-risc0",
                "--features", "macos", "--target", "aarch64-apple-darwin", "--edges", "features",
                "--config", str(args.config.resolve())], cwd=root, env=env, stdout=file, check=True)
    finally:
        local = lock.read_bytes()
        (output / "worker-local-Cargo.lock").write_bytes(local)
        old = tomllib.loads(original.decode())
        new = tomllib.loads(local.decode())
        def packages(data):
            return {(p["name"], p["version"]): p for p in data["package"]}
        before, after = packages(old), packages(new)
        # Only source changes for existing engine packages are expected. If
        # another Cargo operation edited anything else, preserve both snapshots
        # and fail instead of overwriting somebody else's lock changes.
        valid = before.keys() == after.keys()
        for key in before.keys() & after.keys():
            a, b = before[key].copy(), after[key].copy()
            if key[0].startswith("risc0-") or key[0] == "rzup":
                a.pop("source", None); b.pop("source", None)
            if key[0] == "risc0-zkp" and "metal" in a.get("dependencies", []) and "metal" not in b.get("dependencies", []):
                # The explicit Metal feature makes this dependency optional in
                # the CPU build; this is the only intended dependency removal.
                a["dependencies"] = [d for d in a["dependencies"] if d != "metal"]
            valid = valid and a == b
        if not valid:
            raise RuntimeError("unexpected lock changes; root lock retained for review, original saved in target/macos/miner-git-Cargo.lock")
        lock.write_bytes(original)
        print("root Git-source lock restored; local worker lock and metadata retained")

if __name__ == "__main__":
    main()

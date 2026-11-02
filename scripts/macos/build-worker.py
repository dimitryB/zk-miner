#!/usr/bin/env python3
"""Build a source-bound CPU or Metal Mac worker; preserve the root Git-source lock."""
import argparse
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

from worker_provenance import paths, profile, seal, snapshot


def check_lock_delta(original, local):
    def packages(data):
        return {(p["name"], p["version"]): p for p in tomllib.loads(data.decode())["package"]}
    before, after = packages(original), packages(local)
    valid = before.keys() == after.keys()
    for key in before.keys() & after.keys():
        a, b = before[key].copy(), after[key].copy()
        if key[0].startswith("risc0-") or key[0] == "rzup":
            a.pop("source", None)
            b.pop("source", None)
        if key[0] == "risc0-zkp" and "metal" in a.get("dependencies", []) and "metal" not in b.get("dependencies", []):
            # The explicit Metal feature makes this existing dependency optional.
            a["dependencies"] = [d for d in a["dependencies"] if d != "metal"]
        if key[0] == "risc0-circuit-recursion" and "risc0-build-kernel" not in a.get("dependencies", []):
            # Recursion Metal embeds the existing field headers at build time.
            b["dependencies"] = [d for d in b.get("dependencies", []) if d != "risc0-build-kernel"]
        valid = valid and a == b
    return valid


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", required=True, type=Path)
    parser.add_argument("--engine", required=True, type=Path)
    parser.add_argument("--revision", required=True, help="full reviewed engine commit SHA")
    parser.add_argument("--mode", choices=["cpu", "metal", "native", "native-recursion-metal"], default="cpu")
    parser.add_argument("--tests", action="store_true", help="run the selected worker unit tests before sealing")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    output = root / "target/macos"
    config, engine = args.config.resolve(), args.engine.resolve()
    # Reuse the strict origin, full-HEAD, pinned-ancestry and complete-patch generator.
    # The supplied config must have exactly those patches, with no extra build settings.
    with tempfile.TemporaryDirectory(prefix="hemi-worker-config-") as temporary:
        expected = Path(temporary) / "engine.toml"
        subprocess.run(["python3", str(root / "scripts/macos/local-engine.py"), str(engine),
                        str(expected), "--revision", args.revision], check=True)
        if tomllib.loads(config.read_text()) != tomllib.loads(expected.read_text()):
            parser.error("--config does not exactly match the reviewed engine's complete local overrides")
    output.mkdir(parents=True, exist_ok=True)
    name = profile(args.mode)
    artifacts = paths(args.mode)
    lock = root / "Cargo.lock"
    original = lock.read_bytes()
    original_path = output / f"{name}-git-Cargo.lock"
    original_path.write_bytes(original)
    env = os.environ.copy()
    env.pop("RUSTFLAGS", None)
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    env["ZKMINER_RISC0_GUEST_DIR"] = str(output / "guest-build/riscv32im-risc0-zkvm-elf/release")
    feature = "macos" if args.mode == "cpu" else "macos-metal,risc0-zkvm/metal-hybrid"
    metadata_feature = "zkminer-prove-risc0/macos" if args.mode == "cpu" else "zkminer-prove-risc0/macos-metal,risc0-zkvm/metal-hybrid"
    if args.mode == "native":
        feature = "macos-native-wrap,risc0-groth16/native-groth16"
        metadata_feature = "zkminer-prove-risc0/macos-native-wrap,risc0-groth16/native-groth16"
    if args.mode == "native-recursion-metal":
        feature = "macos-recursion-metal,risc0-zkvm/metal-recursion,risc0-groth16/native-groth16"
        metadata_feature = "zkminer-prove-risc0/" + feature
    snapshot(args.mode, engine, config, env)
    try:
        subprocess.run(["cargo", "+1.93.0", "build", "-p", "zkminer-prove-risc0", "--bin", "zkminer-prove-risc0",
            "--features", feature, "--release", "--config", str(config), "--target-dir", str(output / name)],
            cwd=root, env=env, check=True)
        if args.tests:
            subprocess.run(["cargo", "+1.93.0", "test", "-p", "zkminer-prove-risc0", "--bin", "zkminer-prove-risc0",
                "--features", feature, "--release", "--config", str(config), "--target-dir", str(output / name)],
                cwd=root, env=env, check=True)
        with artifacts["metadata"].open("w") as file:
            subprocess.run(["cargo", "+1.93.0", "metadata", "--locked", "--filter-platform", "aarch64-apple-darwin",
                "--features", metadata_feature, "--config", str(config), "--format-version", "1"],
                cwd=root, env=env, stdout=file, check=True)
        subprocess.run(["python3", str(root / "scripts/macos/check-source.py"), str(artifacts["metadata"]),
                        "--engine", str(engine)] + (["--metal-hybrid"] if args.mode == "metal" else ["--metal-recursion"] if args.mode == "native-recursion-metal" else []), check=True)
        with (output / f"{name}-tree.txt").open("w") as file:
            subprocess.run(["cargo", "+1.93.0", "tree", "--locked", "-p", "zkminer-prove-risc0",
                "--features", feature, "--target", "aarch64-apple-darwin", "--edges", "features",
                "--config", str(config)], cwd=root, env=env, stdout=file, check=True)
    finally:
        local = lock.read_bytes()
        artifacts["local_lock"].write_bytes(local)
        # Preserve both snapshots and refuse to overwrite unexpected concurrent edits.
        if not check_lock_delta(original, local):
            raise RuntimeError(f"unexpected lock changes; root lock retained for review, original saved in {original_path}")
        lock.write_bytes(original)
        print("root Git-source lock restored; local worker lock and metadata retained")
    # Source snapshot compares the restored original lock, not its temporary path-source form.
    seal(args.mode, engine, config, env)


if __name__ == "__main__":
    main()

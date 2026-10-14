#!/usr/bin/env python3
"""Run the frozen TKBO-only gate; retain complete logs, payloads and tracked-input bookends."""
import ast
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time

ROOT = Path.cwd()
OUT = Path(sys.argv[1]).resolve()
OUT.mkdir(parents=True, exist_ok=True)
ENV = dict(os.environ, CARGO_BUILD_JOBS="2", CARGO_TARGET_DIR="/tmp/martensite-annular-orientation-target")
CARGO = [str(Path.home() / ".cargo/bin/cargo"), "+1.99.0"]


def capture(arguments):
    return subprocess.check_output(arguments, cwd=ROOT, env=ENV, text=True).strip()


def seal():
    paths = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).split(b"\0")
    return {os.fsdecode(p): hashlib.sha256((ROOT / os.fsdecode(p)).read_bytes()).hexdigest()
            for p in paths if p and (ROOT / os.fsdecode(p)).is_file()}


def retain(label, content):
    (OUT / (label + ".gz")).write_bytes(gzip.compress(content, mtime=0))


def run(label, arguments):
    started = time.monotonic()
    with (OUT / (label + ".log")).open("wb") as log:
        result = subprocess.run(arguments, cwd=ROOT, env=ENV, stdout=log, stderr=subprocess.STDOUT)
    text = (OUT / (label + ".log")).read_text()
    retain(label + ".log", text.encode())
    checks.append(dict(label=label, command=arguments, exit=result.returncode,
                       seconds=time.monotonic() - started, summaries=re.findall(r"test result:.*", text)))
    print(label, result.returncode, flush=True)
    if result.returncode:
        raise RuntimeError(f"{label} failed: {OUT / (label + '.log')}")
    return text


def payload(text, marker, kind):
    value = text.split(marker, 1)[1].splitlines()[0]
    if kind == "hex":
        return bytes.fromhex(value)
    if kind == "array":
        return bytes(ast.literal_eval(value))
    return (marker + value).encode()


def compare_receipt(binary, name, selector, marker, kind, baseline=None):
    results = []
    for index in (1, 2):
        text = run(f"{name}-{index}", [binary, selector, "--exact", "--nocapture", "--test-threads=1"])
        result = payload(text, marker, kind)
        retain(f"{name}-{index}.bin", result)
        results.append(result)
    assert results[0] == results[1], name
    if baseline:
        assert results[0] == gzip.decompress((ROOT / baseline).read_bytes()), name
    replays.append(dict(name=name, bytes=len(results[0]), sha256=hashlib.sha256(results[0]).hexdigest(),
                        independent_processes=2, retained_baseline=baseline))


def replay_gate(binary):
    prefix = "sketch_region_prism::"
    compare_receipt(binary, "orientation-v1", prefix + "region_certificate::orientation::tests::replay_tests::prism_orientation_process_receipt",
                    '{"operation":"martensite.prism-orientation"', "text", BASE + "baseline-orientation-1.bin.gz")
    compare_receipt(binary, "joined-orientation-v1", prefix + "frozen::joined_orientation::tests::replay_tests::joined_orientation_process_receipt",
                    "joined-orientation-v1:", "hex", BASE + "baseline-joined-orientation-1.bin.gz")
    compare_receipt(binary, "publication-v1", prefix + "publication::replay_tests::publication_process_replay",
                    "named-prism-publication-v1=", "hex", BASE + "baseline-publication-1.bin.gz")
    compare_receipt(binary, "annular-join-v2", prefix + "frozen::annular_join::replay_tests::annular_frozen_join_owned_process_receipt",
                    "named-annular-frozen-domain-v2=", "hex")
    compare_receipt(binary, "annular-orientation-v2", prefix + "frozen::annular_orientation::lifecycle_tests::annular_orientation_replay_probe",
                    "ANNULAR_ORIENTATION_REPLAY=", "array")


BASE = "docs/evidence/prism-annular-orientation-v2/shared-signs/"
checks, replays = [], []
before = seal()
revision = capture(["git", "rev-parse", "HEAD"])
assert capture(["git", "status", "--porcelain"]) == "", "freeze tracked and untracked source before gate"
retain("source-before.json", json.dumps(before, sort_keys=True).encode())
receipt = dict(source=revision, platform=platform.platform(), compiler=capture([str(Path.home()/".cargo/bin/rustc"), "+1.99.0", "-Vv"]),
               jobs=2, target=ENV["CARGO_TARGET_DIR"], scope="TKBO all targets/features, docs, strict Clippy/fmt; unchanged TKMath/TKBRep not separately rerun",
               checks=checks, replays=replays, tsan="Not executed: retained host startup limitation; no sanitizer acceptance claim")
try:
    run("clippy", CARGO + ["clippy", "-p", "rustycad-tkbo", "--all-targets", "--all-features", "--", "-D", "warnings"])
    text = run("tests", CARGO + ["test", "-p", "rustycad-tkbo", "--all-targets", "--all-features"])
    binary = re.search(r"Running unittests src/lib.rs \(([^)]+)\)", text).group(1)
    run("doctests", CARGO + ["test", "-p", "rustycad-tkbo", "--doc", "--all-features"])
    run("docs", CARGO + ["doc", "-p", "rustycad-tkbo", "--no-deps", "--all-features"])
    run("fmt", CARGO + ["fmt", "--all", "--", "--check"])
    replay_gate(binary)
finally:
    after = seal()
    retain("source-after.json", json.dumps(after, sort_keys=True).encode())
    receipt["tracked_input_bookends_equal"] = before == after
    receipt["input_files"] = len(before)
    (OUT / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
assert before == after, "tracked inputs changed during frozen gate"

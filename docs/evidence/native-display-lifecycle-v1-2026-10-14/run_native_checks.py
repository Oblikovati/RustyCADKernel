"""Replay the accepted 15 window/GUI cases and six display lifetime cases."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import time

PREVIOUS = Path("docs/evidence/native-window-lifecycle-v1-2026-10-14/run_native_checks.py")
EXAMPLES = ("native_gui_rapid", "native_gui_acceptance", "native_window_lifecycle", "native_display_lifecycle")


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")


def source_snapshot() -> dict[str, object]:
    names = subprocess.check_output(["git", "ls-files", "-z"]).decode().split("\0")
    hashes = {name: hashlib.sha256(Path(name).read_bytes()).hexdigest() for name in names if name}
    return {"revision": subprocess.check_output(["git", "rev-parse", "HEAD"]).decode().strip(), "files": hashes}


def executable_snapshot() -> dict[str, str]:
    return {name: hashlib.sha256((Path("target/debug/examples") / name).read_bytes()).hexdigest() for name in EXAMPLES}


def run_display_case(directory: Path, driver: str, icd: str, index: int) -> dict[str, object]:
    destination = directory / f"{driver}-display-lifecycle-{index}"
    destination.mkdir()
    environment = json.loads((directory / "runs.json").read_text())[0]["environment"]
    environment.update(VK_DRIVER_FILES=f"/usr/share/vulkan/icd.d/{icd}_icd.json", VK_ICD_FILENAMES=f"/usr/share/vulkan/icd.d/{icd}_icd.json")
    command = [str(Path("target/debug/examples/native_display_lifecycle").resolve())]
    start = time.monotonic()
    with (destination / "run.log").open("w") as log:
        try:
            code = subprocess.run(command, env={**os.environ, **environment}, stdout=log, stderr=subprocess.STDOUT, timeout=45).returncode
        except subprocess.TimeoutExpired:
            log.write("\nTimed out after 45 seconds.\n")
            code = 124
    return {"name": destination.name, "command": command, "environment": environment, "exit": code, "wall_seconds_not_performance": time.monotonic() - start}


def replay_display_cases(directory: Path) -> None:
    runs = json.loads((directory / "runs.json").read_text())
    for driver, icd in (("amd", "radeon"), ("lavapipe", "lvp")):
        for index in range(3):
            record = run_display_case(directory, driver, icd, index)
            runs.append(record)
            write_json(directory / "runs.json", runs)
            print(json.dumps(record), flush=True)
            if record["exit"]:
                raise SystemExit(record["exit"])


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("received wrong argument count; expected one new output directory")
    output = Path(sys.argv[1])
    output.mkdir(parents=True)
    write_json(output / "source-before.json", source_snapshot())
    write_json(output / "executables-before.json", executable_snapshot())
    try:
        subprocess.run([sys.executable, str(PREVIOUS), str(output / "native")], check=True)
        replay_display_cases(output / "native")
    finally:
        write_json(output / "source-after.json", source_snapshot())
        write_json(output / "executables-after.json", executable_snapshot())


if __name__ == "__main__":
    main()

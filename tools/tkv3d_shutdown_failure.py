"""Compare the process-fatal virtual Clear callback in OCCT's noexcept destructor."""
from __future__ import annotations
import json
import resource
import signal
import subprocess
from pathlib import Path


def verify_shutdown_failure(evidence: Path, rust_probe: Path) -> None:
    """Require both real probes to abort; verify_shutdown_failure(output, rust_executable)."""
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    request = "v3_ais_context_shutdown 1 1 2\n"
    statuses = {}
    for name, executable in (("native", evidence / "native-probe"), ("rust", rust_probe)):
        result = subprocess.run([str(executable)], input=request, text=True, capture_output=True, timeout=30)
        statuses[name] = result.returncode
        if result.returncode != -signal.SIGABRT:
            raise RuntimeError(f"{name} destructor failure returned {result.returncode}; expected SIGABRT {-signal.SIGABRT}")
    report_path = evidence / "report.json"
    report = json.loads(report_path.read_text())
    report["shutdown_failure"] = {"request": request.strip(), "return_codes": statuses}
    report_path.write_text(json.dumps(report, indent=2) + "\n")
    print("TKV3d: native and Rust noexcept Clear failures both abort")

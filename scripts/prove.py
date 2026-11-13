"""Acceptance proof through the actual extended ruvnet CLI and actual UI binary."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
RUNE = ROOT / "target/rune/release/rune"
UI = ROOT / "target/rune/release/rune-inbox"


def invoke(scenario, expect_success=True):
    command = [str(RUNE), "ui", "--scenario", str(ROOT / scenario)]
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, timeout=180)
    if (result.returncode == 0) != expect_success:
        raise AssertionError(f"unexpected exit {result.returncode}: {result.stderr}\n{result.stdout}")
    if expect_success:
        return json.loads(result.stdout)
    return result.stderr.strip()


def main():
    checks = []
    for scenario in ["fixtures/archive.json", "fixtures/reset.json"]:
        report = invoke(scenario)
        assert report["status"] == "passed"
        checks.append({"check": scenario, "passed": True, "evidence": report["evidence_dir"]})
    for scenario, reason in [("fixtures/reject-postcondition.json", "expected"),
                             ("fixtures/reject-target.json", "missing or ambiguous")]:
        failure = invoke(scenario, False)
        assert reason in failure, failure
        checks.append({"check": scenario, "passed": True, "rejection": failure})
    original = UI.read_bytes()
    try:
        UI.write_bytes(original + b"tampered")
        failure = invoke("fixtures/archive.json", False)
        assert "artifact digest" in failure, failure
        checks.append({"check": "reject-tampered-executable", "passed": True})
    finally:
        UI.write_bytes(original)
    source = ROOT / "crates/rune-inbox/src/lib.rs"
    original = source.read_bytes()
    try:
        source.write_bytes(original + b"\n// stale-plan rejection probe\n")
        failure = invoke("fixtures/archive.json", False)
        assert "source_digest" in failure, failure
        checks.append({"check": "reject-stale-source", "passed": True})
    finally:
        source.write_bytes(original)
    summary = {"schema": "double-rune.acceptance/1", "status": "passed", "checks": checks,
               "controller_sha256": hashlib.sha256(RUNE.read_bytes()).hexdigest(),
               "ui_sha256": hashlib.sha256(UI.read_bytes()).hexdigest(),
               "runtime": "native Linux x86-64; headless GPU rendering",
               "model_calls": 0}
    destination = ROOT / ".rune/proof-summary.json"
    destination.write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()

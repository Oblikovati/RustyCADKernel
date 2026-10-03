"""Verify planned OCCT contexts rerun without touching source retry artifacts."""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import tempfile
import unittest

from tools.api_inventory.inputs import digest
from tools.api_inventory.context_replay_plan import make_plan
from tools.api_inventory.context_replay_run import replay_plan


class FakeOcctExtractor:
    def create(self, path: pathlib.Path, marker: str) -> pathlib.Path:
        path.write_text(
            "#!/usr/bin/env python3\n"
            "import json, pathlib, sys\n"
            "target = pathlib.Path(sys.argv[sys.argv.index('--emit-header') + 1])\n"
            f"missing = {marker == 'missing-header'!r}\n"
            "target = target.with_name('Other.hxx') if missing else target\n"
            "print(json.dumps({'event':'header_seen','source':str(target)}))\n"
            "print(json.dumps({'event':'declaration','source':str(target)}))\n"
            f"# {marker}\n"
        )
        path.chmod(0o755)
        return path


class HeaderContextReplayTests(unittest.TestCase):
    def test_rerun_uses_new_capture_and_preserves_prior_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = ReplayFixture(pathlib.Path(temporary))
            plan, new_capture, new_extractor, old_artifact = fixture.build()
            prior_sha = digest(old_artifact)
            replay_root = fixture.root / "fresh-replay"
            result = replay_plan(
                plan,
                new_capture,
                fixture.source,
                fixture.database,
                new_extractor,
                replay_root,
            )
            self.assertEqual(result["parsed_with_context"], 1)
            replay_row = result["retry_results"][0]
            self.assertTrue(replay_row["target_header_seen"])
            self.assertNotEqual(replay_row["artifact"], str(old_artifact))
            self.assertEqual(digest(old_artifact), prior_sha)
            self.assertEqual(result["original_capture_sha256"], digest(new_capture))

    def test_standalone_parsed_target_is_recorded_without_context_retry(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = ReplayFixture(pathlib.Path(temporary))
            plan, capture, extractor, _ = fixture.build()
            artifact = fixture.root / "standalone.jsonl"
            fixture._write_target_artifact(artifact)
            capture_data = json.loads(capture.read_text())
            capture_data["results"][0].update(
                {
                    "status": "parsed",
                    "returncode": 0,
                    "artifact": str(artifact),
                    "output_sha256": digest(artifact),
                }
            )
            capture.write_text(json.dumps(capture_data))
            replay = replay_plan(
                plan,
                capture,
                fixture.source,
                fixture.database,
                extractor,
                fixture.root / "parsed-replay",
            )
            self.assertEqual(replay["retry_count"], 0)
            self.assertEqual(replay["already_parsed_count"], 1)
            self.assertEqual(
                replay["already_parsed"][0]["status"],
                "already_parsed_in_standalone_capture",
            )

    def test_missing_target_header_is_recorded_as_failed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = ReplayFixture(pathlib.Path(temporary))
            plan, capture, extractor, _ = fixture.build(
                "missing-header", "parse_failed"
            )
            replay = replay_plan(
                plan,
                capture,
                fixture.source,
                fixture.database,
                extractor,
                fixture.root / "failed-replay",
            )
            self.assertEqual(replay["parsed_with_context"], 0)
            self.assertFalse(replay["retry_results"][0]["target_header_seen"])


class ReplayFixture:
    def __init__(self, root: pathlib.Path) -> None:
        self.root = root
        self.source = root / "OCCT"
        self.header = self.source / "src/Module/TKExample/Shape.hxx"
        self.tu = self.source / "src/Module/TKExample/Shape.cxx"
        self.database = root / "compile_commands.json"
        self.fake = FakeOcctExtractor()

    def build(
        self, marker: str = "new", prior_status: str = "parse_failed"
    ) -> tuple[pathlib.Path, pathlib.Path, pathlib.Path, pathlib.Path]:
        self._source_checkout()
        self._compile_database()
        old_extractor = self.fake.create(self.root / "extract-old", "old")
        old_capture = self._capture("old-capture", old_extractor)
        old_artifact = self._prior_artifact()
        retry = self._prior_retry(
            old_capture, old_extractor, old_artifact, prior_status
        )
        retry_path = self.root / "retries.json"
        retry_path.write_text(json.dumps(retry))
        plan_path = self.root / "portable-plan.json"
        plan_path.write_text(
            json.dumps(
                make_plan(
                    retry_path, old_capture, self.source, self.database, old_extractor
                )
            )
        )
        new_extractor = self.fake.create(self.root / "extract-new", marker)
        new_capture = self._capture("new-capture", new_extractor)
        return plan_path, new_capture, new_extractor, old_artifact

    def _source_checkout(self) -> None:
        self.header.parent.mkdir(parents=True)
        self.header.write_text("struct Shape {};\n")
        self.tu.write_text('#include "Shape.hxx"\n')
        subprocess.run(["git", "init", "-q", str(self.source)], check=True)
        subprocess.run(["git", "-C", str(self.source), "add", "src"], check=True)
        subprocess.run(
            [
                "git",
                "-C",
                str(self.source),
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "source",
            ],
            check=True,
        )

    def _compile_database(self) -> None:
        command = f"clang++ -std=c++17 -I{self.source / 'src'} -c {self.tu} -o {self.root / 'shape.o'}"
        self.database.write_text(
            json.dumps(
                [
                    {
                        "directory": str(self.root),
                        "file": str(self.tu),
                        "command": command,
                    }
                ]
            )
        )

    def _capture(self, fingerprint: str, extractor: pathlib.Path) -> pathlib.Path:
        path = self.root / f"{fingerprint}.json"
        manifest = self.root / "header-manifest.json"
        manifest.write_text(json.dumps({"revision": self._revision()}))
        path.write_text(
            json.dumps(
                {
                    "fingerprint": fingerprint,
                    "inputs": {
                        "database_sha256": digest(self.database),
                        "extractor_sha256": digest(extractor),
                        "manifest_sha256": digest(manifest),
                        "file_inputs": {"manifest": str(manifest)},
                    },
                    "results": [
                        {
                            "path": "src/Module/TKExample/Shape.hxx",
                            "status": "parse_failed",
                            "toolkit": "TKExample",
                        }
                    ],
                }
            )
        )
        return path

    def _revision(self) -> str:
        result = subprocess.run(
            ["git", "-C", str(self.source), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()

    def _prior_artifact(self) -> pathlib.Path:
        path = self.root / "old-context.jsonl"
        self._write_target_artifact(path)
        return path

    def _write_target_artifact(self, path: pathlib.Path) -> None:
        header = str(self.header.resolve())
        path.write_text(
            json.dumps({"event": "header_seen", "source": header})
            + "\n"
            + json.dumps({"event": "declaration", "source": header})
            + "\n"
        )

    def _prior_retry(
        self,
        capture: pathlib.Path,
        extractor: pathlib.Path,
        artifact: pathlib.Path,
        status: str,
    ) -> dict[str, object]:
        source_relative = self.tu.relative_to(self.source).as_posix()
        header_relative = self.header.relative_to(self.source).as_posix()
        flags = ("-std=c++17", f"-I{self.source / 'src'}")
        flags_hash = hashlib.sha256(json.dumps(flags).encode()).hexdigest()
        return {
            "original_capture_sha256": digest(capture),
            "original_fingerprint": "old-capture",
            "database_sha256": digest(self.database),
            "extractor_sha256": digest(extractor),
            "retry_results": [
                {
                    "path": header_relative,
                    "toolkit": "TKExample",
                    "status": status,
                    "returncode": 3 if status == "parse_failed" else 0,
                    "context_evidence": "transitive source include graph",
                    "compile_source": str(self.tu),
                    "compile_source_sha256": digest(self.tu),
                    "compile_directory": str(self.root),
                    "compile_flags": list(flags),
                    "compile_flags_sha256": flags_hash,
                    "include_name": "Shape.hxx",
                    "header_sha256": digest(self.header),
                    "target_header_seen": True,
                    "target_declarations": 1,
                    "artifact": str(artifact),
                    "output_sha256": digest(artifact),
                }
            ],
        }


if __name__ == "__main__":
    unittest.main()

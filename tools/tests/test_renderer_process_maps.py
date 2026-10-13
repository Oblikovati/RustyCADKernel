"""Actual-process library observations preserve red exits without guessing loader paths."""

from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch
import renderer_process_maps as maps


class NamedMonitoredChild:
    pid = 12345

    def __init__(self) -> None:
        self.calls = 0

    def communicate(self, timeout: float) -> tuple[str, str]:
        self.calls += 1
        if self.calls == 1:
            raise subprocess.TimeoutExpired(["named-child"], timeout)
        return "retained stdout", "retained stderr"


class NamedMapReader:
    def __init__(self) -> None:
        self.pids: list[int] = []

    def read(self, pid: int) -> set[Path]:
        self.pids.append(pid)
        return {Path("/named/libLLVM.so.20.1")}


class ProcessMapTests(unittest.TestCase):
    def test_only_live_absolute_shared_objects_are_selected(self) -> None:
        snapshot = (
            "1-2 r-xp 0 1:1 1 /named/libm.so.6\n"
            "1-2 r-xp 0 1:1 1 /named/deleted.so (deleted)\n"
            "1-2 rw-p 0 0:0 0 [stack]\n"
            "1-2 r-xp 0 1:1 1 /named/program\n"
        )
        self.assertEqual(maps.library_map_paths(snapshot), {Path("/named/libm.so.6")})

    def test_process_completion_keeps_pipe_output_and_each_observation(self) -> None:
        child, reader = NamedMonitoredChild(), NamedMapReader()
        with patch.object(maps, "sample_process_maps", reader.read):
            stdout, stderr, observed = maps.complete_observed_process(child)
        self.assertEqual((stdout, stderr), ("retained stdout", "retained stderr"))
        self.assertEqual(observed, {Path("/named/libLLVM.so.20.1")})
        self.assertEqual(reader.pids, [12345, 12345])

    def test_retired_owned_process_has_no_invented_library_observation(self) -> None:
        self.assertEqual(maps.sample_process_maps(999999999), set())


if __name__ == "__main__":
    unittest.main()

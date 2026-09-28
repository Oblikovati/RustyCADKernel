"""The shared native build is configured through one path and one lock."""
import fcntl
import tempfile
import unittest
from pathlib import Path
from native_build import lock_path, locked_cmake


class RecordingCmakeRunner:
    """Record cmake calls and whether the build lock is held during each call."""

    def __init__(self, build: Path):
        self.lock = lock_path(build)
        self.calls: list[tuple[list[str], bool]] = []

    def __call__(self, args, log, *, env=None):
        self.calls.append(([str(a) for a in args], self.lock_is_held()))
        return ''

    def lock_is_held(self) -> bool:
        with self.lock.open('w') as probe:
            try:
                fcntl.flock(probe, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError:
                return True
            fcntl.flock(probe, fcntl.LOCK_UN)
            return False


class LockedCmakeTests(unittest.TestCase):
    def test_symlinked_build_is_configured_through_its_real_path_under_lock(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            real = root/'shared/occt'
            real.mkdir(parents=True)
            link = root/'worktree-occt'
            link.symlink_to(real)
            runner = RecordingCmakeRunner(real)
            built = locked_cmake(root/'src', link, ['-DX=1'], ['TKernel', 'gtest_main'], 4,
                                 root/'c.log', root/'b.log', runner, {})
            self.assertEqual(built, real)
            configure, build = runner.calls
            self.assertEqual(configure, (['cmake', '-S', str(root/'src'), '-B', str(real), '-DX=1'], True))
            self.assertEqual(build, (['cmake', '--build', str(real), '--target', 'TKernel', 'gtest_main', '--parallel', '4'], True))
            self.assertFalse(runner.lock_is_held())

    def test_lock_file_sits_beside_the_build(self):
        self.assertEqual(lock_path(Path('/b/occt-tkernel-full')), Path('/b/.occt-tkernel-full.lock'))


if __name__ == '__main__':
    unittest.main()

"""The DRAW oracle tool resolves the native build before it changes directory."""
import os
import tempfile
import unittest
from pathlib import Path

from verify_tkdraw_oracle import absolute_native_build


class AbsoluteNativeBuildTests(unittest.TestCase):
    def test_a_relative_build_resolves_against_the_caller_directory(self):
        caller = Path.cwd()
        with tempfile.TemporaryDirectory() as directory:
            os.chdir(directory)
            try:
                build = absolute_native_build(Path("target/occt-all-full"))
            finally:
                os.chdir(caller)
            self.assertTrue(build.is_absolute())
            self.assertEqual(build, Path(directory).resolve() / "target/occt-all-full")


if __name__ == "__main__":
    unittest.main()

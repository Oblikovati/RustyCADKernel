"""Select legacy Rust case modules after toolkit consolidation.

Example: cargo_case_command(root, 'rustycad-tkernel', 'standard_guid', ['--list']).
"""

from __future__ import annotations

from functools import lru_cache
from pathlib import Path
import re
import tomllib

from toolkit_test_suites import suite_sources


@lru_cache(maxsize=8)
def package_manifests(root: Path) -> dict[str, Path]:
    """Resolve package identifiers without assuming naming conventions; package_manifests(root)."""
    return {tomllib.loads(path.read_text())['package']['name']: path
            for path in (root / 'crates').glob('*/Cargo.toml')}


def compiled_case_target(root: Path, package: str, target: str) -> str:
    """Keep isolated targets and select toolkit modules; compiled_case_target(root, package, target)."""
    manifest = package_manifests(root).get(package)
    if manifest is None:
        return target
    configuration = tomllib.loads(manifest.read_text())
    targets = {entry['name'] for entry in configuration.get('test', [])}
    source = manifest.parent / 'tests' / f'{target}.rs'
    nested = manifest.parent / 'tests' / target / 'main.rs'
    if 'toolkit' in targets and target not in targets and (source.is_file() or nested.is_file()):
        return 'toolkit'
    return target


def cargo_case_command(root: Path, package: str, target: str, flags: list[str]) -> list[str]:
    """Run one original case module with strict namespace filtering; cargo_case_command(root, p, t, [])."""
    compiled = compiled_case_target(root, package, target)
    filtering = [f'{target}::'] if compiled != target else []
    if filtering:
        sources = suite_sources(package_manifests(root)[package])
        namespaces = {source.parent.name if source.name == 'main.rs' else source.stem for source in sources}
        # libtest uses substring filters, so a suffix collision must be explicitly excluded.
        filtering += [argument for name in sorted(namespaces) if name != target and name.endswith(target)
                      for argument in ['--skip', f'{name}::']]
    return ['cargo', 'test', '--locked', '-p', package, '--test', compiled, '--', *filtering, *flags]


# One libtest case: its name, then everything printed until the next case, a blank line or the
# end. A case that writes straight to stdout (OCCT message printers, e.g. Font_FontMgr
# warnings) splits `test name ... ok` across lines, so the status is the chunk's last word.
TEST_CASE_LINE = re.compile(r'^test (\S+) \.\.\. (.*?)(?=^test |^\s*$|\Z)', re.M | re.S)


def passed_case_names(log: str) -> set[str]:
    """Names of the cases libtest reports ok, even when a case printed between its name and
    its status; passed_case_names('test a ... warn\\nok\\n') == {'a'}."""
    return {match.group(1) for match in TEST_CASE_LINE.finditer(log)
            if match.group(2).rstrip().endswith('ok')}


def original_case_names(names: set[str], target: str) -> set[str]:
    """Restore original mapped identities without dropping unexpected cases; original_case_names(names, target)."""
    prefix = f'{target}::'
    return {name.removeprefix(prefix) for name in names}

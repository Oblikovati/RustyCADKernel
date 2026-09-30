"""Reproduce pinned TKHLR's declared-but-undefined Axis member.

Usage: verify_missing_axis(build, output) before the TKHLR numeric gate.
"""
import json
from pathlib import Path
import re
import subprocess
from porting import ROOT
from verify_tkmath_numeric import probe_environment, run


def validate_axis_failure(returncode: int, diagnostic: str) -> None:
    """Accept only the expected missing member; validate_axis_failure(1, linker_stderr)."""
    references = re.findall(r"undefined reference to [`']([^'\n]+)'", diagnostic)
    expected = ['HLRBRep_BSurfaceTool::Axis(BRepAdaptor_Surface const&)']
    if returncode == 0 or references != expected:
        raise RuntimeError(f'Expected failed link for {expected!r}; exit={returncode}, diagnostic={diagnostic!r}')
    permitted = ('in function', 'undefined reference', 'ld returned 1 exit status')
    unexpected = [line for line in diagnostic.splitlines() if not any(text in line for text in permitted)]
    if unexpected:
        raise RuntimeError(f'Expected only Axis missing-symbol diagnostics; unexpected={unexpected!r}')


def verify_missing_axis(build: Path, output: Path) -> None:
    """Compile the declaration, require the exact link failure, and retain evidence; verify_missing_axis(build, out)."""
    output.mkdir(parents=True, exist_ok=True)
    report = output/'axis-gap.json'
    report.unlink(missing_ok=True)
    object_file = output/'missing-axis.o'
    run(['c++', '-std=c++17', '-I'+str(build/'include/opencascade'), '-c',
         str(ROOT/'tools/oracle/tkhlr_bsurface_missing_axis.cxx'), '-o', str(object_file)], output/'axis-compile.log')
    libraries = [next(build.rglob('lib'+name+'.so')) for name in ['TKHLR', 'TKBRep', 'TKG3d', 'TKMath', 'TKernel']]
    command = ['c++', str(object_file), *map(str, libraries), '-Wl,-rpath,'+str(libraries[0].parent),
               '-o', str(output/'missing-axis')]
    result = subprocess.run(command, text=True, capture_output=True, env=probe_environment(), check=False)
    diagnostic = result.stdout+result.stderr
    (output/'axis-link.log').write_text(diagnostic)
    validate_axis_failure(result.returncode, diagnostic)
    report.write_text(json.dumps({'upstream_id': 'HLRBRep_BSurfaceTool::Axis(BRepAdaptor_Surface const&)',
        'status': 'upstream_unimplemented', 'object_compiled': True, 'link_exit': result.returncode}, indent=2)+'\n')

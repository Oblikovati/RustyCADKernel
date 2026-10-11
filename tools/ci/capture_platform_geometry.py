#!/usr/bin/env python3
"""Keep independent native and Rust geometry captures for CI diagnosis.

Example: python3 tools/ci/capture_platform_geometry.py
"""
from pathlib import Path
import hashlib
import json
import os
import platform
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_target_selection import cargo_case_command


def probe_compiler_flags() -> str:
    """Make probe arithmetic obey the library policy; probe_compiler_flags()."""
    if os.name == 'nt':
        return '/fp:precise'
    policy = os.environ.get('MARTENSITE_FP_CONTRACT', 'off')
    if policy not in ('off', 'on', 'fast'):
        raise ValueError(f'contraction {policy!r}: expected off, on, or fast')
    return f'-ffp-contract={policy}'


def write_probe_project(project: Path, name: str, source: Path, include: Path, libraries: list[Path]) -> None:
    """Keep the native probe separate from Rust; write_probe_project(project, name, source, inc, libs)."""
    cmake = ['cmake_minimum_required(VERSION 3.20)', 'project(MartensiteProbe LANGUAGES CXX)', 'set(CMAKE_CXX_STANDARD 17)', f'set(CMAKE_CXX_FLAGS "{probe_compiler_flags()}")', f'add_executable(native-{name} "{source.as_posix()}")', f'target_include_directories(native-{name} PRIVATE "{include.as_posix()}")', f'target_link_libraries(native-{name} PRIVATE ' + ' '.join(f'"{library.as_posix()}"' for library in libraries) + ')']
    (project / 'CMakeLists.txt').write_text('\n'.join(cmake) + '\n')


def capture_probe(root: Path, output: Path, name: str, source: Path) -> bytes:
    """Build against the pinned host libraries; capture_probe(root, out, 'torus', source)."""
    build = root / 'target/native-platform'
    extension = '.lib' if os.name == 'nt' else '.dylib' if platform.system() == 'Darwin' else '.so'
    names = (('TKBO ' if name not in ('cylinder', 'memory') else '') + 'TKMesh TKPrim TKShHealing TKTopAlgo TKGeomAlgo TKBRep TKGeomBase TKG3d TKG2d TKMath TKernel').split()
    libraries = [next(build.rglob(('' if os.name == 'nt' else 'lib') + toolkit + extension)) for toolkit in names]
    include = next(build.rglob('BRepAlgoAPI_Fuse.hxx')).parent
    project = output / name
    project.mkdir(parents=True, exist_ok=True)
    write_probe_project(project, name, source, include, libraries)
    subprocess.run(['cmake', '-S', str(project), '-B', str(project / 'build')], check=True)
    subprocess.run(['cmake', '--build', str(project / 'build'), '--config', 'RelWithDebInfo'], check=True)
    executable = next((project / 'build').rglob(f'native-{name}' + ('.exe' if os.name == 'nt' else '')))
    env = dict(os.environ)
    if os.name == 'nt':
        env['PATH'] = os.pathsep.join({str(path.parent) for path in build.rglob('*.dll')}) + os.pathsep + env['PATH']
    return subprocess.check_output([str(executable)], env=env)


def capture_rust(root: Path, output: Path, package: str, target: str, marker: str, addition: str) -> int:
    """Preserve the original failing assertions and restore source; capture_rust(root,out,pkg,target,marker,addition)."""
    source = root / f'crates/{package}/tests/{target}.rs'
    original = source.read_text()
    if original.count(marker) != 1:
        raise ValueError(f'{target}: marker count {original.count(marker)}; expected one')
    source.write_text(original.replace(marker, addition + '\n' + marker))
    try:
        with (output / f'{target}.log').open('w') as log:
            return subprocess.run(cargo_case_command(root, f'rustycad-{package}', target, ['--nocapture']), stdout=log, stderr=subprocess.STDOUT, cwd=root).returncode
    finally:
        source.write_text(original)


def capture_cylinder_faces(root: Path, output: Path) -> int:
    """Print all face hashes before retaining strict assertions; capture_cylinder_faces(root, out)."""
    addition = '''    let mut captured_faces = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    for captured_index in 0..3 {
        let captured_face = rustycad_tkbrep::topods::face(captured_faces.current()).unwrap();
        let captured_mesh = BRepTool::triangulation(&captured_face, mesh_purpose::NONE).unwrap().0;
        println!("MARTENSITE_CYLINDER_FACE={} NODES={:016x} TRIANGLES={:016x}", captured_index, cylinder_node_hash(&captured_mesh), cylinder_triangle_hash(&captured_mesh));
        captured_faces.next().unwrap();
    }'''
    return capture_rust(root, output, 'tkmesh', 'brep_mesh_planar_face_discret', '    let mut faces = Explorer::new', addition)


def capture_boolean_snapshots(root: Path, output: Path) -> dict[str, int]:
    """Capture each native snapshot before Rust diagnostics; capture_boolean_snapshots(root, out)."""
    probes = [('torus', 'tkbo_bop_torus', 'bop_algo_bop_torus'), ('cells_builder', 'tkbo_cells_builder', 'bop_algo_cells_builder'), ('edges_to_wires', 'tkbo_edges_to_wires', 'bop_algo_edges_to_wires'), ('defeaturing', 'tkbo_defeaturing', 'brep_algo_api_defeaturing')]
    statuses: dict[str, int] = {}
    for name, probe, target in probes:
        native = capture_probe(root, output, name, root / 'tools/oracle' / f'{probe}.cxx')
        (output / f'{name}.expected').write_bytes(native)
        statuses[name] = capture_rust(root, output, 'tkbo', target, '    let expected = ', '    println!("MARTENSITE_SNAPSHOT_BEGIN\\n{rows}MARTENSITE_SNAPSHOT_END");')
    return statuses


def main() -> None:
    """Record raw independently computed results and their provenance; main()."""
    root = Path(__file__).resolve().parents[2]
    output = root / 'target/platform-geometry'
    output.mkdir(parents=True, exist_ok=True)
    statuses: dict[str, int] = {}
    (output / 'cylinder.expected').write_bytes(capture_probe(root, output, 'cylinder', root / 'tools/oracle/tkmesh_cylinder_nodes.cxx'))
    if os.environ.get('MARTENSITE_CYLINDER_ONLY') != 'true':
        statuses.update(capture_boolean_snapshots(root, output))
    statuses['cylinder'] = capture_rust(root, output, 'tkmesh', 'brep_mesh_cylinder_seam_order', '    assert_eq!(cylinder_node_hash', '    println!("MARTENSITE_CYLINDER_HASH={:016x}", cylinder_node_hash(&triangulation));')
    statuses['cylinder_faces'] = capture_cylinder_faces(root, output)
    # Darwin's failing cases use libc; UCRT aligned-zero invokes an invalid-parameter handler.
    if os.name != 'nt':
        (output / 'memory.expected').write_bytes(capture_probe(root, output, 'memory', root / 'tools/oracle/tkernel_standard_memory_storage.cxx'))
    statuses['memory'] = capture_rust(root, output, 'tkernel', 'standard_memory_storage', '    let kind = memory::get_allocator_type()', '    println!("MARTENSITE_NATIVE_MEMORY_TEST");')
    receipt = {'probe_cpp_flags': probe_compiler_flags(), 'platform': platform.platform(), 'machine': platform.machine(), 'rust_tests': statuses, 'occt_revision': subprocess.check_output(['git', '-C', 'target/occt-source', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(), 'source_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(), 'sha256': {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in output.glob('*.expected')}}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    # These are diagnostic captures, not passing parity receipts: every original verdict remains above.
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()

"""Bounded native25 repeat/Vulkan25 controls use frozen release behavior, not inferred pixels."""
from pathlib import Path
import hashlib
import json
import os
import struct
import subprocess

ROOT: Path = Path('/home/vmiguel/git/oblikovati-workspace/RustyCADKernel')
FOLDER: Path = ROOT / 'target/clipping-root-audit/diagnostic-draft/profile-correction/full-family'
CAPTURES: Path = FOLDER / 'captures'
CAPTURES.mkdir(exist_ok=True)
GL_LIBRARY: Path = ROOT / 'target/release-runner-mesa/gl25/extracted/usr/lib/x86_64-linux-gnu'
GL_BINARY: Path = ROOT / 'target/release-runner-mesa/gl25'
STAGE: Path = ROOT / 'target/release-0.2.3-runner-replay'
ENVIRONMENT: dict[str, str] = dict(os.environ)
ENVIRONMENT.update(DISPLAY=':1', LIBGL_ALWAYS_SOFTWARE='1', GALLIUM_DRIVER='llvmpipe', GALLIUM_OVERRIDE_CPU_CAPS='sse2', LP_NATIVE_VECTOR_WIDTH='128', MARTENSITE_VULKAN_DEVICE='0', VK_DRIVER_FILES=str(ROOT / 'target/release-runner-mesa/isolated-lvp.json'), VK_LAYER_PATH=str(ROOT / 'target/vulkan-validation/extracted/usr/share/vulkan/explicit_layer.d'), LD_LIBRARY_PATH=str(GL_LIBRARY) + ':' + str(ROOT / 'target/vulkan-validation/extracted/usr/lib/x86_64-linux-gnu'), LIBGL_DRIVERS_PATH=str(GL_LIBRARY / 'dri'))
ENVIRONMENT.pop('LLVM_TARGET_FEATURES', None)
CASES: list[dict[str, str]] = json.loads((FOLDER / 'cases.json').read_text())
ROWS: list[dict[str, object]] = []
for index, case in enumerate(CASES):
    label: str = f"{case['family']}-{case['asset_name']}"
    commands: list[dict[str, object]] = []
    for kind in ('native', 'repeat', 'vulkan'):
        prefix: Path = CAPTURES / f'{label}-{kind}'
        binary: Path = FOLDER / 'frozen-ffp-family-capture' if kind == 'vulkan' else GL_BINARY / case['native_target']
        arguments: list[str] = [str(binary), case['family'], case['rust_mode'], str(prefix)] if kind == 'vulkan' else [str(binary), case['native_mode'], str(prefix)]
        with Path(str(prefix) + '.log').open('w') as log:
            result: subprocess.CompletedProcess[bytes] = subprocess.run(arguments, cwd=STAGE if kind == 'vulkan' else ROOT, env=ENVIRONMENT, stdout=log, stderr=subprocess.STDOUT, timeout=30)
        commands.append({'kind': kind, 'arguments': arguments, 'exit': result.returncode, 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest()})
    row: dict[str, object] = {'case': case, 'commands': commands}
    if all(command['exit'] == 0 for command in commands):
        colors: list[bytes] = [(CAPTURES / f'{label}-{kind}.rgba').read_bytes() for kind in ('native', 'repeat', 'vulkan')]
        depths: list[bytes] = [(CAPTURES / f'{label}-{kind}.depth').read_bytes() for kind in ('native', 'repeat', 'vulkan')]
        floats: list[tuple[float, ...]] = [struct.unpack('<4096f', payload) for payload in depths]
        delta: list[int] = [abs(a-b) for a, b in zip(colors[0], colors[2])]
        row.update(native_repeat=colors[0] == colors[1] and depths[0] == depths[1], rgba_differences=sum(value != 0 for value in delta), max_rgba_delta=max(delta), max_depth_delta=max(abs(a-b) for a, b in zip(floats[0], floats[2])))
    ROWS.append(row)
    (FOLDER / 'results.json').write_text(json.dumps({'environment': {name: ENVIRONMENT.get(name) for name in ('DISPLAY', 'LIBGL_ALWAYS_SOFTWARE', 'GALLIUM_DRIVER', 'GALLIUM_OVERRIDE_CPU_CAPS', 'LP_NATIVE_VECTOR_WIDTH', 'MARTENSITE_VULKAN_DEVICE', 'VK_DRIVER_FILES', 'VK_LAYER_PATH', 'LD_LIBRARY_PATH', 'LIBGL_DRIVERS_PATH', 'LLVM_TARGET_FEATURES')}, 'cases': ROWS}, indent=2))
    print(json.dumps({'index': index, 'mode': case['mode'], 'exits': [command['exit'] for command in commands], **{key: row[key] for key in ('rgba_differences', 'max_depth_delta') if key in row}}), flush=True)
raise SystemExit(0 if all(row.get('native_repeat') and row.get('rgba_differences') == 0 and float(row.get('max_depth_delta', 99)) <= 1e-6 for row in ROWS) else 1)

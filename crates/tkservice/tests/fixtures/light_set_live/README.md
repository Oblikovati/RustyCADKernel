# Pinned Martensite light-set CPU oracle

OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Generator: `tools/oracle/martensite_light_set_live.cxx`.
The six native files contain 96 sequential snapshots each, one per named filter.
The ordinary toolkit test concatenates these committed assets and replays the
same event stream in Rust. It never loads OCCT or regenerates assets.

Each row contains filter, action, source/index argument, value argument, then:
action result; primary cursor More/value; copied cursor More/value; set revision
and extent; four pairs of raw/enabled type counts; enabled non-ambient/shadow
counts; four ambient IEEE-754 float bit patterns; long/short shader keys; and
ordered stable source ordinals. Empty keys/membership use `-`. Values are stable
ordinals into the six-source array, not native addresses or process-wide IDs.
Exceptions map to `-11` NoSuchObject, `-12` OutOfRange, `-13` ProgramError and
`-14` NotImplemented. The generator catches NotImplemented before its native
ProgramError superclass to preserve exact exception identity.

The native protected skipFiltered hook is invoked by a subclass adapter. Other
operations call real Graphic3d_LightSet/CLight methods. The second cursor copies
native iterator state and receives independent Next calls. Source colors and
intensities exercise unclamped ambient accumulation and bit-exact cached state.
The script deliberately snapshots after every action without implicit cache
updates; only its `update` events call UpdateRevision.

Local regeneration from this checkout uses the existing pinned native build:

```sh
mkdir -p target/oracle
g++ -std=c++17 \
  -I ../RustyCADKernel/target/occt-all-full/include/opencascade \
  tools/oracle/martensite_light_set_live.cxx \
  -L ../RustyCADKernel/target/occt-all-full/lin64/gcc/libi \
  -Wl,-rpath,"$PWD/../RustyCADKernel/target/occt-all-full/lin64/gcc/libi" \
  -lTKService -lTKMath -lTKernel -o target/oracle/martensite-light-set-live
./target/oracle/martensite-light-set-live < \
  crates/tkservice/tests/fixtures/light_set_live/events.txt > target/light-set-native.txt
python3 - <<'PY'
from pathlib import Path
folder = Path('crates/tkservice/tests/fixtures/light_set_live')
rows = Path('target/light-set-native.txt').read_text().splitlines()
for filter in (0, 2, 4, 8, 6, 14):
    selected = [row for row in rows if row.split()[0] == str(filter)]
    (folder / f'native-{filter}.txt').write_text('\n'.join(selected) + '\n')
PY
```

`provenance.json` retains source, installed-header, generator, binary, library
and asset identities. Two native executions produced identical bytes. These
are Linux sequential CPU observations; no GPU context or other-platform oracle
was exercised. Storage retention after dropping a Rust set wrapper is covered
by a separate safety regression and is not inferred from native borrowed memory.

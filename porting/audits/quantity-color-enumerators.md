# Individual named-color API mappings

At Rust source `d186ea3c`, all 509 canonical `Quantity_NameOfColor` values and
12 aliases already have public `NamedColor` associated constants. Their individual
C++ declarations were absent from the API ledger. This audit adds those 521 mappings;
it adds no implementation and no original legacy test declaration.

`quantity-color-enumerators.jsonl` records each compiler declaration ID, native header
hash, public Rust path, Rust source hash, exact initializer, and integer value. Each
path had exactly one associated-constant match in the published Rust inventory;
each C++ name had exactly one enum-constant declaration. Source hashes were checked
against disk. Rustdoc sometimes substitutes `_` for constant initializers, so values
were read from the corresponding hash-verified Rust source, not inferred from that
placeholder.

A GCC 13.3.0 C++17 program compiled against the pinned OCCT headers printed the
integer value of every native enumerator. Every value matched its Rust initializer
(`Self(integer)` or `Self::ALIAS_TARGET`). These are compiled native values compared
with static Rust initializers; this is not a new Rust runtime receipt or a proof of
complete color behavior. The existing table generator check also passed:

```sh
python3 tools/generate_quantity_color_table.py --source ../OCCT --check
```

Reproduce the native values independently of the Rust constants:

```sh
PYTHONPATH=tools python3 - <<'PY'
from pathlib import Path
from generate_quantity_color_table import enum_members
import subprocess
canonical, aliases = enum_members(Path('../OCCT'))
names = canonical + [name for name, target in aliases]
program = '#include <Quantity_NameOfColor.hxx>\n#include <iostream>\nint main(){\n'
program += ''.join(
    f'std::cout << "{name} " << int(Quantity_NOC_{name}) << "\\n";\n'
    for name in names
)
Path('/tmp/quantity_enum_values.cxx').write_text(program + '}\n')
subprocess.run([
    'c++', '-std=c++17', '-O2', '-Itarget/occt-modeling-full/include/opencascade',
    '/tmp/quantity_enum_values.cxx', '-o', '/tmp/quantity_enum_values'
], check=True)
subprocess.run(['/tmp/quantity_enum_values'], check=True)
PY
```

The ledger remains an implementation crosswalk. Full API, configuration, and
behavioral proof flags must remain false while unrelated gaps remain.

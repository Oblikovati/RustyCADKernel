"""Keep native private-helper translation units unchanged and record their provenance."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
from porting import ROOT


def private_translation_unit(original: Path,probe: Path,output: Path,build: Path,libraries: list[str]) -> Path:
    """Include the original source, then the observer, and record the exact build recipe."""
    output.mkdir(parents=True,exist_ok=True)
    wrapper=output/'native-translation-unit.cxx'
    wrapper.write_text('#include '+json.dumps(str(original))+'\n#include '+json.dumps(str(probe))+'\n')
    linked=[str(next(build.rglob('lib'+name+'.so'))) for name in libraries]
    command=['c++','-std=c++17','-O2','-I'+str(build/'include/opencascade'),str(wrapper),*linked,
             '-Wl,-rpath,'+str(Path(linked[0]).parent),'-o',str(output/'native-probe')]
    (output/'native-source.json').write_text(json.dumps(dict(source=str(original),source_sha256=hashlib.sha256(original.read_bytes()).hexdigest(),
        native_command=command,scope='private helpers; no complete API or legacy declaration claim'),indent=2)+'\n')
    return wrapper

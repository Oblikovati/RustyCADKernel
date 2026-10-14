#!/usr/bin/env python3
"""Reuse the committed gate's exact replay commands without rerunning the completed matrix."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

harness = Path('docs/evidence/prism-annular-orientation-v2/gate.py')
namespace = {}
exec(compile(harness.read_text().split('\nchecks, replays = [], []')[0], str(harness), 'exec'), namespace)
namespace['checks'], namespace['replays'] = [], []
compiled = 'db8706a75'
assert subprocess.check_output(['git','diff',compiled,'HEAD','--','crates','Cargo.toml','Cargo.lock']) == b''
binary = '/tmp/martensite-annular-orientation-target/debug/deps/rustycad_tkbo-3743221922cb8a91'
output = Path(sys.argv[1])
before = namespace['seal']()
namespace['retain']('source-before.json', json.dumps(before, sort_keys=True).encode())
receipt = {'compiled_source': subprocess.check_output(['git','rev-parse',compiled],text=True).strip(),
           'harness_revision': subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
           'harness_sha256': hashlib.sha256(harness.read_bytes()).hexdigest(),
           'supplement_runner_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
           'binary':binary,'binary_sha256':hashlib.sha256(Path(binary).read_bytes()).hexdigest(),
           'completed_matrix':'final/receipt.json','crate_and_cargo_diff_since_completed_matrix':'empty',
           'checks':namespace['checks'],'replays':namespace['replays']}
try:
    namespace['replay_gate'](binary)
finally:
    after = namespace['seal']()
    namespace['retain']('source-after.json', json.dumps(after, sort_keys=True).encode())
    receipt['tracked_input_bookends_equal'] = before == after
    (output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
assert before == after

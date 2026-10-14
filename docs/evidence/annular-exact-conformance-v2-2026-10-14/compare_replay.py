"""Compare complete legacy and annular bytes plus the original exact callback trace."""
import gzip, hashlib, json, pathlib, re, sys
out = pathlib.Path(__file__).resolve().parent
phase = sys.argv[1]
baseline = json.loads((out / 'baseline.json').read_text())
current = json.loads((out / (phase + '.json')).read_text())
def payloads(record):
    raw = gzip.decompress((out / record['log']).read_bytes())
    rows = re.findall(rb'([a-z0-9-]+)[:=]([0-9a-f]{100,})', raw)
    if rows:
        return [(tag, bytes.fromhex(value.decode())) for tag, value in rows]
    match = re.search(rb'(\{"operation":"martensite.region-certificate"[^\n]+)', raw)
    if match:
        return [(b'region-json', match.group(1))]
    match = re.search(rb'exact-conformance-callback-baseline:([^\n]+)', raw)
    assert match, record['command']
    return [(b'callback-trace', match.group(1))]
records = []
assert len(baseline['records']) == len(current['records']) == 12
for first, second in zip(baseline['records'][1:], current['records'][1:]):
    assert first['command'] == second['command'] and second['exit'] == 0
    expected, actual = payloads(first), payloads(second)
    assert expected and expected == actual, second['command']
    records.append({'command': second['command'], 'full_bytes_equal': True, 'payloads': [{'tag': tag.decode(), 'bytes': len(value), 'sha256': hashlib.sha256(value).hexdigest()} for tag, value in actual]})
(out / (phase + '-comparison.json')).write_text(json.dumps(records, indent=2) + '\n')
print(json.dumps({'phase': phase, 'complete_payloads_and_trace_equal': len(records)}))

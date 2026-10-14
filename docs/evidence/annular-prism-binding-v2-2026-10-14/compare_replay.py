"""Compare entire emitted payloads against the reviewed pre-style behavior."""
import gzip, hashlib, json, pathlib, re, sys
out = pathlib.Path(__file__).resolve().parent
phase = sys.argv[1]
baseline = json.loads((out / 'prestyle-first.json').read_text())
current = json.loads((out / (phase + '.json')).read_text())
def payloads(record):
    raw = gzip.decompress((out / record['log']).read_bytes())
    rows = re.findall(rb'([a-z0-9-]+)[:=]([0-9a-f]{100,})', raw)
    if record['filter'] == 'portable_region_certificate_process_receipt':
        match = re.search(rb'(\{"operation":"martensite.region-certificate"[^\n]+)', raw)
        assert match
        return [(b'region-json', match.group(1))]
    return [(tag, bytes.fromhex(value.decode())) for tag, value in rows]
records = []
assert len(baseline['commands']) == len(current['commands']) == 8
for first, second in zip(baseline['commands'], current['commands']):
    assert first['filter'] == second['filter'] and second['exit'] == 0
    expected, actual = payloads(first), payloads(second)
    assert expected and expected == actual, second['filter']
    records.append({'filter': second['filter'], 'full_bytes_equal': True, 'payloads': [{'tag': tag.decode(), 'bytes': len(value), 'sha256': hashlib.sha256(value).hexdigest()} for tag, value in actual]})
(out / (phase + '-comparison.json')).write_text(json.dumps(records, indent=2) + '\n')
print(json.dumps({'phase': phase, 'complete_payloads_equal': len(records)}))

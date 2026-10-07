"""The XSMessage resource generator must carry each literal byte for byte, and the embedded
files must match the recorded provenance."""
import hashlib
import json
import unittest

from generate_xsmessage_data import PROVENANCE, RESOURCES, TARGET


class EmbeddedFileTests(unittest.TestCase):
    def test_embedded_files_match_their_provenance(self):
        provenance = json.loads(PROVENANCE.read_text())
        self.assertEqual(set(provenance), {output for _, _, output in RESOURCES})
        for output, record in provenance.items():
            payload = (TARGET/output).read_bytes()
            self.assertEqual(hashlib.sha256(payload).hexdigest(), record['payload_sha256'])
            self.assertEqual(len(payload), record['bytes'])

    def test_the_us_resource_defines_the_first_loading_message(self):
        payload = (TARGET/'XSTEP.us').read_bytes()
        self.assertTrue(payload.startswith(b'!LOADING PHASE\n.XSTEP_1\n'))


if __name__ == '__main__':
    unittest.main()

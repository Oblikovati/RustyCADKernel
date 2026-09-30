"""The SHMessage resource generator must carry each literal byte for byte, and the embedded
files must match the recorded provenance."""
import hashlib
import json
import unittest
from pathlib import Path

from generate_shmessage_data import PROVENANCE, RESOURCES, TARGET, literal_bytes


class LiteralTests(unittest.TestCase):
    def test_adjacent_literals_concatenate_with_their_escapes(self):
        text = ('static const char X[] =\n  "\\n"\n  "! a\\tb\\n"\n  ".K.MSG0\\n";\n'
                'static const char Y[] =\n  "other";\n')
        self.assertEqual(literal_bytes(text, 'X'), b'\n! a\tb\n.K.MSG0\n')
        self.assertEqual(literal_bytes(text, 'Y'), b'other')

    def test_octal_escapes_keep_their_byte(self):
        self.assertEqual(literal_bytes('static const char Z[] =\n  "caf\\351";\n', 'Z'), b'caf\xe9')

    def test_a_missing_literal_names_itself(self):
        with self.assertRaisesRegex(ValueError, 'SHMessage_SHAPE_de'):
            literal_bytes('static const char X[] = "";', 'SHMessage_SHAPE_de')


class EmbeddedFileTests(unittest.TestCase):
    def test_embedded_files_match_their_provenance(self):
        provenance = json.loads(PROVENANCE.read_text())
        self.assertEqual(set(provenance), {output for _, _, output in RESOURCES})
        for output, record in provenance.items():
            payload = (TARGET/output).read_bytes()
            self.assertEqual(hashlib.sha256(payload).hexdigest(), record['payload_sha256'])
            self.assertEqual(len(payload), record['bytes'])

    def test_the_us_resource_defines_the_key_shape_extend_init_checks(self):
        payload = (TARGET/'SHAPE.us').read_bytes()
        self.assertIn(b'\n.ShapeFix.FixSmallSolid.MSG0\n', payload)


if __name__ == '__main__':
    unittest.main()

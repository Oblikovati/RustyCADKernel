"""A compatibility regression must never turn a skipped native assertion into a pass."""
import unittest
import xml.etree.ElementTree as ET
from tkernel_mapping import verified_native_skips, OCC23497, OCC23497_REASON

class NativeSkipTests(unittest.TestCase):
    def case(self, identity=OCC23497, reason=OCC23497_REASON):
        suite,name=identity.rsplit('.',1)
        case=ET.Element('testcase',classname=suite,name=name,status='run')
        ET.SubElement(case,'skipped',message='/pinned/source:46\n'+reason+'\n')
        return case

    def row(self):
        return {'upstream_id':OCC23497,'status':'upstream_skipped',
                'behavior_decision':'preserve_current','rust_test':'occ23497_current_composite_prefix_failure'}

    def test_recorded_skip_is_provenance_not_a_legacy_pass(self):
        result=verified_native_skips([self.case()],[self.row()])
        self.assertFalse(result[0]['legacy_assertions_passed'])
        self.assertEqual(result[0]['reason'],OCC23497_REASON)

    def test_unapproved_or_changed_skips_are_rejected(self):
        for cases,rows in [([self.case()],[]),([self.case(reason='changed')],[self.row()]),
                           ([self.case(identity='Another.Case')],[self.row()]),([],[self.row()])]:
            with self.subTest(cases=cases),self.assertRaises(ValueError): verified_native_skips(cases,rows)

    def test_resolved_native_skip_requires_mapping_review(self):
        case=self.case();case.remove(case.find('skipped'))
        with self.assertRaises(ValueError):verified_native_skips([case],[self.row()])

    def test_failure_cannot_be_accepted_alongside_a_skip(self):
        case=self.case();ET.SubElement(case,'failure')
        with self.assertRaises(ValueError):verified_native_skips([case],[self.row()])

    def test_empty_skip_reason_is_rejected(self):
        case=self.case();case.find('skipped').set('message','')
        with self.assertRaises(ValueError):verified_native_skips([case],[self.row()])

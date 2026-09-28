"""The numeric comparator may ignore only NaN representation, never finite bits."""
import unittest
from verify_tkmath_numeric import canonical_nan_tokens, compare

class NumericComparatorTests(unittest.TestCase):
    def test_nan_sign_and_payload_are_classified_explicitly(self):
        for value in [0x7ff8000000000000,0xfff8000000000000,0x7ff0000000000001]:
            self.assertEqual(canonical_nan_tokens(str(value)), 'NaN')
        compare('9221120237041090560\n', '18444492273895866368\n', 'request\n')

    def test_infinity_and_signed_zero_remain_exact(self):
        for left,right in [('9218868437227405312','18442240474082181120'),
                           ('0','9223372036854775808'),('4607182418800017408','4607182418800017409'),
                           ('9221120237041090560','9218868437227405312')]:
            with self.subTest(left=left),self.assertRaises(RuntimeError): compare(left+'\n',right+'\n','request\n')

    def test_numeric_result_count_cannot_be_relaxed(self):
        with self.assertRaises(RuntimeError):compare('1\n','1\n','first\nsecond\n')
        with self.assertRaises(RuntimeError):compare('1\n','1\n2\n','first\n')

"""Tests of the rustfmt-stable include! slice text shared by the table generators."""
import unittest

from rust_include_list import include_list


class IncludeListTests(unittest.TestCase):
    def test_a_single_fragment_stays_on_one_line(self):
        self.assertEqual(include_list(['t/a_0.rs'], '    '), '&[include!("t/a_0.rs")]')

    def test_several_fragments_take_one_line_each_at_the_given_indent(self):
        self.assertEqual(include_list(['t/a_0.rs', 't/a_1.rs'], '    '),
                         '&[\n        include!("t/a_0.rs"),\n        include!("t/a_1.rs"),\n    ]')


if __name__ == '__main__':
    unittest.main()

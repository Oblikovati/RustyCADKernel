"""Keep native default-argument calls separate from explicit empty-message calls."""

import unittest

from tkernel_exceptions_corpus import CLASSES, MESSAGES, append_exceptions_requests


class ExceptionDefaultCorpusTests(unittest.TestCase):
    def test_default_constructors_are_independent_requests(self) -> None:
        requests: list[str] = []
        append_exceptions_requests(requests)
        defaults = [row for row in requests if row.startswith("exception_default ")]
        self.assertEqual(
            defaults,
            [
                "exception_default Standard_DomainError",
                "exception_default Standard_RangeError",
                "exception_default Standard_NotImplemented",
            ],
        )
        self.assertEqual(len(requests), len(CLASSES) * len(MESSAGES) + 3)
        self.assertEqual(len(requests), len(set(requests)))

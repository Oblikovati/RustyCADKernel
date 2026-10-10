"""Lock the animation verifier to deterministic requests and its native probe operations."""

import unittest
from tkv3d_corpus_ais_animation import append_animation_requests
from verify_ais_animation import corpus


class AnimationCorpusTests(unittest.TestCase):
    def test_corpus_covers_timeline_edits_and_frozen_timers(self) -> None:
        rows: list[str] = []
        append_animation_requests(rows)
        self.assertEqual(len(rows), 319)
        self.assertEqual(len(set(rows)), 319)
        self.assertEqual(
            {row.split()[0] for row in rows},
            {
                "v3_ais_animation",
                "v3_ais_animation_timer",
                "v3_ais_animation_edits",
            },
        )
        self.assertIn("v3_ais_animation_timer 1.0 8.0 -2.0", rows)
        self.assertEqual(corpus(), "\n".join(rows) + "\n")

    def test_append_preserves_existing_requests(self) -> None:
        rows = ["existing"]
        append_animation_requests(rows)
        self.assertEqual(rows[0], "existing")
        self.assertEqual(corpus(), "\n".join(rows[1:]) + "\n")


if __name__ == "__main__":
    unittest.main()

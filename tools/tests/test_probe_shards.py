"""Sharded probe runs deal whole lines round-robin and join the answers in corpus order."""
import sys
import unittest
from probe_shards import SHARD_COUNT, ProbeCommand, ShardResult, interleave_answers, run_shard, run_sharded, split_requests


class EchoShardRunner:
    """Answer each request line with 'OK <line>' and record the shards it was given."""

    def __init__(self):
        self.shards: list[str] = []

    def __call__(self, command, requests):
        self.shards.append(requests)
        return ShardResult(''.join(f'OK {line}' for line in requests.splitlines(keepends=True)), f'{len(requests)}|')


class SplitTests(unittest.TestCase):
    def test_lines_are_dealt_round_robin(self):
        self.assertEqual(split_requests('a\nb\nc\n', 2), [['a\n', 'c\n'], ['b\n']])
        self.assertEqual(split_requests('a\nb\n', 8), [['a\n'], ['b\n']])
        self.assertEqual(split_requests('', 4), [])

    def test_a_shard_count_below_one_is_refused(self):
        with self.assertRaisesRegex(ValueError, 'got 0'):
            split_requests('a\n', 0)


class InterleaveTests(unittest.TestCase):
    def test_answers_return_to_corpus_order(self):
        self.assertEqual(interleave_answers([['a\n', 'c\n'], ['b\n']], ['A\nC\n', 'B\n']), 'A\nB\nC\n')

    def test_a_shard_with_a_missing_answer_is_refused(self):
        with self.assertRaisesRegex(RuntimeError, r"1 lines for 2 requests in the shard that starts with 'a\\n'"):
            interleave_answers([['a\n', 'c\n'], ['b\n']], ['A\n', 'B\n'])


class RunShardedTests(unittest.TestCase):
    def test_outputs_are_joined_in_corpus_order(self):
        runner = EchoShardRunner()
        requests = ''.join(f'op {i}\n' for i in range(100))
        result = run_sharded(ProbeCommand(['probe'], {}), requests, workers=3, runner=runner)
        self.assertEqual(result.stdout, ''.join(f'OK op {i}\n' for i in range(100)))
        self.assertEqual(len(runner.shards), SHARD_COUNT)
        self.assertTrue(runner.shards[0].startswith('op 0\nop 64\n'))

    def test_the_shard_layout_does_not_depend_on_the_worker_count(self):
        requests = ''.join(f'op {i}\n' for i in range(200))
        layouts = []
        for workers in (1, 5, 32):
            runner = EchoShardRunner()
            run_sharded(ProbeCommand(['probe'], {}), requests, workers=workers, runner=runner)
            layouts.append(sorted(runner.shards))
        self.assertEqual(layouts[0], layouts[1])
        self.assertEqual(layouts[0], layouts[2])

    def test_a_single_shard_runs_the_corpus_in_one_process(self):
        runner = EchoShardRunner()
        run_sharded(ProbeCommand(['probe'], {}), 'op 1\nop 2\n', workers=4, runner=runner, shard_count=1)
        self.assertEqual(runner.shards, ['op 1\nop 2\n'])

    def test_an_empty_corpus_still_runs_the_probe_once(self):
        runner = EchoShardRunner()
        self.assertEqual(run_sharded(ProbeCommand(['probe'], {}), '', workers=2, runner=runner).stdout, '')
        self.assertEqual(runner.shards, [''])


class RunShardTests(unittest.TestCase):
    def test_a_failing_probe_names_the_first_request_of_its_shard(self):
        command = ProbeCommand([sys.executable, '-c', 'import sys; sys.exit(3)'], {})
        with self.assertRaisesRegex(RuntimeError, r"Probe failed \(3\).*'op 7'"):
            run_shard(command, 'op 7\nop 8\n')

    def test_a_passing_probe_returns_its_output(self):
        command = ProbeCommand([sys.executable, '-c', 'import sys; print(sys.stdin.read().upper(), end="")'], {})
        self.assertEqual(run_shard(command, 'op 1\n'), ShardResult('OP 1\n', ''))


if __name__ == '__main__':
    unittest.main()

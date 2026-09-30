"""Persistent per-shell sewing, tolerance selection and rebuilt topology."""

from itertools import product
from tktopalgo_corpus_breplib_records import numbers


def shell_sewing_record(
    kind: int,
    tolerance: float,
    gap: float = 0.0,
    reverse: int = 0,
    scale: float = 1.0,
    previous: int = 0,
) -> list[float]:
    """Build kind/gap/offset/scale/orientation/tolerance/source; shell_sewing_record(4,1e-6)."""
    return [kind, gap, 10.0, scale, reverse, tolerance, previous]


def append_shell_sewing_request(
    requests: list[str], operations: list[list[float]]
) -> None:
    """Retain one instance across operations; append_shell_sewing_request(rows,records)."""
    values = [len(operations), *(value for record in operations for value in record)]
    requests.append("sh_shell_sewing " + numbers(values))


def append_shell_sewing_requests(requests: list[str]) -> None:
    """Exercise topology and exact mass on all geometry families; append_shell_sewing_requests(rows)."""
    for kind, tolerance, reverse, scale in product(
        range(9), [-1.0, 0.0, 1e-7, 1e-4, 1e-3, 0.1], [0, 1], [0.1, 1.0]
    ):
        record = shell_sewing_record(kind, tolerance, 1e-4, reverse, scale)
        append_shell_sewing_request(requests, [record, [*record[:-1], 1]])
    append_shell_sewing_gaps(requests)
    append_shell_sewing_histories(requests)


def append_shell_sewing_gaps(requests: list[str]) -> None:
    """Cross native merge thresholds without changing inputs; append_shell_sewing_gaps(rows)."""
    for gap, tolerance, reverse in product(
        [-1e-4, 0.0, 1e-8, 1e-4, 1e-2, 0.5], [1e-7, 1e-4, 1e-3, 0.1], [0, 1]
    ):
        record = shell_sewing_record(3, tolerance, gap, reverse)
        append_shell_sewing_request(requests, [record, record])


def append_shell_sewing_histories(requests: list[str]) -> None:
    """Expose retained shells across unrelated inputs and null/no-shell calls; append_shell_sewing_histories(rows)."""
    for first, last in product(range(9), repeat=2):
        operations = [
            shell_sewing_record(first, 1e-6),
            shell_sewing_record(last, 0.0, scale=2.0),
        ]
        append_shell_sewing_request(requests, operations)

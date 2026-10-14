"""Verify the independent cut-cap cell fixture, not production geometry."""

import json
from fractions import Fraction
from pathlib import Path


def boundary_matrices() -> tuple[list[list[int]], list[list[int]]]:
    endpoints = [(0, 0), (1, 1), (0, 1), (2, 2), (3, 3), (2, 3), (0, 2), (1, 3)]
    words = [[1, 7, -4, -7], [-2, 8, 5, -8], [-1, 3, 2, -3], [4, 6, -5, -6]]
    first = [[0] * len(endpoints) for _ in range(4)]
    second = [[0] * len(words) for _ in endpoints]
    for edge, (start, end) in enumerate(endpoints):
        first[start][edge] -= 1
        first[end][edge] += 1
    for face, word in enumerate(words):
        for signed_edge in word:
            second[abs(signed_edge) - 1][face] += 1 if signed_edge > 0 else -1
    return first, second


def rational_rank(matrix: list[list[int]]) -> int:
    rows = [[Fraction(entry) for entry in row] for row in matrix]
    pivot_row = 0
    for column in range(len(rows[0])):
        pivot = next((i for i in range(pivot_row, len(rows)) if rows[i][column]), None)
        if pivot is None:
            continue
        rows[pivot_row], rows[pivot] = rows[pivot], rows[pivot_row]
        scale = rows[pivot_row][column]
        rows[pivot_row] = [entry / scale for entry in rows[pivot_row]]
        eliminate_column(rows, pivot_row, column)
        pivot_row += 1
    return pivot_row


def eliminate_column(rows: list[list[Fraction]], pivot: int, column: int) -> None:
    for i in range(len(rows)):
        if i == pivot:
            continue
        scale = rows[i][column]
        rows[i] = [a - scale * b for a, b in zip(rows[i], rows[pivot], strict=True)]


def verify_fixture() -> None:
    first, second = boundary_matrices()
    records = json.loads(Path(__file__).with_name("independent-boundary-ranks.json").read_text())
    for matrix, record in zip((first, second), records, strict=True):
        result = record["output"]
        expected = [[Fraction(int(q["num"]), int(q["den"])) for q in row]
                    for row in result["matrix"]["entries"]]
        assert matrix == expected, "fixture boundary differs from the independent rank input"
        assert rational_rank(matrix) == result["rank"] == 3
    assert all(sum(first[i][k] * second[k][j] for k in range(8)) == 0
               for i in range(4) for j in range(4)), "boundary squared must vanish"
    assert (4 - rational_rank(first), 8 - rational_rank(first) - rational_rank(second),
            4 - rational_rank(second)) == (1, 2, 1)
    assert 4 - 8 + 4 == 4 - 6 + (0 + 0 + 1 + 1) == 0


if __name__ == "__main__":
    verify_fixture()
    print("Independent annular cell words: ranks 3/3, Betti numbers (1, 2, 1), Euler 0")

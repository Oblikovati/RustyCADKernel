"""Append, edit and HExtendedString requests of the extended_string_api area.

See tkernel_extended_string_api_corpus.py for the formats and the excluded (undefined
behavior) inputs; every generator here respects those exclusions.
"""
import random

from tkernel_extended_string_api_corpus import INT_MAX, INT_MIN, REALS, STRINGS, UTF8, bits, indices, raw, u16, units_of

OTHERS = ["", "x", "ab", "Hello", [0xD83D, 0xDE00], "a\0b", [0xFFFF]]
EDIT_STRINGS = STRINGS[:14] + [STRINGS[25], STRINGS[26], STRINGS[28]]


def span_arguments(target: list[int], other) -> list[str]:
    """(pointer, length) pairs Cat accepts: a length within the span, or a total below 0."""
    count = len(units_of(other))
    overflow = -(len(target) + 1)
    pairs = [f"{u16(other)} {n}" for n in sorted({0, count, min(1, count), overflow, INT_MIN})]
    return pairs + [f"null {n}" for n in (0, overflow, INT_MIN)]


def append_requests() -> list[str]:
    """Every AssignCat/Cat/operator form, Prepend and the aliased self forms."""
    out = []
    for text in EDIT_STRINGS:
        target = u16(text)
        units = units_of(text)
        for other in OTHERS:
            out += [f"extended_string_append {kind} {target} {u16(other)}" for kind in ("str", "u16", "view", "prepend")]
        out += [f"extended_string_append span {target} {pair}" for other in OTHERS for pair in span_arguments(units, other)]
        out += [f"extended_string_append int {target} {v}" for v in (0, -7, INT_MIN, INT_MAX)]
        out += [f"extended_string_append real {target} {bits(v)}" for v in REALS]
        out += [f"extended_string_append char {target} {c}" for c in (0, 33, 200)]
        out += [f"extended_string_append unit {target} {c}" for c in (0, 33, 0x20AC)]
        out += [f"extended_string_append self {target} {offset} {length}"
                for offset in sorted({0, len(units) // 2, len(units) - 1}) if 0 <= offset < len(units)
                for length in sorted({0, 1, len(units) - offset}) if length <= len(units) - offset]
    return out


def indexed_edits(text) -> list[str]:
    """Insert, SetValue, Remove, Trunc, Split and SubString at boundary indices."""
    target, n = u16(text), len(units_of(text))
    out = []
    for i in indices(n):
        out += [f"extended_string_edit insert_unit {target} {i} {u}" for u in (0, 0x41)]
        out += [f"extended_string_edit set_unit {target} {i} {u}" for u in (0, 0x41)]
        for other in ("", "xy", "a\0b"):
            out += [f"extended_string_edit {op} {target} {i} {u16(other)}" for op in ("insert_str", "insert_u16", "set_str", "set_u16")]
        spans = [f"{u16('xy')} 2", f"{u16('xy')} 1", f"{u16('xy')} 0", f"{u16('xy')} -1", "null 0", "null 3"]
        out += [f"extended_string_edit {op} {target} {i} {s}" for op in ("insert_span", "set_span") for s in spans]
        out += [f"extended_string_edit remove {target} {i} {c}" for c in sorted({INT_MIN, -1, 0, 1, 2, n, n + 1})]
        out += [f"extended_string_edit {op} {target} {i}" for op in ("remove_one", "trunc", "split")]
        out += [f"extended_string_edit sub {target} {i} {j}" for j in (0, 1, n, n + 1, INT_MAX)]
    return out


def whole_edits(text) -> list[str]:
    """RemoveAll, ChangeAll, Copy, assignment, Clear, Swap, Move, adjust and justify."""
    target, n = u16(text), len(units_of(text))
    out = [f"extended_string_edit remove_all {target} {u}" for u in (0, 0x20, 0x41, 0x6C)]
    out += [f"extended_string_edit change {target} {a} {b}" for a, b in ((0x6C, 0x4C), (0, 0x2A), (0x20, 0))]
    for other in OTHERS:
        out += [f"extended_string_edit {op} {target} {u16(other)}"
                for op in ("copy_u16", "copy_str", "assign_u16", "assign_view", "swap", "move")]
        count = len(units_of(other))
        out += [f"extended_string_edit copy_span {target} {u16(other)} {c}" for c in sorted({0, count, -1})]
    out += [f"extended_string_edit copy_span {target} null {c}" for c in (0, 2)]
    out += [f"extended_string_edit {op} {target}" for op in ("clear", "left_adjust", "right_adjust", "capitalize")]
    widths = sorted({INT_MIN, -1, 0, max(n - 1, 0), n, n + 3, n + 4})
    out += [f"extended_string_edit {op} {target} {w} {f}" for op in ("left_justify", "right_justify", "center")
            for w in widths for f in (0x2D, 0)]
    return out


def edit_requests(rng: random.Random) -> list[str]:
    """Append and edit requests, plus seeded random insert/remove sequences' single steps."""
    out = append_requests()
    for text in EDIT_STRINGS:
        out += indexed_edits(text) + whole_edits(text)
    for _ in range(300):
        units = [rng.choice([0x20, 0x41, 0x61, 0, 0xE9, 0xD83D]) for _ in range(rng.randrange(0, 9))]
        n = len(units)
        out.append(f"extended_string_edit remove {u16(units)} {rng.randrange(-1, n + 2)} {rng.randrange(-1, n + 2)}")
        out.append(f"extended_string_token {u16(units)} {u16([0x20, 0])} {rng.randrange(-1, 5)}")
    return out


def handle_requests() -> list[str]:
    """HExtendedString constructors, members and the handle hash/equality specializations."""
    out = ["extended_string_hnew empty", "extended_string_hnew hascii -"]
    out += [f"extended_string_hnew cstr {raw(data)}" for data in UTF8]
    out += [f"extended_string_hnew {kind} {u16(text)}" for text in STRINGS for kind in ("u16", "str", "hcopy")]
    out += [f"extended_string_hnew unit {c}" for c in (0, 65, 0x20AC)]
    out += [f"extended_string_hnew fill {n} {c}" for n in (INT_MIN, -1, 0, 2) for c in (0, 45)]
    for text in EDIT_STRINGS:
        target, n = u16(text), len(units_of(text))
        out += [f"extended_string_hedit {op} {target} {u16(other)}" for op in ("assign_cat", "cat", "query")
                for other in OTHERS]
        out += [f"extended_string_hedit {op} {target}" for op in ("assign_self", "clear", "print")]
        out += [f"extended_string_hedit change {target} 108 76", f"extended_string_hedit remove_all {target} 32"]
        for i in indices(n):
            out += [f"extended_string_hedit {op} {target} {i} 65" for op in ("insert_unit", "set_unit")]
            out += [f"extended_string_hedit {op} {target} {i} {u16('xy')}" for op in ("insert", "set")]
            out += [f"extended_string_hedit remove {target} {i} {c}" for c in (-1, 0, 1, n)]
            out += [f"extended_string_hedit {op} {target} {i}" for op in ("remove_one", "split", "trunc", "value")]
        out += [f"extended_string_hedit token {target} {u16(' ')} {w}" for w in (0, 1, 2, 3)]
    out += [f"extended_string_hkey {u16(a)} {u16(b)}" for a in OTHERS for b in OTHERS]
    return out

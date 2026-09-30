"""The rustfmt-stable Rust text of a slice of `include!` fragments, shared by the table generators.

rustfmt keeps a one-element slice on one line, so a generator that always wrote the
multi-line form produced files that `cargo fmt` rewrote and `--check` then reported as
stale.
"""
from __future__ import annotations


def include_list(paths: list[str], indent: str) -> str:
    """The slice expression over `paths`, formatted as rustfmt formats it.

    `indent` is the indentation of the line that holds the slice.
    include_list(['t/a_0.rs'], '    ') == '&[include!("t/a_0.rs")]'
    """
    if len(paths) == 1:
        return f'&[include!("{paths[0]}")]'
    body = ''.join(f'{indent}    include!("{path}"),\n' for path in paths)
    return f'&[\n{body}{indent}]'

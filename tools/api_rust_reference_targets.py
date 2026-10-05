"""Resolve UFCS receiver crates while retaining exact borrowed method identity."""

from __future__ import annotations


def nominal_reference_owner(owner: str) -> str:
    """Strip valid reference qualifiers; e.g. `nominal_reference_owner("&mut crate::Value")`."""
    if not owner.startswith("&"):
        return owner
    receiver = owner[1:]
    if receiver.startswith("'"):
        lifetime, separator, receiver = receiver.partition(" ")
        if not separator or not lifetime[1:].isidentifier():
            return ""
    if receiver.startswith("mut "):
        receiver = receiver[4:]
    crate, separator, tail = receiver.partition("::")
    return receiver if crate.isidentifier() and separator and tail else ""

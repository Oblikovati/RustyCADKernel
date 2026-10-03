#!/usr/bin/env python3
import pathlib
import sys


sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[3]))
from tools.api_inventory.merge import declaration_identity


def main():
    base = {
        "usr": "clang:colliding-function-template-usr",
        "kind": "FunctionTemplate",
        "signature": "decltype(provider.Nb()) (const Provider &) const",
        "template_signature": "template <typename Provider>",
    }
    size_overload = {
        **base,
        "signature": "decltype(provider.Size()) (const Provider &) const",
    }
    constrained_overload = {
        **base,
        "template_signature": "template <typename Provider> requires HasSize<Provider>",
    }
    assert declaration_identity(base) == declaration_identity(dict(base))
    assert declaration_identity(base) != declaration_identity(size_overload)
    assert declaration_identity(base) != declaration_identity(constrained_overload)


if __name__ == "__main__":
    main()

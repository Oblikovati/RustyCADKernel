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
        "identity_signature": "decltype(__argument_0.Nb()) (const Provider &) const",
        "template_signature": "template <typename __template_param_0>",
    }
    size_overload = {
        **base,
        "identity_signature": "decltype(__argument_0.Size()) (const Provider &) const",
    }
    constrained_overload = {
        **base,
        "template_signature": "template <typename __template_param_0> requires HasSize<__template_param_0>",
    }
    assert declaration_identity(base) == declaration_identity(dict(base))
    assert declaration_identity(base) != declaration_identity(size_overload)
    assert declaration_identity(base) != declaration_identity(constrained_overload)
    renamed_parameter = {
        **base,
        "signature": "decltype(other.Nb()) (const Provider &) const",
        "identity_signature": "decltype(__argument_0.Nb()) (const Provider &) const",
    }
    assert declaration_identity(base) == declaration_identity(renamed_parameter)


if __name__ == "__main__":
    main()

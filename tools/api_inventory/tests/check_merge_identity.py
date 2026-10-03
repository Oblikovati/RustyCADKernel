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
    defaulted = {
        **base,
        "template_signature": "template <typename __template_param_0 = int>",
    }
    redeclared = {
        **base,
        "template_signature": "template <typename __template_param_0>",
    }
    assert declaration_identity(defaulted) == declaration_identity(redeclared)
    nested_default = {
        **base,
        "template_signature": "template <typename __template_param_0 = std::pair<int, long>>",
    }
    assert declaration_identity(nested_default) == declaration_identity(redeclared)
    template_template_default = {
        **base,
        "template_signature": "template <template <typename T = int> class C>",
    }
    template_template_redecl = {
        **base,
        "template_signature": "template <template <typename T> class C>",
    }
    assert declaration_identity(template_template_default) == declaration_identity(
        template_template_redecl
    )
    renamed_parameter = {
        **base,
        "signature": "decltype(other.Nb()) (const Provider &) const",
        "identity_signature": "decltype(__argument_0.Nb()) (const Provider &) const",
    }
    assert declaration_identity(base) == declaration_identity(renamed_parameter)


if __name__ == "__main__":
    main()

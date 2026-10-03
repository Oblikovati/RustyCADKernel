#!/usr/bin/env python3
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[3]))
from tools.api_inventory.merge import declaration_identity


def base_template() -> dict[str, str]:
    return {
        "usr": "clang:colliding-function-template-usr",
        "kind": "FunctionTemplate",
        "signature": "decltype(provider.Nb()) (const Provider &) const",
        "identity_signature": "decltype(__argument_0.Nb()) (const Provider &) const",
        "template_signature": "template <typename __template_param_0>",
    }


def test_callable_signatures_keep_overloads() -> None:
    base = base_template()
    size_overload = {
        **base,
        "identity_signature": "decltype(__argument_0.Size()) (const Provider &) const",
    }
    constrained_overload = {
        **base,
        "template_signature": "template <typename __template_param_0> requires HasSize<__template_param_0>",
    }
    assert declaration_identity(base) != declaration_identity(size_overload)
    assert declaration_identity(base) != declaration_identity(constrained_overload)


def test_defaults_do_not_split_redeclarations() -> None:
    base = base_template()
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


def test_nested_template_template_default_is_removed() -> None:
    base = base_template()
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


def test_literals_and_relational_defaults_are_lexed() -> None:
    base = base_template()
    literal = {
        **base,
        "template_signature": "template <char A = '>', char B = ',', bool C = (1 < 2)>",
    }
    plain = {
        **base,
        "template_signature": "template <char A, char B, bool C>",
    }
    assert declaration_identity(literal) == declaration_identity(plain)


def test_parameter_placeholder_substring_is_not_keyword() -> None:
    base = base_template()
    defaulted = {
        **base,
        "template_signature": "template <typename __template_param_0 = int>",
    }
    plain = {
        **base,
        "template_signature": "template <typename __template_param_0>",
    }
    assert declaration_identity(defaulted) == declaration_identity(plain)


def test_parameter_names_do_not_change_identity() -> None:
    base = base_template()
    renamed_parameter = {
        **base,
        "signature": "decltype(other.Nb()) (const Provider &) const",
        "identity_signature": "decltype(__argument_0.Nb()) (const Provider &) const",
    }
    assert declaration_identity(base) == declaration_identity(renamed_parameter)


def main() -> None:
    test_callable_signatures_keep_overloads()
    test_defaults_do_not_split_redeclarations()
    test_nested_template_template_default_is_removed()
    test_literals_and_relational_defaults_are_lexed()
    test_parameter_placeholder_substring_is_not_keyword()
    test_parameter_names_do_not_change_identity()


if __name__ == "__main__":
    main()

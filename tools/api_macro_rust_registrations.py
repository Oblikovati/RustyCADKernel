"""Find every runtime-type registration in the Rust sources, through wrapper macros too.

A registration macro is any `macro_rules!` arm that writes `RuntimeTyped for $class`
with `TYPE_NAME = $name` and a parent `<$parent as ..RuntimeTyped>`, or that forwards
to such a macro (`impl_transient!`, `message_runtime_type!`, `concrete_tshape!`, ...).
Each invocation with a string-literal name yields (Rust type, descriptor name, parent).
"""

from __future__ import annotations

from dataclasses import dataclass
import re

from api_macro_rust_source import RustSourceTree, TransientRegistration, squeeze

DEFINITION = re.compile(r"macro_rules!\s*(\w+)\s*\{")
INTERFACE_MACRO = "impl_transient_interface"


@dataclass(frozen=True)
class RegistrationSpec:
    """Argument positions of one registration macro; `parent` is a position or a type."""

    class_position: int
    name_position: int
    parent: int | str
    interface: bool


def balanced(text: str, start: int, opening: str, closing: str) -> int:
    """Index of the bracket closing the one at `start`, or -1."""
    depth = 0
    for index in range(start, len(text)):
        depth += (text[index] == opening) - (text[index] == closing)
        if depth == 0:
            return index
    return -1


def macro_arms(text: str, start: int) -> list[tuple[list[str], str]]:
    """(parameter names, body) of each arm of the `macro_rules!` body opening at `start`."""
    end = balanced(text, start, "{", "}")
    body, arms, cursor = text[start + 1 : end], [], 0
    while (opening := body.find("(", cursor)) >= 0:
        matcher_end = balanced(body, opening, "(", ")")
        arrow = body.find("{", matcher_end)
        arm_end = balanced(body, arrow, "{", "}")
        if matcher_end < 0 or arrow < 0 or arm_end < 0:
            break
        arms.append((matcher_parameters(body[opening + 1 : matcher_end]), body[arrow : arm_end + 1]))
        cursor = arm_end + 1
    return arms


def matcher_parameters(matcher: str) -> list[str]:
    """Top-level `$name:frag` parameters in order; repetition groups are skipped."""
    flat = re.sub(r"\$\((?:[^()]|\([^()]*\))*\)\s*[*+?]", " ", matcher)
    return re.findall(r"\$(\w+)\s*:\s*\w+", flat)


def primitive_spec(parameters: list[str], body: str) -> RegistrationSpec | None:
    """Spec of an arm that itself writes the RuntimeTyped impl and parent handle."""
    target = re.search(r"RuntimeTyped for \$(\w+)", body)
    name = re.search(r"TYPE_NAME\s*:\s*&'static str\s*=\s*\$(\w+)", body)
    parent = re.search(r"<\s*\$(\w+)\s+as\s+[\w:$]*RuntimeTyped\s*>", body)
    found = [match.group(1) if match else None for match in (target, name, parent)]
    if None in found or any(item not in parameters for item in found):
        return None
    positions = [parameters.index(str(item)) for item in found]
    return RegistrationSpec(positions[0], positions[1], positions[2], False)


def forwarding_spec(
    parameters: list[str], body: str, known: dict[str, RegistrationSpec]
) -> RegistrationSpec | None:
    """Spec of an arm that invokes an already known registration macro."""
    for name, spec in known.items():
        for args in invocation_arguments(body, name):
            mapped = map_forwarded(parameters, args, spec)
            if mapped is not None:
                return mapped
    return None


def map_forwarded(
    parameters: list[str], args: list[str], spec: RegistrationSpec
) -> RegistrationSpec | None:
    needed = max(spec.class_position, spec.name_position, spec.parent if isinstance(spec.parent, int) else 0)
    if len(args) <= needed:
        return None
    positions = [placeholder(args[spec.class_position], parameters), placeholder(args[spec.name_position], parameters)]
    if None in positions:
        return None
    parent_arg = args[spec.parent] if isinstance(spec.parent, int) else spec.parent
    parent_position = placeholder(parent_arg, parameters)
    parent: int | str = parent_position if parent_position is not None else parent_arg
    return RegistrationSpec(int(positions[0]), int(positions[1]), parent, spec.interface)


def placeholder(argument: str, parameters: list[str]) -> int | None:
    """Position of `$name` in the arm's parameters, or None for a non-placeholder."""
    match = re.fullmatch(r"\$(\w+)", argument.strip())
    return parameters.index(match.group(1)) if match and match.group(1) in parameters else None


def invocation_arguments(text: str, macro: str) -> list[list[str]]:
    """Top-level comma-separated arguments of every `macro!(...)` call in `text`."""
    calls = []
    for match in re.finditer(rf"\b{re.escape(macro)}!\s*\(", text):
        end = balanced(text, match.end() - 1, "(", ")")
        if end > 0:
            calls.append(split_arguments(strip_attributes(text[match.end() : end])))
    return calls


def strip_attributes(text: str) -> str:
    """Drop doc comments and `#[...]` attributes that wrapper macros accept first."""
    text = re.sub(r"//[^\n]*", " ", text)
    return re.sub(r"#\s*\[[^\]]*\]", " ", text)


def without_line_comments(text: str) -> str:
    """Blank whole-line `//` and `///` comments, keeping line numbers, so a doc example
    such as `impl_transient_interface!(dyn Curve, "Geom2d_Curve", ..)` never counts."""
    return "\n".join("" if line.lstrip().startswith("//") else line for line in text.split("\n"))


def macro_definitions(tree: RustSourceTree) -> dict[str, list[tuple[list[str], str]]]:
    """Every `macro_rules!` by name with its arms; a name defined twice keeps both."""
    found: dict[str, list[tuple[list[str], str]]] = {}
    for _, raw in tree.rust_files():
        text = without_line_comments(raw)
        for match in DEFINITION.finditer(text):
            found.setdefault(match.group(1), []).extend(macro_arms(text, match.end() - 1))
    return found


def registration_specs(tree: RustSourceTree) -> dict[str, RegistrationSpec]:
    """Resolve primitive and forwarding registration macros to a fixed point."""
    definitions = macro_definitions(tree)
    known: dict[str, RegistrationSpec] = {}
    changed = True
    while changed:
        changed = False
        for name, arms in sorted(definitions.items()):
            spec = None if name in known else arm_spec(name, arms, known)
            if spec is not None:
                known[name], changed = spec, True
    return known


def arm_spec(
    name: str, arms: list[tuple[list[str], str]], known: dict[str, RegistrationSpec]
) -> RegistrationSpec | None:
    for parameters, body in arms:
        spec = primitive_spec(parameters, body) or forwarding_spec(parameters, body, known)
        if spec is not None:
            interface = spec.interface or name == INTERFACE_MACRO
            return RegistrationSpec(spec.class_position, spec.name_position, spec.parent, interface)
    return None


def transient_registrations(tree: RustSourceTree) -> list[TransientRegistration]:
    """Every literal-named registration, e.g. `impl_transient!(Line, "Geom2d_Line", ...)`."""
    specs = registration_specs(tree)
    found = []
    for relative, raw in tree.rust_files():
        text = without_line_comments(raw)
        for name, spec in sorted(specs.items()):
            found.extend(file_registrations(relative, text, name, spec))
    return sorted(found, key=lambda item: (item.file, item.line, item.cxx_name))


def file_registrations(
    relative: str, text: str, name: str, spec: RegistrationSpec
) -> list[TransientRegistration]:
    found = []
    for match in re.finditer(rf"\b{re.escape(name)}!\s*\(", text):
        end = balanced(text, match.end() - 1, "(", ")")
        args = split_arguments(strip_attributes(text[match.end() : end])) if end > 0 else []
        line = text.count("\n", 0, match.start()) + 1
        registration = registration_from_args(args, spec, relative, line, name)
        if registration is not None:
            found.append(registration)
    return found


def registration_from_args(
    args: list[str], spec: RegistrationSpec, relative: str, line: int, macro: str
) -> TransientRegistration | None:
    parent_index = spec.parent if isinstance(spec.parent, int) else -1
    if len(args) <= max(spec.class_position, spec.name_position, parent_index):
        return None
    literal = re.fullmatch(r"\"([^\"]+)\"", args[spec.name_position])
    if literal is None:
        return None
    parent = args[spec.parent] if isinstance(spec.parent, int) else spec.parent
    return TransientRegistration(args[spec.class_position], literal.group(1), squeeze(parent),
                                 spec.interface, relative, line, macro)


def split_arguments(text: str) -> list[str]:
    parts, depth, start = [], 0, 0
    for index, char in enumerate(text):
        depth += (char in "([{<") - (char in ")]}>")
        if char == "," and depth == 0:
            parts.append(squeeze(text[start:index]))
            start = index + 1
    return parts + [squeeze(text[start:])]

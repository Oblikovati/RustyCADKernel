"""Conservatively normalize template defaults for declaration identity."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class TemplateIdentity:
    normalized: str
    supported: bool
    reason: str | None = None


def normalize_template_signature(signature: str) -> TemplateIdentity:
    """Remove redeclaration defaults only when bounded syntax parsing succeeds."""
    bounds = template_parameter_bounds(signature)
    if bounds is None:
        return TemplateIdentity(signature, False, "template_parameter_list_unparsed")
    start, end = bounds
    suffix = lexical_tokens(signature[end + 1 :])
    if suffix is None or any(token == ">" for token, _, _ in suffix):
        return TemplateIdentity(signature, False, "template_suffix_unparsed")
    return normalize_bounded_signature(signature, start, end)


def normalize_bounded_signature(
    signature: str, start: int, end: int
) -> TemplateIdentity:
    """Normalize each parameter in an already validated signature span."""
    parts = split_template_parameters(signature[start + 1 : end])
    if parts is None:
        return TemplateIdentity(signature, False, "template_parameters_unparsed")
    normalized = normalize_parameters(parts)
    if normalized is None:
        return TemplateIdentity(signature, False, "template_parameter_unparsed")
    replacement = ", ".join(normalized)
    return TemplateIdentity(
        signature[: start + 1] + replacement + signature[end:], True
    )


def normalize_parameters(parameters: list[str]) -> list[str] | None:
    """Normalize all parameter defaults or fail the complete signature closed."""
    results = [normalize_parameter(parameter) for parameter in parameters]
    if any(not result.supported for result in results):
        return None
    return [result.normalized for result in results]


def normalize_parameter(parameter: str) -> TemplateIdentity:
    """Normalize nested template-template defaults before its own default."""
    nested = normalize_nested_lists(parameter)
    if nested is None:
        return TemplateIdentity(parameter, False, "nested_template_list_unparsed")
    default = top_level_default(nested)
    if default is None:
        return TemplateIdentity(parameter, False, "template_default_unparsed")
    if default < 0:
        return TemplateIdentity(nested, True)
    return TemplateIdentity(nested[:default].rstrip(), True)


def normalize_nested_lists(parameter: str) -> str | None:
    """Normalize exact `template<...>` tokens without matching identifier substrings."""
    tokens = lexical_tokens(parameter)
    if tokens is None:
        return None
    spans = outermost_template_spans(template_keyword_bounds(parameter, tokens))
    if len(spans) > 1:
        return None
    return (
        normalize_template_parameter_span(parameter, spans[0]) if spans else parameter
    )


def outermost_template_spans(
    spans: list[tuple[int, int]],
) -> list[tuple[int, int]]:
    """Keep only non-overlapping outer spans for recursive, offset-safe edits."""
    return [
        span
        for span in spans
        if not any(other[0] < span[0] and span[1] < other[1] for other in spans)
    ]


def normalize_template_parameter_span(
    parameter: str, bounds: tuple[int, int]
) -> str | None:
    """Normalize one template-template list through recursion before replacing it."""
    start, end = bounds
    parts = split_template_parameters(parameter[start + 1 : end])
    normalized = None if parts is None else normalize_parameters(parts)
    if normalized is None:
        return None
    return parameter[: start + 1] + ", ".join(normalized) + parameter[end:]


def template_parameter_bounds(signature: str) -> tuple[int, int] | None:
    """Find the first exact template parameter list with lexical delimiter rules."""
    tokens = lexical_tokens(signature)
    if tokens is None:
        return None
    for index, token in enumerate(tokens[:-1]):
        if token[0] != "template" or tokens[index + 1][0] != "<":
            continue
        return balanced_angle_bounds(signature, tokens[index + 1][1])
    return None


def template_keyword_bounds(
    signature: str, tokens: list[tuple[str, int, int]]
) -> list[tuple[int, int]]:
    """Return nested template-list spans, excluding the outer signature list."""
    spans: list[tuple[int, int]] = []
    for index, token in enumerate(tokens[:-1]):
        if token[0] != "template" or tokens[index + 1][0] != "<":
            continue
        bounds = balanced_angle_bounds(signature, tokens[index + 1][1])
        if bounds is not None:
            spans.append(bounds)
    return spans


def balanced_angle_bounds(text: str, start: int) -> tuple[int, int] | None:
    """Match angle brackets while ignoring parenthesized operators and literals."""
    tokens = lexical_tokens_from(text, start)
    return None if tokens is None else find_angle_closer(tokens, start)


def find_angle_closer(
    tokens: list[tuple[str, int, int]], start: int
) -> tuple[int, int] | None:
    """Find a matching `>` token while tracking nested delimiters."""
    depths = {"<": 1, "(": 0, "[": 0, "{": 0}
    closing = {">": "<", ")": "(", "]": "[", "}": "{"}
    for token, left, _right in tokens:
        if left == start and token == "<":
            continue
        if token == ">" and not depths["("]:
            depths["<"] -= 1
            if depths["<"] == 0:
                return start, left
            if depths["<"] < 0:
                return None
            continue
        if not advance_depths(depths, token, closing):
            return None
    return None


def advance_depths(depths: dict[str, int], token: str, closing: dict[str, str]) -> bool:
    """Update nested delimiter depths, returning false for an unmatched closer."""
    if token in closing:
        if token == ">" and depths["("]:
            return True
        depths[closing[token]] -= 1
        return depths[closing[token]] >= 0
    if token in depths and (token != "<" or not depths["("]):
        depths[token] += 1
    return True


def split_template_parameters(signature: str) -> list[str] | None:
    """Split top-level commas while respecting nested delimiters and C++ literals."""
    tokens = lexical_tokens(signature)
    if tokens is None:
        return None
    depths = {"<": 0, "(": 0, "[": 0, "{": 0}
    closing = {">": "<", ")": "(", "]": "[", "}": "{"}
    chunks: list[str] = []
    start = 0
    for token, left, right in tokens:
        if token == "," and not any(depths.values()):
            chunks.append(signature[start:left].strip())
            start = right
            continue
        if not advance_depths(depths, token, closing):
            return None
    if any(depths.values()):
        return None
    chunks.append(signature[start:].strip())
    return chunks


def top_level_default(parameter: str) -> int | None:
    """Find a real top-level assignment, ignoring comparisons and literals."""
    tokens = lexical_tokens(parameter)
    if tokens is None:
        return None
    depths = {"<": 0, "(": 0, "[": 0, "{": 0}
    closing = {">": "<", ")": "(", "]": "[", "}": "{"}
    for index, (token, start, end) in enumerate(tokens):
        if assignment_token(tokens, index, token, start, depths):
            return start
        if not advance_depths(depths, token, closing):
            return None
    return -1 if not any(depths.values()) else None


def assignment_token(
    tokens: list[tuple[str, int, int]],
    index: int,
    token: str,
    start: int,
    depths: dict[str, int],
) -> bool:
    """Recognize a top-level default assignment rather than a comparison token."""
    if token != "=" or any(depths.values()):
        return False
    previous = tokens[index - 1][0] if index else ""
    following = tokens[index + 1][0] if index + 1 < len(tokens) else ""
    return previous not in {"=", "!", "<", ">"} and following != "="


def lexical_tokens(text: str) -> list[tuple[str, int, int]] | None:
    """Tokenize delimiters, identifiers, and operators while skipping literals."""
    return lexical_tokens_from(text, 0)


def lexical_tokens_from(text: str, start: int) -> list[tuple[str, int, int]] | None:
    """Return C++ delimiters and names while keeping literal contents opaque."""
    tokens: list[tuple[str, int, int]] = []
    index = start
    while index < len(text):
        skipped = skip_trivia_or_literal(text, index)
        if skipped is None:
            return None
        index = skipped
        if index >= len(text):
            break
        token_end = token_bounds(text, index)
        end = token_end if token_end is not None else index + 1
        if token_end is not None or text[index] in "<>()[]{} ,=>!":
            tokens.append((text[index:end], index, end))
        index = end
    return tokens


def skip_trivia_or_literal(text: str, index: int) -> int | None:
    """Skip whitespace, comments, and quoted literals; malformed quotes fail closed."""
    while index < len(text) and text[index].isspace():
        index += 1
    if text.startswith("//", index):
        newline = text.find("\n", index)
        return len(text) if newline < 0 else skip_trivia_or_literal(text, newline + 1)
    if text.startswith("/*", index):
        end = text.find("*/", index + 2)
        return None if end < 0 else skip_trivia_or_literal(text, end + 2)
    raw_end = raw_literal_end(text, index)
    if raw_end is not None:
        return raw_end if raw_end >= 0 else None
    quote = quote_start(text, index)
    return skip_quoted(text, quote) if quote is not None else index


def raw_literal_end(text: str, index: int) -> int | None:
    """Skip raw strings by their exact delimiter; malformed raw literals fail closed."""
    prefixes = ('u8R"', 'uR"', 'UR"', 'LR"', 'R"')
    prefix = next((item for item in prefixes if text.startswith(item, index)), None)
    if prefix is None:
        return None
    delimiter_start = index + len(prefix)
    open_paren = text.find("(", delimiter_start, delimiter_start + 17)
    if open_paren < 0:
        return -1
    delimiter = text[delimiter_start:open_paren]
    if any(character.isspace() or character in "()\\" for character in delimiter):
        return -1
    terminator = ")" + delimiter + '"'
    end = text.find(terminator, open_paren + 1)
    return end + len(terminator) if end >= 0 else -1


def quote_start(text: str, index: int) -> int | None:
    """Locate ordinary, prefixed, or raw C++ string and character literals."""
    for prefix in (
        'u8R"',
        'uR"',
        'UR"',
        'LR"',
        'R"',
        'u8"',
        'u"',
        'U"',
        'L"',
        '"',
        "u'",
        "U'",
        "L'",
        "'",
    ):
        if text.startswith(prefix, index):
            return index + len(prefix) - 1
    return None


def skip_quoted(text: str, quote: int) -> int | None:
    """Skip one escaped quoted literal, returning end-of-input if unterminated."""
    delimiter = text[quote]
    index = quote + 1
    while index < len(text):
        if text[index] == "\\":
            index += 2
            continue
        if text[index] == delimiter:
            return index + 1
        index += 1
    return None


def token_bounds(text: str, start: int) -> int | None:
    """Return end of an identifier or recognized paired operator token."""
    if text[start].isalpha() or text[start] == "_":
        end = start + 1
        while end < len(text) and (text[end].isalnum() or text[end] == "_"):
            end += 1
        return end
    for operator in ("::", "&&", "||", "==", "!=", "<=", ">=", "<<"):
        if text.startswith(operator, start):
            return start + len(operator)
    return None

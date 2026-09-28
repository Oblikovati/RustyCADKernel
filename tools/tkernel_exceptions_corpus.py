"""Deterministic requests for the TKernel exception class tree and failure printing.

Every class is thrown with an empty message (Print omits the ": " separator), a plain
message, a message with an embedded NUL (the C string ends there) and a UTF-8 message.
"""

CLASSES = [
    "Standard_Failure", "Standard_AbortiveTransaction", "Standard_ConstructionError",
    "Standard_DimensionError", "Standard_DimensionMismatch", "Standard_DivideByZero",
    "Standard_DomainError", "Standard_ImmutableObject", "Standard_LicenseError",
    "Standard_LicenseNotFound", "Standard_MultiplyDefined", "Standard_NegativeValue",
    "Standard_NoMoreObject", "Standard_NoSuchObject", "Standard_NotImplemented",
    "Standard_NullObject", "Standard_NullValue", "Standard_NumericError", "Standard_OutOfRange",
    "Standard_Overflow", "Standard_ProgramError", "Standard_RangeError", "Standard_TypeMismatch",
    "Standard_Underflow", "StdFail_InfiniteSolutions", "StdFail_NotDone", "StdFail_Undefined",
    "StdFail_UndefinedDerivative", "StdFail_UndefinedValue", "Plugin_Failure",
]
MESSAGES = [b"", b"index 7 out of 1..3", b"head\0tail", "Überlauf".encode()]


def append_exceptions_requests(requests: list[str]) -> None:
    """Append one request per class and message; append_exceptions_requests(requests)."""
    for name in CLASSES:
        requests.extend(f"exception_kind {name} {message.hex() or '-'}" for message in MESSAGES)

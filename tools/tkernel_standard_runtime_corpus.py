"""Deterministic requests for Standard handles, transients, type descriptors, failures, error handler and TColStd handles.

Excluded inputs, each C++ undefined behavior or a platform-dependent value:
- `Standard_Type::Size()` and `SystemName()` (C++ object sizes and mangled names are not Rust's);
  `Print` is compared with the descriptor address replaced by `ADDR`.
- Handle `<` and `std::hash` (they order and hash object addresses).
- Stack traces whose frames only partly fit the buffer (the frame text is Rust's, not C++'s);
  the corpus uses buffers of at most 31 bytes, where no frame fits, or of 4096 bytes and more
  with at most 3 frames, where all fit. Prefixes never hold a NUL and always fit the buffer
  (C++ `strcat` would overflow it). Stack lengths stay small: C++ multiplies the length by 200
  in `int` and `alloca`s one pointer per frame. No skip count exceeds the stack depth: C++ then
  passes a negative count to `backtrace_symbols`, which crashes the native probe (SIGSEGV).
- `Standard_ErrorHandler::Raise()` without an error and `Abort` without a handler exit the process;
  scripts never unregister a callback after its handler object is gone.
"""
import random

TYPES = ["Standard_Transient", "Standard_Type", "Standard_Persistent", "TColStd_HPackedMapOfInteger",
         "NCollection_BaseAllocator", "NCollection_IncAllocator", "NCollection_HeapAllocator"]
CLASSES = ["Standard_Transient", "Standard_Persistent", "TColStd_HPackedMapOfInteger", "NCollection_IncAllocator"]
SIGNALS = ["OSD_SIGBUS", "OSD_SIGHUP", "OSD_SIGILL", "OSD_SIGINT", "OSD_SIGKILL", "OSD_SIGQUIT", "OSD_SIGSEGV",
           "OSD_SIGSYS", "OSD_Exception_ACCESS_VIOLATION", "OSD_Exception_ARRAY_BOUNDS_EXCEEDED",
           "OSD_Exception_ILLEGAL_INSTRUCTION", "OSD_Exception_IN_PAGE_ERROR", "OSD_Exception_INT_OVERFLOW",
           "OSD_Exception_INVALID_DISPOSITION", "OSD_Exception_NONCONTINUABLE_EXCEPTION",
           "OSD_Exception_PRIV_INSTRUCTION", "OSD_Exception_STACK_OVERFLOW", "OSD_Exception_STATUS_NO_MEMORY",
           "Standard_DivideByZero", "Standard_NumericError", "Standard_Overflow", "Standard_ProgramError",
           "Standard_Underflow"]
MESSAGES = [b"", b"plain message", b"head\0tail", "Überlauf".encode(), b"a" * 300]
INT_MIN, INT_MAX = -(2 ** 31), 2 ** 31 - 1


def hexed(data: bytes) -> str:
    """Request field for raw bytes, '-' when empty; hexed(b'A') == '41'."""
    return data.hex() or "-"


def append_standard_runtime_requests(requests: list[str]) -> None:
    """Append this area's requests with its own seed; append_standard_runtime_requests(requests)."""
    requests.extend(area_requests(random.Random(20260931)))


def area_requests(rng: random.Random) -> list[str]:
    """Every request of this area, drawn from `rng`; area_requests(random.Random(1))."""
    return (type_requests() + object_requests() + handle_requests(rng) + hpacked_requests(rng)
            + failure_requests() + stack_requests() + handler_requests(rng) + abort_requests())


def type_requests() -> list[str]:
    """Every descriptor, and SubType by name with exact, prefix, NUL, empty and non-ASCII names."""
    names = [name.encode() for name in TYPES] + [b"", b"Standard", b"Standard_Transient\0x", b"Standard_Transient ",
                                                  "Überlauf".encode(), b"standard_transient", b"Standard_Failure"]
    requests = [f"runtime_type {name}" for name in TYPES]
    requests += [f"runtime_subtype_name {name} {hexed(query)}" for name in TYPES for query in names]
    return requests


def object_requests() -> list[str]:
    """IsInstance/IsKind by every class name and a few odd names, for each probe class."""
    names = [name.encode() for name in TYPES] + [b"", b"Standard_Persistent\0", b"Standard_Tr", b"RuntimeProbe"]
    return [f"runtime_object {cls} {hexed(name)}" for cls in CLASSES for name in names]


def handle_requests(rng: random.Random) -> list[str]:
    """Fixed scripts for each operation, then random scripts over three slots."""
    fixed = ["T0 c10 c21 x0 x1 x2", "T0 m10 m01 m00", "D0 d10 d20 x0 x1 x2", "T0 d10 s21 p01 c00 p00",
             "P0 r0 z0 z1", "T0 T1 c01 c10 m12", "D0 c10 p21 x1 m01 z2", "x0 c01 m12 p20 d01 s11"]
    ops = ["T", "D", "P", "x", "r", "z", "c", "m", "d", "s", "p"]
    scripts = list(fixed)
    for _ in range(60):
        script = []
        for _ in range(rng.randint(1, 14)):
            op = rng.choice(ops)
            slots = rng.randint(0, 2), rng.randint(0, 2)
            script.append(f"{op}{slots[0]}" if op in "TDPxrz" else f"{op}{slots[0]}{slots[1]}")
        scripts.append(" ".join(script))
    return [f"runtime_handle_script {script}" for script in scripts]


def hpacked_requests(rng: random.Random) -> list[str]:
    """Bucket counts at and around the int limits (no values for huge ones) and random value sets."""
    requests = [f"runtime_hpacked {buckets}" for buckets in (INT_MIN, -1, 0, 1, 2, 100, INT_MAX)]
    edges = [0, 1, -1, 31, 32, 33, 63, 64, INT_MIN, INT_MAX, INT_MIN + 1, INT_MAX - 1]
    requests.append("runtime_hpacked 1 " + " ".join(map(str, edges + edges)))
    for buckets in (0, 1, 5, 100):
        values = [rng.randint(-5000, 5000) for _ in range(rng.randint(1, 60))]
        requests.append(f"runtime_hpacked {buckets} " + " ".join(map(str, values)))
    return requests


def failure_requests() -> list[str]:
    """Standard_Failure messages, explicit stacks, and Standard_OutOfMemory around its 1023-byte buffer."""
    requests = [f"runtime_failure {hexed(message)}" for message in MESSAGES]
    stacks = [b"", b"\ntrace line", b"stack\0cut", "Überlauf".encode()]
    requests += [f"runtime_failure_stack {hexed(m)} {hexed(s)}" for m in MESSAGES[:4] for s in stacks]
    long_texts = [b"b" * 1022, b"c" * 1023, b"d" * 1024, b"e" * 3000, b"f" * 1022 + "é".encode()]
    oom = ["null", "-", hexed(b"no memory"), hexed(b"cut\0off"), hexed("Überlauf".encode())]
    oom += [hexed(text) for text in long_texts]
    requests += [f"runtime_out_of_memory {first} {second}" for first in oom for second in ("null", "-", oom[2], oom[-1])]
    return requests


def stack_requests() -> list[str]:
    """Stack-trace lengths, and StackTrace arguments whose result does not depend on frame text."""
    requests = [f"runtime_stack_length {length}" for length in (INT_MIN, -1, 0, 1, 2, 5, 50)]
    for size in (0, -1, 1, 14, 15, 16, 20, 29, 30, 31, 4096, 65536):
        for traces, skip in ((1, 0), (3, 1), (0, 0), (-2, 0), (2, -1)):
            for prefix in (b"", b"abc"):
                if size > 0 and len(prefix) + 1 > size:
                    continue
                requests.append(f"runtime_stack_trace {size} {traces} {skip} {hexed(prefix)}")
    return requests


def handler_requests(rng: random.Random) -> list[str]:
    """Fixed handler/callback scripts for each rule, then random ones."""
    fixed = ["t h t h t D1 t D0 t", "h h D1 t D1 t", "c R0 t h c c R1 R2 R0 D0 t", "h c c R0 R1 U0 D0",
             "h c R0 D0 h R0 D1 U0 R0 D1", "h h c c R0 D0 R1 t", "h c R0 h c R1 D0 t", "c c t U0 U1 h R0 R0 U1 D0",
             "h c c R0 R1 D0 U0 U1 h R1 R0 D1", "h h h c R0 D1 t D2 t D0 t"]
    scripts = list(fixed)
    for _ in range(40):
        handlers = callbacks = 0
        script = []
        for _ in range(rng.randint(1, 16)):
            choice = rng.choice("hhccDRRUt")
            if choice in "hct":
                handlers += choice == "h"
                callbacks += choice == "c"
                script.append(choice)
            elif choice == "D" and handlers:
                script.append(f"D{rng.randrange(handlers)}")
            elif choice in "RU" and callbacks:
                script.append(f"{choice}{rng.randrange(callbacks)}")
        scripts.append(" ".join(script) or "t")
    return [f"runtime_handler_script {script}" for script in scripts]


def abort_requests() -> list[str]:
    """Every class of the signal variant, at one to three nested handlers."""
    messages = [b"", b"signal", "Überlauf".encode()]
    return [f"runtime_abort {cls} {hexed(messages[index % 3])} {1 + index % 3}"
            for index, cls in enumerate(SIGNALS)] + [f"runtime_abort Standard_Overflow {hexed(b'x')} {depth}"
                                                    for depth in (1, 2, 3)]

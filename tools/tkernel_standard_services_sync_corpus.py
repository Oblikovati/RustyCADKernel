"""Requests for Standard_Mutex / Standard_Condition token scripts and Standard_CLocaleSentry
(operation prefix `services_`).

Excluded inputs: scripts that end with the mutex still locked (destroying a locked
pthread mutex is undefined) and waits whose outcome depends on scheduling (every timed
wait either has no setter or a generous limit).
"""
import math
import struct

MUTEX_SCRIPTS = [
    "T U",
    "L t U t",
    "L L t U t U t",
    "T T t U U t",
    "L u t U t",
    "U t",
    "S t s t",
    "N t s t",
    "S S L t U s s t",
    "L S U t s t",
    "S T U t s",
]

CONDITION_SCRIPTS = [
    "0 C X C",
    "1 C X C X",
    "0 S C R C",
    "0 W0 W-5 W10 S W0 W-1 W10",
    "1 W0 X W0",
    "0 D20,5000 C",
    "0 D0,5000 X",
    "0 I20 C R I0 X",
    "1 I50 C",
]

LOCALE_VALUES = [1.5, -2.25, 0.0, -0.0, 1e300, 1e-300, 5e-324, 123456789.0, 0.1, math.inf, -math.inf, math.nan]


def bits(value: float) -> int:
    """IEEE-754 bit pattern of a double, as the probes read it; bits(1.0) == 0x3FF0000000000000."""
    return struct.unpack("<Q", struct.pack("<d", value))[0]


def sync_requests() -> list[str]:
    """Mutex and condition scripts; sync_requests()."""
    return ([f"services_mutex {script}" for script in MUTEX_SCRIPTS]
            + [f"services_condition {script}" for script in CONDITION_SCRIPTS]
            + [f"services_clocale {bits(value)}" for value in LOCALE_VALUES])

"""Request lines for the OSD process probe family (OSD_Process, OSD_Host, OSD_Environment,
OSD_SharedLibrary, OSD_MemInfo).

Environment sequences cover valid names, `$` in names and values, non-printable bytes, an
empty name, a name holding `=`, redefinition and Unix Remove-to-empty; working-directory
changes cover a real directory, a relative one and a missing one; the loader covers lazy and
eager opens of system libraries, the main program, a missing library and missing symbols;
MemInfo covers every single-counter activity mask with and without Update, plus mixes.
"""
from __future__ import annotations

ENV_SEQUENCES = [
    "OSD_PROBE_A one two",
    "OSD_PROBE_A three four",
    "OSD_PROBE_B - value",
    "OSD_PROBE_C $HOME/x plain",
    "OSD_PROBE_D plain $dollar",
    "OSD_PROBE_$E v w",
    "OSD_PROBE_F café x",
    "OSD_PROBE_G a café",
    "- empty_name other",
    "OSD_PROBE_H=I nested value",
    "OSD_PROBE_J ~!@#%^&*()_+{}|:<>? ,./;'[]",
]
ENV_NAMES = [
    "OSD_PROBE_A OSD_PROBE_B",
    "OSD_PROBE_A OSD_PROBE_UNSET",
    "OSD_PROBE_UNSET OSD_PROBE_$X",
    "OSD_PROBE_$X OSD_PROBE_A",
    "LANG LC_ALL",
    "- OSD_PROBE_C",
    "OSD_PROBE_A é",
]
CHDIRS = ["/tmp/", "/", "/tmp/../usr/", "/nonexistent_osd_probe_dir/", "tools/", "-"]
SHARED_LIBRARIES = [
    "libc.so.6 0 malloc",
    "libc.so.6 1 malloc",
    "libm.so.6 1 cos",
    "libm.so.6 0 no_such_symbol_osd_probe",
    "- 0 malloc",
    "- 1 free",
    "libosd_probe_missing.so 0 malloc",
    "/nonexistent/libosd_probe.so 1 malloc",
]


def cases() -> list[str]:
    """Every process request line, in a fixed order.

    >>> sum(1 for c in cases() if c.startswith("osd_meminfo"))
    22
    """
    lines = [f"osd_env {sequence}" for sequence in ENV_SEQUENCES]
    lines += [f"osd_env_name {pair}" for pair in ENV_NAMES]
    lines += ["osd_host", "osd_process"]
    lines += [f"osd_chdir {target}" for target in CHDIRS]
    lines += [f"osd_shlib {request}" for request in SHARED_LIBRARIES]
    masks = [0, 127] + [1 << counter for counter in range(7)] + [0b0110000, 0b1000001]
    lines += [f"osd_meminfo {mask} {update}" for mask in masks for update in (0, 1)]
    return lines

"""Requests for Plugin::Load (operation prefix `services_plugin`).

Only failure paths are probed: a successful load needs a native plugin library exporting
PLUGINFACTORY. Library names never name an existing library (a real libc.so, for example,
is a linker script whose dlopen error text differs by system).
"""

GUIDS = ["f69e10e5-9a4e-11d4-8b8c-0060b0ee281b", "00000000-0000-0000-0000-000000000000",
         "ABCDEF01-2345-6789-ABCD-EF0123456789"]


def plugin_requests() -> list[str]:
    """Missing resources and libraries that cannot be opened, quiet and verbose."""
    requests = []
    for guid in GUIDS:
        files = [b"", b"Other.Location: RustyCadNoSuchLibrary\n",
                 f"{guid.lower()}.Location: RustyCadNoSuchLibrary\n".encode(),
                 f"{guid.lower()}.Location:\n".encode(),
                 f"{guid.lower()}.Location: RustyCad No Such\n".encode(),
                 f"{guid.upper()}.Location: RustyCadUpperCase\n".encode()]
        for data in files:
            requests += [f"services_plugin {guid} {data.hex() or '-'} {verbose}" for verbose in (0, 1)]
    return requests

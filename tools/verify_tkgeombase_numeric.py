#!/usr/bin/env python3
"""Compare the TKGeomBase port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkgeombase_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkgeombase_probe.cxx against libTKGeomBase (and the toolkits it
depends on) and the Rust twin crates/tkgeombase/examples/numeric_probe.rs, runs both on
tools/tkgeombase_corpus.py and requires identical bits (NaN compared by classification),
through the shared driver tools/toolkit_numeric.py. The AppCont tables are checked
against the OCCT sources (tools/generate_appcont_tables.py --check) first.
"""
from __future__ import annotations
from tkgeombase_corpus import append_tkgeombase_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKGeomBase request corpus, one request per line."""
    requests: list[str] = []
    append_tkgeombase_requests(requests)
    requests += [row.replace(row.split()[0], row.split()[0]+"_owned", 1)
                 for row in requests if row.split()[0] in ("ex_locpc", "ex_locpc2d")]
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKGeomBase', 'tools/oracle/tkgeombase_probe.cxx',
                       ['TKGeomBase', 'TKG3d', 'TKG2d', 'TKMath', 'TKernel'], 'rustycad-tkgeombase', corpus,
                       generators=('tools/generate_appcont_tables.py',))


if __name__ == '__main__':
    verify_numeric(ORACLE)

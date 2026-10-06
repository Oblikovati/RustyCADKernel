# Returned private-type capture, version 1

This staging snapshot preserves the published primary inventory policy. Coverage
credit and API denominator completeness remain false. The coordinating agent
reviews any later publication to `porting/api-inventory` separately.

## Evidence

- Global capture: 6,317 header candidates; 5,639 standalone parses, 260 fresh
  translation-unit context parses, 384 included fragments, six parse failures,
  and 28 headers without compilation configuration.
- All 271,643 published primary identities have unchanged semantic projections,
  including 108,622 API candidates. The baseline is main commit
  `14dba9ccefe050fe5cd7aeccd6552381a3e8b61f`.
- Returned-type evidence has 7,352 observations and 1,418 distinct edges. There
  are 15 distinct inferred-public members on two private record types. The
  7,341 template/dependent observations remain unresolved.
- The existing 5,224 conflicting-declaration observations remain open, including
  the public `math_Vector` alias. No API parity credit follows from these counts.

The global and TKernel reports retain exact IDs, overloaded signatures, access
observations, and declaration-field comparisons. TKernel additionally records
outside-scope implementation rows, first-observation default-argument differences,
and existing non-API identity collisions. The global comparison is unchanged.

## Replay

```sh
PYTHONPATH=tools python3 tools/verify_returned_type_recapture.py --deep
```

The receipt is `porting/audits/returned-type-scope-recapture-v1.json`.
The native evidence bundle is
`porting/runtime-evidence/returned-type-recapture-native-v1.tar.zst`.
It preserves raw compiler output, status files, input manifests, real-TU retries,
CMake configuration, generated headers, and the exact extractor executable.
Original absolute paths are part of the capture provenance; a fresh capture at
another location needs its own configuration and fingerprints.

Rebuilding the extractor and rerunning the capture remains necessary for further
platform, dependency, template, inherited-member, or callability coverage. The
published snapshot and all earlier audit receipts are unchanged by this staging.

# Martensite lighting evidence

The integrated renderer remains partial. `summary.json` records the actual main
execution revision and failing gates. `functional.json.gz` contains CPU output,
all 252 native capture comparisons, both runtime probes, raw timing samples and
source fingerprints. Exact RGBA comparisons were not relaxed.

`captures.tar.gz` contains only generated RGBA/depth captures, PNGs and diff
masks from that run. It contains no checkout, Git metadata or build artifacts.
Extract it with `tar -xzf captures.tar.gz`; case labels match the receipt.
The four loose PNGs show the retained facet captures and exact-color error masks.

The forward and reverse timing reports compare the previous 20 workloads with
freshly rebuilt main before lighting. Both orders, including their failures,
are retained. The new lighting timing baseline is collected, not accepted.
Main Clippy and Python test logs are included. Full inventory completion is false.

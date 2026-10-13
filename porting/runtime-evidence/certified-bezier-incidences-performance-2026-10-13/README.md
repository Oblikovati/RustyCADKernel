# Frozen certified incidence timings

Frozen source/evidence revision: `93127260b05efe4621e43f2bae13506bb0ec2a9f`.
Executable was built from `c83c0f08a41008a58f2262d51749042fac7bde73`;
the intervening commit only added acceptance receipts. Its SHA-256 is
`580f24d9b5eb2ea31b900ebe6f7f79ae74492ed670ae1d8e703e91cf8869f8ff`.
The complete 45,774-file frozen manifest and executable were verified unchanged
before and after the batch. Root independently audited the freeze and granted
an exclusive quiet CPU timing window. No compilation or source change occurred.

Each of six identical accepted workloads ran in its own `/usr/bin/time -v`
process: twenty warmups and one hundred retained samples. The executable asserts
complete result, expected contact count, denominator root-freedom and achieved
world error before each sample is retained. Timing includes the entire query,
acceptance checks and result destruction. Peak RSS includes process startup and
corpus preparation; it is not an incremental allocation estimate. GPU memory is
not applicable to this CPU-only operation.

All 600 raw samples are positive and retained in the original JSON files.
Original RSS/stdout/stderr, exact commands, host/environment details and fourteen
process observations are included. Those observations recorded no heavy
compiler jobs; they are snapshots rather than continuous host-load telemetry.
All six processes returned zero. `summary.json` derives median, arithmetic mean
and nearest-rank p95 from the raw vectors, alongside whole-process peak RSS.
No uncertified or failed workload is an equivalent comparator. No comparative
baseline is accepted (`baseline_accepted=false`).

These metrics apply to this complete isolated Bézier/line operation and its six
accepted inputs, not whole application blocker #10, other curve families,
coincident overlap construction, or reusable wire/arrangement incidence.
Source/package acceptance is recorded separately in the preceding archive
receipt; mapped API parity, extension acceptance and application acceptance
remain distinct.

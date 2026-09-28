"""Report, level, printer and algorithm requests of the Message probe family.

Used by `tkernel_message_corpus.cases`; see that module for the protocol conventions.
"""
from __future__ import annotations

import random

PROBE_DIR = "target/oracle-tkernel/message-probe"
KINDS = ["P", "X", "Y", "N", "R", "Ea", "Eb", "Mm", "Ss", "Oo", "E"]
EXTENDED_KINDS = ["Ea", "Eb", "Mm", "Ss", "Oo", "E"]
METRICS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]


def hexb(data: bytes) -> str:
    """Byte pairs, `-` for none.

    >>> hexb(b"A")
    '41'
    """
    return data.hex() if data else "-"


def report_steps(rng: random.Random, count: int) -> list[str]:
    """Random report steps, never `cleart` (its native loop hangs on foreign alerts)."""
    steps = []
    for _ in range(count):
        gravity = rng.randrange(5)
        choice = rng.random()
        if choice < 0.45:
            steps.append(f"add:{gravity}:{rng.choice(KINDS)}")
        elif choice < 0.6:
            steps.append(f"ext:{gravity}:{rng.choice(EXTENDED_KINDS)}")
        elif choice < 0.7:
            steps.append(f"nest:{gravity}:{rng.choice(KINDS)}")
        elif choice < 0.75:
            steps.append(f"limit:{rng.choice([-1, 0, 1, 2, 5])}")
        elif choice < 0.8:
            steps.append(rng.choice(["clear", f"clearg:{gravity}", "clearmetrics"]))
        elif choice < 0.87:
            steps.append(f"metric:{rng.choice(METRICS)}:{rng.randrange(2)}")
        elif choice < 0.94:
            steps.append(f"has:{rng.choice('PXYNRE')}")
        else:
            steps.append(f"merge:{rng.choice(['a', str(gravity)])}")
    return steps


def report_cases(rng: random.Random) -> list[str]:
    """Merging, limits, clearing by type (only when safe) and random scripts."""
    cases = ["report_run", "report_run add:1:P add:1:P add:1:P", "report_run limit:2 add:1:X add:1:X add:1:X",
             "report_run add:2:N add:2:N add:2:R add:2:R", "report_run add:0:X add:0:Y add:0:X has:X has:Y has:P",
             "report_run cleart:X", "report_run add:1:X add:3:X nest:1:P cleart:X",
             "report_run ext:1:Ea nest:2:X nest:2:X add:1:Ea add:1:E has:E cleart:E",
             "report_run ext:4:Mm nest:0:Mm nest:0:Mm metric:5:1 metric:7:1 metric:5:0 metric:12:1",
             "report_run add:1:Ea add:1:Ea add:1:Ea merge:a merge:1"]
    for _ in range(250):
        cases.append("report_run " + " ".join(report_steps(rng, rng.randint(1, 14))))
    for kind in "PXYNRE":
        steps = [f"add:{rng.randrange(5)}:{kind if kind != 'E' else 'Ea'}" for _ in range(rng.randint(0, 4))]
        cases.append("report_run " + " ".join(steps + [f"cleart:{kind}", f"has:{kind}"]))
    return cases


def level_steps(rng: random.Random, count: int) -> list[str]:
    """Level scripts: opening/closing levels while alerts and messages arrive."""
    steps = []
    for index in range(count):
        gravity = rng.randrange(5)
        choice = rng.random()
        if choice < 0.2:
            steps.append(f"open:lvl{index}")
        elif choice < 0.3:
            steps.append("close")
        elif choice < 0.5:
            steps.append(f"add:{gravity}:{rng.choice(KINDS)}")
        elif choice < 0.6:
            steps.append(f"levadd:{gravity}:{rng.choice(KINDS)}")
        elif choice < 0.75:
            steps.append(f"send:{gravity}:{hexb(rng.choice([b'msg', b'second', b'', b'key: value']))}")
        elif choice < 0.87:
            steps.append(f"stream:{gravity}:{hexb(rng.choice([b'name', b'\"Key\": 2', b'', b'plain text']))}")
        else:
            steps.append(f"metric:{rng.choice(METRICS)}:1")
    return steps


def level_cases(rng: random.Random) -> list[str]:
    """Levels of the default report, with and without alert metrics."""
    cases = ["level_run", "level_run open:a add:1:Ea add:1:Eb close add:1:Ea",
             "level_run open:a open:b add:2:Ea close add:2:Eb close",
             "level_run metric:5:1 metric:10:1 metric:2:1 open:m add:1:Ea add:1:Eb close",
             "level_run open:a levadd:1:P levadd:1:Ea send:1:6d7367 stream:1:6e616d65 close"]
    cases += ["level_run " + " ".join(level_steps(rng, rng.randint(1, 16))) for _ in range(150)]
    return cases


def printer_to_report_cases(rng: random.Random) -> list[str]:
    """PrinterToReport: pending names, JSON streams, objects, trace levels, metrics."""
    texts = [b"name", b"\"Key\": 1", b"", b"other", b"a: b", "café".encode()]
    cases = ["ptr_run send:1:6e616d65 stream:1:6e616d65 stream:1:6e616d65 send:1:6f",
             "ptr_run stream:0:6e stream:0:22613a2031 obj:0 obj:4",
             "ptr_run trace:3 send:1:61 send:4:62 cstr:0:63 ext:2:64,65 stream:0:66",
             "ptr_run metric:5:1 send:1:61 stream:1:62 obj:1 stream:1:223a2031"]
    for _ in range(150):
        steps = []
        for _ in range(rng.randint(1, 10)):
            gravity = rng.randrange(5)
            kind = rng.choice(["send", "cstr", "stream", "stream", "ext", "obj", "trace", "metric"])
            if kind in ("send", "cstr", "stream"):
                steps.append(f"{kind}:{gravity}:{hexb(rng.choice(texts))}")
            elif kind == "ext":
                steps.append(f"ext:{gravity}:{rng.choice(['-', '61', '20ac,31'])}")
            elif kind == "metric":
                steps.append(f"metric:{rng.choice(METRICS)}:{rng.randrange(2)}")
            else:
                steps.append(f"{kind}:{gravity}")
        cases.append("ptr_run " + " ".join(steps))
    return cases


STATUSES = [0x100, 0x101, 0x11F, 0x200, 0x201, 0x21F, 0x400, 0x402, 0x800, 0x801, 0x81F, 0]


def algo_steps(rng: random.Random, count: int) -> list[str]:
    """Algorithm scripts over both algorithms, the catalog and message sending."""
    steps = []
    for _ in range(count):
        status = rng.choice(STATUSES)
        choice = rng.random()
        prefix = "o" if rng.random() < 0.3 else ""
        if choice < 0.15:
            steps.append(f"{prefix}st:{status}")
        elif choice < 0.3:
            steps.append(f"{prefix}si:{status}:{rng.choice([1, 7, 42, 1000, -3, 64, 65])}")
        elif choice < 0.45:
            kind = rng.choice(["ss", "sa"])
            steps.append(f"{prefix}{kind}:{status}:{hexb(rng.choice([b'edge', b'face', b'', 'café'.encode()]))}:{rng.randrange(2)}")
        elif choice < 0.55:
            steps.append(f"{prefix}se:{status}:{rng.choice(['-', '61,62', '20ac'])}:{rng.randrange(2)}")
        elif choice < 0.6:
            steps.append(f"sm:{status}:{rng.choice(['25,64', '41,20,25,73', '4f,4b'])}")
        elif choice < 0.65:
            steps.append(rng.choice(["clear", "add", f"addf:{status}", f"sn:{status}"]))
        elif choice < 0.8:
            steps.append(rng.choice([f"send:{rng.randrange(5)}:{rng.choice([0, 1, 2, 20])}",
                                     f"sendst:{status}:{rng.randrange(5)}:{rng.choice([0, 1, 3])}"]))
        else:
            steps.append(f"{rng.choice(['nums', 'strs'])}:{status}:{rng.choice([0, 1, 2, 3, 20])}")
    return steps


def algo_cases(rng: random.Random) -> list[str]:
    """Algorithm statuses, catalog message names and report text."""
    keys = [b"Message_Algorithm.Warn1", b"Message_Algorithm.Fail2", b"Standard_Transient.Done1"]
    texts = ["57,61,72,6e,20,25,73", "46,61,69,6c,20,25,64,20,25,73", "44,6f,6e,65"]
    cases = [f"algo_run cat:{hexb(key)}:{text}" for key, text in zip(keys, texts)]
    cases += ["algo_run si:512:3 si:512:1 ss:512:6162:1 ss:512:6162:1 ss:512:6364:0 send:2:20",
              "algo_run sm:2049:25,64,25,73 sendst:2049:4:20 sendst:2049:4:20",
              "algo_run osi:512:5 osi:512:9 ose:512:61:1 ose:512:62:1 ose:512:63:1 add strs:512:20 nums:512:20",
              "algo_run si:1024:1 si:1024:2 si:1024:3 nums:1024:2 nums:1024:0 strs:1024:0 send:3:2"]
    cases += ["algo_run " + " ".join(algo_steps(rng, rng.randint(1, 14))) for _ in range(200)]
    return cases


def printer_cases(rng: random.Random) -> list[str]:
    """Messenger printer sequences, file printers and console colors."""
    cases = [f"printer_console {color} {intense}" for color in range(9) for intense in range(2)]
    cases.append(f"printer_badfile {hexb(PROBE_DIR.encode())}")
    for index in range(40):
        path = hexb(f"{PROBE_DIR}/printer{index % 4}.log".encode())
        messages = [f"{rng.randrange(5)}:{hexb(rng.choice([b'line', b'', b'second line', b'a' + bytes([0]) + b'b']))}"
                    for _ in range(rng.randint(0, 5))]
        prefill = hexb(rng.choice([b"", b"old" + bytes([10])]))
        cases.append(f"printer_file {path} {prefill} {rng.randrange(2)} {rng.randrange(5)} {rng.randrange(2)} "
                     + " ".join(messages))
    log = hexb(f"{PROBE_DIR}/messenger.log".encode())
    cases += [f"messenger_ops p p dup rm rm json r o:{log} rmt:B json n",
              f"messenger_ops p r o:{log} rmt:R json send:1:6869 rmt:O p send:0:74 trace:0 send:0:75",
              "messenger_ops p trace:3 send:2:61 send:3:62 send:4:63 r rmt:B"]
    for _ in range(80):
        steps = ["p"]
        for _ in range(rng.randint(1, 10)):
            steps.append(rng.choice(["p", "r", "dup", "rm", "rmt:B", "rmt:R", "rmt:O", "json", f"o:{log}",
                                     f"send:{rng.randrange(5)}:{hexb(b'text')}", f"trace:{rng.randrange(5)}"]))
        cases.append("messenger_ops " + " ".join(steps))
    return cases


def cases(rng: random.Random) -> list[str]:
    """Report, level, printer-to-report, algorithm and printer requests.

    >>> len(cases(random.Random(1))) > 0
    True
    """
    return (report_cases(rng) + level_cases(rng) + printer_to_report_cases(rng) + algo_cases(rng)
            + printer_cases(rng))

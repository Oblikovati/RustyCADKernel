"""Request lines for the OSD files probe family (OSD_FileNode, OSD_File, OSD_Directory,
OSD_DirectoryIterator, OSD_FileIterator).

Each request is one script run in a fresh "osdf" directory: building, opening and
appending in every mode, reads/writes/seeks mixed with the buffered ReadLine and Rewind,
end-of-file state, pending-error re-raises, every precondition failure, locks, node
removal/move/copy/protection on files, directories, read-only files, FIFOs, symlinks and
missing paths, directory builds with missing parents, temporaries, and the iterators with
their masks. Scripts avoid what is undefined natively: a FIFO open, a file object replaced
while open, Rewind of a never-opened file (NULL FILE*), Protection of a missing node (an
uninitialized stat buffer) and UnLock (an uninitialized flock).
"""
from __future__ import annotations

import random


def hexed(text: str) -> str:
    """Lowercase hex of the bytes, "-" for empty.

    >>> hexed("a\\n")
    '610a'
    """
    return text.encode().hex() if text else "-"


LINES = "first line\nsecond\n\nthird line is longer than sixteen bytes\nlast"

SCRIPTS = [
    # Build/Open/Append in every mode, then the kind, size and access predicates.
    "file a.txt build 2 write {lines} 60 seek 0 0 read 5 read 100 atend size kind isopen readable close atend",
    "file a.txt build 0 isopen readable size close",
    "file a.txt build 1 write {abc} 3 read 2",
    "file a.txt build 1 write {abc} 4 close file a.txt open 0 readb 10 atend",
    "file a.txt open 0 failed error perror error failed reset failed",
    "file a.txt open 2 isopen failed perror read 3",
    "fixture a.txt {lines} file a.txt open 0 readline 100 readline 100 readline 100 readline 100 readline 100 readline 100 atend readline 100",
    "fixture a.txt {lines} file a.txt open 0 readline 5 readline 5 readline 1 readline 0 readline -1",
    "fixture a.txt {lines} file a.txt open 0 readline 100 readb 6 readline 100 rewind readline 100 readb 3",
    "fixture a.txt {lines} file a.txt open 2 readline 100 seek 0 0 readline 100 read 4 rewind read 4 readline 7",
    "fixture a.txt {short} file a.txt open 0 readline 100 rewind readb 5 readline 100",
    "fixture a.txt {short} file a.txt open 2 readline 100 readline 100 atend write {abc} 3 readline 100 lastline 0 2 rewind lastline 0 1",
    "fixture a.txt {short} file a.txt open 1 write {abc} 3 readline 5 rewind close",
    "fixture a.txt {short} file a.txt append 1 write {abc} 3 seek 0 0 write {abc} 2 close file a.txt open 0 read 100",
    "file new.txt append 2 write {abc} 3 rewind readline 10 seek 0 0 readline 10 size",
    "file a.txt append 0 isopen failed",
    "fixture a.txt {short} file a.txt open 0 seek -5 0 failed seek 2 0 read 100 seek 0 2 read 3 atend",
    "fixture a.txt {short} file a.txt open 0 seek -5 0 read 2",
    "fixture a.txt {short} file a.txt open 0 seek 100 0 readb 4 atend seek 1 0 readb 2 atend",
    "fixture a.txt {nul} file a.txt open 0 read 10 seek 0 0 readb 10 seek 0 0 readline 10",
    # Precondition failures.
    "file a.txt read 1 readline 1 readb 1 write {abc} 1 seek 0 0 close atend lock 1 unlock",
    "file a.txt build 2 build 2 open 2 append 2 read 0 readline 0 readb 0 write {abc} 0 close",
    "file a.txt build 0 write {abc} 1 close file a.txt build 1 read 1 readline 1",
    "file - build 2 open 2 append 2 size exists kind readable",
    "file sub/ build 2",
    "dir sub dbuild file sub build 2 open 0 append 1 read 1 kind size",
    "file sub/x/ build 2 open 2 size failed perror",
    "file missing size failed error perror kind readable",
    # Locks.
    "file a.txt build 2 lock 1 lock 2 lock 3 prot lock 0 close",
    "file a.txt build 2 lock 3 close file a.txt prot",
    "file a.txt build 0 lock 2 failed perror",
    "file a.txt build 1 lock 1 failed perror",
    # Node operations on files.
    "fixture a.txt {abc} file a.txt exists prot setprot 0 7 1 0 prot readable remove exists",
    "fixture a.txt {abc} chmod a.txt 444 file a.txt remove failed perror exists",
    "fixture a.txt {abc} chmod a.txt 000 file a.txt readable open 0 failed perror",
    "fixture a.txt {abc} file a.txt move b.txt failed exists file b.txt exists copy c.txt failed file c.txt open 0 read 10 prot",
    "file none move b.txt failed perror copy c.txt failed perror remove failed perror",
    "fixture a.txt {abc} file a.txt copy sub/c.txt failed perror copy a.txt failed",
    "fixture a.txt {lines} file a.txt copy b.txt file b.txt size",
    "mkfifo p file p kind exists remove failed exists",
    "fixture a.txt {abc} symlink a.txt l file l kind size readable remove exists file a.txt exists",
    "symlink nowhere l file l exists kind remove failed perror",
    "file a.txt setprot 0 7 0 0 failed perror",
    "fixture a.txt {abc} file a.txt path moment",
    "file none moment",
    # Directories.
    "dir d dbuild dexists dfailed dprot dremove dexists",
    "dir d/e/f dbuild dfailed dexists dir d/e dexists dremove dfailed dperror",
    "dir d/e/f dbuildp 0 7 5 5 dfailed dexists dprot dir d/e dexists dremove dfailed dperror",
    "dir d/e.x/f.y dbuildp 0 7 5 5 dfailed dexists dir d/e.x dexists",
    "dir d dbuildp 0 7 5 0 dprot dsetprot 0 5 0 0 dprot dsetprot 0 7 0 0 dremove",
    "dir d dbuild dbuild dfailed",
    "fixture a {abc} dir a dbuild dfailed dir a/b dbuild dfailed dperror",
    "dir - dbuild",
    "dir d.x/e.y dbuild dfailed dexists",
    "dir d dbuild dir d dremove dexists dremove dfailed dperror",
    "dir d dbuildp 0 7 5 5 fixture d/f {abc} dir d dremove dfailed dperror",
    "dir d dbuild file d exists kind remove exists",
    "dir d dbuild dmoment dpath",
    "tempdir dexists dremove dexists",
    "tempfile write {abc} 3 seek 0 0 read 10 size close remove exists",
    "fixture a.txt {abc} file a.txt open 0 tempfile isopen close remove",
]

ITERATION_TREE = ("dir t dbuildp 0 7 5 5 dir t/sub.d dbuildp 0 7 5 5 dir t/other dbuildp 0 7 5 5 fixture t/a.step {abc} "
                  "fixture t/b.tar.gz {abc} fixture t/.hidden {abc} fixture t/noext {abc} "
                  "fixture t/x.c {abc} symlink a.step t/link.step symlink sub.d t/dlink")
MASKS = ["*", "", "*.step", "*.c", "a*", "*b*", "?", "noext", "*.*", "**", "sub*", ".*", "*x",
         "a.step", "*.d", "o*r", "*g*z", "b.*.gz"]


def cases() -> list[str]:
    """Every files request line, in a fixed order.

    >>> all(line.startswith("osd_files ") for line in cases())
    True
    """
    values = {"lines": hexed(LINES), "abc": hexed("abc"), "short": hexed("ab\nc"),
              "nul": "61620063640a65"}
    lines = [f"osd_files {script.format(**values)}" for script in SCRIPTS]
    for mask in MASKS:
        tree = ITERATION_TREE.format(**values)
        lines.append(f"osd_files {tree} iter d {hexed(mask)} t iter f {hexed(mask)} t")
    lines.append(f"osd_files iter f {hexed('*')} missing iter d {hexed('*')} missing")
    rng = random.Random(710)
    for _ in range(40):
        count = rng.randrange(1, 12)
        ops = " ".join(rng.choice([f"readline {rng.randrange(1, 9)}", f"readb {rng.randrange(1, 9)}",
                                   f"read {rng.randrange(1, 9)}", "rewind", f"seek {rng.randrange(0, 40)} 0",
                                   "atend"]) for _ in range(count))
        lines.append(f"osd_files fixture r {hexed(LINES)} file r open 0 {ops}")
    return lines

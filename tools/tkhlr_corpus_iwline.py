"""Original walking-line template contract with Contap runtime identity."""
from itertools import product
from tkgeomalgo_corpus_intpatch import _iwline_requests
from tktopalgo_corpus_breplib_records import numbers


def append_iwline_requests(requests: list[str]) -> None:
    """Retain the shared template scripts and expand every endpoint flag combination."""
    scripts: list[str] = []
    _iwline_requests(scripts)
    requests.extend(line.replace('ip_iwline ', 'ct_iwline ', 1) for line in scripts)
    points = [[1, i, 2*i, 3*i, 0.1*i, 0.2*i, 0.3*i, 0.4*i] for i in range(5)]
    for closed, first, last, reverse, cut in product(range(2),range(2),range(2),range(2),[1,3,5]):
        commands = points + [[3,closed,first,1,1,2,3,0.1,0.2], [6,last,5,4,5,6,0.3,0.4],
                             [7,2],[7,4],[8,1,2,3,2],[9,first],[10,last]]
        commands += [[11]] if reverse else []
        commands += [[12,cut]]
        requests.append('ct_iwline '+numbers([len(commands),*[x for command in commands for x in command]]))

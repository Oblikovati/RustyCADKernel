"""TopOpeBRepDS_InterferenceTool factory and shared parameter cases."""
from itertools import product


def append_interference_tool_requests(requests: list[str]) -> None:
    """Append all orientations/configurations and signed-zero parameters; e.g. append_interference_tool_requests([])."""
    for orientation, indices, parameter, config, bound in product(
        range(4), ((-2, 7), (0, 0), (4, -3)), ('-0', '0.25', '-1.5', '1e200'), range(3), (0, 1)
    ):
        requests.append(f'ds_interference_tool {orientation} {indices[0]} {indices[1]} {parameter} {config} {bound}')

"""All ShapeAnalysis_Curve projection overloads and output-preserving failure paths."""
from itertools import product
from tkshhealing_corpus_shapeconstruct import construct_curves_3d, construct_curves_2d
from tktopalgo_corpus_breplib_records import numbers


def append_analysis_curve_requests(requests:list[str])->None:
    """Append direct/native extrema and fallback inputs; append_analysis_curve_requests(lines)."""
    for operation,curve,point,tolerance,adjust in product(
            range(6),construct_curves_3d(),[(0,0,0),(0.1,0.2,0.3),(3,4,5)],[-1,0,1e-7,10],range(2)):
        requests.append('sh_analysis_projection '+numbers([operation,*curve,*point,tolerance,0,1,0.4,adjust]))
    for operation,curve,bounds,previous in product(
            [1,2,3,4,5],construct_curves_3d(),[(1,0),(0.5,0.5),(-2,3),(5,8)],[-3,6]):
        requests.append('sh_analysis_projection '+numbers([operation,*curve,1,2,3,1e-7,*bounds,previous,0]))
    for value,first,last in product([-1e20,-7,-0.0,0,7,1e20],[-3,0,4],[-6,0,1e-110,6]):
        requests.append('sh_analysis_period '+numbers([value,first,last]))
    for curve, bounds in product(construct_curves_2d(), [(0,1),(-1,2),(1,0),(0.5,0.5),(1e-9,1-1e-9)]):
        requests.append('sh_analysis_samples2d '+numbers([*curve,*bounds]))

"""All ShapeAnalysis_Curve projection overloads and output-preserving failure paths."""
from itertools import product
from tkshhealing_corpus_shapeconstruct import construct_curves_3d, construct_curves_2d
from tktopalgo_corpus_breplib_records import numbers, geom2d_gp, geom2d_bezier, geom_bezier, geom_bspline


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
    for curve, tolerance in product(construct_curves_3d(), [-1,0,1e-7,1,10]):
        requests.append('sh_analysis_closure '+numbers([3,*curve,tolerance]))
    for curve in construct_curves_2d():
        requests.append('sh_analysis_closure '+numbers([2,*curve]))
    for curve, bounds in product(construct_curves_3d(), [(0,1),(-1,2),(1,0),(0.5,0.5),(1e-9,1-1e-9),(0,8)]):
        requests.append('sh_analysis_samples3d '+numbers([*curve,*bounds]))
    seams = construct_curves_2d() + [geom2d_gp(0,p=p,d=d) for p,d in product([(0,0),(2,-3)],[(1,0),(-1,0),(0,1),(0,-1),(1,1)])]
    seams += [geom2d_bezier([(0,0),(0,0)]), geom2d_bezier([(0,0),(1e-100,0)])]
    for first, second in product(seams, repeat=2):
        requests.append('sh_analysis_seam '+numbers([*first,*second]))
    closed = [(0,0,0),(1,2,0),(0,0,0)]
    nearly_closed = [(0,0,0),(1,2,0),(1e-5,0,0)]
    range_curves = construct_curves_3d() + [geom_bezier(closed), geom_bspline(2,False,nearly_closed,[(0,3),(1,3)])]
    for curve, bounds, tolerance in product(range_curves, [(0,1),(1,0),(-2,3),(0.5,0.5),(0.8,0.2),(1,0.2),(0.2,0),(7,-7)],[-1,0,1e-7,1e-3,10]):
        requests.append('sh_analysis_validate '+numbers([*curve,*bounds,tolerance]))
    for curve, bounds, count, exact, initial in product(construct_curves_2d(),[(0,1),(-1,2),(1,0),(0.5,0.5)],[-1,0,1,2,5,20],range(2),range(2)):
        requests.append('sh_analysis_bounds '+numbers([initial,*curve,*bounds,count,exact]))
    normals = [(0,0,0),(0,0,1),(0,0,3),(1,0,0),(1,2,3),(1e-9,0,0)]
    points_sets = [[(0,0,0),(1,0,0)],[(0,0,0),(0,0,1)],[(1,2,3)]*3,
        [(0,0,0),(1,0,0),(1,1,0),(0,1,0)],[(0,0,0),(1,0,0),(1,1,1),(0,1,0)],
        [(0,0,0),(1,0,0),(0,0,0),(2,0,0)],[(0,0,0),(1,0,0),(1,1,1e-8),(0,1,0)]]
    for points, normal, tolerance in product(points_sets,normals,[-1,0,1e-7,1]):
        requests.append('sh_analysis_planar '+numbers([0,len(points),*[v for p in points for v in p],*normal,tolerance]))
    for curve, normal, tolerance in product(construct_curves_3d(),normals,[-1,0,1e-7,1]):
        requests.append('sh_analysis_planar '+numbers([1,*curve,*normal,tolerance]))
    for first, second, normal in product(construct_curves_3d(),construct_curves_3d(),normals):
        requests.append('sh_analysis_planar '+numbers([2,2,*first,*second,*normal,1e-7]))

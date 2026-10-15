//! The static helpers of TopOpeBRepBuild_FuseFace.cxx: the grouping of shapes by their
//! shared edges, of edges by their vertices, the edges of the groups and the support
//! test of two edges.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{Array1, DataMap, List, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// Shapes by shape.
pub(super) type ShapeLists = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The groups of `a` and `b` merged: each shape of one group gets the other group.
fn merge_groups(map: &mut ShapeLists, shap1: &Shape, shap2: &Shape) -> Result<(), Failure> {
    let lmap1 = map.find(shap1)?.clone();
    let lmap2 = map.find(shap2)?.clone();
    for s in lmap1.iter() {
        let mut l3 = lmap2.clone();
        map.change_find(s)?.append_list(&mut l3);
    }
    for s in lmap2.iter() {
        let mut l3 = lmap1.clone();
        map.change_find(s)?.append_list(&mut l3);
    }
    Ok(())
}

/// `GroupShape(mylist, Keep_Edge, mymapShLSh)` (static): the shapes grouped by their
/// shared (oriented, or all with `keep_edge`) edges: one group per shape (the shapes
/// by edge with `keep_edge`).
pub(super) fn group_shape(
    mylist: &List<Shape>,
    keep_edge: bool,
    mymap: &mut ShapeLists,
) -> Result<(), Failure> {
    let (mut map_edg, mut map_sh) = (ShapeLists::default(), ShapeLists::default());
    for shap1 in mylist.iter() {
        let mut l = List::default();
        l.append(shap1.clone());
        map_sh.bind(shap1.clone(), l);
        for edg1 in Explorer::new(shap1, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge_ok = !matches!(
                edg1.orientation(),
                Orientation::Internal | Orientation::External
            );
            if !(edge_ok || keep_edge) {
                continue;
            }
            if !map_edg.is_bound(&edg1) {
                let mut l = List::default();
                l.append(shap1.clone());
                map_edg.bind(edg1, l);
                continue;
            }
            let lmap_edg = map_edg.change_find(&edg1)?;
            lmap_edg.append(shap1.clone());
            if keep_edge {
                continue;
            }
            let shap2 = lmap_edg.first().cloned().unwrap_or_default();
            if !map_sh.find(&shap2)?.iter().any(|s| s.is_same(shap1)) {
                merge_groups(&mut map_sh, shap1, &shap2)?;
            }
        }
    }
    mymap.clear(true);
    if keep_edge {
        *mymap = map_edg;
        return Ok(());
    }
    let mut m: Map<Shape, ShapeMapHasher> = Map::default();
    for (shap1, lmap) in map_sh.items() {
        if m.add(shap1.clone()) {
            mymap.bind(shap1.clone(), lmap.clone());
            for s in lmap.iter() {
                m.add(s.clone());
            }
        }
    }
    Ok(())
}

/// `GroupEdge(mymapVerLEdg, mymapEdgLEdg)` (static): the edges of the vertices of two
/// edges grouped into chains.
pub(super) fn group_edge(
    mymap_ver: &ShapeLists,
    mymap_edg: &mut ShapeLists,
) -> Result<(), Failure> {
    let mut map = ShapeLists::default();
    for (_, lmap_edg) in mymap_ver.items() {
        let mut it = lmap_edg.iter();
        let edg1 = it.next().cloned().unwrap_or_default();
        let edg2 = it.next().cloned().unwrap_or_default();
        let (add1, add2) = (!map.is_bound(&edg1), !map.is_bound(&edg2));
        if !(add1 || add2) {
            continue;
        }
        if add1 && add2 {
            map.bind(edg1, lmap_edg.clone());
            map.bind(edg2, lmap_edg.clone());
            continue;
        }
        let (edgold, edgnew) = if add1 { (edg2, edg1) } else { (edg1, edg2) };
        let mut l11 = List::default();
        l11.append(edgnew.clone());
        map.bind(edgnew.clone(), l11);
        let lmap1 = map.find(&edgold)?.clone();
        for edg22 in lmap1.iter() {
            map.change_find(&edgnew)?.append(edg22.clone());
            map.change_find(edg22)?.append(edgnew.clone());
        }
    }
    let mut m: Map<Shape, ShapeMapHasher> = Map::default();
    for (edg1, lmap) in map.items() {
        if m.add(edg1.clone()) {
            mymap_edg.bind(edg1.clone(), lmap.clone());
            for e in lmap.iter() {
                m.add(e.clone());
            }
        }
    }
    Ok(())
}

/// The end vertices (once in the group) of a group of edges, None unless two.
fn group_ends(lmap_edg: &List<Shape>) -> Result<Option<(Shape, Shape)>, Failure> {
    let mut map_ver_int: DataMap<Shape, i32, ShapeMapHasher> = DataMap::default();
    for edg2 in lmap_edg.iter() {
        for ver in Explorer::new(edg2, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            let extrem = if map_ver_int.is_bound(&ver) { 0 } else { 1 };
            map_ver_int.bind(ver, extrem);
        }
    }
    let lv: Vec<Shape> = map_ver_int
        .items()
        .filter(|(_, v)| **v == 1)
        .map(|(k, _)| k.clone())
        .collect();
    Ok((lv.len() == 2).then(|| (lv[0].clone(), lv[1].clone())))
}

/// `MakeEdge(mymapEdgLEdg)` (static): per group, an edge on the curve of its key edge
/// between its two end vertices (stopping at a group without two ends).
pub(super) fn make_edge(mymap: &mut ShapeLists) -> Result<(), Failure> {
    let mut map = ShapeLists::default();
    for (edg1, lmap_edg) in mymap.items() {
        let Some((v1, v2)) = group_ends(lmap_edg)? else {
            return Ok(());
        };
        let (verf, verl) = (
            topods::vertex(&v1.oriented(Orientation::Forward))?,
            topods::vertex(&v2.oriented(Orientation::Forward))?,
        );
        let found = BRepTool::curve_with_location(&topods::edge(edg1)?)?;
        let me = BRepLibMakeEdge::from_curve_vertices(&found.curve, &verf, &verl)?;
        map.bind(me.edge()?.into_shape(), lmap_edg.clone());
    }
    *mymap = map;
    Ok(())
}

/// The 3d curve of the edge in its location, the basis of a trimmed curve.
fn support_curve(e: &Edge) -> Result<Handle<dyn GeomCurve>, Failure> {
    let found = BRepTool::curve_with_location(e)?;
    let mut c = found.curve;
    if !found.location.is_identity() {
        c = c
            .get()
            .ok_or_else(|| Failure::new("SameSupport: null curve"))?
            .transformed(found.location.transformation())?
            .cast();
    }
    if c.is_instance_name("Geom_TrimmedCurve") {
        let t: Handle<GeomTrimmedCurve> = c.clone().cast();
        c = t
            .get()
            .ok_or_else(|| Failure::new("SameSupport: null trimmed curve"))?
            .basis_curve();
    }
    Ok(c)
}

/// The same poles within a tolerance.
fn same_poles(p1: &Array1<'_, Point3>, p2: &Array1<'_, Point3>, tol: f64) -> bool {
    p1.iter().zip(p2.iter()).all(|(a, b)| a.distance(b) <= tol)
}

/// The same weights within a tolerance (none for a non-rational curve).
fn same_weights(w1: Option<Array1<'_, f64>>, w2: Option<Array1<'_, f64>>, tol: f64) -> bool {
    match (w1, w2) {
        (Some(w1), Some(w2)) => w1.iter().zip(w2.iter()).all(|(a, b)| (a - b).abs() <= tol),
        _ => true,
    }
}

/// `SameSupport(E1, E2)` (static): the two edges lie on the same line, circle,
/// ellipse, B-spline or Bezier curve.
pub(super) fn same_support(e1: &Edge, e2: &Edge) -> Result<bool, Failure> {
    if e1.is_null() || e2.is_null() {
        return Ok(false);
    }
    let (c1, c2) = (support_curve(e1)?, support_curve(e2)?);
    let n1 = c1
        .dynamic_type()
        .map(|t| t.name().to_string())
        .unwrap_or_default();
    let n2 = c2
        .dynamic_type()
        .map(|t| t.name().to_string())
        .unwrap_or_default();
    if n1 != n2 {
        return Ok(false);
    }
    let (tollin, tolang) = (precision::confusion(), precision::angular());
    match n1.as_str() {
        "Geom_Line" => {
            let (l1, l2): (Handle<GeomLine>, Handle<GeomLine>) = (c1.cast(), c2.cast());
            let (li1, li2) = (l1.get().map(|l| l.lin()), l2.get().map(|l| l.lin()));
            let (Some(li1), Some(li2)) = (li1, li2) else {
                return Ok(false);
            };
            Ok(li1.angle(li2).abs() <= tolang
                && li1.location().square_distance(&li2.location()) <= tollin * tollin)
        }
        "Geom_Circle" => {
            let (a, b): (Handle<GeomCircle>, Handle<GeomCircle>) = (c1.cast(), c2.cast());
            let (Some(a), Some(b)) = (
                a.get().map(|c| c.circ()).transpose()?,
                b.get().map(|c| c.circ()).transpose()?,
            ) else {
                return Ok(false);
            };
            Ok((a.radius() - b.radius()).abs() <= tollin
                && a.location().square_distance(&b.location()) <= tollin * tollin)
        }
        "Geom_Ellipse" => {
            let (a, b): (Handle<GeomEllipse>, Handle<GeomEllipse>) = (c1.cast(), c2.cast());
            let (Some(a), Some(b)) = (
                a.get().map(|c| c.elips()).transpose()?,
                b.get().map(|c| c.elips()).transpose()?,
            ) else {
                return Ok(false);
            };
            Ok((a.major_radius() - b.major_radius()).abs() <= tollin
                && (a.minor_radius() - b.minor_radius()).abs() <= tollin
                && a.location().square_distance(&b.location()) <= tollin * tollin)
        }
        "Geom_BSplineCurve" => {
            let (a, b): (Handle<GeomBSplineCurve>, Handle<GeomBSplineCurve>) =
                (c1.cast(), c2.cast());
            let (Some(a), Some(b)) = (a.get(), b.get()) else {
                return Ok(false);
            };
            if a.nb_poles() != b.nb_poles() || a.nb_knots() != b.nb_knots() {
                return Ok(false);
            }
            if !same_poles(&a.poles(), &b.poles(), BRepTool::edge_tolerance(e1)?) {
                return Ok(false);
            }
            let (k1, k2, m1, m2) = (a.knots(), b.knots(), a.multiplicities(), b.multiplicities());
            for (k, kk) in (k1.lower()..=k1.upper()).zip(k2.lower()..=k2.upper()) {
                if (k1[k] - k2[kk]) > tollin || f64::from((m1[k] - m2[kk]).abs()) > tollin {
                    return Ok(false);
                }
            }
            if a.is_rational() != b.is_rational() {
                return Ok(false);
            }
            Ok(!a.is_rational() || same_weights(a.weights(), b.weights(), tollin))
        }
        "Geom_BezierCurve" => {
            let (a, b): (Handle<GeomBezierCurve>, Handle<GeomBezierCurve>) = (c1.cast(), c2.cast());
            let (Some(a), Some(b)) = (a.get(), b.get()) else {
                return Ok(false);
            };
            if a.nb_poles() != b.nb_poles() || !same_poles(&a.poles(), &b.poles(), tollin) {
                return Ok(false);
            }
            if a.is_rational() != b.is_rational() {
                return Ok(false);
            }
            Ok(!a.is_rational() || same_weights(a.weights(), b.weights(), tollin))
        }
        _ => Ok(false),
    }
}

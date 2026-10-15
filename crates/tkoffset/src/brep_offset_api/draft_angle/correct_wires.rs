// Rust translation of BRepOffsetAPI_DraftAngle::CorrectWires of
// ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_DraftAngle.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inner wires of a drafted face crossing its seam edges, split at the seams and
//! merged into the wires of the seams.
use super::BRepOffsetAPIDraftAngle;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepGCurve, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d};
use rustycad_tkbrep::brep_tools::{self, Substitution};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkmath::gp::{Point3, Vector2};
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;
use std::f64::consts::PI;

/// `Seq(i)` of a 1-based sequence: Standard_OutOfRange outside 1..=Length().
fn at<T>(seq: &[T], i: usize) -> Result<&T, Failure> {
    i.checked_sub(1)
        .and_then(|k| seq.get(k))
        .ok_or_else(|| Failure::with_message("Standard_OutOfRange", "NCollection_Sequence::Value"))
}

/// The value bound to a key: Standard_NoSuchObject when it is not bound.
fn found<'a, V>(map: &'a DataMap<Shape, V, ShapeMapHasher>, key: &Shape) -> Result<&'a V, Failure> {
    map.seek(key)
        .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find"))
}

/// The value bound to a key, to write: Standard_NoSuchObject when it is not bound.
fn found_mut<'a, V>(map: &'a mut DataMap<Shape, V, ShapeMapHasher>, key: &Shape) -> Result<&'a mut V, Failure> {
    map.change_seek(key)
        .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::ChangeFind"))
}

/// The keys of a data map in its iteration order.
fn keys<V>(map: &DataMap<Shape, V, ShapeMapHasher>) -> Vec<Shape> {
    map.items().map(|(k, _)| k.clone()).collect()
}

/// The edge of an empty copy with the range, FORWARD.
fn empty_copy_ranged(builder: &BRepBuilder, an_edge: &Edge, first: f64, last: f64) -> Result<Edge, Failure> {
    let new_e = topods::edge(&an_edge.empty_copied()?)?;
    builder.range(&new_e, first, last, false)?;
    Ok(new_e)
}

impl BRepOffsetAPIDraftAngle {
    /// `CorrectWires()`: the inner wires of the faces intersecting their seam edges are
    /// split at the intersections and joined to the seam wires.
    pub fn correct_wires(&mut self) -> Result<(), Failure> {
        let tol_inter = 1.0e-7;

        let mut eseq: Vec<Shape> = Vec::new();
        let mut wseq: Vec<Shape> = Vec::new();
        let mut fseq: Vec<Shape> = Vec::new();

        let my_shape = self.modify.my_shape.clone();
        for cur_face in Explorer::new(&my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            for cur_wire in ShapeIterator::new(&cur_face, true, true) {
                let mut emap = MapOfShape::default();
                for e in ShapeIterator::new(&cur_wire, true, true) {
                    emap.add(e);
                }
                for cur_edge in emap.iter() {
                    if brep_tools::is_really_closed(&topods::edge(cur_edge)?, &topods::face(&cur_face)?)? {
                        eseq.push(cur_edge.clone());
                        wseq.push(cur_wire.clone());
                        fseq.push(cur_face.clone());
                    }
                }
            }
        }

        let mut emap: DataMap<Shape, Vec<f64>, ShapeMapHasher> = DataMap::default();

        let mut non_seam: Vec<Shape> = Vec::new();
        let mut non_seam_wires: Vec<Shape> = Vec::new();
        let mut pars_non_seam: Vec<Vec<f64>> = Vec::new();
        let mut seam: Vec<Vec<Shape>> = Vec::new();
        let mut pars_seam: Vec<Vec<f64>> = Vec::new();

        let mut wf_map: DataMap<Shape, Shape, ShapeMapHasher> = DataMap::default();
        let mut ww_map: DataMap<Shape, Vec<Shape>, ShapeMapHasher> = DataMap::default();
        for i in 0..eseq.len() {
            let cur_edge = eseq[i].clone();
            let cur_wire = wseq[i].clone();
            let cur_face = fseq[i].clone();
            let a_face = topods::face(&cur_face)?;

            // Prepare 2D adaptors for intersection.
            // The seam edge has two 2D curve, thus we have to create 2 adaptors
            let a_bac2d1 = BRepAdaptorCurve2d::with_edge_on_face(&topods::edge(&cur_edge)?, &a_face)?;
            let a_bac2d1r = BRepAdaptorCurve2d::with_edge_on_face(&topods::edge(&cur_edge.reversed())?, &a_face)?;
            // Get surface of the face to get 3D intersection point
            let (a_surf, _) = BRepTool::surface_with_location(&a_face)?;
            // Get the tolerance of the current edge to compare intersection points
            let a_tol_cur_e = BRepTool::edge_tolerance(&topods::edge(&cur_edge)?)?;

            for a_wire in ShapeIterator::new(&cur_face, true, true) {
                if a_wire.is_same(&cur_wire) {
                    continue;
                }
                let mut pts: Vec<Point3> = Vec::new();
                let mut wadd = false;
                for an_edge in ShapeIterator::new(&a_wire, true, true) {
                    let an_edge = topods::edge(&an_edge)?;

                    // Prepare 2D adaptor for intersection
                    let a_bac2d2 = BRepAdaptorCurve2d::with_edge_on_face(&an_edge, &a_face)?;
                    // Perform intersection
                    let mut a_g_inter =
                        GInter::between(a_bac2d1.geom2d_adaptor(), a_bac2d2.geom2d_adaptor(), tol_inter, tol_inter)?;
                    if !a_g_inter.is_done() || a_g_inter.is_empty()? {
                        // If first intersection is empty try intersection with reversed edge
                        a_g_inter = GInter::between(
                            a_bac2d1r.geom2d_adaptor(),
                            a_bac2d2.geom2d_adaptor(),
                            tol_inter,
                            tol_inter,
                        )?;
                        if !a_g_inter.is_done() || a_g_inter.is_empty()? {
                            continue;
                        }
                    }

                    wadd = true;
                    if !wf_map.is_bound(&a_wire) {
                        wf_map.bind(a_wire.clone(), cur_face.clone());
                    }
                    let mut ind = 0;
                    for (j, ns) in non_seam.iter().enumerate() {
                        if an_edge.is_same(ns) {
                            ind = j + 1;
                            break;
                        }
                    }
                    if ind == 0 {
                        non_seam.push(an_edge.clone().into_shape());
                        non_seam_wires.push(a_wire.clone());
                        ind = non_seam.len();
                        pars_non_seam.push(Vec::new());
                        seam.push(Vec::new());
                        pars_seam.push(Vec::new());
                    }
                    if !emap.is_bound(&cur_edge) {
                        emap.bind(cur_edge.clone(), Vec::new());
                    }

                    // Get the tolerance of edge to compare intersection points
                    let a_tol_e = BRepTool::edge_tolerance(&an_edge)?;
                    // Tolerance to compare the intersection points is the maximal
                    // tolerance of intersecting edges
                    let a_tol_cmp = a_tol_cur_e.max(a_tol_e);

                    let a_nb_int_pnt = a_g_inter.nb_points()?;
                    for k in 1..=a_nb_int_pnt {
                        let a_p2d_int = a_g_inter.point(k)?;
                        let a_p2d = a_p2d_int.value();
                        let a_p3d = live_surface(&a_surf)?.value(a_p2d.x(), a_p2d.y())?;

                        // Check if the intersection point is new
                        let is_new = !pts.iter().any(|p| a_p3d.is_equal(p, a_tol_cmp));
                        if is_new {
                            pts.push(a_p3d);
                            found_mut(&mut emap, &cur_edge)?.push(a_p2d_int.param_on_first());
                            pars_non_seam[ind - 1].push(a_p2d_int.param_on_second());
                            seam[ind - 1].push(cur_edge.clone());
                            pars_seam[ind - 1].push(a_p2d_int.param_on_first());
                        }
                    }
                }
                if wadd {
                    if !ww_map.is_bound(&cur_wire) {
                        ww_map.bind(cur_wire.clone(), Vec::new());
                    }
                    found_mut(&mut ww_map, &cur_wire)?.push(a_wire.clone());
                }
            }
        }

        // Sorting
        for i in 0..non_seam.len() {
            let n = pars_non_seam[i].len();
            for j in 0..n.saturating_sub(1) {
                for k in j + 1..n {
                    if pars_non_seam[i][k] < pars_non_seam[i][j] {
                        pars_non_seam[i].swap(j, k);
                        seam[i].swap(j, k);
                        pars_seam[i].swap(j, k);
                    }
                }
            }
        }
        for key in keys(&emap) {
            let seq = found_mut(&mut emap, &key)?;
            let n = seq.len();
            for i in 0..n.saturating_sub(1) {
                for j in i + 1..n {
                    if seq[j] < seq[i] {
                        seq.swap(i, j);
                    }
                }
            }
        }
        let mut ep_map: DataMap<Shape, Vec<f64>, ShapeMapHasher> = DataMap::default();
        let mut ev_map: DataMap<Shape, Vec<Shape>, ShapeMapHasher> = DataMap::default(); // Seam
        let mut ew_map: DataMap<Shape, Vec<Shape>, ShapeMapHasher> = DataMap::default(); // Seam and wires intersecting it
        for key in keys(&emap) {
            ep_map.bind(key.clone(), Vec::new());
            ev_map.bind(key.clone(), Vec::new());
            ew_map.bind(key, Vec::new());
        }

        // Reconstruction of non-seam edges
        let mut a_sub = Substitution::new();
        let bb = BRepBuilder::default();
        for i in 1..=non_seam.len() {
            let an_edge = topods::edge(at(&non_seam, i)?)?;
            let mut new_edges: Vec<Shape> = Vec::new();
            let (vfirst, vlast) = topexp::vertices(&an_edge, false);
            let (first_par, last_par) = BRepTool::range(&an_edge)?;
            let mut firstind = 1;
            let pars = at(&pars_non_seam, i)?.clone();
            let seams = at(&seam, i)?.clone();
            let pseams = at(&pars_seam, i)?.clone();
            let mut par = *at(&pars, 1)?;
            let mut seam_edge = topods::edge(at(&seams, 1)?)?;
            // Find the face (the native search result is not used further)
            if (par - first_par).abs() <= precision::confusion() {
                bb.update_vertex_on_edge(&vfirst, *at(&pseams, 1)?, &seam_edge, BRepTool::vertex_tolerance(&vfirst)?)?;
                found_mut(&mut ep_map, &seam_edge)?.push(*at(&pseams, 1)?);
                found_mut(&mut ev_map, &seam_edge)?.push(vfirst.clone().into_shape());
                found_mut(&mut ew_map, &seam_edge)?.push(at(&non_seam_wires, i)?.clone());
                firstind = 2;
            }
            let mut prevpar = first_par;
            let mut prev_v = vfirst.clone();
            let mut j = firstind;
            while j <= pars.len() {
                let mut new_e = topods::edge(&an_edge.empty_copied()?)?;
                let new_v: Vertex;
                par = *at(&pars, j)?;
                bb.range(&new_e, prevpar, par, false)?;
                seam_edge = topods::edge(at(&seams, j)?)?;
                if j == pars.len() && (par - last_par).abs() <= precision::confusion() {
                    new_v = vlast.clone();
                    if firstind == 2 && j == 2 {
                        bb.update_vertex_on_edge(&vlast, *at(&pseams, j)?, &seam_edge, BRepTool::vertex_tolerance(&vlast)?)?;
                        found_mut(&mut ep_map, &seam_edge)?.push(*at(&pseams, j)?);
                        found_mut(&mut ev_map, &seam_edge)?.push(vlast.clone().into_shape());
                        found_mut(&mut ew_map, &seam_edge)?.push(at(&non_seam_wires, i)?.clone());
                        break;
                    }
                } else {
                    let bcur = BRepAdaptorCurve::with_edge(&new_e)?;
                    let point = bcur.value(par)?;
                    new_v = BRepLibMakeVertex::new(point)?.vertex()?;
                    bb.update_vertex_on_edge(&new_v, par, &new_e, 10. * precision::confusion())?;
                }
                bb.update_vertex_on_edge(&new_v, *at(&pseams, j)?, &seam_edge, 10. * precision::confusion())?;
                new_e.set_orientation(Orientation::Forward);
                bb.add(&mut new_e, &prev_v.oriented(Orientation::Forward))?;
                bb.add(&mut new_e, &new_v.oriented(Orientation::Reversed))?;

                new_edges.push(new_e.into_shape());
                found_mut(&mut ep_map, &seam_edge)?.push(*at(&pseams, j)?);
                found_mut(&mut ev_map, &seam_edge)?.push(new_v.clone().into_shape());
                found_mut(&mut ew_map, &seam_edge)?.push(at(&non_seam_wires, i)?.clone());

                prevpar = par;
                prev_v = new_v;
                j += 1;
            }
            // The last edge
            let mut new_e = topods::edge(&an_edge.empty_copied()?)?;
            par = last_par;
            if (prevpar - par).abs() > precision::confusion() {
                bb.range(&new_e, prevpar, par, false)?;
                new_e.set_orientation(Orientation::Forward);
                bb.add(&mut new_e, &prev_v.oriented(Orientation::Forward))?;
                bb.add(&mut new_e, &vlast.oriented(Orientation::Reversed))?;
                new_edges.push(new_e.into_shape());
            }

            // Substitute anEdge by NewEdges
            a_sub.substitute(&an_edge, &new_edges)?;
        }

        // Sorting of EPmap and EVmap and removing repeating points from them
        for key in keys(&ep_map) {
            let mut seq = found(&ep_map, &key)?.clone();
            let mut seq_shape = found(&ev_map, &key)?.clone();
            let mut seq_shape2 = found(&ew_map, &key)?.clone();
            let n = seq.len();
            for i in 0..n.saturating_sub(1) {
                for j in i + 1..n {
                    if seq[j] < seq[i] {
                        seq.swap(i, j);
                        seq_shape.swap(i, j);
                        seq_shape2.swap(i, j);
                    }
                }
            }
            *found_mut(&mut ep_map, &key)? = seq;
            *found_mut(&mut ev_map, &key)? = seq_shape;
            *found_mut(&mut ew_map, &key)? = seq_shape2;
        }
        for key in keys(&ep_map) {
            let mut seq = found(&ep_map, &key)?.clone();
            let mut seq_shape = found(&ev_map, &key)?.clone();
            let mut seq_shape2 = found(&ew_map, &key)?.clone();
            let mut remove = true;
            while remove {
                remove = false;
                let mut i = 1;
                while i < seq.len() {
                    if (seq[i - 1] - seq[i]).abs() <= precision::confusion() {
                        seq.remove(i);
                        seq_shape.remove(i);
                        seq_shape2.remove(i);
                        remove = true;
                    }
                    i += 1;
                }
            }
            *found_mut(&mut ep_map, &key)? = seq;
            *found_mut(&mut ev_map, &key)? = seq_shape;
            *found_mut(&mut ew_map, &key)? = seq_shape2;
        }

        // Reconstruction of seam edges
        let mut ve_map: DataMap<Shape, Shape, ShapeMapHasher> = DataMap::default();
        for key in keys(&emap) {
            let an_edge = topods::edge(&key)?;
            let mut onepoint = false;
            let mut new_edges: Vec<Shape> = Vec::new();
            let seq = found(&emap, &key)?.clone();
            let seq2 = found(&ep_map, &key)?.clone();
            let seq_ver = found(&ev_map, &key)?.clone();
            let (vfirst, vlast) = topexp::vertices(&an_edge, false);
            let (first_par, last_par) = BRepTool::range(&an_edge)?;
            let mut fpar = first_par;
            let mut lpar = *at(&seq, 1)?;
            let mut firstind = 1;
            if (fpar - lpar).abs() <= precision::confusion() {
                firstind = 2;
                fpar = *at(&seq, 1)?;
                lpar = *at(&seq, 2)?;
            } else if seq.len() % 2 != 0 {
                ve_map.bind(vfirst.clone().into_shape(), an_edge.clone().into_shape());
                firstind = 2;
                fpar = *at(&seq, 1)?;
                if seq.len() > 2 {
                    lpar = *at(&seq, 2)?;
                } else {
                    onepoint = true;
                }
            }
            if !onepoint {
                let mut new_e = empty_copy_ranged(&bb, &an_edge, fpar, lpar)?;
                new_e.set_orientation(Orientation::Forward);
                if firstind == 1 {
                    bb.add(&mut new_e, &vfirst.oriented(Orientation::Forward))?;
                    bb.add(&mut new_e, &topods::vertex(&at(&seq_ver, 1)?.oriented(Orientation::Reversed))?.into_shape())?;
                } else {
                    bb.add(&mut new_e, &topods::vertex(&at(&seq_ver, 1)?.oriented(Orientation::Forward))?.into_shape())?;
                    bb.add(&mut new_e, &topods::vertex(&at(&seq_ver, 2)?.oriented(Orientation::Reversed))?.into_shape())?;
                }
                new_edges.push(new_e.into_shape());

                firstind += 1;
                let mut i = firstind;
                while i < seq.len() {
                    fpar = *at(&seq, i)?;
                    lpar = *at(&seq, i + 1)?;
                    let mut new_e = empty_copy_ranged(&bb, &an_edge, fpar, lpar)?;
                    // Find vertices
                    let mut j = 1;
                    while j <= seq2.len() {
                        if (fpar - *at(&seq2, j)?).abs() <= precision::confusion() {
                            break;
                        }
                        j += 1;
                    }
                    new_e.set_orientation(Orientation::Forward);
                    bb.add(&mut new_e, &topods::vertex(&at(&seq_ver, j)?.oriented(Orientation::Forward))?.into_shape())?;
                    bb.add(&mut new_e, &topods::vertex(&at(&seq_ver, j + 1)?.oriented(Orientation::Reversed))?.into_shape())?;
                    new_edges.push(new_e.into_shape());
                    i += 2;
                }
            }

            fpar = *at(&seq, seq.len())?;
            lpar = last_par;
            if (fpar - lpar).abs() <= precision::confusion() {
                continue;
            }
            let mut new_e = empty_copy_ranged(&bb, &an_edge, fpar, lpar)?;
            new_e.set_orientation(Orientation::Forward);
            bb.add(&mut new_e, &topods::vertex(&at(&seq_ver, seq_ver.len())?.oriented(Orientation::Forward))?.into_shape())?;
            bb.add(&mut new_e, &vlast.oriented(Orientation::Reversed))?;
            new_edges.push(new_e.into_shape());

            // Substitute anEdge by NewEdges
            a_sub.substitute(&an_edge, &new_edges)?;
        }

        // Removing edges connected with missing extremities of seam edges
        for (v, e) in ve_map.items() {
            let mut w = Shape::default();
            for (i, s) in eseq.iter().enumerate() {
                if e.is_same(s) {
                    w = wseq[i].clone();
                    break;
                }
            }
            let mut etoremove = Shape::default();
            for cur_e in ShapeIterator::new(&w, true, true) {
                let cur_e = topods::edge(&cur_e)?;
                if cur_e.is_same(e) {
                    continue;
                }
                let (vfirst, vlast) = topexp::vertices(&cur_e, false);
                if vfirst.is_same(v) || vlast.is_same(v) {
                    etoremove = cur_e.into_shape();
                    break;
                }
            }
            if !etoremove.is_null() {
                w.tshape().set_free(true);
                bb.remove(&mut w, &etoremove)?;
            }
        }

        a_sub.build(&my_shape)?;
        if a_sub.is_copied(&my_shape) {
            let list_sh = a_sub.copy(&my_shape)?;
            if let Some(first) = list_sh.first() {
                self.modify.my_shape = first.clone();
            }
        }

        // Reconstruction of wires
        for (key, wires) in ww_map.items() {
            let the_copy = a_sub.copy(key)?;
            let mut cur_wire = the_copy
                .first()
                .cloned()
                .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_List::First"))?;
            cur_wire.tshape().set_free(true);
            for a_wire in wires {
                let mut cur_face = found(&wf_map, a_wire)?.clone();
                let the_copy = a_sub.copy(a_wire)?;
                let a_wire = the_copy
                    .first()
                    .cloned()
                    .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_List::First"))?;
                // Adjusting period
                let face = topods::face(&cur_face)?;
                let (the_surf, l) = BRepTool::surface_with_location(&face)?;
                for an_edge in ShapeIterator::new(&a_wire, true, true) {
                    let an_edge = topods::edge(&an_edge)?;
                    let (pfirst, plast) = BRepTool::uv_points_on_face(&an_edge, &face)?;
                    let bc2d = BRepAdaptorCurve2d::with_edge_on_face(&an_edge, &face)?;
                    let pmid = bc2d.value((bc2d.first_parameter()? + bc2d.last_parameter()?) / 2.)?;
                    let mut offset = Vector2::default();
                    let mut translate = false;
                    if pfirst.x() - 2. * PI > precision::confusion()
                        || plast.x() - 2. * PI > precision::confusion()
                        || pmid.x() - 2. * PI > precision::confusion()
                    {
                        offset = Vector2::new(-2. * PI, 0.);
                        translate = true;
                    }
                    if pfirst.x() < -precision::confusion()
                        || plast.x() < -precision::confusion()
                        || pmid.x() < -precision::confusion()
                    {
                        offset = Vector2::new(2. * PI, 0.);
                        translate = true;
                    }
                    if translate {
                        let te = edge_tshape(&an_edge, "BRepOffsetAPI_DraftAngle::CorrectWires")?;
                        te.change_curves(|curves| -> Result<(), Failure> {
                            for idx in 0..curves.len() {
                                let gc = curves[idx].clone().cast::<dyn BRepGCurve>();
                                let Some(g) = gc.get() else { continue };
                                if g.is_curve_on_surface_at(&the_surf, &l) {
                                    let pc: Handle<dyn Geom2dCurve> = g.pcurve()?;
                                    let pc: Handle<dyn Geom2dCurve> = pc
                                        .get()
                                        .ok_or_else(|| Failure::with_message("Standard_NullObject", "BRep_GCurve::PCurve"))?
                                        .translated(offset)?
                                        .cast();
                                    g.set_pcurve(&pc)?;
                                    let item = curves.remove(idx);
                                    curves.push(item);
                                    break;
                                }
                            }
                            Ok(())
                        })?;
                    }
                }
                ///////////////////
                for an_edge in ShapeIterator::new(&a_wire, false, true) {
                    bb.add(&mut cur_wire, &an_edge)?;
                }
                if a_sub.is_copied(&cur_face) {
                    let the_copy = a_sub.copy(&cur_face)?;
                    if let Some(first) = the_copy.first() {
                        cur_face = first.clone();
                    }
                }
                cur_face.tshape().set_free(true);
                bb.remove(&mut cur_face, &a_wire)?;
            }
        }
        Ok(())
    }
}

/// The surface of a live handle.
fn live_surface(
    s: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
) -> Result<&dyn rustycad_tkg3d::geom::GeomSurface, Failure> {
    s.get().ok_or_else(|| Failure::with_message("Standard_NullObject", "BRepOffsetAPI_DraftAngle: null surface"))
}

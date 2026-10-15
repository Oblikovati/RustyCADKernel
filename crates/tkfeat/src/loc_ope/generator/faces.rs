// Rust translation of the face fusion part of LocOpe_Generator::Perform of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Generator.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Fusion des faces, ou reconstruction: each face fused with generated faces (or to
//! rebuild) gets the edges of its untouched wires, of the untouched wires of its fused
//! faces, its rebuilt edges and the parts of the fused edges, and BRepAlgo_Loop makes
//! the new faces of them. On a non-planar face the p-curves are projected.
use super::Run;
use crate::errors::null_object;
use crate::loc_ope::find_edges::basis_curve;
use rustycad_tkbool::brep_algo::BRepAlgoLoop;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::gp::{Vector2, Vector3};
use std::f64::consts::PI;

/// The state of the rebuild of one face; the curve, its range and the tolerance carry
/// over from edge to edge as the native locals do.
struct FaceWork {
    fac: Face,
    fused: Vec<Shape>,
    orface: Orientation,
    newface: Face,
    surface: Handle<dyn GeomSurface>,
    on_plane: bool,
    listofedg: List<Shape>,
    edg_added: MapOfShape,
    mod_face: bool,
    c: Handle<dyn GeomCurve>,
    f: f64,
    l: f64,
    tol: f64,
}

/// The (located) basis surface of a face, a rectangular trimmed surface standing for
/// its basis.
fn basis_surface(fac: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
    let s = BRepTool::surface(fac)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    Ok(trimmed.get().map_or(s.clone(), |t| t.basis_surface()))
}

impl FaceWork {
    /// The p-curve of the current curve projected on the surface (the tolerance updated).
    fn project(&mut self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let (c2d, tol) = geom_proj_lib::curve2d_range_tolerance(
            &self.c,
            (self.f, self.l),
            &self.surface,
            self.tol,
        )?;
        self.tol = tol;
        Ok(c2d)
    }

    /// The 3D curve of `edg` (its basis when trimmed) becomes the current curve.
    fn take_curve(&mut self, edg: &Edge) -> Result<(), Failure> {
        let (c, f, l) = basis_curve(edg)?;
        (self.c, self.f, self.l) = (c, f, l);
        Ok(())
    }

    /// Projects the curve of `edg` on the surface, recalibrated by periods of 2 pi in
    /// `[uminc, umaxc]` (tentative de recalage dans la facette), and stores it on the
    /// new face unless `edg` is really closed on `facbis`.
    fn put_pcurve(
        &mut self,
        edg: &Edge,
        facbis: &Face,
        (uminc, umaxc): (f64, f64),
    ) -> Result<(), Failure> {
        // on met les courbes 2d si on n`est pas sur un plan
        // on ne devrait pas avoir de pb d`edge de couture.
        self.tol = BRepTool::edge_tolerance(edg)?;
        self.take_curve(edg)?;
        let c2d = self.project()?;
        let curve = c2d.get().ok_or_else(|| {
            null_object("LocOpe_Generator::Perform", "the projected p-curve is null")
        })?;
        let tttol = precision::angular();
        let (mut pf, mut pl) = (curve.value(self.f)?, curve.value(self.l)?);
        while pf.x().min(pl.x()) >= umaxc - tttol {
            curve.translate(Vector2::new(-2. * PI, 0.))?;
            (pf, pl) = (curve.value(self.f)?, curve.value(self.l)?);
        }
        while pf.x().max(pl.x()) <= uminc + tttol {
            curve.translate(Vector2::new(2. * PI, 0.))?;
            (pf, pl) = (curve.value(self.f)?, curve.value(self.l)?);
        }
        if !brep_tools::is_really_closed(edg, facbis)? {
            BRepBuilder::default().update_edge_pcurve_on_face(
                edg,
                &c2d,
                &self.newface,
                self.tol,
            )?;
        }
        Ok(())
    }
}

/// The edges of `wire` in order.
fn wire_edges(wire: &Shape) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)?.collect())
}

/// The first vertex of `newedg` that is also a vertex of `edgbis`.
fn common_vertex(newedg: &Edge, edgbis: &Edge) -> Option<Shape> {
    ShapeIterator::new(newedg, true, true)
        .find(|v1| ShapeIterator::new(edgbis, true, true).any(|v2| v1.is_same(&v2)))
}

impl Run<'_> {
    /// `facbis` with its orientation among the generated faces, else in the shape; and
    /// whether it is generated.
    fn oriented_fused_face(&mut self, facbis: &Shape) -> Result<(Face, bool), Failure> {
        let mut face = topods::face(facbis)?;
        if let Some(x) = self.g.oriented_faces()?.iter().find(|x| x.is_same(facbis)) {
            face.set_orientation(x.orientation());
            return Ok((face, true));
        }
        // on fusionne avec une autre face du shape...
        if let Some(x) = Explorer::new(self.shape, ShapeEnum::Face, ShapeEnum::Shape)?
            .find(|x| x.is_same(facbis))
        {
            face.set_orientation(x.orientation());
        }
        Ok((face, false))
    }

    /// The work on the face `key` fused with `fused`.
    fn face_work(&self, key: &Shape, fused: Vec<Shape>) -> Result<FaceWork, Failure> {
        let fac = topods::face(key)?;
        let in_shape =
            Explorer::new(self.shape, ShapeEnum::Face, ShapeEnum::Shape)?.find(|x| x.is_same(&fac));
        let orface = in_shape.map_or(self.shape.orientation(), |x| x.orientation());
        let mut newface = topods::face(&fac.empty_copied()?)?;
        newface.set_orientation(Orientation::Forward);
        let surface = basis_surface(&fac)?;
        let on_plane = !surface.cast::<GeomPlane>().is_null();
        Ok(FaceWork {
            fac,
            fused,
            orface,
            newface,
            surface,
            on_plane,
            listofedg: List::default(),
            edg_added: MapOfShape::default(),
            mod_face: false,
            c: Handle::default(),
            f: 0.,
            l: 0.,
            tol: 0.,
        })
    }

    /// Premier passage: the untouched wires of the fused faces, their edges oriented
    /// by the face orientation.
    fn untouched_fused_wires(
        &mut self,
        w: &mut FaceWork,
        bounds: (f64, f64),
    ) -> Result<(), Failure> {
        let fused = w.fused.clone();
        for facbis in &fused {
            let (facbis, _) = self.oriented_fused_face(facbis)?;
            for wire in Explorer::new(&facbis, ShapeEnum::Wire, ShapeEnum::Shape)? {
                let edges = wire_edges(&wire)?;
                if edges.iter().any(|e| self.to_remove.contains(e)) {
                    continue;
                }
                for e in &edges {
                    let edg = topods::edge(e)?;
                    let orient = top_abs::compose(w.orface, edg.orientation());
                    w.listofedg.append(edg.oriented(orient));
                    w.edg_added.add(e.clone());
                    if !w.on_plane {
                        w.put_pcurve(&edg, &facbis, bounds)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// The p-curves of a rebuilt or fused edge `newedg` (image of `edg`) on the new face,
    /// from those of `edg` on the face; both of a seam once its second side comes.
    fn rebuilt_edge_pcurves(
        &self,
        w: &mut FaceWork,
        edg: &Edge,
        newedg: &Edge,
        orient: Orientation,
        c2d1: &mut Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        w.tol = BRepTool::edge_tolerance(newedg)?;
        let local_edge = topods::edge(&edg.oriented(orient))?;
        let local_face = topods::face(&w.fac.oriented(Orientation::Forward))?;
        let c2d = BRepTool::curve_on_face(&local_edge, &local_face)?.pcurve;
        if !brep_tools::is_really_closed(edg, &w.fac)? {
            b.update_edge_pcurve_on_face(newedg, &c2d, &w.newface, w.tol)?;
        } else if c2d1.is_null() {
            *c2d1 = c2d;
        } else if orient == Orientation::Forward {
            b.update_edge_pcurves_on_face(newedg, (&c2d, c2d1), &w.newface, w.tol)?;
        } else {
            b.update_edge_pcurves_on_face(newedg, (c2d1, &c2d), &w.newface, w.tol)?;
        }
        Ok(())
    }

    /// Whether the parts of the edges fused with `edg` are added after it.
    fn add_part(&self, w: &FaceWork, edg: &Edge, orient: Orientation) -> Result<bool, Failure> {
        let Some(dont) = self.dont_fuse.seek(edg) else {
            return Ok(false);
        };
        let modified = self.mod_shapes.is_bound(edg);
        if !brep_tools::is_really_closed(edg, &w.fac)? {
            if dont.is_same(&w.fac) {
                return Ok(modified);
            }
            return Ok(!self.to_remove.contains(edg));
        }
        if modified {
            // edg raccourci
            return Ok(orient == dont.orientation());
        }
        Ok(orient == top_abs::reverse(dont.orientation()))
    }

    /// The U of the start of `edg` on the face, from which a fused p-curve is shifted
    /// by periods.
    fn reference_u(&self, w: &FaceWork, edg: &Edge) -> Result<f64, Failure> {
        let local_face = topods::face(&w.fac.oriented(Orientation::Forward))?;
        let (pf, _) = match self.dont_fuse.seek(edg) {
            Some(dont) => {
                let oredge = dont.orientation();
                let or = if self.mod_shapes.is_bound(edg) {
                    oredge
                } else {
                    top_abs::reverse(oredge)
                };
                BRepTool::uv_points_on_face(&topods::edge(&edg.oriented(or))?, &local_face)?
            }
            None => BRepTool::uv_points_on_face(edg, &w.fac)?,
        };
        Ok(pf.x())
    }

    /// The parts of the edges fused with `edg` not added yet, oriented as `newedg` runs
    /// at their common vertex, with their p-curves on a non-planar face.
    fn add_fused_parts(
        &self,
        w: &mut FaceWork,
        edg: &Edge,
        newedg: &Edge,
        orient: Orientation,
    ) -> Result<Vec<Shape>, Failure> {
        let mut added = Vec::new();
        let parts: Vec<Shape> = self.the_ee_map.find(edg)?.iter().cloned().collect();
        for part in &parts {
            if w.edg_added.contains(part) {
                continue;
            }
            let edgbis = topods::edge(part)?;
            let (dir1, dir2) = match common_vertex(newedg, &edgbis) {
                Some(v) => {
                    let vertex = topods::vertex(&v)?;
                    let curve = w.c.get().ok_or_else(|| {
                        null_object("LocOpe_Generator::Perform", "edge without 3D curve")
                    })?;
                    let (_, dir1) = curve.d1(BRepTool::parameter(&vertex, newedg)?)?;
                    w.take_curve(&edgbis)?;
                    let curve = w.c.get().ok_or_else(|| {
                        null_object("LocOpe_Generator::Perform", "edge without 3D curve")
                    })?;
                    let (_, dir2) = curve.d1(BRepTool::parameter(&vertex, &edgbis)?)?;
                    (dir1, dir2)
                }
                None => (Vector3::new(1., 0., 0.), Vector3::new(1., 0., 0.)),
            };
            w.edg_added.add(part.clone());
            let or = if dir1.dot(dir2) < 0. {
                top_abs::reverse(orient)
            } else {
                orient
            };
            added.push(edgbis.oriented(or));
            if !w.on_plane {
                self.fused_part_pcurve(w, edg, &edgbis)?;
            }
        }
        Ok(added)
    }

    /// The projection of the current curve (that of the part, or still the one of the
    /// rebuilt edge when they share no vertex) as the p-curve of the part `edgbis`,
    /// shifted onto the U of `edg` on a U-periodic surface.
    fn fused_part_pcurve(
        &self,
        w: &mut FaceWork,
        edg: &Edge,
        edgbis: &Edge,
    ) -> Result<(), Failure> {
        // C est la courbe de edgbis, f et l s`y rapportent
        let ptc = w.project()?;
        let surface = w
            .surface
            .get()
            .ok_or_else(|| null_object("LocOpe_Generator::Perform", "face without surface"))?;
        if surface.is_u_periodic() {
            let uref = self.reference_u(w, edg)?;
            let curve = ptc.get().ok_or_else(|| {
                null_object("LocOpe_Generator::Perform", "the projected p-curve is null")
            })?;
            let new_u = curve.value(w.f)?.x();
            if (new_u - uref).abs() > epsilon(surface.u_period()?) {
                curve.translate(Vector2::new(uref - new_u, 0.))?;
            }
        }
        BRepBuilder::default().update_edge_pcurve_on_face(edgbis, &ptc, &w.newface, w.tol)
    }

    /// Reconstruction du wire: its kept edges, the images of its rebuilt or fused edges
    /// and the parts of the edges fused with them.
    fn rebuild_wire(&self, w: &mut FaceWork, wir: &Wire) -> Result<(), Failure> {
        let mut c2d1: Handle<dyn Geom2dCurve> = Handle::default();
        for e in wire_edges(wir)? {
            let edg = topods::edge(&e)?;
            let orient = edg.orientation();
            let fused = self.the_ee_map.is_bound(&e);
            if !self.to_remove.contains(&e) && !fused {
                w.listofedg.append(e);
                continue;
            }
            let modified = self.mod_shapes.seek(&e);
            if modified.is_none() && !fused {
                continue;
            }
            let newedg = match modified {
                Some(l) => topods::edge(l.first().ok_or_else(|| {
                    null_object("LocOpe_Generator::Perform", "rebuilt edge without image")
                })?)?,
                None => edg.clone(),
            };
            w.listofedg.append(newedg.oriented(orient));
            w.take_curve(&newedg)?;
            if !w.on_plane {
                // on met les courbes 2d si on n`est pas sur un plan
                self.rebuilt_edge_pcurves(w, &edg, &newedg, orient, &mut c2d1)?;
            }
            if self.add_part(w, &edg, orient)? {
                for part in self.add_fused_parts(w, &edg, &newedg, orient)? {
                    w.listofedg.append(part);
                }
            }
        }
        Ok(())
    }

    /// Recuperation des edges sur les faces a fusionner: once a generated face was met,
    /// the kept edges of the wires of the fused faces not added yet.
    fn fused_face_edges(&mut self, w: &mut FaceWork, bounds: (f64, f64)) -> Result<(), Failure> {
        let mut includeinw = false;
        let fused = w.fused.clone();
        for facbis in &fused {
            let (facbis, genface) = self.oriented_fused_face(facbis)?;
            for wire in Explorer::new(&facbis, ShapeEnum::Wire, ShapeEnum::Shape)? {
                // The native isIncludedInW of a face of the shape is computed and not
                // used: only a generated face opens the inclusion.
                if genface {
                    includeinw = true;
                }
                if !includeinw {
                    continue;
                }
                for e in wire_edges(&wire)? {
                    if self.to_remove.contains(&e) || w.edg_added.contains(&e) {
                        continue;
                    }
                    let edg = topods::edge(&e)?;
                    let orient = top_abs::compose(w.orface, edg.orientation());
                    w.listofedg.append(edg.oriented(orient));
                    w.edg_added.add(e);
                    if !w.on_plane {
                        w.put_pcurve(&edg, &facbis, bounds)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// The edges of the face: its untouched wires as they are, the others rebuilt.
    fn collect_face_edges(
        &mut self,
        w: &mut FaceWork,
        bounds: &mut (f64, f64),
    ) -> Result<(), Failure> {
        for x in Explorer::new(
            &w.fac.oriented(Orientation::Forward),
            ShapeEnum::Wire,
            ShapeEnum::Shape,
        )? {
            let wir = topods::wire(&x)?;
            let edges = wire_edges(&x)?;
            let touched = edges
                .iter()
                .any(|e| self.to_remove.contains(e) || self.mod_shapes.is_bound(e));
            if !touched {
                // wire non modifie
                for e in edges {
                    w.listofedg.append(e);
                }
                continue;
            }
            if !w.mod_face {
                // Petit truc crad pour les p-curves sur les cylindres...
                if !w.on_plane {
                    let (umin, umax, _, _) = brep_tools::uv_bounds(&w.fac)?;
                    *bounds = (umin, umax);
                }
                // premier passage : on met les wires non touches des faces en vis a vis
                self.untouched_fused_wires(w, *bounds)?;
                w.mod_face = true;
            }
            self.rebuild_wire(w, &wir)?;
            self.fused_face_edges(w, *bounds)?;
        }
        Ok(())
    }

    /// Fusion des faces, ou reconstruction: the new faces of each face of `the_ff_map`
    /// (and of the faces fused with it), made by BRepAlgo_Loop.
    pub(super) fn fuse_faces(&mut self) -> Result<(), Failure> {
        let entries: Vec<(Shape, Vec<Shape>)> = self
            .the_ff_map
            .items()
            .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
            .collect();
        let mut bounds = (0., 0.);
        for (key, fused) in entries {
            let mut w = self.face_work(&key, fused)?;
            self.collect_face_edges(&mut w, &mut bounds)?;
            if w.listofedg.is_empty() {
                continue;
            }
            let mut l = BRepAlgoLoop::new();
            l.init(&w.newface);
            l.add_const_edges(&w.listofedg);
            l.perform()?;
            l.wires_to_faces()?;
            let listoffaces = l.new_faces().clone();
            self.to_remove.add(key.clone());
            self.mod_shapes.bind(key.clone(), listoffaces.clone());
            for facbis in &w.fused {
                self.mod_shapes.bind(facbis.clone(), listoffaces.clone());
            }
        }
        Ok(())
    }
}

// Rust translation of BRepOffset_MakeOffset::UpdateFaceOffset and CorrectConicalFaces of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The propagation of the specific face offsets to the tangent faces, and the
//! spherical faces closing the apexes of offset cones (arc joins).
use super::BRepOffsetMakeOffset;
use super::by_arc::entries;
use super::statics::{self, compound, not_bound};
use crate::brep_offset::tool::live;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d};
use rustycad_tkbrep::brep_tools::Substitution;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Shape, ShapeIterator, Solid, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSphericalSurface, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Axis2, Axis3, Direction2, Direction3, Point2, Point3, Sphere3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeFace, BRepLibMakeVertex};
use std::f64::consts::{FRAC_PI_2, PI};

impl BRepOffsetMakeOffset {
    /// `UpdateFaceOffset()`: the specific offset of each face given to the faces tangent
    /// to it (on the offset side for arc joins).
    pub(super) fn update_face_offset(&mut self) -> Result<(), Failure> {
        let mut m = MapOfShape::default();
        let copied_map = entries(&self.my_face_offset)?;
        let rt = if self.my_offset < 0. {
            TypeOfConcavity::Concave
        } else {
            TypeOfConcavity::Convex
        };
        for (fs, cur_offset) in copied_map {
            if !m.add(fs.clone()) {
                continue;
            }
            let f = topods::face(&fs)?;
            let mut c = Compound::default();
            BRepBuilder::default().make_compound(&mut c);
            let mut co = c.into_shape();
            BRepBuilder::default().add(&mut co, &fs)?;
            let mut co = topods::compound(&co)?;
            let mut dummy = MapOfShape::default();
            if self.my_join == JoinType::Arc {
                self.my_analyse.add_faces_two_types(
                    &f,
                    &mut co,
                    &mut dummy,
                    TypeOfConcavity::Tangential,
                    rt,
                )?;
            } else {
                self.my_analyse
                    .add_faces(&f, &mut co, &mut dummy, TypeOfConcavity::Tangential)?;
            }
            for ff in Explorer::new(co.as_ref(), ShapeEnum::Face, ShapeEnum::Shape)? {
                if !m.add(ff.clone()) {
                    continue;
                }
                self.my_face_offset.unbind(&ff);
                self.my_face_offset.bind(ff, cur_offset);
            }
        }
        Ok(())
    }

    /// `CorrectConicalFaces()`: for arc joins, the faces whose degenerated edges are not
    /// points (the offsets of cones at their apex) closed by a spherical face at the
    /// apex, added to the shell; the result rebuilt as solids.
    pub(super) fn correct_conical_faces(&mut self) -> Result<(), Failure> {
        if self.my_offset_shape.is_null() {
            return Ok(());
        }
        let tol_apex = 1.0e-5;
        let mut faces_of_cone: DataMap<Shape, List<Shape>, ShapeMapHasher> = DataMap::default();
        if self.my_join == JoinType::Arc {
            for fs in Explorer::new(&self.my_offset_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
                let a_face = topods::face(&fs)?;
                let mut emap = IndexedMapOfShape::default();
                topexp::map_shapes_of_type(&fs, ShapeEnum::Edge, &mut emap)?;
                for i in 1..=emap.extent() {
                    let an_edge = topods::edge(emap.find_key(i)?)?;
                    if !BRepTool::degenerated(&an_edge)? {
                        continue;
                    }
                    // Whether the edge is really degenerated.
                    let ba_curve = BRepAdaptorCurve::with_edge_on_face(&an_edge, &a_face)?;
                    let (bf, bl) = (ba_curve.first_parameter()?, ba_curve.last_parameter()?);
                    let pfirst = ba_curve.value(bf)?;
                    let plast = ba_curve.value(bl)?;
                    let pmid = ba_curve.value((bf + bl) / 2.)?;
                    if pfirst.distance(&plast) <= tol_apex && pfirst.distance(&pmid) <= tol_apex {
                        continue;
                    }
                    let or_edge = topods::edge(&self.my_init_offset_edge.root(an_edge.as_ref())?)?;
                    let vf = topexp::first_vertex(&or_edge, false);
                    faces_of_cone
                        .try_emplaced(vf.as_ref().clone(), List::default)
                        .append(fs.clone());
                }
            }
        }
        let builder = BRepBuilder::default();
        let l = Location::default();
        let mut is_modified = false;
        for (apex_s, faces) in entries(&faces_of_cone)? {
            let an_apex = topods::vertex(&apex_s)?;
            let mut sphere_wire_w = Wire::default();
            builder.make_wire(&mut sphere_wire_w);
            let mut sphere_wire = sphere_wire_w.into_shape();
            let mut the_sphere = Sphere3::default();
            let mut a_sph_surf: Handle<dyn GeomSurface> = Handle::default();
            let mut is_first_face = true;
            let mut first_point = Point3::default();
            let mut the_first_vertex = Vertex::default();
            let mut cur_first_vertex = Vertex::default();
            for fs in faces.iter() {
                let a_face = topods::face(fs)?;
                let mut deg_edge = Edge::default();
                for es in Explorer::new(fs, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    deg_edge = topods::edge(&es)?;
                    if BRepTool::degenerated(&deg_edge)? {
                        let or_edge = topods::edge(&self.my_init_offset_edge.root(&es)?)?;
                        let vf = topexp::first_vertex(&or_edge, false);
                        if vf.is_same(&an_apex) {
                            break;
                        }
                    }
                }
                let cur_edge = topods::edge(&deg_edge.oriented(Orientation::Forward))?;
                builder.degenerated(&cur_edge, false)?;
                builder.same_range(&cur_edge, false)?;
                builder.same_parameter(&cur_edge, false)?;
                let (f_pnt, m_pnt, l_pnt) = statics::get_edge_points(&cur_edge, &a_face)?;
                if is_first_face {
                    let a_vec1 = Vector3::between(f_pnt, m_pnt);
                    let a_vec2 = Vector3::between(f_pnt, l_pnt);
                    let a_norm = a_vec1.crossed(a_vec2);
                    let the_apex = BRepTool::pnt(&an_apex)?;
                    let apex_to_fpnt = Vector3::between(the_apex, f_pnt);
                    let ydir = a_norm.crossed(apex_to_fpnt);
                    let xdir = ydir.crossed(a_norm);
                    let an_ax2 = Axis2::with_x_direction(
                        the_apex,
                        Direction3::from_vector(a_norm)?,
                        Direction3::from_vector(xdir)?,
                    )?;
                    the_sphere.set_radius(self.my_offset)?;
                    the_sphere.set_position(Axis3::from_axis2(an_ax2));
                    a_sph_surf = Handle::new(GeomSphericalSurface::from_sphere(the_sphere)).cast();
                    first_point = f_pnt;
                    the_first_vertex = BRepLibMakeVertex::new(f_pnt)?.vertex()?;
                    cur_first_vertex = the_first_vertex.clone();
                }
                let (v1, v2) = topexp::vertices(&cur_edge, false);
                let first_vert = cur_first_vertex.clone();
                let end_vert = if l_pnt.distance(&first_point) <= precision::confusion() {
                    the_first_vertex.clone()
                } else {
                    BRepLibMakeVertex::new(l_pnt)?.vertex()?
                };
                cur_edge.tshape().set_free(true);
                let mut ce = cur_edge.as_ref().clone();
                builder.remove(&mut ce, v1.as_ref())?;
                builder.remove(&mut ce, v2.as_ref())?;
                builder.add(&mut ce, &first_vert.oriented(Orientation::Forward))?;
                builder.add(&mut ce, &end_vert.oriented(Orientation::Reversed))?;
                // The curve of the sphere on the edge.
                let (uf, vf) =
                    elslib::sphere_parameters(the_sphere.position(), the_sphere.radius(), f_pnt);
                let (mut ul, _vl) =
                    elslib::sphere_parameters(the_sphere.position(), the_sphere.radius(), l_pnt);
                if ul.abs() <= precision::confusion() {
                    ul = 2. * PI;
                }
                let a_curv = live(&a_sph_surf, "sphere")?.v_iso(vf)?;
                let a_trim_curv: Handle<dyn GeomCurve> =
                    Handle::new(GeomTrimmedCurve::new(&a_curv, uf, ul, true, true)?).cast();
                builder.update_edge_curve(&cur_edge, &a_trim_curv, precision::confusion())?;
                builder.range(&cur_edge, uf, ul, true)?;
                let the_lin2d: Handle<dyn Geom2dCurve> =
                    Handle::new(Geom2dLine::from_point(Point2::new(0., vf), Direction2::X)).cast();
                let the_trim_lin2d: Handle<dyn Geom2dCurve> =
                    Handle::new(Geom2dTrimmedCurve::new(&the_lin2d, uf, ul, true, true)?).cast();
                builder.update_edge_pcurve(
                    &cur_edge,
                    &the_trim_lin2d,
                    &a_sph_surf,
                    &l,
                    precision::confusion(),
                    None,
                )?;
                builder.range_on_surface(&cur_edge, &a_sph_surf, &l, uf, ul)?;
                brep_lib::same_parameter_edge(&cur_edge, 1.0e-5)?;
                builder.add(&mut sphere_wire, cur_edge.as_ref())?;
                // The corresponding edges of the face: the vertices common with CurEdge
                // substituted.
                let b_ac2d = BRepAdaptorCurve2d::with_edge_on_face(&cur_edge, &a_face)?;
                let f_pnt2d = b_ac2d.value(b_ac2d.first_parameter()?)?;
                let mut emap = IndexedMapOfShape::default();
                topexp::map_shapes_of_type(fs, ShapeEnum::Edge, &mut emap)?;
                let mut ee = Vec::new();
                for k in 1..=emap.extent() {
                    let an_edge = topods::edge(emap.find_key(k)?)?;
                    if !BRepTool::degenerated(&an_edge)? {
                        let (ev1, ev2) = topexp::vertices(&an_edge, false);
                        if ev1.is_same(&v1) || ev2.is_same(&v1) {
                            ee.push(an_edge);
                        }
                    }
                }
                for e in &ee {
                    let mut eforward = e.oriented(Orientation::Forward);
                    eforward.tshape().set_free(true);
                    let ef = topods::edge(&eforward)?;
                    let (ev1, ev2) = topexp::vertices(&ef, false);
                    let eec = BRepAdaptorCurve2d::with_edge_on_face(&ef, &a_face)?;
                    let p2d1 = eec.value(eec.first_parameter()?)?;
                    let p2d2 = eec.value(eec.last_parameter()?)?;
                    if ev1.is_same(&v1) {
                        let new_v = if p2d1.distance(f_pnt2d) <= precision::confusion() {
                            &first_vert
                        } else {
                            &end_vert
                        };
                        builder.remove(&mut eforward, ev1.as_ref())?;
                        builder.add(&mut eforward, &new_v.oriented(Orientation::Forward))?;
                    } else {
                        let new_v = if p2d2.distance(f_pnt2d) <= precision::confusion() {
                            &first_vert
                        } else {
                            &end_vert
                        };
                        builder.remove(&mut eforward, ev2.as_ref())?;
                        builder.add(&mut eforward, &new_v.oriented(Orientation::Reversed))?;
                    }
                }
                is_first_face = false;
                cur_first_vertex = end_vert;
            }
            // The new spherical face.
            let (mut ufirst, mut ulast) = (f64::MAX, f64::MIN);
            let mut edges_of_wire = Vec::new();
            let mut itw = ShapeIterator::new(&sphere_wire, true, true);
            while itw.more() {
                let an_edge = topods::edge(itw.value()?)?;
                itw.next();
                edges_of_wire.push(an_edge.as_ref().clone());
                let pc = BRepTool::curve_on_surface(&an_edge, &a_sph_surf, &l)?;
                let c = live(&pc.pcurve, "pcurve")?;
                let (p2d1, p2d2) = (c.value(pc.first)?, c.value(pc.last)?);
                for x in [p2d1.x(), p2d2.x()] {
                    if x < ufirst {
                        ufirst = x;
                    }
                    if x > ulast {
                        ulast = x;
                    }
                }
            }
            let mut new_edges = Vec::new();
            let mut first_edge = Edge::default();
            let mut p2d1 = Point2::default();
            let mut k = 0;
            while k < edges_of_wire.len() {
                first_edge = topods::edge(&edges_of_wire[k])?;
                let pc = BRepTool::curve_on_surface(&first_edge, &a_sph_surf, &l)?;
                let c = live(&pc.pcurve, "pcurve")?;
                p2d1 = c.value(pc.first)?;
                if (p2d1.x() - ufirst).abs() <= precision::confusion() {
                    edges_of_wire.remove(k);
                    break;
                }
                k += 1;
            }
            new_edges.push(first_edge.oriented(Orientation::Forward));
            let (vf1, mut cur_vertex) = topexp::vertices(&first_edge, false);
            let mut k = 0;
            while k < edges_of_wire.len() {
                let an_edge = topods::edge(&edges_of_wire[k])?;
                let (ev1, ev2) = topexp::vertices(&an_edge, false);
                if ev1.is_same(&cur_vertex) || ev2.is_same(&cur_vertex) {
                    new_edges.push(an_edge.oriented(Orientation::Forward));
                    cur_vertex = if ev1.is_same(&cur_vertex) { ev2 } else { ev1 };
                    edges_of_wire.remove(k);
                } else {
                    k += 1;
                }
            }
            let (vfirst, vlast) = if p2d1.y() > 0. {
                (p2d1.y(), FRAC_PI_2)
            } else {
                (-FRAC_PI_2, p2d1.y())
            };
            let mut new_spherical_face = BRepLibMakeFace::from_surface_bounds(
                &a_sph_surf,
                (ufirst, ulast, vfirst, vlast),
                precision::confusion(),
            )?
            .face()?;
            let mut deg_edge = Edge::default();
            for es in Explorer::new(
                new_spherical_face.as_ref(),
                ShapeEnum::Edge,
                ShapeEnum::Shape,
            )? {
                deg_edge = topods::edge(&es)?;
                if BRepTool::degenerated(&deg_edge)? {
                    break;
                }
            }
            let deg_vertex = topexp::first_vertex(&deg_edge, false);
            let mut old_edge = Edge::default();
            for es in Explorer::new(
                new_spherical_face.as_ref(),
                ShapeEnum::Edge,
                ShapeEnum::Shape,
            )? {
                old_edge = topods::edge(&es)?;
                let (ov1, ov2) = topexp::vertices(&old_edge, false);
                if !ov1.is_same(&deg_vertex) && !ov2.is_same(&deg_vertex) {
                    break;
                }
            }
            let (ov1, ov2) = topexp::vertices(&old_edge, false);
            let lv1 = vec![vf1.oriented(Orientation::Forward)];
            let lv2 = vec![cur_vertex.oriented(Orientation::Forward)];
            let mut the_substitutor = Substitution::new();
            the_substitutor.substitute(&ov1.oriented(Orientation::Forward), &lv1)?;
            if !ov1.is_same(&ov2) {
                the_substitutor.substitute(&ov2.oriented(Orientation::Forward), &lv2)?;
            }
            the_substitutor.substitute(&old_edge.oriented(Orientation::Forward), &new_edges)?;
            the_substitutor.build(new_spherical_face.as_ref())?;
            if the_substitutor.is_copied(new_spherical_face.as_ref()) {
                let list_sh = the_substitutor.copy(new_spherical_face.as_ref())?;
                new_spherical_face = topods::face(list_sh.first().ok_or_else(not_bound)?)?;
            }
            // The new spherical face added to the shell.
            let mut the_shell =
                Explorer::new(&self.my_offset_shape, ShapeEnum::Shell, ShapeEnum::Shape)?
                    .current()
                    .clone();
            the_shell.tshape().set_free(true);
            builder.add(&mut the_shell, new_spherical_face.as_ref())?;
            is_modified = true;
            if !the_shell.tshape().closed() && BRepTool::is_closed(&the_shell)? {
                the_shell.tshape().set_closed(true);
            }
        }
        if !is_modified {
            return Ok(());
        }
        if self.my_shape.shape_type()? == ShapeEnum::Solid || self.my_thickening {
            let mut nb_shell = 0;
            let mut nc = compound();
            let mut s1 = Shape::default();
            let mut sol_s = Solid::default();
            builder.make_solid(&mut sol_s);
            let mut sol = sol_s.into_shape();
            sol.tshape().set_closed(true);
            for sh in Explorer::new(&self.my_offset_shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
                nb_shell += 1;
                if sh.tshape().closed() {
                    builder.add(&mut sol, &sh)?;
                } else {
                    builder.add(&mut nc, &sh)?;
                    if nb_shell == 1 {
                        s1 = sh;
                    }
                }
            }
            let nbs = ShapeIterator::new(&sol, true, true).count();
            let mut sol_is_null = nbs == 0;
            // The check of the solid.
            if nbs > 1 {
                let a_check = BRepCheckAnalyzer::new(&sol, false, false, false)?;
                if !a_check.is_valid()? {
                    let mut a_sol_list = List::default();
                    statics::correct_solid(&mut sol, &mut a_sol_list)?;
                    if !a_sol_list.is_empty() {
                        builder.add(&mut nc, &sol)?;
                        for s in a_sol_list.iter() {
                            builder.add(&mut nc, s)?;
                        }
                        sol_is_null = true;
                    }
                }
            }
            let nc_is_null = ShapeIterator::new(&nc, true, true).count() == 0;
            if !sol_is_null && !nc_is_null {
                builder.add(&mut nc, &sol)?;
                self.my_offset_shape = nc;
            } else if sol_is_null && !nc_is_null {
                self.my_offset_shape = if nb_shell == 1 { s1 } else { nc };
            } else if !sol_is_null && nc_is_null {
                self.my_offset_shape = sol;
            } else {
                self.my_offset_shape = nc;
            }
        }
        Ok(())
    }
}

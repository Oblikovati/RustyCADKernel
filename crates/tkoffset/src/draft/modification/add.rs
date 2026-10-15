// Rust translation of Draft_Modification::InternalAdd and Draft_Modification::Propagate of
// ModelingAlgorithms/TKOffset/Draft/Draft_Modification_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The faces added to the draft with their tangent neighbours, and the propagation of
//! the modification to the edges, vertices and faces they touch.
use super::geometry::{new_curve, new_surface, orientation};
use super::{DraftState, live, untrimmed_curve, untrimmed_surface};
use crate::draft::info::{DraftEdgeInfo, DraftErrorStatus, DraftFaceInfo, DraftVertexInfo};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConic, GeomCurve, GeomCylindricalSurface, GeomElementarySurface, GeomLine,
    GeomPlane, GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSweptSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurfaceOfLinearExtrusion};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::int_ana::IntConicQuad;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{Shape as Continuity, SurfaceType};
use rustycad_tkmath::gp::{Axis1, Axis3, Circle3, Direction3, Plane3, Vector3};
use std::f64::consts::PI;

/// The transformed copy of a surface (`S->Transformed(L.Transformation())`, a copy even
/// at the identity).
pub(super) fn located_surface(face: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (s, lo) = BRepTool::surface_with_location(face)?;
    Ok(live(&s, "surface")?
        .transformed(lo.transformation())?
        .cast())
}

/// The transformed copy of the 3d curve of an edge.
pub(super) fn located_curve(edge: &Edge) -> Result<Handle<dyn GeomCurve>, Failure> {
    let c = BRepTool::curve_with_location(edge)?;
    Ok(live(&c.curve, "curve")?
        .transformed(c.location.transformation())?
        .cast())
}

/// The trimmed copy of a surface in `[0, umax] x [vmin, vmax]`.
pub(super) fn trimmed(
    s: &Handle<dyn GeomSurface>,
    umax: f64,
    vmin: f64,
    vmax: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(Handle::new(GeomRectangularTrimmedSurface::new(
        s,
        (0., umax),
        (vmin, vmax),
        (true, true),
    )?)
    .cast())
}

impl DraftState {
    /// `InternalAdd(F, Direction, Angle, NeutralPlane, Flag)`.
    pub(super) fn internal_add(
        &mut self,
        f: &Face,
        direction: Direction3,
        angle: f64,
        neutral_plane: &Plane3,
        flag: bool,
    ) -> Result<bool, Failure> {
        if self.my_f_map.contains(f) {
            return Ok(self.bad_shape.is_null());
        }

        let oris = orientation(&self.my_shape, f)?;
        let mut s = located_surface(f)?;
        s = untrimmed_surface(&s)?;
        let mut new_s: Handle<dyn GeomSurface> = Handle::default();
        let mut the_circle: Handle<dyn GeomCurve> = Handle::default();

        let mut postponed = !flag;
        if postponed {
            let surface = live(&s, "surface")?;
            if surface.is_instance_name("Geom_CylindricalSurface")
                || surface.is_instance_name("Geom_SurfaceOfLinearExtrusion")
            {
                let mut cir: Circle3;
                if surface.is_instance_name("Geom_CylindricalSurface") {
                    let cyl = live(&s.clone().cast::<GeomCylindricalSurface>(), "cylinder")?
                        .cylinder()?;
                    let axcyl = cyl.axis();
                    cir = elslib::cylinder_v_iso(cyl.position(), cyl.radius(), 0.);
                    let vv = Vector3::between(cyl.location(), neutral_plane.location());
                    cir.translate(
                        Vector3::from(axcyl.direction()) * vv.dot(Vector3::from(axcyl.direction())),
                    );
                } else {
                    let extrusion = s.clone().cast::<GeomSurfaceOfLinearExtrusion>();
                    let extrusion = live(&extrusion, "extrusion")?;
                    let mut cbas = extrusion.basis_curve();
                    let the_dirextr = extrusion.direction();

                    if live(&cbas, "curve")?.is_kind_name("Geom_TrimmedCurve") {
                        cbas = untrimmed_curve(&cbas)?;
                    }
                    if live(&cbas, "curve")?.is_kind_name("Geom_Circle") {
                        cir = live(&cbas.clone().cast::<GeomCircle>(), "circle")?.circ()?;
                        let dircir = cir.axis().direction();
                        if !direction.is_parallel(dircir, precision::angular()) {
                            self.bad_shape = f.clone().into_shape();
                            self.err_stat = DraftErrorStatus::FaceRecomputation;
                            return Ok(false);
                        }
                    } else {
                        self.bad_shape = f.clone().into_shape();
                        self.err_stat = DraftErrorStatus::FaceRecomputation;
                        return Ok(false);
                    }

                    let axis: Axis3 = neutral_plane.position();
                    let l = Vector3::between(cir.location(), axis.location())
                        .dot(Vector3::from(axis.direction()));
                    let cos = the_dirextr.dot(axis.direction());
                    let vv = Vector3::from(the_dirextr) * (l / cos);
                    cir.translate(vv);
                }

                the_circle = Handle::new(GeomCircle::from_circ(cir)).cast();
            } else {
                postponed = false;
            }
        }

        if !postponed {
            new_s = new_surface(&s, oris, direction, angle, neutral_plane)?;
            if new_s.is_null() {
                self.bad_shape = f.clone().into_shape();
                self.err_stat = DraftErrorStatus::FaceRecomputation;
                return Ok(false);
            }
            // To avoid some problems with infinite restrictions
            let typs = live(&new_s, "surface")?;
            let is_cylinder = typs.is_instance_name("Geom_CylindricalSurface");
            if is_cylinder || typs.is_instance_name("Geom_ConicalSurface") {
                let (_umin, _umax, mut vmin, mut vmax) = brep_tools::uv_bounds(f)?;
                if !precision::is_negative_infinite(vmin) && !precision::is_positive_infinite(vmax)
                {
                    let deltav = 10. * (vmax - vmin);
                    if is_cylinder {
                        vmin -= deltav;
                        vmax += deltav;
                    } else {
                        let co = live(
                            &new_s
                                .clone()
                                .cast::<rustycad_tkg3d::geom::GeomConicalSurface>(),
                            "cone",
                        )?
                        .cone()?;
                        let vapex = -co.ref_radius() / co.semi_angle().sin();
                        if vmin < vapex {
                            // vmax should not exceed Vapex
                            if vmax + deltav > vapex {
                                vmax = vapex;
                                vmin -= 10. * (vmax - vmin);
                                // JAG debug to avoid apex
                                vmax -= precision::confusion();
                            } else {
                                vmin -= deltav;
                                vmax += deltav;
                            }
                        } else {
                            // Vapex <= vmin < vmax
                            if vmin - deltav < vapex {
                                vmin = vapex;
                                vmax += 10. * (vmax - vmin);
                                // JAG debug to avoid apex
                                vmin += precision::confusion();
                            } else {
                                vmin -= deltav;
                                vmax += deltav;
                            }
                        }
                    }
                    new_s = trimmed(&new_s, 2. * PI, vmin, vmax)?;
                }
            }
        }

        if postponed || s != new_s {
            let mut fi = DraftFaceInfo::with_surface(&new_s, true);
            fi.set_root_face(&self.cur_face);
            self.my_f_map.add(f.clone().into_shape(), fi);
            if postponed {
                *self.my_f_map.change_from_key(f)?.change_curve() = the_circle.clone();
            }
        }

        let mut a_exp = Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)?;
        let mut map_of_e = MapOfShape::default();
        while a_exp.more() && self.bad_shape.is_null() {
            let edg = topods::edge(a_exp.current())?;
            if !self.my_e_map.contains(&edg) {
                let mut addedg = false;
                let mut addface = false;
                let mut other_f = Face::default();
                if brep_tools::is_really_closed(&edg, f)? {
                    addedg = true;
                    addface = false;
                } else {
                    // Find the other face containing the edge.
                    let mut nbother = 0;
                    for it in self.my_ef_map.find_from_key(&edg)?.iter() {
                        if !it.is_same(f) {
                            if other_f.is_null() {
                                other_f = topods::face(it)?;
                            }
                            nbother += 1;
                        }
                    }
                    if nbother >= 2 {
                        self.bad_shape = edg.clone().into_shape();
                        self.err_stat = DraftErrorStatus::EdgeRecomputation;
                    } else if !other_f.is_null()
                        && BRepTool::continuity_on_faces(&edg, f, &other_f)? >= Continuity::G1
                    {
                        addface = true;
                        addedg = true;
                    }
                    // nbother == 0: the native code leaves the face alone.
                }
                if addedg {
                    if postponed {
                        self.my_f_map.change_from_key(f)?.add(&other_f);
                    }
                    let mut c = located_curve(&edg)?;
                    c = untrimmed_curve(&c)?;
                    let mut new_c: Handle<dyn GeomCurve> = Handle::default();
                    let mut e_inf = DraftEdgeInfo::with_new_geometry(true);
                    if postponed {
                        e_inf.add(f)?;
                        e_inf.add(&other_f)?;

                        // find fixed point
                        let a_local_geom = c.clone().cast::<GeomLine>();
                        match a_local_geom.get() {
                            None => {
                                self.bad_shape = edg.clone().into_shape();
                                self.err_stat = DraftErrorStatus::EdgeRecomputation;
                            }
                            Some(line) => {
                                let lin = line.lin();
                                let ilipl = IntConicQuad::line_plane(
                                    lin,
                                    *neutral_plane,
                                    precision::angular(),
                                    0.,
                                    0.,
                                )?;
                                if ilipl.is_done() && ilipl.nb_points()? != 0 {
                                    e_inf.tangent(ilipl.point(1)?);
                                } else {
                                    self.bad_shape = edg.clone().into_shape();
                                    self.err_stat = DraftErrorStatus::EdgeRecomputation;
                                }
                            }
                        }
                    } else {
                        new_c = new_curve(&c, &s, oris, direction, angle, neutral_plane)?;
                        if new_c.is_null() {
                            self.bad_shape = edg.clone().into_shape();
                            self.err_stat = DraftErrorStatus::EdgeRecomputation;
                        }
                    }

                    new_c = super::down_cast_basis(&new_c);
                    *e_inf.change_geometry() = new_c;

                    e_inf.set_root_face(&self.cur_face);
                    self.my_e_map.add(edg.clone().into_shape(), e_inf);
                    map_of_e.add(edg.clone().into_shape());
                    if addface {
                        let fl = flag;
                        let (mut alocal_surface, _) = BRepTool::surface_with_location(&other_f)?;
                        alocal_surface = untrimmed_surface(&alocal_surface)?;
                        let typs = live(&alocal_surface, "surface")?;
                        if (typs.is_instance_name("Geom_CylindricalSurface")
                            || typs.is_instance_name("Geom_SurfaceOfLinearExtrusion"))
                            && self.my_f_map.contains(f)
                            && !flag
                            && !postponed
                        {
                            self.my_f_map.remove_key(f);
                            for k in map_of_e.iter() {
                                self.my_e_map.remove_key(k);
                            }
                        }
                        self.internal_add(&other_f, direction, angle, neutral_plane, fl)?;
                    }
                }
            }
            a_exp.next()?;
        }
        Ok(self.bad_shape.is_null())
    }

    /// `Propagate()`: the edges and vertices of the modified faces, the edges through
    /// the modified vertices and the faces along the modified edges, then the planes of
    /// the free borders; false with the problematic shape on failure.
    pub(super) fn propagate(&mut self) -> Result<bool, Failure> {
        if !self.bad_shape.is_null() {
            return Ok(false);
        }

        // Set all edges and vertices of modified faces
        for i in 1..=self.my_f_map.extent() {
            let fc = topods::face(self.my_f_map.find_key(i)?)?;

            // Exploration of the edges of the face
            for e in Explorer::new(&fc, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let e = topods::edge(&e)?;
                if !self.my_e_map.contains(&e) {
                    self.my_e_map.add(
                        e.clone().into_shape(),
                        DraftEdgeInfo::with_new_geometry(true),
                    );
                }
                self.my_e_map.change_from_key(&e)?.add(&fc)?;

                // Exploration of the vertices of the edge
                for v in Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                    let v = topods::vertex(&v)?;
                    if !self.my_v_map.contains(&v) {
                        self.my_v_map
                            .add(v.clone().into_shape(), DraftVertexInfo::new());
                    }
                    let vinf = self.my_v_map.change_from_key(&v)?;
                    vinf.add(&e);
                    *vinf.change_parameter(&e)? = BRepTool::parameter(&v, &e)?;
                }
            }
        }

        // Set edges containing modified vertices.
        for i in 1..=self.my_v_map.extent() {
            let vt = topods::vertex(self.my_v_map.find_key(i)?)?;

            // Exploration of the ancestors of the vertex
            for e in Explorer::new(&self.my_shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let e = topods::edge(&e)?;
                let mut found = false;
                for v in Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                    if vt.is_same(&v) {
                        found = true;
                        break;
                    }
                }
                if found {
                    if !self.my_e_map.contains(&e) {
                        self.my_e_map.add(
                            e.clone().into_shape(),
                            DraftEdgeInfo::with_new_geometry(false),
                        );
                    }
                    let vinf = self.my_v_map.change_from_key(&vt)?;
                    vinf.add(&e);
                    *vinf.change_parameter(&e)? = BRepTool::parameter(&vt, &e)?;
                }
            }
        }

        // Set faces containing modified edges
        for i in 1..=self.my_e_map.extent() {
            let ed = self.my_e_map.find_key(i)?.clone();
            let ancestors: Vec<Shape> =
                self.my_ef_map.find_from_key(&ed)?.iter().cloned().collect();
            for it in &ancestors {
                let f = topods::face(it)?;
                if !self.my_f_map.contains(&f) {
                    let (s, l) = BRepTool::surface_with_location(&f)?;
                    let mut new_s: Handle<dyn GeomSurface> =
                        live(&s, "surface")?.transformed(l.transformation())?.cast();

                    let typs = live(&s, "surface")?;
                    if typs.is_instance_name("Geom_CylindricalSurface")
                        || typs.is_instance_name("Geom_ConicalSurface")
                    {
                        let (_umin, _umax, mut vmin, mut vmax) = brep_tools::uv_bounds(&f)?;
                        if !precision::is_negative_infinite(vmin)
                            && !precision::is_positive_infinite(vmax)
                        {
                            let deltav = 10. * (vmax - vmin);
                            vmin -= deltav;
                            vmax += deltav;
                            new_s = trimmed(&new_s, 2. * PI, vmin, vmax)?;
                        }
                    }

                    self.my_f_map.add(
                        f.clone().into_shape(),
                        DraftFaceInfo::with_surface(&new_s, false),
                    );
                }
                self.my_e_map.change_from_index(i)?.add(&f)?;
            }
        }

        //  Try to add faces for free borders...
        // JAG 09.11.95
        let mut i = 1;
        while i <= self.my_e_map.extent() {
            let einf = self.my_e_map.find_from_index(i)?;
            if einf.new_geometry() && einf.geometry().is_null() && einf.second_face().is_null() {
                let first_face = einf.first_face().clone();
                let s1 = located_surface(&first_face)?;
                let mut s2: Handle<dyn GeomSurface> = Handle::default();

                let ek = topods::edge(self.my_e_map.find_key(i)?)?;
                let mut c = located_curve(&ek)?;
                c = untrimmed_curve(&c)?;
                let s1_surface = live(&s1, "surface")?;
                let curve = live(&c, "curve")?;
                if !s1_surface.is_kind_name("Geom_Plane") {
                    if curve.is_kind_name("Geom_Conic") {
                        let the_pl = Axis3::from_axis2(
                            live(&c.clone().cast::<dyn GeomConic>(), "conic")?.position(),
                        );
                        s2 = Handle::new(GeomPlane::new(the_pl)).cast();
                    } else if curve.is_instance_name("Geom_Line") {
                        let axis: Axis1 =
                            if s1_surface.is_instance_name("Geom_RectangularTrimmedSurface") {
                                let basis = live(
                                    &s1.clone().cast::<GeomRectangularTrimmedSurface>(),
                                    "surface",
                                )?
                                .basis_surface();
                                live(
                                    &basis.cast::<dyn GeomElementarySurface>(),
                                    "elementary surface",
                                )?
                                .axis()
                            } else {
                                live(
                                    &s1.clone().cast::<dyn GeomElementarySurface>(),
                                    "elementary surface",
                                )?
                                .axis()
                            };
                        let they = Vector3::between(axis.location(), curve.value(0.)?);
                        let axz =
                            Direction3::from_vector(Vector3::from(axis.direction()).crossed(they))?;
                        s2 = Handle::new(GeomPlane::new(Axis3::new(
                            axis.location(),
                            axz,
                            axis.direction(),
                        )?))
                        .cast();
                    } else {
                        self.bad_shape = ek.into_shape();
                        self.err_stat = DraftErrorStatus::EdgeRecomputation;
                        break; // leave from for
                    }
                } else {
                    // on the plane
                    for j in 1..=self.my_v_map.extent() {
                        let vinf = self.my_v_map.change_from_index(j)?;
                        vinf.init_edge_iterator();
                        while vinf.more_edge() {
                            if vinf.edge()?.is_same(&ek) {
                                break;
                            }
                            vinf.next_edge();
                        }
                        if vinf.more_edge() {
                            vinf.init_edge_iterator();
                            while vinf.more_edge() {
                                let edg = vinf.edge()?;
                                if !edg.is_same(&ek) {
                                    let ei = self.my_e_map.find_from_key(&edg)?;
                                    if !ei.first_face().is_same(&first_face)
                                        && (ei.second_face().is_null()
                                            || !ei.second_face().is_same(&first_face))
                                    {
                                        break;
                                    }
                                }
                                vinf.next_edge();
                            }
                            if vinf.more_edge() {
                                let mut c2 = located_curve(&vinf.edge()?)?;
                                c2 = untrimmed_curve(&c2)?;
                                let (h_cur, direc): (Handle<dyn GeomCurve>, Direction3) =
                                    if curve.is_instance_name("Geom_Line") {
                                        (
                                            c2.clone(),
                                            live(&c.clone().cast::<GeomLine>(), "line")?
                                                .lin()
                                                .direction(),
                                        )
                                    } else if live(&c2, "curve")?.is_instance_name("Geom_Line") {
                                        (
                                            c.clone(),
                                            live(&c2.clone().cast::<GeomLine>(), "line")?
                                                .lin()
                                                .direction(),
                                        )
                                    } else {
                                        self.bad_shape = ek.clone().into_shape();
                                        self.err_stat = DraftErrorStatus::EdgeRecomputation;
                                        break; // leave from while
                                    };
                                let h_cur_adaptor =
                                    Handle::new(GeomAdaptorCurve::with_curve(&h_cur)?);
                                let sle =
                                    GeomAdaptorSurfaceOfLinearExtrusion::with_curve_and_direction(
                                        &h_cur_adaptor.clone().cast(),
                                        direc,
                                    );
                                s2 = match sle.get_type()? {
                                    SurfaceType::Plane => {
                                        Handle::new(GeomPlane::from_pln(sle.plane()?)).cast()
                                    }
                                    SurfaceType::Cylinder => Handle::new(
                                        GeomCylindricalSurface::from_cylinder(sle.cylinder()?),
                                    )
                                    .cast(),
                                    _ => Handle::new(GeomSurfaceOfLinearExtrusion::new(
                                        &live(&h_cur_adaptor, "adaptor")?.curve(),
                                        direc,
                                    )?)
                                    .cast(),
                                };
                            } else {
                                self.bad_shape = ek.clone().into_shape();
                                self.err_stat = DraftErrorStatus::EdgeRecomputation;
                                break; // leave from while
                            }
                            break;
                        }
                    }
                }

                if self.bad_shape.is_null() {
                    let mut the_new_face = Face::default();
                    BRepBuilder::default().make_face_on_surface(
                        &mut the_new_face,
                        &s2,
                        precision::confusion(),
                    )?;
                    self.my_e_map.change_from_index(i)?.add(&the_new_face)?;
                    self.my_f_map.add(
                        the_new_face.into_shape(),
                        DraftFaceInfo::with_surface(&s2, false),
                    );
                } else {
                    break; // leave from for
                }
                // Fin JAG 09.11.95
            }
            i += 1;
        }
        Ok(self.bad_shape.is_null())
    }
}

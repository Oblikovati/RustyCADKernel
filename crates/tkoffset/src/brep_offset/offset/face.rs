// Rust translation of BRepOffset_Offset::Init(Face, Offset, Created, OffsetOutside,
// JoinType) of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Offset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The offset face of a face: the offset surface (trimmed at the apex of a cone or of
//! an offset surface with degenerated edges) bounded by the images of the edges.
use super::curve3d::compute_curve3d;
use super::{BRepOffsetOffset, CreatedShapes, update_edge_pcurve, update_edge_pcurves};
use crate::brep_offset::tool::live;
use crate::brep_offset::{BRepOffsetTool, surface};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{update_face, uv_bounds};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{
    GeomBoundedSurface, GeomConicalSurface, GeomElementarySurface, GeomLine, GeomOffsetSurface,
    GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::{ExtremaCurveCurve, ProjectPointOnCurve};
use rustycad_tkgeombase::geom_lib::extend_surf_by_length;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::toploc::Location;

/// The apex tolerance of the degenerated-edge processing.
const TOL_APEX: f64 = 1.0e-5;

/// The degenerated sides of the face and the apex points of its offset.
#[derive(Default)]
struct Singularity {
    umin: bool,
    umax: bool,
    vmin: bool,
    vmax: bool,
    min_apex: Point3,
    max_apex: Point3,
    has: bool,
}

/// A rectangular trimmed surface.
fn trimmed(
    s: &Handle<dyn GeomSurface>,
    u: (f64, f64),
    v: (f64, f64),
) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(Handle::new(GeomRectangularTrimmedSurface::new(s, u, v, (true, true))?).cast())
}

/// The parameters of one side treatment of an offset surface with a degenerated side.
struct SideApex {
    /// The iso of the degenerated side.
    iso_u: bool,
    /// The parameter of that iso.
    iso: f64,
    /// The basis point `(u, v)` of the apex.
    apex: (f64, f64),
    /// The quarter and middle points along the other direction.
    quart: (f64, f64),
    mid: (f64, f64),
    /// The derivative orders of the generatrix.
    dn: (i32, i32),
    /// The `ExtendSurfByLength` direction and side.
    in_u: bool,
    after: bool,
}

impl BRepOffsetOffset {
    /// `Init(Face, Offset, Created, OffsetOutside, JoinType)`: the offset face of a face,
    /// the images of its edges and vertices taken from `created` when bound;
    /// `o.init_face_created(&face, 2.0, &created, true, JoinType::Arc)?`.
    pub fn init_face_created(
        &mut self,
        face: &Face,
        offset: f64,
        created: &CreatedShapes,
        offset_outside: bool,
        join_type: JoinType,
    ) -> Result<(), Failure> {
        self.my_shape = face.as_ref().clone();
        let mut my_offset = offset;
        if face.orientation() == Orientation::Reversed {
            my_offset *= -1.;
        }
        let (mut s, l) = BRepTool::surface_with_location(face)?;
        // The surfaces are untrimmed, which avoids copies in the extensions.
        if s.is_instance_name("Geom_RectangularTrimmedSurface") {
            s = live(
                &s.clone().cast::<GeomRectangularTrimmedSurface>(),
                "surface",
            )?
            .basis_surface();
        }
        let mut is_transformed = false;
        if (s.is_kind_name("Geom_BSplineSurface")
            || s.is_kind_name("Geom_SurfaceOfLinearExtrusion")
            || s.is_kind_name("Geom_SurfaceOfRevolution")
            || s.is_kind_name("Geom_OffsetSurface"))
            && !l.is_identity()
        {
            s = live(&s, "surface")?.copy()?.cast();
            live(&s, "surface")?.transform(l.transformation())?;
            is_transformed = true;
        }
        // The particular case of the cone.
        if s.is_instance_name("Geom_ConicalSurface") {
            let co = s.clone().cast::<GeomConicalSurface>();
            let co = live(&co, "cone")?;
            let cone = co.cone()?;
            let (_, vc) = elslib::cone_parameters(
                cone.position(),
                cone.ref_radius(),
                cone.semi_angle(),
                co.apex(),
            );
            let (_, _, vv1, vv2) = uv_bounds(face)?;
            if (vv2 < vc && co.semi_angle() > 0.) || (vv1 > vc && co.semi_angle() < 0.) {
                my_offset *= -1.;
            }
            if !co.position().direct()? {
                my_offset *= -1.;
            }
        }
        let (mut the_surf, status) = surface(&s, my_offset, false)?;
        self.my_status = status;

        // The offsets of faces with possible degenerated edges.
        let (uf1, uf2, vf1, vf2) = uv_bounds(face)?;
        let mut sing = Singularity::default();
        if (!offset_outside || join_type != JoinType::Arc)
            && (the_surf.is_instance_name("Geom_ConicalSurface")
                || the_surf.is_instance_name("Geom_OffsetSurface"))
        {
            let deg_edges = singular_degenerated_edges(face, &s)?;
            if !deg_edges.is_empty() {
                let the_deg_edge = &deg_edges[0];
                let pc = BRepTool::curve_on_face(the_deg_edge, face)?;
                let a_curve = live(&pc.pcurve, "pcurve")?;
                let fp2d = a_curve.value(pc.first)?;
                let lp2d = a_curve.value(pc.last)?;
                let uiso_degen = (fp2d.x() - lp2d.x()).abs() <= precision::p_confusion();
                if deg_edges.len() == 2 {
                    if uiso_degen {
                        sing.umin = true;
                        sing.umax = true;
                    } else {
                        sing.vmin = true;
                        sing.vmax = true;
                    }
                } else if uiso_degen {
                    if (fp2d.x() - uf1).abs() <= precision::confusion() {
                        sing.umin = true;
                    } else {
                        sing.umax = true;
                    }
                } else if (fp2d.y() - vf1).abs() <= precision::confusion() {
                    sing.vmin = true;
                } else {
                    sing.vmax = true;
                }
                if the_surf.is_instance_name("Geom_ConicalSurface") {
                    let the_cone =
                        live(&the_surf.clone().cast::<GeomConicalSurface>(), "cone")?.cone()?;
                    let apex = the_cone.apex();
                    let (_, vapex) = elslib::cone_parameters(
                        the_cone.position(),
                        the_cone.ref_radius(),
                        the_cone.semi_angle(),
                        apex,
                    );
                    if sing.vmin {
                        the_surf = trimmed(&the_surf, (uf1, uf2), (vapex, vf2))?;
                        sing.min_apex = apex;
                        sing.has = true;
                    } else if sing.vmax {
                        the_surf = trimmed(&the_surf, (uf1, uf2), (vf1, vapex))?;
                        sing.max_apex = apex;
                        sing.has = true;
                    }
                } else {
                    let (u14, u12) = (0.75 * uf1 + 0.25 * uf2, 0.5 * (uf1 + uf2));
                    let (v14, v12) = (0.75 * vf1 + 0.25 * vf2, 0.5 * (vf1 + vf2));
                    let bounds = (uf1, uf2, vf1, vf2);
                    if sing.umin {
                        let side = SideApex {
                            iso_u: true,
                            iso: uf1,
                            apex: (uf1, vf1),
                            quart: (uf1, v14),
                            mid: (uf1, v12),
                            dn: (1, 0),
                            in_u: true,
                            after: false,
                        };
                        if let Some(p) = side_apex(
                            &mut the_surf,
                            &side,
                            bounds,
                            offset_outside,
                            ApexEnd::FirstU,
                        )? {
                            sing.min_apex = p;
                            sing.has = true;
                        }
                    }
                    if sing.umax {
                        let side = SideApex {
                            iso_u: true,
                            iso: uf2,
                            apex: (uf2, vf1),
                            quart: (uf2, v14),
                            mid: (uf2, v12),
                            dn: (1, 0),
                            in_u: true,
                            after: true,
                        };
                        if let Some(p) =
                            side_apex(&mut the_surf, &side, bounds, offset_outside, ApexEnd::LastU)?
                        {
                            sing.max_apex = p;
                            sing.has = true;
                        }
                    }
                    if sing.vmin {
                        let side = SideApex {
                            iso_u: false,
                            iso: vf1,
                            apex: (uf1, vf1),
                            quart: (u14, vf1),
                            mid: (u12, vf1),
                            dn: (0, 1),
                            in_u: false,
                            after: false,
                        };
                        if let Some(p) = side_apex(
                            &mut the_surf,
                            &side,
                            bounds,
                            offset_outside,
                            ApexEnd::FirstV,
                        )? {
                            sing.min_apex = p;
                            sing.has = true;
                        }
                    }
                    if sing.vmax {
                        let side = SideApex {
                            iso_u: false,
                            iso: vf2,
                            apex: (uf1, vf2),
                            quart: (u14, vf2),
                            mid: (u12, vf2),
                            dn: (0, 1),
                            in_u: false,
                            after: true,
                        };
                        if let Some(p) =
                            side_apex(&mut the_surf, &side, bounds, offset_outside, ApexEnd::LastV)?
                        {
                            sing.max_apex = p;
                            sing.has = true;
                        }
                    }
                }
            }
        }

        // The p-curves of the edges of the face.
        let builder = BRepBuilder::default();
        let mut my_face = Face::default();
        builder.make_face(&mut my_face);
        let face_location = if is_transformed {
            Location::default()
        } else {
            l.clone()
        };
        builder.update_face(
            &my_face,
            &the_surf,
            &face_location,
            BRepTool::face_tolerance(face)?,
        )?;
        let ctx = FaceContext {
            cur_face: topods::face(&face.oriented(Orientation::Forward))?,
            my_face: my_face.clone(),
            the_surf: the_surf.clone(),
            l: l.clone(),
            is_transformed,
            sing,
            vf1,
            bounds: live(&the_surf, "surface")?.bounds(),
        };
        let (mut map_ss, von_degen) = ctx.initial_images(created)?;
        let mut expw = Explorer::new(ctx.cur_face.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)?;
        while expw.more() {
            let w = topods::wire(expw.current())?;
            expw.next()?;
            let mut ow = Wire::default();
            builder.make_wire(&mut ow);
            let mut ow_shape = ow.into_shape();
            let mut expe = Explorer::new(
                &w.oriented(Orientation::Forward),
                ShapeEnum::Edge,
                ShapeEnum::Shape,
            )?;
            while expe.more() {
                let e = topods::edge(expe.current())?;
                expe.next()?;
                let oe = ctx.edge_image(&e, created, &mut map_ss, &von_degen)?;
                builder.add(&mut ow_shape, &oe.oriented(e.orientation()))?;
            }
            let mut face_shape = my_face.as_ref().clone();
            builder.add(&mut face_shape, &ow_shape.oriented(w.orientation()))?;
        }
        my_face.set_orientation(face.orientation());
        update_face(&my_face)?;
        self.my_face = my_face;
        Ok(())
    }
}

/// The degenerated edges of the face whose ends meet on the surface (a true
/// singularity; a degenerated edge with distinct ends is not one).
fn singular_degenerated_edges(
    face: &Face,
    s: &Handle<dyn GeomSurface>,
) -> Result<Vec<Edge>, Failure> {
    let mut deg_edges = Vec::new();
    let mut explo = Explorer::new(face.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explo.more() {
        let an_edge = topods::edge(explo.current())?;
        explo.next()?;
        if !BRepTool::degenerated(&an_edge)? {
            continue;
        }
        let pc = BRepTool::curve_on_face(&an_edge, face)?;
        let c2d = live(&pc.pcurve, "pcurve")?;
        let (a_f, a_l) = (c2d.value(pc.first)?, c2d.value(pc.last)?);
        let surf = live(s, "surface")?;
        let a_fpnt = surf.value(a_f.x(), a_f.y())?;
        let a_lpnt = surf.value(a_l.x(), a_l.y())?;
        if a_fpnt.square_distance(&a_lpnt) < precision::square_confusion() {
            deg_edges.push(an_edge);
        }
    }
    Ok(deg_edges)
}

/// The side whose apex a degenerated-side treatment computes.
#[derive(Clone, Copy)]
enum ApexEnd {
    FirstU,
    LastU,
    FirstV,
    LastV,
}

/// One degenerated side of an offset surface: when the offset iso is not a point, the
/// surface extended to (outside) or trimmed at (inside) the nearest point of the
/// generatrix to the apex line; the apex point, `None` when the iso is a point.
fn side_apex(
    the_surf: &mut Handle<dyn GeomSurface>,
    side: &SideApex,
    (uf1, uf2, vf1, vf2): (f64, f64, f64, f64),
    offset_outside: bool,
    end: ApexEnd,
) -> Result<Option<Point3>, Failure> {
    let surf = live(the_surf, "surface")?;
    let iso = if side.iso_u {
        surf.u_iso(side.iso)?
    } else {
        surf.v_iso(side.iso)?
    };
    if BRepOffsetTool::gabarit(&iso)? <= TOL_APEX {
        return Ok(None);
    }
    let basis_surf = live(
        &the_surf.clone().cast::<GeomOffsetSurface>(),
        "offset surface",
    )?
    .basis_surface();
    let basis = live(&basis_surf, "surface")?;
    let papex = basis.value(side.apex.0, side.apex.1)?;
    let pfirst = surf.value(side.apex.0, side.apex.1)?;
    let pquart = surf.value(side.quart.0, side.quart.1)?;
    let pmid = surf.value(side.mid.0, side.mid.1)?;
    let dir_apex = Vector3::between(pfirst, pquart).crossed(Vector3::between(pfirst, pmid));
    let line_apex: Handle<dyn rustycad_tkg3d::geom::GeomCurve> = Handle::new(GeomLine::from_point(
        papex,
        Direction3::from_vector(dir_apex)?,
    ))
    .cast();
    let dir_generatrix = basis.dn(side.apex.0, side.apex.1, side.dn.0, side.dn.1)?;
    let line_generatrix: Handle<dyn rustycad_tkg3d::geom::GeomCurve> = Handle::new(
        GeomLine::from_point(pfirst, Direction3::from_vector(dir_generatrix)?),
    )
    .cast();
    let the_extrema = ExtremaCurveCurve::with_curves(&line_generatrix, &line_apex)?;
    let (pint1, _) = the_extrema.nearest_points()?;
    let length = pfirst.distance(&pint1);
    if offset_outside {
        let mut a_surf: Handle<dyn GeomBoundedSurface> = Handle::new(
            GeomRectangularTrimmedSurface::new(the_surf, (uf1, uf2), (vf1, vf2), (true, true))?,
        )
        .cast();
        extend_surf_by_length(&mut a_surf, length, 1, side.in_u, side.after)?;
        *the_surf = a_surf.cast();
        let (u1, u2, v1, v2) = live(the_surf, "surface")?.bounds();
        let s = live(the_surf, "surface")?;
        return Ok(Some(match end {
            ApexEnd::FirstU => s.value(u1, vf1)?,
            ApexEnd::LastU => s.value(u2, vf1)?,
            ApexEnd::FirstV => s.value(uf1, v1)?,
            ApexEnd::LastV => s.value(uf1, v2)?,
        }));
    }
    let other_iso = if side.iso_u {
        surf.v_iso(vf1)?
    } else {
        surf.u_iso(uf1)?
    };
    let projector = ProjectPointOnCurve::with_curve(pint1, &other_iso)?;
    let new_par = projector.lower_distance_parameter()?;
    let (u, v) = match end {
        ApexEnd::FirstU => ((new_par, uf2), (vf1, vf2)),
        ApexEnd::LastU => ((uf1, new_par), (vf1, vf2)),
        ApexEnd::FirstV => ((uf1, uf2), (new_par, vf2)),
        ApexEnd::LastV => ((uf1, uf2), (vf1, new_par)),
    };
    *the_surf = trimmed(the_surf, u, v)?;
    let s = live(the_surf, "surface")?;
    Ok(Some(match end {
        ApexEnd::FirstU | ApexEnd::LastU => s.value(new_par, vf1)?,
        ApexEnd::FirstV | ApexEnd::LastV => s.value(uf1, new_par)?,
    }))
}

/// The state of the edge images of `Init(Face, ...)`.
struct FaceContext {
    cur_face: Face,
    my_face: Face,
    the_surf: Handle<dyn GeomSurface>,
    l: Location,
    is_transformed: bool,
    sing: Singularity,
    vf1: f64,
    bounds: (f64, f64, f64, f64),
}

type ShapeMap = DataMap<Shape, Shape, ShapeMapHasher>;

impl FaceContext {
    /// The vertex images known from the created edges and vertices (`MapSS`), and the
    /// vertices on degenerated edges of a face with a singularity (`VonDegen`).
    fn initial_images(&self, created: &CreatedShapes) -> Result<(ShapeMap, MapOfShape), Failure> {
        let mut map_ss = ShapeMap::default();
        let mut von_degen = MapOfShape::default();
        let mut exp = Explorer::new(self.cur_face.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
        while exp.more() {
            let e = topods::edge(exp.current())?;
            exp.next()?;
            let (v1, v2) = topexp::vertices(&e, false);
            if self.sing.has && BRepTool::degenerated(&e)? {
                von_degen.add(v1.as_ref().clone());
            }
            if let Some(oe) = created.seek(e.as_ref()) {
                let (ov1, ov2) = topexp::vertices(&topods::edge(oe)?, false);
                if map_ss.seek(v1.as_ref()).is_none() {
                    map_ss.bind(v1.as_ref().clone(), ov1.as_ref().clone());
                }
                if map_ss.seek(v2.as_ref()).is_none() {
                    map_ss.bind(v2.as_ref().clone(), ov2.as_ref().clone());
                }
            }
            for v in [&v1, &v2] {
                if let Some(cv) = created.seek(v.as_ref())
                    && map_ss.seek(v.as_ref()).is_none()
                {
                    map_ss.bind(v.as_ref().clone(), cv.clone());
                }
            }
        }
        Ok((map_ss, von_degen))
    }

    /// The point and parameter of a vertex image: the apex of a vertex on a
    /// degenerated edge, the offset surface at the p-curve end otherwise.
    fn vertex_point(&self, p2d: Point2, on_degen: bool) -> Result<(Point3, f64), Failure> {
        let (_, _, v1, v2) = self.bounds;
        if on_degen {
            if (p2d.y() - self.vf1).abs() <= precision::confusion() {
                return Ok((self.sing.min_apex, v1));
            }
            return Ok((self.sing.max_apex, v2));
        }
        let mut p = live(&self.the_surf, "surface")?.d0(p2d.x(), p2d.y())?;
        if !self.l.is_identity() && !self.is_transformed {
            p = p.transformed(self.l.transformation());
        }
        Ok((p, p2d.y()))
    }

    /// The image of an edge of the face, built (or completed for a seam or created
    /// edge) with its p-curves on the offset face.
    fn edge_image(
        &self,
        e: &Edge,
        created: &CreatedShapes,
        map_ss: &mut ShapeMap,
        von_degen: &MapOfShape,
    ) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let (v1, v2) = topexp::vertices(e, false);
        let on1 = von_degen.contains(v1.as_ref());
        let on2 = von_degen.contains(v2.as_ref());
        let pc = BRepTool::curve_on_face(e, &self.cur_face)?;
        let (mut c2d, mut f, mut l) = (pc.pcurve.clone(), pc.first, pc.last);
        let tol_e = BRepTool::edge_tolerance(e)?;
        if let (Some(oe_shape), false, false) = (map_ss.seek(e.as_ref()).cloned(), on1, on2) {
            // A seam edge: its second p-curve.
            let oe = topods::edge(&oe_shape)?;
            let pcr = BRepTool::curve_on_face(&topods::edge(&e.reversed())?, &self.cur_face)?;
            (f, l) = (pcr.first, pcr.last);
            if e.orientation() == Orientation::Forward {
                update_edge_pcurves(&oe, &c2d, &pcr.pcurve, &self.my_face, tol_e)?;
            } else {
                update_edge_pcurves(&oe, &pcr.pcurve, &c2d, &self.my_face, tol_e)?;
            }
            builder.range(&oe, f, l, false)?;
            return Ok(oe_shape);
        }
        let eforward = topods::edge(&e.oriented(Orientation::Forward))?;
        let c2 = live(&c2d, "pcurve")?;
        let mut p2d1 = c2.value(BRepTool::parameter_on_face(&v1, &eforward, &self.cur_face)?)?;
        let mut p2d2 = c2.value(BRepTool::parameter_on_face(&v2, &eforward, &self.cur_face)?)?;
        let (p1, vstart) = self.vertex_point(p2d1, on1)?;
        let (p2, vend) = self.vertex_point(p2d2, on2)?;
        // The image of E in the created shapes, or the seam, or a new edge.
        let oe = if let Some(c) = created.seek(e.as_ref()) {
            topods::edge(c)?
        } else if let Some(c) = map_ss.seek(e.as_ref()) {
            topods::edge(c)?
        } else {
            let mut oe = Edge::default();
            builder.make_edge(&mut oe)?;
            let ov1 = vertex_image(&v1, p1, map_ss)?;
            let ov2 = vertex_image(&v2, p2, map_ss)?;
            let mut oe_shape = oe.as_ref().clone();
            builder.add(&mut oe_shape, &ov1.oriented(v1.orientation()))?;
            builder.add(&mut oe_shape, &ov2.oriented(v2.orientation()))?;
            if BRepTool::degenerated(e)? {
                builder.degenerated(&oe, true)?;
            }
            oe
        };
        if on1 || on2 {
            if on1 {
                p2d1.set_y(vstart);
            }
            if on2 {
                p2d2.set_y(vend);
            }
            c2d = line_through(p2d1, p2d2)?;
            f = 0.;
            l = p2d1.distance(p2d2);
            if map_ss.seek(e.as_ref()).is_some() {
                // A seam edge.
                let c2d_1 = BRepTool::curve_on_face(&oe, &self.my_face)?;
                (f, l) = (c2d_1.first, c2d_1.last);
                if e.orientation() == Orientation::Forward {
                    update_edge_pcurves(&oe, &c2d, &c2d_1.pcurve, &self.my_face, tol_e)?;
                } else {
                    update_edge_pcurves(&oe, &c2d_1.pcurve, &c2d, &self.my_face, tol_e)?;
                }
            } else {
                update_edge_pcurve(&oe, &c2d, &self.my_face, tol_e)?;
            }
            builder.range_on_face(&oe, &self.my_face, f, l)?;
            if !BRepTool::degenerated(e)? && live(&self.the_surf, "surface")?.is_u_closed()? {
                let pcr = BRepTool::curve_on_face(&topods::edge(&e.reversed())?, &self.cur_face)?;
                let c2r = live(&pcr.pcurve, "pcurve")?;
                p2d1 = c2r.value(BRepTool::parameter_on_face(&v1, e, &self.cur_face)?)?;
                p2d2 = c2r.value(BRepTool::parameter_on_face(&v2, e, &self.cur_face)?)?;
                if on1 {
                    p2d1.set_y(vstart);
                }
                if on2 {
                    p2d2.set_y(vend);
                }
                let c2d_1 = line_through(p2d1, p2d2)?;
                if e.orientation() == Orientation::Forward {
                    update_edge_pcurves(&oe, &c2d, &c2d_1, &self.my_face, tol_e)?;
                } else {
                    update_edge_pcurves(&oe, &c2d_1, &c2d, &self.my_face, tol_e)?;
                }
            }
        } else {
            update_edge_pcurve(&oe, &c2d, &self.my_face, tol_e)?;
            builder.range(&oe, f, l, false)?;
        }
        if !BRepTool::degenerated(&oe)? {
            compute_curve3d(&oe, &c2d, &self.the_surf, &self.l, tol_e)?;
        }
        map_ss.bind(e.as_ref().clone(), oe.as_ref().clone());
        Ok(oe.as_ref().clone())
    }
}

/// The image of a vertex: the bound one, or a new vertex at the point with the
/// tolerance of the vertex (then bound).
fn vertex_image(v: &Vertex, p: Point3, map_ss: &mut ShapeMap) -> Result<Shape, Failure> {
    if let Some(ov) = map_ss.seek(v.as_ref()) {
        return Ok(ov.clone());
    }
    let builder = BRepBuilder::default();
    let mut ov = Vertex::default();
    builder.make_vertex(&mut ov);
    builder.update_vertex_point(&ov, p, BRepTool::vertex_tolerance(v)?)?;
    map_ss.bind(v.as_ref().clone(), ov.as_ref().clone());
    Ok(ov.as_ref().clone())
}

/// `new Geom2d_Line(P1, gp_Vec2d(P1, P2))`.
fn line_through(p1: Point2, p2: Point2) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let d = rustycad_tkmath::gp::Direction2::from_vector(Vector2::between(p1, p2))?;
    Ok(Handle::new(Geom2dLine::from_point(p1, d)).cast())
}

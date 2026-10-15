// Rust translation of the storage of the curves projected on several faces and of the
// tangent free borders in ChFi3d_Builder::PerformMoreThreeCorner
// (ChFi3d_Builder_CnCrn.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projections of the Hermite curves crossing several faces given to the plate and
//! stored with their points on the edges between the faces, and the free borders when
//! they are tangent at the vertex.
use super::plate_bounds::{add_constraint, curve_on};
use super::{Corner, indices, slot};
use crate::chfi3d::builder::ChFi3dBuilder;
use crate::chfi3d::fil_curve_in_ds;
use crate::chfi3d::fil_point_in_ds;
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSPoint};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_plate::GeomPlateBuildPlateSurface;
use rustycad_tkmath::gp::Point3;

/// The failure of a missing curve of a projected edge.
fn construction(what: &str) -> Failure {
    Failure::with_message("Standard_ConstructionError", what)
}

/// The `k`-th (from 1) element of a sequence, Standard_OutOfRange outside it.
fn nth<T: Clone>(v: &[T], k: usize) -> Result<T, Failure> {
    v.get(k.wrapping_sub(1))
        .cloned()
        .ok_or_else(|| Failure::out_of_range("PerformMoreThreeCorner: sequence"))
}

/// A pcurve and its 3d curve.
type CurvePair = (Handle<dyn Geom2dCurve>, Handle<dyn GeomCurve>);

/// The pcurve and the 3d curve of a projected edge on the face, trimmed to its range.
fn projected_curves(e: &Edge, f: &Face) -> Result<CurvePair, Failure> {
    let pc = BRepTool::curve_on_face(e, f)?;
    if pc.pcurve.is_null() {
        return Err(construction("Failed to get p-curve of edge"));
    }
    let proj2d: Handle<dyn Geom2dCurve> = Handle::new(Geom2dTrimmedCurve::new(
        &pc.pcurve, pc.first, pc.last, true, true,
    )?)
    .cast();
    let (projc, up1, up2) = BRepTool::curve(e)?;
    if projc.is_null() {
        return Err(construction("Failed to get 3D curve of edge"));
    }
    Ok((
        proj2d,
        Handle::new(GeomTrimmedCurve::new(&projc, up1, up2, true, true)?).cast(),
    ))
}

/// The orientation of the edge in the face taken FORWARD (`oredge` unchanged when it is
/// not there).
fn edge_in_face(f: &Face, e: &Edge, oredge: &mut Orientation) -> Result<(), Failure> {
    let mut ex = Explorer::new(
        &f.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while ex.more() {
        if e.is_same(ex.current()) {
            *oredge = ex.current().orientation();
            return Ok(());
        }
        ex.next()?;
    }
    Ok(())
}

/// The running state of the storage of one projected connection: the last point made
/// (`ind`) and the orientation of the last common edge in its face (`oredge`).
struct Run {
    ind: i32,
    oredge: Orientation,
}

impl ChFi3dBuilder {
    /// The curves projected on several faces given to the plate and stored.
    pub(super) fn store_projected(
        &mut self,
        c: &mut Corner,
        psurf: &mut GeomPlateBuildPlateSurface,
    ) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if !c.moresurf[ic] {
                continue;
            }
            let (icplus, _) = indices(c.nedge, ic);
            let ip = [c.indpoint[ic][1], c.indpoint[icplus][0]];
            let t1 = self.ds.change_point(ip[0])?.tolerance() + c.error;
            self.ds.change_point(ip[0])?.set_tolerance(t1);
            // The second tolerance is made from the first one, as natively.
            let t2 = self.ds.change_point(ip[0])?.tolerance() + c.error;
            self.ds.change_point(ip[1])?.set_tolerance(t2);
            let mut run = Run {
                ind: 0,
                oredge: Orientation::Forward,
            };
            for nb in 1..=c.nbface.max(0) as usize {
                self.store_projection(c, (ic, icplus, nb), &mut run, psurf)?;
            }
        }
        Ok(())
    }

    /// The projection on the `nb`-th face, and the point on the common edge after it.
    fn store_projection(
        &mut self,
        c: &mut Corner,
        (ic, icplus, nb): (usize, usize, usize),
        run: &mut Run,
        psurf: &mut GeomPlateBuildPlateSurface,
    ) -> Result<(), Failure> {
        let ecom = nth(&c.ecom, nb)?;
        let sommet = BRepTool::pnt(&c.v1)?;
        let pf = BRepTool::pnt(&topexp::first_vertex(&ecom, false))?;
        let pl = BRepTool::pnt(&topexp::last_vertex(&ecom, false))?;
        let para = nth(&c.parcom, nb)?;
        let (ccom, _, _) = BRepTool::curve(&ecom)?;
        let pcom = ccom
            .get()
            .ok_or_else(|| construction("Failed to get 3D curve of edge"))?
            .value(para)?;
        let orvt = if pf.distance(&sommet) < pl.distance(&sommet) {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        let eproj = nth(&c.eproj, nb)?;
        if !eproj.is_null() {
            self.store_projected_curve(
                c,
                (ic, icplus, nb),
                (&eproj, &ecom, pcom, orvt),
                run,
                psurf,
            )?;
        }
        if nb == c.nbface as usize {
            return Ok(());
        }
        let mut indice = run.ind;
        if eproj.is_null() {
            indice = c.indpoint[ic][1];
        }
        if nth(&c.eproj, nb + 1)?.is_null() {
            indice = c.indpoint[icplus][0];
        }
        *slot(&mut c.indice, c.n3d)? = indice;
        let iarc1 = self.ds.add_shape(ecom.as_ref());
        let interf = fil_point_in_ds(orvt, iarc1, indice, para, false);
        self.ds
            .change_shape_interferences_index(iarc1)?
            .append(interf);
        Ok(())
    }

    /// The projected edge on the `nb`-th face: a G1 constraint of the plate, its curve,
    /// its points (new ones away from the ends of the connection) and its pcurve on the
    /// face.
    fn store_projected_curve(
        &mut self,
        c: &mut Corner,
        (ic, icplus, nb): (usize, usize, usize),
        (eproj, ecom, pcom, orvt): (&Edge, &Edge, Point3, Orientation),
        run: &mut Run,
        psurf: &mut GeomPlateBuildPlateSurface,
    ) -> Result<(), Failure> {
        c.n3d += 1;
        let fproj = nth(&c.fproj, nb)?;
        let (proj2d, cproj) = projected_curves(eproj, &fproj)?;
        let curve = cproj
            .get()
            .ok_or_else(|| construction("Failed to get 3D curve of edge"))?;
        let (pardeb, parfin) = (curve.first_parameter(), curve.last_parameter());
        let (p1, p2) = (curve.value(pardeb)?, curve.value(parfin)?);
        let ip1 = if p1.distance(&self.ds.change_point(c.indpoint[ic][1])?.point()) < 1.0e-3 {
            c.indpoint[ic][1]
        } else {
            run.ind
        };
        let ip2 = if p2.distance(&self.ds.change_point(c.indpoint[icplus][0])?.point()) < 1.0e-3 {
            c.indpoint[icplus][0]
        } else {
            run.ind = self.ds.add_point(&TopOpeBRepDSPoint::new(p2, c.error));
            run.ind
        };
        let asurf = Handle::new(GeomAdaptorSurface::with_surface(&BRepTool::surface(
            &fproj,
        )?)?);
        let hcons = curve_on(&proj2d, &asurf)?;
        *slot(&mut c.order, c.n3d)? = 1;
        add_constraint(psurf, &hcons, (1, 10), (self.tolapp3d, self.angular))?;
        let icurve = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(&cproj, c.error, false));
        *slot(&mut c.indcurve3d, c.n3d)? = icurve;
        let i1 = fil_point_in_ds(Orientation::Forward, icurve, ip1, pardeb, false);
        let i2 = fil_point_in_ds(Orientation::Reversed, icurve, ip2, parfin, false);
        self.ds.change_curve_interferences(icurve).append(i1);
        self.ds.change_curve_interferences(icurve).append(i2);
        let num = self.ds.add_shape(fproj.as_ref());
        edge_in_face(&fproj, ecom, &mut run.oredge)?;
        let along = (p1.distance(&pcom) > 1.0e-4) == (orvt == Orientation::Forward);
        let orinterf = if along {
            run.oredge
        } else {
            top_abs::reverse(run.oredge)
        };
        let interfc = fil_curve_in_ds(icurve, num, &proj2d, orinterf);
        self.ds
            .change_shape_interferences_index(num)?
            .append(interfc);
        Ok(())
    }

    /// The tangent free borders: from the vertex to their parameter, G0 constraints of the
    /// plate stored with the vertex.
    pub(super) fn free_borders(
        &mut self,
        c: &mut Corner,
        psurf: &mut GeomPlateBuildPlateSurface,
    ) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if !c.is_libre_edge(ic) {
                continue;
            }
            c.n3d += 1;
            let (icplus, icmoins) = indices(c.nedge, ic);
            let e = c.evive[ic].clone();
            let face = c.fvive[ic][icplus].clone();
            let curve2d = BRepTool::curve_on_face(&e, &face)?.pcurve;
            let (curve, ufirst, ulast) = BRepTool::curve(&e)?;
            let from_v1 = topexp::first_vertex(&e, false).is_same(&c.v1);
            let (a, b) = if from_v1 {
                (ufirst, c.p[ic][icmoins])
            } else {
                (c.p[ic][icmoins], ulast)
            };
            let ctrim: Handle<dyn GeomCurve> =
                Handle::new(GeomTrimmedCurve::new(&curve, a, b, true, true)?).cast();
            let ctrim2d: Handle<dyn Geom2dCurve> =
                Handle::new(Geom2dTrimmedCurve::new(&curve2d, a, b, true, true)?).cast();
            let ivtx = self.ds.add_shape(c.v1.as_ref());
            let mut ends = if from_v1 {
                ([ivtx, c.indpoint[ic][1]], [true, false])
            } else {
                ([c.indpoint[ic][1], ivtx], [false, true])
            };
            let reverse = if c.libre[ic] {
                from_v1
            } else {
                topexp::last_vertex(&e, false).is_same(&c.v1)
            };
            if reverse {
                ctrim
                    .get()
                    .ok_or_else(|| construction("free border"))?
                    .reverse()?;
                ctrim2d
                    .get()
                    .ok_or_else(|| construction("free border"))?
                    .reverse()?;
                ends = if c.libre[ic] {
                    ([c.indpoint[ic][1], ivtx], [false, true])
                } else {
                    ([ivtx, c.indpoint[ic][1]], [true, false])
                };
            }
            self.store_free_border(c, (&ctrim, &ctrim2d, &face), ends, psurf)?;
        }
        Ok(())
    }

    /// A free border given to the plate and stored with its ends.
    fn store_free_border(
        &mut self,
        c: &mut Corner,
        (ctrim, ctrim2d, face): (&Handle<dyn GeomCurve>, &Handle<dyn Geom2dCurve>, &Face),
        (ip, isvt): ([i32; 2], [bool; 2]),
        psurf: &mut GeomPlateBuildPlateSurface,
    ) -> Result<(), Failure> {
        let curve = ctrim.get().ok_or_else(|| construction("free border"))?;
        let (ufirst, ulast) = (curve.first_parameter(), curve.last_parameter());
        let asurf = Handle::new(GeomAdaptorSurface::with_surface(&BRepTool::surface(face)?)?);
        let hcons = curve_on(ctrim2d, &asurf)?;
        *slot(&mut c.order, c.n3d)? = 0;
        add_constraint(psurf, &hcons, (0, 10), (self.tolapp3d, self.angular))?;
        let icurve = self
            .ds
            .add_curve(&mut TopOpeBRepDSCurve::new(ctrim, 1.0e-4, false));
        *slot(&mut c.indcurve3d, c.n3d)? = icurve;
        let i1 = fil_point_in_ds(Orientation::Forward, icurve, ip[0], ufirst, isvt[0]);
        let i2 = fil_point_in_ds(Orientation::Reversed, icurve, ip[1], ulast, isvt[1]);
        self.ds.change_curve_interferences(icurve).append(i1);
        self.ds.change_curve_interferences(icurve).append(i2);
        Ok(())
    }
}

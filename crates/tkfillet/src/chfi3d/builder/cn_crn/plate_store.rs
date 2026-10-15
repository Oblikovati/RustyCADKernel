// Rust translation of the approximation and the storage of the plate surface in
// ChFi3d_Builder::PerformMoreThreeCorner (ChFi3d_Builder_CnCrn.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The plate approximated by a B-spline surface and stored: its interference with the
//! solid, the orientation of its pcurves, the vertex removed from the sharp edges of two
//! tangent fillets, and the edges of the plate between the free borders.
use super::statics::{parametre_plate, plate_orientation};
use super::{Corner, indices};
use crate::chfi_ds::{CommonPoint, lock};
use crate::chfi3d::builder::{ChFi3dBuilder, spine_of};
use crate::chfi3d::{
    Arete, compute_arete, fil_curve_in_ds, fil_point_in_ds, fil_vertex_in_ds, surf_data_at,
};
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSCurve, TopOpeBRepDSKind, TopOpeBRepDSSolidSurfaceInterference, TopOpeBRepDSSurface,
    TopOpeBRepDSTransition,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_plate::{
    GeomPlateBuildPlateSurface, GeomPlateMakeApprox, GeomPlatePlateG0Criterion,
};
use rustycad_tkgeombase::adv_app2var::{AdvApp2VarCriterionRepartition, AdvApp2VarCriterionType};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Xy, Xyz};

/// The stored plate: its index and surface, the approximation error (`apperror`) and
/// the orientation of its pcurves (`orien`).
pub(super) struct Plate {
    pub isurf: i32,
    pub surf: Handle<dyn GeomSurface>,
    pub apperror: f64,
    pub orien: Orientation,
}

/// The approximation of the plate: the B-spline surface, its approximation error and
/// its criterion error.
fn approximate(
    psurf: &mut GeomPlateBuildPlateSurface,
) -> Result<(Handle<dyn GeomSurface>, f64, f64), Failure> {
    let gp_plate = psurf.surface();
    let (mut s2d, mut s3d) = (Sequence::<Xy>::default(), Sequence::<Xyz>::default());
    psurf.disc2d_contour(4, &mut s2d)?;
    psurf.disc3d_contour(4, 0, &mut s3d)?;
    let seuil = f64::max(1.0e-4, 10.0 * psurf.g0_error());
    let critere = GeomPlatePlateG0Criterion::new(
        &s2d,
        &s3d,
        seuil,
        AdvApp2VarCriterionType::Absolute,
        AdvApp2VarCriterionRepartition::Regular,
    );
    let mapp = GeomPlateMakeApprox::with_criterion(
        &gp_plate,
        &critere,
        1.0e-4,
        9,
        8,
        Continuity::C1,
        1.1,
    )?;
    Ok((
        mapp.surface().cast(),
        mapp.approx_error(),
        mapp.criterion_error(),
    ))
}

impl ChFi3dBuilder {
    /// The plate approximated and stored with all its curves.
    pub(super) fn store_plate(
        &mut self,
        c: &mut Corner,
        psurf: &mut GeomPlateBuildPlateSurface,
    ) -> Result<(), Failure> {
        let (surf, approx_error, criterion_error) = approximate(psurf)?;
        let isurf = self
            .ds
            .add_surface(&TopOpeBRepDSSurface::new(&surf, approx_error));
        let key = c.v1.as_ref().clone();
        if !self.my_evi_map.is_bound(&key) {
            self.my_evi_map.bind(key.clone(), Vec::new());
        }
        self.my_evi_map.change_find(&key)?.push(isurf);
        let sol_ind = lock(c.cd(0)?).solid_index();
        if c.deuxconges {
            self.vertex_off_spines(c)?;
        }
        let orplate = plate_orientation(&surf, &psurf.curves2d()?, &c.sum_normal)?;
        let ssi = TopOpeBRepDSSolidSurfaceInterference::new(
            &TopOpeBRepDSTransition::from_orientation(orplate),
            TopOpeBRepDSKind::Solid,
            sol_ind,
            TopOpeBRepDSKind::Surface,
            isurf,
        );
        self.ds
            .change_shape_interferences_index(sol_ind)?
            .append(Handle::new(ssi).cast());
        let orien = self.pcurve_orientation(c, orplate)?;
        let plate = Plate {
            isurf,
            surf,
            apperror: criterion_error * 1.1,
            orien,
        };
        if !c.droit {
            self.free_border_edges(c, psurf, &plate)?;
        }
        self.store_plate_curves(c, psurf, &plate)
    }

    /// With two tangent fillets the vertex is put on the end edges of their spines with
    /// the reversed orientation, so that the sharp edges reaching it are removed.
    fn vertex_off_spines(&mut self, c: &Corner) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if c.sharp[ic] {
                continue;
            }
            let sp = spine_of(c.cd(ic)?)?;
            let arcspine = {
                let g = lock(&sp);
                g.edges(if c.sens[ic] == 1 { 1 } else { g.nb_edges() })?
                    .clone()
            };
            let iarcspine = self.ds.add_shape(arcspine.as_ref());
            let mut ovtx = Orientation::Forward;
            let mut ex = Explorer::new(
                &arcspine.oriented(Orientation::Forward),
                ShapeEnum::Vertex,
                ShapeEnum::Shape,
            )?;
            while ex.more() {
                if c.v1.is_same(ex.current()) {
                    ovtx = ex.current().orientation();
                    break;
                }
                ex.next()?;
            }
            let par_vtx = BRepTool::parameter(&c.v1, &arcspine)?;
            let interfv = fil_vertex_in_ds(top_abs::reverse(ovtx), iarcspine, c.ivtx, par_vtx);
            self.ds
                .change_shape_interferences_index(iarcspine)?
                .append(interfv);
        }
        Ok(())
    }

    /// The orientation of the pcurves of the plate, from the surface data at the end of
    /// the first contour and the orientation of the plate.
    fn pcurve_orientation(&self, c: &Corner, orplate: Orientation) -> Result<Orientation, Failure> {
        let (_, icmoins) = indices(c.nedge, 0);
        let isfirst = c.sens[0] == 1;
        let fd = surf_data_at(c.cd(0)?, c.i[0][icmoins])?;
        let g = lock(&fd);
        let (ishape1, ishape2) = (g.index_of_s1(), g.index_of_s2());
        let orsurfdata = g.orientation();
        let trafil1 = if ishape1 != 0 {
            let mut t = if ishape1 > 0 {
                self.ds.shape(ishape1, true).orientation()
            } else {
                Orientation::Forward
            };
            t = top_abs::compose(t, g.orientation());
            top_abs::compose(top_abs::reverse(g.interference_on_s1().transition()), t)
        } else {
            let mut t = if ishape2 > 0 {
                self.ds.shape(ishape2, true).orientation()
            } else {
                Orientation::Forward
            };
            t = top_abs::compose(t, g.orientation());
            top_abs::reverse(top_abs::compose(
                top_abs::reverse(g.interference_on_s2().transition()),
                t,
            ))
        };
        drop(g);
        let st = lock(c.cd(0)?);
        let orpcurve = if isfirst {
            top_abs::compose(top_abs::reverse(trafil1), st.first_pcurve_orientation())
        } else {
            top_abs::compose(trafil1, st.last_pcurve_orientation())
        };
        Ok(if orsurfdata == orplate {
            top_abs::reverse(orpcurve)
        } else {
            orpcurve
        })
    }

    /// Between a free border and the next edge, the edge of the plate joining their
    /// ends.
    fn free_border_edges(
        &mut self,
        c: &Corner,
        psurf: &GeomPlateBuildPlateSurface,
        plate: &Plate,
    ) -> Result<(), Failure> {
        for ic in 0..=c.nedge {
            if !c.libre[ic] {
                continue;
            }
            let (icplus, _) = indices(c.nedge, ic);
            let (par1, par2) = (c.p[ic][icplus], c.p[icplus][ic]);
            let ptic = BRepAdaptorCurve::with_edge(&c.evive[ic])?.value(par1)?;
            let pticplus = BRepAdaptorCurve::with_edge(&c.evive[icplus])?.value(par2)?;
            let uv1 = parametre_plate(c.n3d, psurf, &plate.surf, ptic, plate.apperror)?;
            let uv2 = parametre_plate(c.n3d, psurf, &plate.surf, pticplus, plate.apperror)?;
            let (mut cp1, mut cp2) = (CommonPoint::new(), CommonPoint::new());
            cp1.set_arc(1.0e-3, &c.evive[ic], par1, Orientation::Forward);
            cp1.set_point(ptic);
            cp2.set_arc(1.0e-3, &c.evive[icplus], par2, Orientation::Forward);
            cp2.set_point(pticplus);
            let mut arete = Arete::default();
            compute_arete(
                (&cp1, uv1),
                (&cp2, uv2),
                &plate.surf,
                &mut arete,
                (1.0e-3, 1.0e-6),
                0,
            )?;
            let indcurv = self.ds.add_curve(&mut TopOpeBRepDSCurve::new(
                &arete.c3d,
                arete.tolreached,
                false,
            ));
            let i1 = fil_point_in_ds(
                Orientation::Forward,
                indcurv,
                c.indpoint[ic][0],
                arete.pardeb,
                false,
            );
            let i2 = fil_point_in_ds(
                Orientation::Reversed,
                indcurv,
                c.indpoint[icplus][0],
                arete.parfin,
                false,
            );
            self.ds.change_curve_interferences(indcurv).append(i1);
            self.ds.change_curve_interferences(indcurv).append(i2);
            let interfc = fil_curve_in_ds(indcurv, plate.isurf, &arete.pcurv, plate.orien);
            self.ds
                .change_surface_interferences(plate.isurf)
                .append(interfc);
        }
        Ok(())
    }
}

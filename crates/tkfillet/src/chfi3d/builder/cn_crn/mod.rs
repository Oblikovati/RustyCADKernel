// Rust translation of ChFi3d_Builder::PerformMoreThreeCorner and its file statics
// (ChFi3d_Builder_CnCrn.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of a vertex with more than three edges, or of contours that do not meet: the
//! ends of the fillets and the sharp edges at the vertex are joined by curves (the
//! intersections of neighbouring fillets, straight lines, battens or projections) and the
//! hole they bound is filled by a `GeomPlate` surface.
//!
//! The arrays of the native function are the fields of [`Corner`], indexed from 0 like
//! the native ones; the steps of the function are the methods of the files of this module,
//! called in the native order by [`ChFi3dBuilder::perform_more_three_corner`].
mod batten;
mod connections;
mod hermite;
mod inters;
mod params;
mod parse;
mod plate_bounds;
mod plate_store;
mod plate_store_curves;
mod projected;
mod projections;
mod regul;
mod same_ext;
mod statics;
#[cfg(test)]
mod tests;

use super::ChFi3dBuilder;
use crate::chfi_ds::StripeHandle;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeomalgo::geom_plate::GeomPlateBuildPlateSurface;
use rustycad_tkmath::gp::Vector3;

/// `Indices(n, ic, icplus, icmoins)`: the next and previous indices around the vertex.
///
/// ```text
/// assert_eq!(indices(4, 3), (0, 2));
/// ```
pub(super) fn indices(n: usize, ic: usize) -> (usize, usize) {
    let icplus = if ic == n - 1 { 0 } else { ic + 1 };
    let icmoins = if ic == 0 { n - 1 } else { ic - 1 };
    (icplus, icmoins)
}

/// The slot `k` of an array indexed by the number of plate boundaries (`indcurve3d`,
/// `Order`, `Indice`), Standard_OutOfRange outside it as natively.
pub(super) fn slot(v: &mut [i32], k: i32) -> Result<&mut i32, Failure> {
    usize::try_from(k)
        .ok()
        .and_then(|k| v.get_mut(k))
        .ok_or_else(|| Failure::out_of_range("PerformMoreThreeCorner: plate boundary index"))
}

/// The state of `PerformMoreThreeCorner`: its arrays (sized `2 nedge + 1`) and the
/// variables shared by its steps.
pub(super) struct Corner {
    pub v1: Vertex,
    /// The index of the vertex in the map of the contours at the vertices.
    pub jndex: i32,
    pub nconges: i32,
    pub nedge: usize,
    /// `edgelibre1`, `edgelibre2`: the free borders at the vertex.
    pub edgelibre: [Edge; 2],
    pub droit: bool,
    pub cd: Vec<Option<StripeHandle>>,
    pub jf: Vec<i32>,
    pub index: Vec<i32>,
    pub indice: Vec<i32>,
    pub sens: Vec<i32>,
    pub indcurve3d: Vec<i32>,
    pub numfa: Vec<Vec<i32>>,
    pub order: Vec<i32>,
    pub i: Vec<Vec<i32>>,
    pub indpoint: Vec<[i32; 2]>,
    pub oksea: Vec<bool>,
    pub sharp: Vec<bool>,
    pub regul: Vec<bool>,
    pub p: Vec<Vec<f64>>,
    pub errapp: Vec<f64>,
    pub evive: Vec<Edge>,
    pub fvive: Vec<Vec<Face>>,
    pub ponctuel: Vec<bool>,
    pub samedge: Vec<bool>,
    pub moresurf: Vec<bool>,
    pub libre: Vec<bool>,
    pub tangentregul: Vec<bool>,
    pub is_g1: Vec<bool>,
    pub couture: bool,
    pub facecouture: Face,
    pub edgecouture: Edge,
    pub fproj: Vec<Face>,
    /// `SumFaceNormalAtV1`: the sum of the normals of the faces at the vertex.
    pub sum_normal: Vector3,
    pub deuxconges: bool,
    pub deuxcgnontg: bool,
    pub on_same_diff: bool,
    pub inters: bool,
    pub distmini: bool,
    pub distmin: f64,
    pub tolcp: f64,
    pub nbface: i32,
    pub error: f64,
    pub proj2d1: Vec<Handle<dyn Geom2dCurve>>,
    pub proj2d2: Vec<Handle<dyn Geom2dCurve>>,
    pub cproj1: Vec<Handle<dyn GeomCurve>>,
    pub cproj2: Vec<Handle<dyn GeomCurve>>,
    pub ecom: Vec<Edge>,
    pub eproj: Vec<Edge>,
    pub parcom: Vec<f64>,
    pub n3d: i32,
    /// `Curv3d`: the last 3d curve built from a pcurve (kept when an approximation
    /// fails, as natively).
    pub curv3d: Handle<dyn GeomCurve>,
    pub ivtx: i32,
    pub maxapp: f64,
    pub maxapp1: f64,
}

impl Corner {
    /// The arrays of a vertex of `nedge` edges, initialized as natively (the integer and
    /// real arrays the native code leaves uninitialized are zero).
    fn new(
        (v1, jndex): (Vertex, i32),
        nconges: i32,
        nedge: usize,
        edgelibre: [Edge; 2],
        droit: bool,
    ) -> Self {
        let n = 2 * nedge + 1;
        Self {
            v1,
            jndex,
            nconges,
            nedge,
            edgelibre,
            droit,
            cd: vec![None; n],
            jf: vec![0; n],
            index: vec![0; n],
            indice: vec![0; n],
            sens: vec![0; n],
            indcurve3d: vec![0; n],
            numfa: vec![vec![0; n]; n],
            order: vec![0; n],
            i: vec![vec![0; n]; n],
            indpoint: vec![[0, 0]; n],
            oksea: vec![false; n],
            sharp: vec![false; n],
            regul: vec![false; n],
            p: vec![vec![0.0; n]; n],
            errapp: vec![0.0; n],
            evive: vec![Edge::default(); n],
            fvive: vec![vec![Face::default(); n]; n],
            ponctuel: vec![false; n],
            samedge: vec![false; n],
            moresurf: vec![false; n],
            libre: vec![false; n],
            tangentregul: vec![false; n],
            is_g1: vec![false; n],
            couture: false,
            facecouture: Face::default(),
            edgecouture: Edge::default(),
            fproj: Vec::new(),
            sum_normal: Vector3::new(0.0, 0.0, 0.0),
            deuxconges: false,
            deuxcgnontg: false,
            on_same_diff: false,
            inters: true,
            distmini: false,
            distmin: 1.0e30,
            tolcp: 0.0,
            nbface: 0,
            error: 0.0,
            proj2d1: vec![Handle::default(); n],
            proj2d2: vec![Handle::default(); n],
            cproj1: vec![Handle::default(); n],
            cproj2: vec![Handle::default(); n],
            ecom: Vec::new(),
            eproj: Vec::new(),
            parcom: Vec::new(),
            n3d: 0,
            curv3d: Handle::default(),
            ivtx: 0,
            maxapp: 0.0,
            maxapp1: 0.0,
        }
    }

    /// `CD(ic)`: the contour of the `ic`-th edge (Standard_NullObject on a sharp edge).
    pub fn cd(&self, ic: usize) -> Result<&StripeHandle, Failure> {
        self.cd[ic].as_ref().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "PerformMoreThreeCorner: no contour")
        })
    }

    /// `Evive(ic)` is a free border.
    pub fn is_libre_edge(&self, ic: usize) -> bool {
        self.evive[ic].is_same(&self.edgelibre[0]) || self.evive[ic].is_same(&self.edgelibre[1])
    }
}

impl ChFi3dBuilder {
    /// `PerformMoreThreeCorner(Jndex, nconges)`: the corner of the `jndex`-th vertex,
    /// where `nconges` contours end, filled by a plate surface; `done` false with a partial
    /// result when the plate fails.
    pub(crate) fn perform_more_three_corner(
        &mut self,
        jndex: i32,
        nconges: i32,
    ) -> Result<(), Failure> {
        let mut c = self.corner_init(jndex, nconges)?;
        self.parse_edges(&mut c, jndex)?;
        self.update_regul(&mut c)?;
        self.find_evive_anew(&mut c)?;
        self.limits(&mut c)?;
        self.intersections(&mut c)?;
        if !c.inters {
            self.params_without_inters(&mut c)?;
        }
        self.limit_by_common_points(&mut c)?;
        self.free_border_params(&mut c)?;
        self.regul_projections(&mut c)?;
        self.more_surf_projections(&mut c)?;
        if self.two_fillets_by_intersection(&mut c)? {
            return Ok(());
        }
        self.fill_plate(&mut c)
    }

    /// The plate of the corner: its boundaries, then the surface and its storage.
    fn fill_plate(&mut self, c: &mut Corner) -> Result<(), Failure> {
        let mut psurf = GeomPlateBuildPlateSurface::new(
            3,
            10,
            1,
            self.tol2d,
            self.tolapp3d,
            self.angular,
            0.1,
            false,
        )?;
        self.stripe_bounds(c, &mut psurf)?;
        self.sharp_points(c)?;
        if !c.inters {
            self.connections(c, &mut psurf)?;
        }
        self.store_projected(c, &mut psurf)?;
        if c.droit {
            self.free_borders(c, &mut psurf)?;
        }
        psurf.perform(&Default::default())?;
        if psurf.is_done() {
            self.store_plate(c, &mut psurf)
        } else {
            self.partial_result(c)
        }
    }
}

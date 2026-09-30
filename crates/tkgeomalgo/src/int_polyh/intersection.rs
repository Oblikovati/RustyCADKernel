// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_Intersection.hxx and
// IntPolyh_Intersection.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPolyh_Intersection`: the intersection of two surfaces by their triangulations.
use super::array::{
    IntPolyhArrayOfPointNormal, IntPolyhArrayOfSectionLines, IntPolyhArrayOfTangentZones,
};
use super::edge::IntPolyhCouple;
use super::maillage::IntPolyhMaillageAffinage;
use super::start_point::IntPolyhStartPoint;
use super::tools::IntPolyhTools;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use std::collections::HashSet;

/// The cosine of 5 degrees: interfering triangles more parallel than that are almost
/// tangent.
const COS_5_DEG: f64 = 0.996;

/// The sampling of both surfaces: `(UPars1, VPars1)` and `(UPars2, VPars2)`.
type Samplings<'a> = [(&'a Array1<'a, f64>, &'a Array1<'a, f64>); 2];

/// `IntPolyh_Intersection`: the section lines and tangent zones of the intersection of
/// two surfaces, computed by the constructors.
///
/// ```ignore
/// let inter = IntPolyhIntersection::new(&s1, &s2)?;
/// assert!(inter.is_done());
/// ```
pub struct IntPolyhIntersection {
    surf1: Adaptor3dSurfaceHandle,
    surf2: Adaptor3dSurfaceHandle,
    nb_su1: i32,
    nb_sv1: i32,
    nb_su2: i32,
    nb_sv2: i32,
    is_done: bool,
    section_lines: IntPolyhArrayOfSectionLines,
    tangent_zones: IntPolyhArrayOfTangentZones,
    is_parallel: bool,
}

impl IntPolyhIntersection {
    fn unperformed(
        s1: &Adaptor3dSurfaceHandle,
        s2: &Adaptor3dSurfaceHandle,
        samples: [i32; 4],
    ) -> Self {
        let mut inter = Self {
            surf1: s1.clone(),
            surf2: s2.clone(),
            nb_su1: samples[0],
            nb_sv1: samples[1],
            nb_su2: samples[2],
            nb_sv2: samples[3],
            is_done: false,
            section_lines: IntPolyhArrayOfSectionLines::default(),
            tangent_zones: IntPolyhArrayOfTangentZones::default(),
            is_parallel: false,
        };
        inter.section_lines.init(1000);
        inter.tangent_zones.init(10000);
        inter
    }

    /// `IntPolyh_Intersection(theS1, theS2)`: 10 x 10 samples on each surface.
    /// `IntPolyhIntersection::new(&s1, &s2)?`.
    pub fn new(s1: &Adaptor3dSurfaceHandle, s2: &Adaptor3dSurfaceHandle) -> Result<Self, Failure> {
        let mut inter = Self::unperformed(s1, s2, [10; 4]);
        inter.perform()?;
        Ok(inter)
    }

    /// `IntPolyh_Intersection(theS1, theNbSU1, theNbSV1, theS2, theNbSU2, theNbSV2)`.
    /// `IntPolyhIntersection::with_samples(&s1, (10, 10), &s2, (12, 8))?`.
    pub fn with_samples(
        s1: &Adaptor3dSurfaceHandle,
        (nb_su1, nb_sv1): (i32, i32),
        s2: &Adaptor3dSurfaceHandle,
        (nb_su2, nb_sv2): (i32, i32),
    ) -> Result<Self, Failure> {
        let mut inter = Self::unperformed(s1, s2, [nb_su1, nb_sv1, nb_su2, nb_sv2]);
        inter.perform()?;
        Ok(inter)
    }

    /// `IntPolyh_Intersection(theS1, theUPars1, theVPars1, theS2, theUPars2, theVPars2)`:
    /// the given sampling. `IntPolyhIntersection::with_parameters(&s1, (&u1, &v1), &s2, (&u2, &v2))?`.
    pub fn with_parameters(
        s1: &Adaptor3dSurfaceHandle,
        (u_pars1, v_pars1): (&Array1<'_, f64>, &Array1<'_, f64>),
        s2: &Adaptor3dSurfaceHandle,
        (u_pars2, v_pars2): (&Array1<'_, f64>, &Array1<'_, f64>),
    ) -> Result<Self, Failure> {
        let samples = [
            u_pars1.length(),
            v_pars1.length(),
            u_pars2.length(),
            v_pars2.length(),
        ];
        let mut inter = Self::unperformed(s1, s2, samples);
        inter.perform_on([(u_pars1, v_pars1), (u_pars2, v_pars2)])?;
        Ok(inter)
    }

    /// `IsDone()`. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }

    /// `IsParallel()`: too many almost parallel interfering triangles. `inter.is_parallel()`.
    pub fn is_parallel(&self) -> bool {
        self.is_parallel
    }

    /// `NbSectionLines()`. `inter.nb_section_lines()`.
    pub fn nb_section_lines(&self) -> i32 {
        self.section_lines.nb_items()
    }

    /// `NbPointsInLine(IndexLine)`: the points of the line `1..=NbSectionLines()`; past
    /// the lines the allocated empty lines answer -1 (natively too), past the storage
    /// Standard_OutOfRange (undefined natively). `inter.nb_points_in_line(1)?`.
    pub fn nb_points_in_line(&self, index_line: i32) -> Result<i32, Failure> {
        Ok(self
            .section_lines
            .value(index_line.wrapping_sub(1))?
            .nb_start_points())
    }

    /// `NbTangentZones()`. `inter.nb_tangent_zones()`.
    pub fn nb_tangent_zones(&self) -> i32 {
        self.tangent_zones.nb_items()
    }

    /// `NbPointsInTangentZone(int)`: always 1. `inter.nb_points_in_tangent_zone(1)`.
    pub fn nb_points_in_tangent_zone(&self, _index: i32) -> i32 {
        1
    }

    /// `GetLinePoint(IndexLine, IndexPoint, x, y, z, u1, v1, u2, v2, incidence)`: the point
    /// `[x, y, z, u1, v1, u2, v2, incidence]`; Standard_OutOfRange outside the points of
    /// the line (and past the storage of the lines). `inter.get_line_point(1, 1)?`.
    pub fn get_line_point(&self, index_line: i32, index_point: i32) -> Result<[f64; 8], Failure> {
        let line = self.section_lines.value(index_line.wrapping_sub(1))?;
        let sp: &IntPolyhStartPoint = line.value(index_point.wrapping_sub(1))?;
        Ok([
            sp.x(),
            sp.y(),
            sp.z(),
            sp.u1(),
            sp.v1(),
            sp.u2(),
            sp.v2(),
            sp.get_angle(),
        ])
    }

    /// `GetTangentZonePoint(IndexLine, IndexPoint, x, y, z, u1, v1, u2, v2)`: the zone
    /// `IndexLine` (`IndexPoint` is not read) as `[x, y, z, u1, v1, u2, v2]`; past the
    /// zones the allocated default points answer, past the storage Standard_OutOfRange.
    /// `inter.get_tangent_zone_point(1, 1)?`.
    pub fn get_tangent_zone_point(
        &self,
        index_zone: i32,
        _index_point: i32,
    ) -> Result<[f64; 7], Failure> {
        let sp: &IntPolyhStartPoint = self.tangent_zones.value(index_zone.wrapping_sub(1))?;
        Ok([sp.x(), sp.y(), sp.z(), sp.u1(), sp.v1(), sp.u2(), sp.v2()])
    }

    /// `Perform()`: the default sampling of both surfaces, then the intersection.
    fn perform(&mut self) -> Result<(), Failure> {
        let (mut u_pars1, mut v_pars1) = (Array1::default(), Array1::default());
        let (mut u_pars2, mut v_pars2) = (Array1::default(), Array1::default());
        IntPolyhTools::make_sampling(
            &self.surf1,
            (self.nb_su1, self.nb_sv1),
            false,
            &mut u_pars1,
            &mut v_pars1,
        )?;
        IntPolyhTools::make_sampling(
            &self.surf2,
            (self.nb_su2, self.nb_sv2),
            false,
            &mut u_pars2,
            &mut v_pars2,
        )?;
        self.perform_on([(&u_pars1, &v_pars1), (&u_pars2, &v_pars2)])
    }

    /// `Perform(theUPars1, theVPars1, theUPars2, theVPars2)`: the standard intersection,
    /// or the advanced one (four shifted triangulations) when it finds too few or too
    /// tangent couples.
    fn perform_on(&mut self, pars: Samplings<'_>) -> Result<(), Failure> {
        self.is_done = true;
        let defl_tol1: f64 = IntPolyhTools::compute_deflection(&self.surf1, pars[0].0, pars[0].1)?;
        let defl_tol2: f64 = IntPolyhTools::compute_deflection(&self.surf2, pars[1].0, pars[1].1)?;
        let defl: [f64; 2] = [defl_tol1, defl_tol2];
        let Some(mut maillage_std) = self.perform_maillage(pars, defl)? else {
            self.is_done = false;
            return Ok(());
        };
        let nb_couples_std: i32 = maillage_std.get_couples().extent();
        if !self.is_adv_required(&mut maillage_std) {
            // The default interference is good.
            return self.chain(&mut maillage_std).map(|_| ());
        }
        // Too few interferences: four intersections with shifted triangulations.
        let adv = self.perform_adv(pars, defl)?;
        match adv {
            Some((mut maillages, nb_couples_adv)) if nb_couples_adv > 0 => {
                for maillage in &mut maillages {
                    self.chain(maillage)?;
                }
            }
            _ => {
                if nb_couples_std > 0 {
                    self.chain(&mut maillage_std)?;
                }
            }
        }
        Ok(())
    }

    fn chain(&mut self, maillage: &mut IntPolyhMaillageAffinage) -> Result<i32, Failure> {
        maillage.start_points_chain(&mut self.section_lines, &mut self.tangent_zones)
    }

    /// `PerformAdv`: the four shifted intersections (FR, RF, FF, RR), stopped at the first
    /// failure, and their couples merged; `None` when one is not done. The maillages are
    /// returned in the order FF, FR, RF, RR.
    #[allow(clippy::type_complexity)]
    fn perform_adv(
        &mut self,
        pars: Samplings<'_>,
        defl: [f64; 2],
    ) -> Result<Option<([IntPolyhMaillageAffinage; 4], i32)>, Failure> {
        let (mut points1, mut points2) = (
            IntPolyhArrayOfPointNormal::default(),
            IntPolyhArrayOfPointNormal::default(),
        );
        IntPolyhTools::fill_array_of_point_normal(&self.surf1, pars[0].0, pars[0].1, &mut points1)?;
        IntPolyhTools::fill_array_of_point_normal(&self.surf2, pars[1].0, pars[1].1, &mut points2)?;
        let mut done: Vec<IntPolyhMaillageAffinage> = Vec::with_capacity(4);
        for shifts in [(true, false), (false, true), (true, true), (false, false)] {
            match self.perform_maillage_shifted(pars, defl, (&points1, &points2), shifts)? {
                Some(maillage) => done.push(maillage),
                None => return Ok(None),
            }
        }
        let [mut fr, mut rf, mut ff, mut rr]: [IntPolyhMaillageAffinage; 4] = done
            .try_into()
            .unwrap_or_else(|_| unreachable!("four maillages"));
        let nb_couples: i32 = ff.get_couples().extent()
            + fr.get_couples().extent()
            + rf.get_couples().extent()
            + rr.get_couples().extent();
        if nb_couples > 0 {
            Self::merge_couples([
                ff.get_couples(),
                fr.get_couples(),
                rf.get_couples(),
                rr.get_couples(),
            ]);
        }
        Ok(Some(([ff, fr, rf, rr], nb_couples)))
    }

    /// `PerformMaillage(theUPars1, ..., theDeflTol2, theMaillage)`: the standard
    /// intersection, retried on the enlarged surfaces when no triangles interfere;
    /// `None` when `AnalyzeIntersection` fails.
    fn perform_maillage(
        &mut self,
        pars: Samplings<'_>,
        defl: [f64; 2],
    ) -> Result<Option<IntPolyhMaillageAffinage>, Failure> {
        let mut maillage = self.new_maillage(pars);
        maillage.fill_array_of_pnt_params(1, pars[0].0, pars[0].1, Some(defl[0]))?;
        maillage.fill_array_of_pnt_params(2, pars[1].0, pars[1].1, Some(defl[1]))?;
        let fin_ttc: i32 = compute_intersection(&mut maillage)?;
        if fin_ttc == 0 {
            // No interfering triangles: try the enlarged surfaces.
            let (u1, v1) = IntPolyhTools::is_enlarge_possible(&self.surf1)?;
            let (u2, v2) = IntPolyhTools::is_enlarge_possible(&self.surf2)?;
            if u1 || v1 || u2 || v2 {
                maillage.set_enlarge_zone(true);
                maillage.fill_array_of_pnt(1)?;
                maillage.fill_array_of_pnt(2)?;
                compute_intersection(&mut maillage)?;
                maillage.set_enlarge_zone(false);
            }
        }
        Ok(self.analyze_intersection(&mut maillage).then_some(maillage))
    }

    /// `PerformMaillage(..., thePoints1, thePoints2, theIsFirstFwd, theIsSecondFwd,
    /// theMaillage)`: the intersection of the shifted triangulations.
    fn perform_maillage_shifted(
        &mut self,
        pars: Samplings<'_>,
        defl: [f64; 2],
        (points1, points2): (&IntPolyhArrayOfPointNormal, &IntPolyhArrayOfPointNormal),
        (first_fwd, second_fwd): (bool, bool),
    ) -> Result<Option<IntPolyhMaillageAffinage>, Failure> {
        let mut maillage = self.new_maillage(pars);
        maillage.fill_array_of_pnt_normals(1, first_fwd, points1, pars[0], defl[0])?;
        maillage.fill_array_of_pnt_normals(2, second_fwd, points2, pars[1], defl[1])?;
        compute_intersection(&mut maillage)?;
        Ok(self.analyze_intersection(&mut maillage).then_some(maillage))
    }

    fn new_maillage(&self, pars: Samplings<'_>) -> IntPolyhMaillageAffinage {
        IntPolyhMaillageAffinage::with_samples(
            &self.surf1,
            (pars[0].0.length(), pars[0].1.length()),
            &self.surf2,
            (pars[1].0.length(), pars[1].1.length()),
            0,
        )
    }

    /// `MergeCouples`: a couple found in several lists is kept in the first one only.
    fn merge_couples(lists: [&mut rustycad_tkernel::ncollection::List<IntPolyhCouple>; 4]) {
        let mut fence: HashSet<IntPolyhCouple> = HashSet::new();
        for list in lists {
            list.retain(|couple| fence.insert(*couple));
        }
    }

    /// `IsAdvRequired`: no couple (unless parallel), or at most 10 couples with one
    /// almost tangent.
    fn is_adv_required(&self, maillage: &mut IntPolyhMaillageAffinage) -> bool {
        let couples = maillage.get_couples();
        let nb_couples: i32 = couples.extent();
        if nb_couples == 0 && !self.is_parallel() {
            return true;
        }
        if nb_couples > 10 {
            return false;
        }
        couples.iter().any(|c| c.angle().abs() > COS_5_DEG)
    }

    /// `AnalyzeIntersection`: with more than 200 couples of which at least as many as the
    /// triangles of a surface are almost parallel, the surfaces are parallel and the
    /// couples dropped. Always true natively for an existing maillage.
    fn analyze_intersection(&mut self, maillage: &mut IntPolyhMaillageAffinage) -> bool {
        let nb_t1: i32 = maillage.get_array_of_triangles(1).nb_items();
        let nb_t2: i32 = maillage.get_array_of_triangles(2).nb_items();
        let couples = maillage.get_couples();
        if couples.extent() > 200 {
            let npara: i32 = couples
                .iter()
                .filter(|c| c.angle().abs() > COS_5_DEG)
                .count() as i32;
            if npara >= nb_t1 || npara >= nb_t2 {
                couples.clear();
                self.is_parallel = true;
            }
        }
        true
    }
}

/// `ComputeIntersection(theMaillage)`: the common box, the grids, the refinement and the
/// comparison of the triangles; the number of couples in contact.
fn compute_intersection(maillage: &mut IntPolyhMaillageAffinage) -> Result<i32, Failure> {
    maillage.common_box()?;
    maillage.fill_array_of_triangles(1);
    maillage.fill_array_of_triangles(2);
    maillage.fill_array_of_edges(1);
    maillage.fill_array_of_edges(2);
    maillage.triangles_deflections_refinement_bsb()?;
    maillage.triangle_compare()
}

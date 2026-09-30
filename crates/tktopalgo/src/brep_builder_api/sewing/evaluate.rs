// Rust translation of BRepBuilderAPI_Sewing::EvaluateAngulars and EvaluateDistances
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The comparison of a reference section with the following sections: the mean angle
//! between the normals of their faces, and their distance, length and mutual
//! orientation.
use super::BRepBuilderAPISewing;
use super::cutting::Projections;
use super::failures::{array1, null_object};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::{Array1, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::gcpnts::UniformAbscissa;
use rustycad_tkmath::gp::{self, Point3, Vector3};
use std::f64::consts::PI;

/// The number of samples of `EvaluateAngulars`.
const NB_ANGULAR_SAMPLES: i32 = 4;
/// The number of samples of `EvaluateDistances`.
const NB_DISTANCE_SAMPLES: i32 = 8;

/// The 3d curve of an edge in global coordinates (a transformed copy when located),
/// with its range; a null curve when the edge has none.
pub(super) fn global_curve(edge: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let Some(curve) = found.curve.get() else {
        return Ok((found.curve.clone(), found.first, found.last));
    };
    if found.location.is_identity() {
        return Ok((found.curve.clone(), found.first, found.last));
    }
    let copy = curve.copy()?.cast::<dyn GeomCurve>();
    if let Some(c) = copy.get() {
        c.transform(found.location.transformation())?;
    }
    Ok((copy, found.first, found.last))
}

/// A curve and its parameter range.
type CurveRange<'c> = (&'c Handle<dyn GeomCurve>, f64, f64);

/// The samples of the reference and of the current section, and the reference curve.
struct DistanceSamples {
    pts_ref: Array1<'static, Point3>,
    pts_sec: Array1<'static, Point3>,
    reference: (Handle<dyn GeomCurve>, f64, f64),
}

impl DistanceSamples {
    /// Samples a section (the reference when `is_ref`, kept with its curve) and
    /// measures it against the reference.
    fn sample(
        &mut self,
        is_ref: bool,
        (c3d, first, last): CurveRange<'_>,
    ) -> Result<SampleDistances, Failure> {
        let curve = c3d
            .get()
            .ok_or_else(|| null_object("EvaluateDistances: null curve"))?;
        if is_ref {
            self.reference = (c3d.clone(), first, last);
            sample_curve(curve, (first, last), &mut self.pts_ref)?;
            return Ok(sample_distances(&self.pts_ref, &self.pts_ref, true));
        }
        sample_curve(curve, (first, last), &mut self.pts_sec)?;
        Ok(sample_distances(&self.pts_sec, &self.pts_ref, false))
    }

    /// The samples to project and the curve to project them on: the section's onto the
    /// reference curve when the reference is longer, else the reference's onto the
    /// section's curve.
    fn projection<'c>(
        &'c self,
        longer_reference: bool,
        section: CurveRange<'c>,
    ) -> (&'c Array1<'static, Point3>, CurveRange<'c>) {
        if longer_reference {
            return (
                &self.pts_sec,
                (&self.reference.0, self.reference.1, self.reference.2),
            );
        }
        (&self.pts_ref, section)
    }
}

/// The distances between one section's samples and the reference's, both ways.
struct SampleDistances {
    dist_for: f64,
    dist_rev: f64,
    min_dist: f64,
    nb_found: i32,
    len2: f64,
}

impl BRepBuilderAPISewing {
    /// `EvaluateAngulars(sequenceSec, secForward, tabAng, indRef)`: for each section
    /// after the reference that bounds a face, the mean angle (folded below pi/2)
    /// between its face normal and the reference's at 4 points of equal abscissa;
    /// -1 when none can be computed. It stops when the reference bounds no face.
    #[allow(dead_code)]
    pub(crate) fn evaluate_angulars(
        &self,
        sequence_sec: &Sequence<Shape>,
        sec_forward: &Array1<'_, bool>,
        tab_ang: &mut Array1<'_, f64>,
        ind_ref: i32,
    ) -> Result<(), Failure> {
        tab_ang.init(-1.0);
        let mut norm_ref = array1(NB_ANGULAR_SAMPLES, Vector3::default())?;
        for i in ind_ref..=sequence_sec.length() {
            let edge = topods::edge(sequence_sec.value(i as usize)?)?;
            let Some(faces) = self.my_bound_faces.seek(&self.bound_of(&edge)) else {
                if i == ind_ref {
                    return Ok(());
                }
                continue;
            };
            let face = topods::face(faces.first().ok_or_else(|| {
                super::failures::no_such_object("EvaluateAngulars: a boundary without faces")
            })?)?;
            let normals = face_normals(&edge, &face, sec_forward[i] || i == ind_ref)?;
            if i == ind_ref {
                for (j, normal) in normals.into_iter().enumerate() {
                    norm_ref[j as i32 + 1] = normal;
                }
                continue;
            }
            let (mut cumulate, mut nb) = (0.0, 0);
            for (j, normal) in normals.into_iter().enumerate() {
                let reference = norm_ref[j as i32 + 1];
                if normal.magnitude() > gp::resolution() && reference.magnitude() > gp::resolution()
                {
                    nb += 1;
                    let mut angular: f64 = normal.angle(reference)?;
                    if angular > PI / 2. {
                        angular = PI - angular;
                    }
                    cumulate += angular;
                }
            }
            if nb != 0 {
                tab_ang[i] = cumulate / f64::from(nb);
            }
        }
        Ok(())
    }

    /// `EvaluateDistances(sequenceSec, secForward, tabDst, arrLen, tabMinDist,
    /// indRef)`: for each section from the reference on, its polygon length at 8
    /// samples, its orientation (the nearer of the sample-to-sample distances both
    /// ways) and, when close and overlapping, the largest and smallest distances;
    /// otherwise the distances of the projections of the shorter section's samples on
    /// the longer one's curve, when more than one projects.
    pub(crate) fn evaluate_distances(
        &self,
        sequence_sec: &Sequence<Shape>,
        sec_forward: &mut Array1<'_, bool>,
        tab_dst: &mut Array1<'_, f64>,
        arr_len: &mut Array1<'_, f64>,
        tab_min_dist: &mut Array1<'_, f64>,
        ind_ref: i32,
    ) -> Result<(), Failure> {
        sec_forward.init(true);
        tab_dst.init(-1.0);
        arr_len.init(0.);
        tab_min_dist.init(precision::infinite());
        let mut samples = DistanceSamples {
            pts_ref: array1(NB_DISTANCE_SAMPLES, Point3::default())?,
            pts_sec: array1(NB_DISTANCE_SAMPLES, Point3::default())?,
            reference: (Handle::default(), 0., 0.),
        };
        for i in ind_ref..=sequence_sec.length() {
            let sec = topods::edge(sequence_sec.value(i as usize)?)?;
            let (c3d, first, last) = global_curve(&sec)?;
            if c3d.is_null() {
                continue;
            }
            let d = samples.sample(i == ind_ref, (&c3d, first, last))?;
            arr_len[i] = d.len2.sqrt();
            // szv debug: <=
            let is_forward = d.dist_for < d.dist_rev;
            sec_forward[i] = is_forward;
            let dist = if is_forward { d.dist_for } else { d.dist_rev };
            let close = dist < self.my_tolerance
                && f64::from(d.nb_found) >= f64::from(NB_DISTANCE_SAMPLES) * 0.5;
            if i == ind_ref || close {
                tab_dst[i] = dist;
                tab_min_dist[i] = d.min_dist;
                continue;
            }
            let longer_reference = arr_len[ind_ref] >= arr_len[i];
            if let Some((max, min)) =
                self.projected_distances(samples.projection(longer_reference, (&c3d, first, last)))?
            {
                tab_dst[i] = max;
                tab_min_dist[i] = min;
            }
        }
        Ok(())
    }

    /// The largest and smallest distances of the projected samples, when more than one
    /// projects.
    fn projected_distances(
        &self,
        (points, (curve, first, last)): (&Array1<'_, Point3>, CurveRange<'_>),
    ) -> Result<Option<(f64, f64)>, Failure> {
        let mut out = Projections::new(NB_DISTANCE_SAMPLES)?;
        self.project_points_on_curve(points, curve, (first, last), &mut out, false)?;
        let arr_dist = &out.dist;
        let (mut nb_found, mut min_dist, mut dist) = (0, precision::infinite(), -1.0);
        for j in 1..=NB_DISTANCE_SAMPLES {
            if arr_dist[j] < 0. {
                continue;
            }
            if dist < arr_dist[j] {
                dist = arr_dist[j];
            }
            if min_dist > arr_dist[j] {
                min_dist = arr_dist[j];
            }
            nb_found += 1;
        }
        Ok((nb_found > 1).then_some((dist, min_dist)))
    }
}

/// The normals of the face along the edge, at 4 points of equal abscissa on its 3d
/// curve (in reverse order when `forward` is false), through its p-curve.
fn face_normals(
    edge: &Edge,
    face: &rustycad_tkbrep::topods::Face,
    forward: bool,
) -> Result<Vec<Vector3>, Failure> {
    let (mut surf, loc) = BRepTool::surface_with_location(face)?;
    if !loc.is_identity()
        && let Some(s) = surf.get()
    {
        let copy = s.copy()?.cast::<dyn GeomSurface>();
        if let Some(c) = copy.get() {
            c.transform(loc.transformation())?;
        }
        surf = copy;
    }
    let c2d = BRepTool::curve_on_face(edge, face)?.pcurve;
    let (c3d, first, last) = global_curve(edge)?;
    let adaptor = GeomAdaptorCurve::with_curve(&c3d)?;
    let uni_abs =
        UniformAbscissa::with_nb_points_range(&adaptor, NB_ANGULAR_SAMPLES, first, last, -1.0)?;
    let s = surf
        .get()
        .ok_or_else(|| null_object("EvaluateAngulars: null surface"))?;
    let c = c2d
        .get()
        .ok_or_else(|| null_object("EvaluateAngulars: null p-curve"))?;
    let mut normals = Vec::new();
    for j in 1..=NB_ANGULAR_SAMPLES {
        let index = if forward {
            j
        } else {
            NB_ANGULAR_SAMPLES - j + 1
        };
        let p = c.d0(uni_abs.parameter(index)?)?;
        let d1 = s.d1(p.x(), p.y())?;
        normals.push(d1.d1u.crossed(d1.d1v));
    }
    Ok(normals)
}

/// The 8 samples of a section: the ends exactly, the others at uniform steps.
fn sample_curve(
    curve: &dyn GeomCurve,
    (first, last): (f64, f64),
    samples: &mut Array1<'_, Point3>,
) -> Result<(), Failure> {
    let delta_t: f64 = (last - first) / f64::from(NB_DISTANCE_SAMPLES - 1);
    for j in 1..=NB_DISTANCE_SAMPLES {
        let t = if j == 1 {
            first
        } else if j == NB_DISTANCE_SAMPLES {
            last
        } else {
            first + f64::from(j - 1) * delta_t
        };
        samples[j] = curve.value(t)?;
    }
    Ok(())
}

/// The squared polygon length of the samples and, for a section other than the
/// reference, its distances to the reference samples both ways and the number of
/// samples that lie between the reference ends.
fn sample_distances(
    samples: &Array1<'_, Point3>,
    pts_ref: &Array1<'_, Point3>,
    is_ref: bool,
) -> SampleDistances {
    let npt = NB_DISTANCE_SAMPLES;
    let mut d = SampleDistances {
        dist_for: -1.0,
        dist_rev: -1.0,
        min_dist: precision::infinite(),
        nb_found: 0,
        len2: 0.,
    };
    for j in 1..=npt {
        let pt = samples[j];
        if j > 1 {
            d.len2 += pt.square_distance(&samples[j - 1]);
        }
        if is_ref {
            continue;
        }
        let dist = pt.distance(&pts_ref[j]);
        d.min_dist = std_min(d.min_dist, dist);
        if d.dist_for < dist {
            d.dist_for = dist;
        }
        let dist = pt.distance(&pts_ref[npt - j + 1]);
        d.min_dist = std_min(d.min_dist, dist);
        if d.dist_rev < dist {
            d.dist_rev = dist;
        }
        let (p11, p12) = (pts_ref[1], pts_ref[npt]);
        let (vec1, vec2, vec_ref) = (
            Vector3::between(pt, p11),
            Vector3::between(pt, p12),
            Vector3::between(p11, p12),
        );
        if vec_ref.dot(vec1) * vec_ref.dot(vec2) < 0. {
            d.nb_found += 1;
        }
    }
    d
}

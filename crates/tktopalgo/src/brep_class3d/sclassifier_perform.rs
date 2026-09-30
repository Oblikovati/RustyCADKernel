// Rust translation of BRepClass3d_SClassifier::Perform (OCC578) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepClass3d_SClassifier::Perform`: the parity of the crossings of a segment from
//! the point with the boundary of the solid, the segment chosen again while it
//! passes through an edge or a vertex.
use super::bnd_box_tree::BRepClass3dBndBoxTreeSelectorLine;
use super::transitions::{get_transi, trans};
use super::{BRepClass3dSClassifier, BRepClass3dSolidExplorer};
use crate::int_curves_face::IntCurvesFaceIntersector;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeomalgo::int_curve_surface::TransitionOnCurve;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPS};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line3, Point2, Point3};

/// The line of one attempt and what its crossings found so far: `parmin` (the
/// parameter of the crossing that decides), `NearFaultPar` (the parameter of the
/// nearest crossing through an edge or vertex) and whether the line is faulty.
struct LineAttempt {
    line: Line3,
    par: f64,
    parmin: f64,
    near_fault_par: f64,
    faulty: bool,
}

impl BRepClass3dSClassifier {
    /// `Perform(S, P, Tol)`: IN for a solid without faces; ON when the point is in the
    /// tolerance zone of an edge or vertex; else, over the segments of the explorer
    /// (`Segment`, then `OtherSegment` while the segment passes through an edge or
    /// vertex before its decisive crossing): ON for a point on an infinite face (or
    /// IN/ON a face through its projection) or within `tol` of a crossing, OUT when
    /// the explorer finds degenerate face bounds, the error state when the face index
    /// of the segments stops increasing; otherwise IN or OUT from the transition of
    /// the crossing (with a face or through an edge between two faces) nearest to the
    /// point. A face met parallel to the segment gives ON when the point projects IN or
    /// ON it within `tol`; `classifier.perform(&mut explorer, &p, 1e-7)?`.
    pub fn perform(
        &mut self,
        explorer: &mut BRepClass3dSolidExplorer,
        point: &Point3,
        tol: f64,
    ) -> Result<(), Failure> {
        if explorer.reject(point) {
            self.my_state = 3; // Solid without faces => the whole space. Always in.
            return Ok(());
        }
        if explorer.selects_point(*point)? {
            self.my_state = 2; // the point lays inside the tolerance area of vertices/edges
            return Ok(());
        }
        let mut map_ef = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            explorer.get_shape(),
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut map_ef,
        )?;
        self.my_face = Face::default();
        self.my_state = 0;
        let mut attempt = LineAttempt {
            line: Line3::default(),
            par: 0.0,
            parmin: 0.0,
            near_fault_par: f64::MAX,
            faulty: true,
        };
        let mut ind_face: i32 = 0;
        while attempt.faulty {
            let flag = if ind_face == 0 {
                explorer.segment(point, &mut attempt.line, &mut attempt.par)?
            } else {
                explorer.other_segment(point, &mut attempt.line, &mut attempt.par)?
            };
            let cur_ind = explorer.get_face_segment_index();
            if cur_ind <= ind_face {
                self.my_state = 1; // Faulty.
                return Ok(());
            }
            ind_face = cur_ind;
            match flag {
                1 => self.my_state = 2, // the face is infinite: ON
                2 => self.my_state = 4, // OUT
                // The point is ON the surface but OUT of the face: skip this face.
                3 => continue,
                _ => {}
            }
            if flag == 1 || flag == 2 {
                return Ok(());
            }
            self.classify_along(explorer, point, tol, &map_ef, &mut attempt)?;
        }
        Ok(())
    }

    /// One line: its crossings through edges and vertices, then with the faces.
    fn classify_along(
        &mut self,
        explorer: &mut BRepClass3dSolidExplorer,
        point: &Point3,
        tol: f64,
        map_ef: &IndexedDataMapOfShapeListOfShape,
        attempt: &mut LineAttempt,
    ) -> Result<(), Failure> {
        attempt.faulty = false;
        attempt.parmin = f64::MAX;
        attempt.near_fault_par = f64::MAX; // Parameter on line.
        self.edge_crossings(explorer, map_ef, attempt)?;
        self.face_crossings(explorer, point, tol, attempt)?;
        if attempt.near_fault_par != f64::MAX
            && attempt.parmin.abs() >= attempt.near_fault_par.abs() - precision::p_confusion()
        {
            attempt.faulty = true;
        }
        Ok(())
    }

    /// The vertices and edges the segment passes through: the nearest vertex is a
    /// fault; an edge between two faces (whose vertices are not crossed) gives the
    /// state from its transition when it is nearer than `parmin`, else a fault.
    fn edge_crossings(
        &mut self,
        explorer: &BRepClass3dSolidExplorer,
        map_ef: &IndexedDataMapOfShapeListOfShape,
        attempt: &mut LineAttempt,
    ) -> Result<(), Failure> {
        let mut selector = BRepClass3dBndBoxTreeSelectorLine::new(explorer.get_map_ev());
        selector.clear_results();
        selector.set_current_line(attempt.line, attempt.par)?;
        let selected: i32 = explorer.get_tree().select(&mut selector);
        if let Some(failure) = selector.take_failure() {
            return Err(failure);
        }
        if selected <= 0 {
            return Ok(());
        }
        let mut crossed_vertices = MapOfShape::default();
        for i in 1..=selector.get_nb_vert_param() {
            let (vertex, lp) = selector.get_vert_param(i)?;
            crossed_vertices.add(vertex.into_shape());
            if lp.abs() < attempt.near_fault_par.abs() {
                attempt.near_fault_par = lp;
            }
        }
        for i in 1..=selector.get_nb_edge_param() {
            let (edge, param, lpar) = selector.get_edge_param(i)?;
            let faces = map_ef.find_from_key(&edge)?; // ffs size == 2
            if faces.extent() != 2 {
                continue;
            }
            let (f1, f2) = (
                topods::face(faces.first().unwrap())?,
                topods::face(faces.last().unwrap())?,
            );
            let (v1, v2) = topexp::vertices(&edge, false);
            if crossed_vertices.contains(&v1) || crossed_vertices.contains(&v2) {
                continue;
            }
            let mut tran = TransitionOnCurve::Tangent;
            let tst = get_transi(&f1, &f2, &edge, param, &attempt.line, &mut tran)?;
            if tst == 1 && lpar.abs() < attempt.parmin.abs() {
                attempt.parmin = lpar;
                trans(attempt.parmin, &mut tran, &mut self.my_state);
            } else if lpar.abs() < attempt.near_fault_par.abs() {
                attempt.near_fault_par = lpar;
            }
        }
        Ok(())
    }

    /// The crossings with the faces of every shell, until the state is ON or the line
    /// faulty.
    fn face_crossings(
        &mut self,
        explorer: &mut BRepClass3dSolidExplorer,
        point: &Point3,
        tol: f64,
        attempt: &mut LineAttempt,
    ) -> Result<(), Failure> {
        explorer.init_shell()?;
        while explorer.more_shell() && !attempt.faulty {
            if explorer.reject_shell(&attempt.line) {
                self.my_state = 1;
            } else {
                explorer.init_face()?;
                while explorer.more_face() && !attempt.faulty {
                    if !explorer.reject_face(&attempt.line)
                        && self.face_crossing(explorer, point, tol, attempt)?
                    {
                        break;
                    }
                    explorer.next_face()?;
                }
                if self.my_state == 2 {
                    break;
                }
            }
            explorer.next_shell()?;
        }
        Ok(())
    }

    /// The crossings of the prolonged segment with the current face; true when the
    /// state became ON (the face loop stops).
    fn face_crossing(
        &mut self,
        explorer: &mut BRepClass3dSolidExplorer,
        point: &Point3,
        tol: f64,
        attempt: &mut LineAttempt,
    ) -> Result<bool, Failure> {
        let face = explorer.current_face()?;
        let intersector = explorer.change_intersector(&face)?;
        // Prolong segment, since there are cases when the intersector does not find
        // intersection points with the original segment due to rough triangulation of
        // a parameterized surface.
        let (min_w, max_w) =
            prolonged_segment(&attempt.line, attempt.par, tol, &intersector.bounding());
        intersector.perform_line(&attempt.line, min_w, max_w)?;
        if !intersector.is_done() {
            self.my_state = 1;
            return Ok(false);
        }
        if intersector.nb_pnt() == 0
            && intersector.is_parallel()
            && on_parallel_face(intersector, &face, point, tol)?
        {
            self.my_state = 2;
            self.my_face = face;
            attempt.parmin = 0.0;
            return Ok(true);
        }
        self.keep_crossings(intersector, &face, tol, attempt)?;
        Ok(self.my_state == 2)
    }

    /// The points of the face intersector nearer than `parmin` (by more than
    /// Precision::PConfusion()): within `tol` it is ON, IN the face it gives the state
    /// from its transition (tangent points ignored), ON the face boundary the line is
    /// faulty.
    fn keep_crossings(
        &mut self,
        intersector: &IntCurvesFaceIntersector,
        face: &Face,
        tol: f64,
        attempt: &mut LineAttempt,
    ) -> Result<(), Failure> {
        for i in 1..=intersector.nb_pnt() {
            let w = intersector.w_parameter(i)?;
            if w.abs() >= attempt.parmin.abs() - precision::p_confusion() {
                continue; // a point with a greater parameter
            }
            attempt.parmin = w;
            let state = intersector.state(i)?;
            if attempt.parmin.abs() <= tol {
                self.my_state = 2;
                self.my_face = face.clone();
                return Ok(());
            }
            if state == State::In {
                let mut tran = intersector.transition(i)?;
                if tran == TransitionOnCurve::Tangent {
                    continue; // ignore this point
                }
                trans(attempt.parmin, &mut tran, &mut self.my_state);
                self.my_face = face.clone();
            } else if state == State::On {
                // Another line is needed: repeat the whole procedure.
                attempt.faulty = true;
                return Ok(());
            }
        }
        Ok(())
    }
}

/// The parameter range `[-addW, min(10 Par, Par + addW')]` of the prolonged segment:
/// `addW = max(10 Tol, 0.01 Par)`, raised for the upper end to reach the corners of a
/// finite face box.
fn prolonged_segment(line: &Line3, par: f64, tol: f64, face_box: &BoundingBox3) -> (f64, f64) {
    let add_w_initial: f64 = std_max(10.0 * tol, 0.01 * par);
    let mut add_w = add_w_initial;
    // The box must be finite in order to correctly prolong the segment to its bounds.
    if !face_box.is_void() && !face_box.is_whole() {
        add_w = std_max(add_w, get_add_to_param(line, par, face_box));
    }
    (-add_w_initial, std_min(par * 10.0, par + add_w))
}

/// `std::max(a, b)`.
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// `std::min(a, b)`.
fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `GetAddToParam(L, P, B)`: how far the parameter of the farthest box corner lies
/// beyond `par` (0 when none does), 1e20 when a corner is 1e20 or more away from the
/// line origin along an axis.
fn get_add_to_param(line: &Line3, par: f64, face_box: &BoundingBox3) -> f64 {
    let Ok([xmin, ymin, zmin, xmax, ymax, zmax]) = face_box.get() else {
        return 0.0;
    };
    let origin = line.location();
    let mut farthest = par;
    for x in [xmin, xmax] {
        for y in [ymin, ymax] {
            for z in [zmin, zmax] {
                let near = (x - origin.x()).abs() < 1.0e20
                    && (y - origin.y()).abs() < 1.0e20
                    && (z - origin.z()).abs() < 1.0e20;
                if !near {
                    return 1.0e20;
                }
                let parameter = elclib::line_parameter(line.position(), Point3::new(x, y, z));
                if parameter > farthest {
                    farthest = parameter;
                }
            }
        }
    }
    farthest - par
}

/// A face parallel to the segment holds the point: its projection on the
/// unrestricted surface (Extrema_ExtPS, MIN) nearest within `tol` is IN or ON.
fn on_parallel_face(
    intersector: &IntCurvesFaceIntersector,
    face: &Face,
    point: &Point3,
    tol: f64,
) -> Result<bool, Failure> {
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    let pconfusion = precision::p_confusion();
    let projection = ExtPS::with_surface(
        *point,
        &surface,
        pconfusion,
        pconfusion,
        ExtFlag::Min,
        ExtAlgo::Grad,
    )?;
    if !(projection.is_done() && projection.nb_ext()? > 0) {
        return Ok(false);
    }
    let (mut indmin, mut d) = (0, f64::MAX);
    for i in 1..=projection.nb_ext()? {
        if projection.square_distance(i)? < d {
            (indmin, d) = (i, projection.square_distance(i)?);
        }
    }
    if indmin == 0 || d > tol * tol {
        return Ok(false);
    }
    let (u, v) = projection.point(indmin)?.parameter();
    Ok(matches!(
        intersector.classify_uv_point(Point2::new(u, v))?,
        State::In | State::On
    ))
}

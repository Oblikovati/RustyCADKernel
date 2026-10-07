//! HLRBRep_Intersector.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{
    CInter, EdgeData, InterCSurf, Surface, SurfaceTool, ThePolygonOfInterCSurf,
    ThePolyhedronOfInterCSurf,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_curve_surface::{
    Intersection as CSIntersection, IntersectionPoint as CSPoint, IntersectionSegment as CSSegment,
    SamplePolyhedron,
};
use rustycad_tkgeomalgo::int_imp_par_gen::determine_crossing_transition;
use rustycad_tkgeomalgo::int_res2d::{
    Domain, IntersectionPoint, IntersectionSegment, Position, Transition,
};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Line3, Point2, Vector2};

/// HLRBRep_Intersector: the intersections of the hidden line algorithm: one edge
/// with itself or with another edge in the view plane (HLRBRep_CInter), a simulated
/// single point, and a sight line with a face (HLRBRep_InterCSurf).
///
/// The native class keeps a pointer to the loaded surface and the
/// HLRBRep_InterCSurf object; the port takes the loaded surface again in
/// [`Intersector::perform_line`] and keeps the result of the last line-surface
/// intersection (each native Perform restarts from empty fields). The polyhedron of
/// a loaded surface is built on the first line and kept until the next `load`.
///
/// ```
/// use rustycad_tkhlr::hlr_brep::Intersector;
/// let inter = Intersector::new();
/// assert!(inter.is_done().unwrap());
/// assert_eq!(inter.nb_points().unwrap(), 1);
/// ```
pub struct Intersector {
    single_point: IntersectionPoint,
    type_perform: i32,
    intersector: CInter,
    cs_result: CSIntersection,
    polyhedron: Option<ThePolyhedronOfInterCSurf>,
}

impl Default for Intersector {
    fn default() -> Self {
        Self::new()
    }
}

fn edge_bounds(e: &EdgeData) -> (f64, f32, f64, f32) {
    e.status().bounds()
}

impl Intersector {
    /// HLRBRep_Intersector(): the curve intersector with at least 4 samples;
    /// `Intersector::new()`.
    pub fn new() -> Self {
        let mut intersector = CInter::new();
        // Set minimal number of samples in case of HLR polygonal intersector.
        let a_min_nb_hlr_samples = 4;
        intersector.set_min_nb_samples(a_min_nb_hlr_samples);
        Self {
            single_point: IntersectionPoint::default(),
            type_perform: 0,
            intersector,
            cs_result: CSIntersection::new(),
            polyhedron: None,
        }
    }

    /// Perform(Edge1, Da1, Db1): the self intersection of the edge on its visible
    /// range, shortened by the fractions `da1` and `db1` at its ends;
    /// `inter.perform_self(&edge, 0.0, 0.0)?`.
    pub fn perform_self(&mut self, edge1: &EdgeData, da1: f64, db1: f64) -> Result<(), Failure> {
        let my_c1 = edge1.curve();
        self.type_perform = 1;
        let (mut a, ta, mut b, tb) = edge_bounds(edge1);
        let d = b - a;
        if da1 != 0. {
            a += d * da1;
        }
        if db1 != 0. {
            b -= d * db1;
        }
        let (mut pa, mut pb) = (Point2::default(), Point2::default());
        my_c1.d0(a, &mut pa)?;
        my_c1.d0(b, &mut pb)?;
        let a = my_c1.parameter_2d(a)?;
        let b = my_c1.parameter_2d(b)?;
        let d1 = Domain::bounded(pa, a, f64::from(ta), pb, b, f64::from(tb));
        // modified by jgv, 18.04.2016 for OCC27341
        let tol = precision::confusion();
        self.intersector.perform_self_on(my_c1, &d1, tol, tol)
    }

    /// SimulateOnePoint(Edge1, U, Edge2, V): one crossing point at the 2D
    /// parameters `u` and `v` with the transitions of the tangents there;
    /// `inter.simulate_one_point(&e1, 0.5, &e2, 0.25)?`.
    pub fn simulate_one_point(
        &mut self,
        edge1: &EdgeData,
        u: f64,
        edge2: &EdgeData,
        v: f64,
    ) -> Result<(), Failure> {
        let (my_c1, my_c2) = (edge1.curve(), edge2.curve());
        let u3 = my_c1.parameter_3d(u)?;
        let v3 = my_c2.parameter_3d(v)?;
        let (mut p13, mut p23) = (Point2::default(), Point2::default());
        let (mut t13, mut t23) = (Vector2::default(), Vector2::default());
        my_c1.d1(u3, &mut p13, &mut t13)?;
        my_c2.d1(v3, &mut p23, &mut t23)?;
        let (mut tr1, mut tr2) = (Transition::new(), Transition::new());
        let (pos1, pos2) = (Position::Middle, Position::Middle);
        determine_crossing_transition((pos1, t13, &mut tr1), (pos2, t23, &mut tr2), 0.0);
        self.type_perform = 0;
        self.single_point.set_values(p13, u, v, tr1, tr2, false);
        Ok(())
    }

    /// Load(Surface): forgets the polyhedron of the previous surface;
    /// `inter.load()`.
    pub fn load(&mut self) {
        self.polyhedron = None;
    }

    /// Perform(L, P): the sight line `l` against the loaded surface `surface`: the
    /// analytic intersection on the elementary types, else the polyhedral one on the
    /// part of the line inside the box of the polyhedron and before the parameter `p`;
    /// `inter.perform_line(&surface, &line, 10.0)?`.
    pub fn perform_line(&mut self, surface: &Surface, l: &Line3, p: f64) -> Result<(), Failure> {
        self.type_perform = 2;
        let mut cs = InterCSurf::new();
        match SurfaceTool::get_type(surface) {
            SurfaceType::Plane
            | SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus => {
                let result = cs.perform(l, &surface);
                self.cs_result = (*cs).clone();
                result
            }
            _ => {
                if self.polyhedron.is_none() {
                    self.polyhedron = Some(Self::surface_polyhedron(surface)?);
                }
                let polyhedron = self
                    .polyhedron
                    .as_ref()
                    .ok_or_else(|| Failure::new("HLRBRep_Intersector: no polyhedron"))?;
                let (pmin, pmax) = sight_range(l, polyhedron, p)?;
                let polygon = ThePolygonOfInterCSurf::with_range(l, pmin, pmax, 3)?;
                let result = cs.perform_polygon_polyhedron(l, &polygon, &surface, polyhedron);
                self.cs_result = (*cs).clone();
                result
            }
        }
    }

    // The uniform polyhedron of the whole surface at its sample counts.
    fn surface_polyhedron(surface: &Surface) -> Result<ThePolyhedronOfInterCSurf, Failure> {
        let u1 = SurfaceTool::first_u_parameter(surface)?;
        let v1 = SurfaceTool::first_v_parameter(surface)?;
        let u2 = SurfaceTool::last_u_parameter(surface)?;
        let v2 = SurfaceTool::last_v_parameter(surface)?;
        let nbsu = SurfaceTool::nb_samples_u_in(surface, u1, u2)?;
        let nbsv = SurfaceTool::nb_samples_v_in(surface, v1, v2)?;
        SamplePolyhedron::<SurfaceTool<'_>>::uniform(&surface, nbsu, nbsv, [u1, v1, u2, v2])
    }

    /// IsDone(); `inter.is_done()?`.
    pub fn is_done(&self) -> Result<bool, Failure> {
        Ok(match self.type_perform {
            1 => self.intersector.is_done(),
            2 => self.cs_result.is_done(),
            _ => true,
        })
    }

    /// NbPoints(); `inter.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        match self.type_perform {
            43 => Ok(0),
            1 => self.intersector.nb_points(),
            2 => self.cs_result.nb_points(),
            _ => Ok(1),
        }
    }

    /// Point(N): the simulated point, or the N-th curve intersection point;
    /// `inter.point(1)?`.
    pub fn point(&self, n: i32) -> Result<IntersectionPoint, Failure> {
        if self.type_perform == 0 {
            return Ok(self.single_point);
        }
        self.intersector.point(n)
    }

    /// CSPoint(N); `inter.cs_point(1)?`.
    pub fn cs_point(&self, n: i32) -> Result<CSPoint, Failure> {
        self.cs_result.point(n)
    }

    /// NbSegments(); `inter.nb_segments()?`.
    pub fn nb_segments(&self) -> Result<i32, Failure> {
        match self.type_perform {
            1 => self.intersector.nb_segments(),
            2 => self.cs_result.nb_segments(),
            _ => Ok(0),
        }
    }

    /// Segment(N); `inter.segment(1)?`.
    pub fn segment(&self, n: i32) -> Result<IntersectionSegment, Failure> {
        self.intersector.segment(n)
    }

    /// CSSegment(N); `inter.cs_segment(1)?`.
    pub fn cs_segment(&self, n: i32) -> Result<CSSegment, Failure> {
        self.cs_result.segment(n)
    }

    /// Destroy(): forgets the polyhedron; `inter.destroy()`.
    pub fn destroy(&mut self) {
        self.polyhedron = None;
    }

    pub(super) fn curve_intersector(&mut self) -> &mut CInter {
        &mut self.intersector
    }
    pub(super) fn set_type_perform(&mut self, t: i32) {
        self.type_perform = t;
    }
}

// The parameters on the sight line of the corners of the polyhedron box, widened by
// 1e-6; a box beyond `p` is pushed past itself, else the range stops just after `p`.
fn sight_range(
    l: &Line3,
    polyhedron: &ThePolyhedronOfInterCSurf,
    p: f64,
) -> Result<(f64, f64), Failure> {
    let [x0, y0, z0, x1, y1, z1] = polyhedron.bounding().get()?;
    let corners = [
        (x0, y0, z0),
        (x0, y0, z1),
        (x1, y0, z0),
        (x1, y0, z1),
        (x0, y1, z0),
        (x0, y1, z1),
        (x1, y1, z0),
        (x1, y1, z1),
    ];
    let par = |(x, y, z): (f64, f64, f64)| {
        rustycad_tkmath::elclib::line_parameter(
            l.position(),
            rustycad_tkmath::gp::Point3::new(x, y, z),
        )
    };
    let mut pmin = par(corners[0]);
    let mut pmax = pmin;
    for c in &corners[1..] {
        let q = par(*c);
        if pmin > q {
            pmin = q;
        }
        if pmax < q {
            pmax = q;
        }
    }
    pmin -= 0.000001;
    pmax += 0.000001;
    if pmin > p {
        //-- on va rejeter avec les boites
        pmin = pmax + 1.;
        pmax += 2.;
    } else if pmax > p {
        pmax = p + 0.0000001;
    }
    Ok((pmin, pmax))
}

mod edges;

//! IntWalk_IWalking.gxx probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_iwalk.cxx: the generic is instantiated with
//! `HSurfaceTool`, `IntImpZerImpFunc` (IntPatch_TheSurfFunction) and [`ProbeIWLine`], a
//! probe port of IntPatch_TheIWLineOfTheIWalking (IntPatch is not ported yet).
use super::common::{Reader, attempt, end, flag, int, pnt, real, show, xyz};
use super::int_surf::quadric;
use super::int_walk::{pnt2s, surface};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{BaseAllocator, INC_DEFAULT_BLOCK_SIZE};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkgeomalgo::int_imp::{HSurfaceTool, IntImpZerImpFunc};
use rustycad_tkgeomalgo::int_surf::{
    IntSurfInteriorPoint, IntSurfLineOn2S, IntSurfPathPoint, IntSurfPntOn2S, IntSurfQuadricTool,
};
use rustycad_tkgeomalgo::int_walk::{IWalkingTypes, IntWalkIWalking, TheIWFunction, TheIWLine};
use rustycad_tkmath::gp::{Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

type SurfFunction = IntImpZerImpFunc<HSurfaceTool, IntSurfQuadricTool>;

/// The fields of IntPatch_TheIWLineOfTheIWalking.
struct LineState {
    couple: Vec<(i32, i32)>,
    closed: bool,
    has_first: bool,
    has_last: bool,
    first_index: i32,
    last_index: i32,
    first_point: IntSurfPathPoint,
    last_point: IntSurfPathPoint,
    indextg: i32,
    vcttg: Vector3,
    istgtbeg: bool,
    istgtend: bool,
}

/// IntPatch_TheIWLineOfTheIWalking: the walking line of the IntPatch instance.
pub struct ProbeIWLine {
    parent: StandardTransient,
    line: Handle<IntSurfLineOn2S>,
    state: SharedState<LineState>,
}
impl_transient!(ProbeIWLine, "GaProbe_TheIWLine", StandardTransient, parent);

impl ProbeIWLine {
    fn points(&self) -> &IntSurfLineOn2S {
        self.line.get().expect("probe line")
    }
    fn first_point_index(&self) -> Result<i32, Failure> {
        let s = self.state.read();
        if !s.has_first {
            return Err(Failure::with_type("Standard_DomainError", "", ""));
        }
        Ok(s.first_index)
    }
    fn last_point_index(&self) -> Result<i32, Failure> {
        let s = self.state.read();
        if !s.has_last {
            return Err(Failure::with_type("Standard_DomainError", "", ""));
        }
        Ok(s.last_index)
    }
    fn first_point(&self) -> Result<IntSurfPathPoint, Failure> {
        let s = self.state.read();
        if !s.has_first {
            return Err(Failure::with_type("Standard_DomainError", "", ""));
        }
        Ok(s.first_point.clone())
    }
    fn last_point(&self) -> Result<IntSurfPathPoint, Failure> {
        let s = self.state.read();
        if !s.has_last {
            return Err(Failure::with_type("Standard_DomainError", "", ""));
        }
        Ok(s.last_point.clone())
    }
}

impl TheIWLine for ProbeIWLine {
    fn with_inc_allocator() -> Self {
        let allocator: BaseAllocator = BaseAllocator::incremental(INC_DEFAULT_BLOCK_SIZE);
        Self {
            parent: StandardTransient::default(),
            line: Handle::new(IntSurfLineOn2S::new(Some(allocator))),
            state: SharedState::new(LineState {
                couple: Vec::new(),
                closed: false,
                has_first: false,
                has_last: false,
                first_index: -1,
                last_index: -1,
                first_point: IntSurfPathPoint::new(),
                last_point: IntSurfPathPoint::new(),
                indextg: -1,
                vcttg: Vector3::default(),
                istgtbeg: false,
                istgtend: false,
            }),
        }
    }
    fn reverse(&self) {
        self.points().reverse();
        let n = self.points().nb_points();
        for couple in &mut self.state.write().couple {
            *couple = (n - couple.0 + 1, couple.1);
        }
    }
    fn cut(&self, index: i32) -> Result<(), Failure> {
        self.points().split(index).map(|_| ())
    }
    fn add_point(&self, p: &IntSurfPntOn2S) {
        self.points().add(p);
    }
    fn add_status_first(&self, closed: bool, has_first: bool) {
        let mut s = self.state.write();
        (s.closed, s.has_first) = (closed, has_first);
    }
    fn add_status_first_point(
        &self,
        closed: bool,
        has_first: bool,
        index: i32,
        p: &IntSurfPathPoint,
    ) {
        let mut s = self.state.write();
        (s.closed, s.has_first, s.first_index, s.first_point) =
            (closed, has_first, index, p.clone());
    }
    fn add_status_first_last(&self, closed: bool, has_first: bool, has_last: bool) {
        let mut s = self.state.write();
        (s.closed, s.has_first, s.has_last) = (closed, has_first, has_last);
    }
    fn add_status_last(&self, has_last: bool) {
        self.state.write().has_last = has_last;
    }
    fn add_status_last_point(&self, has_last: bool, index: i32, p: &IntSurfPathPoint) {
        let mut s = self.state.write();
        (s.has_last, s.last_index, s.last_point) = (has_last, index, p.clone());
    }
    fn add_index_passing(&self, index: i32) {
        let n = self.points().nb_points();
        self.state.write().couple.push((n + 1, index));
    }
    fn set_tangent_vector(&self, v: Vector3, index: i32) {
        let mut s = self.state.write();
        (s.indextg, s.vcttg) = (index, v);
    }
    fn set_tangency_at_begining(&self, is_tangent: bool) {
        self.state.write().istgtbeg = is_tangent;
    }
    fn set_tangency_at_end(&self, is_tangent: bool) {
        self.state.write().istgtend = is_tangent;
    }
    fn nb_points(&self) -> i32 {
        self.points().nb_points()
    }
    fn value(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        self.points().value(index)
    }
    fn line(&self) -> Handle<IntSurfLineOn2S> {
        self.line.clone()
    }
    fn tangent_vector(&self) -> (Vector3, i32) {
        let s = self.state.read();
        (s.vcttg, s.indextg)
    }
    fn is_tangent_at_begining(&self) -> bool {
        self.state.read().istgtbeg
    }
    fn is_tangent_at_end(&self) -> bool {
        self.state.read().istgtend
    }
}

/// The probe instance: the types of IntPatch_TheIWalking.
pub struct ProbeTypes;
impl IWalkingTypes for ProbeTypes {
    type SurfaceTool = HSurfaceTool;
    type Function = SurfFunction;
    type Line = ProbeIWLine;
}

fn evaluate(f: &mut SurfFunction, u: f64, v: f64) -> Result<(), Failure> {
    let mut x = Vector::with_bounds(1, 2)?;
    let mut fv = Vector::with_bounds(1, 1)?;
    let mut d = Matrix::with_bounds(1, 1, 1, 2).expect("probe matrix");
    (x[1], x[2]) = (u, v);
    f.checked_values(&x, &mut fv, &mut d)?;
    Ok(())
}

/// `ga_iw_path_point`.
fn path_point(r: &mut Reader, f: &mut SurfFunction) -> Result<IntSurfPathPoint, Failure> {
    let (u, v) = (r.real(), r.real());
    let (pass, tangent, nmult) = (r.flag(), r.flag(), r.integer());
    evaluate(f, u, v)?;
    let mut p = IntSurfPathPoint::with_value(f.point(), u, v);
    if !tangent {
        let d3 = TheIWFunction::direction3d(f)?;
        p.set_directions(d3, TheIWFunction::direction2d(f)?);
    }
    p.set_passing(pass);
    for _ in 0..nmult {
        let a = r.real();
        p.add_uv(a, r.real())?;
    }
    Ok(p)
}

/// `ga_iw_interior_point`.
fn interior_point(r: &mut Reader, f: &mut SurfFunction) -> Result<IntSurfInteriorPoint, Failure> {
    let (u, v) = (r.real(), r.real());
    evaluate(f, u, v)?;
    if TheIWFunction::is_tangent(f)? {
        return Ok(IntSurfInteriorPoint::with_value(
            f.point(),
            u,
            v,
            Vector3::new(0.0, 0.0, 0.0),
            Vector2::new(1.0, 0.0),
        ));
    }
    let d2 = TheIWFunction::direction2d(f)?;
    let d3 = TheIWFunction::direction3d(f)?;
    Ok(IntSurfInteriorPoint::with_value(
        f.point(),
        u,
        v,
        d3,
        Vector2::new(d2.x(), d2.y()),
    ))
}

fn line_report(l: &ProbeIWLine) {
    let n = l.nb_points();
    int(n);
    for i in 1..=n {
        pnt2s(&l.value(i).expect("probe line point"));
    }
    let s = l.state.read();
    flag(s.closed);
    flag(s.has_first);
    flag(s.has_last);
    drop(s);
    show(l.first_point_index(), int);
    show(l.first_point(), |p| pnt(p.value()));
    show(l.last_point_index(), int);
    show(l.last_point(), |p| pnt(p.value()));
    let couples = l.state.read().couple.clone();
    int(couples.len() as i32);
    for (a, b) in couples {
        int(a);
        int(b);
    }
    let (v, idx) = l.tangent_vector();
    xyz(*v.xyz());
    int(idx);
    flag(l.is_tangent_at_begining());
    flag(l.is_tangent_at_end());
}

fn walk_report(w: &IntWalkIWalking<ProbeTypes>) {
    flag(w.is_done());
    attempt(|| {
        let n = w.nb_lines()?;
        int(n);
        for i in 1..=n {
            line_report(w.value(i)?);
        }
        Ok(())
    });
    attempt(|| {
        let n = w.nb_single_pnts()?;
        int(n);
        for i in 1..=n {
            let p = w.single_pnt(i)?;
            pnt(p.value());
            let (u, v) = p.value2d()?;
            real(u);
            real(v);
        }
        Ok(())
    });
    attempt(|| w.value(0).map(|_| ()));
    attempt(|| w.value(w.nb_lines()? + 1).map(|_| ()));
    attempt(|| w.single_pnt(0).map(|_| ()));
}

/// `surface quadric tol eps defl step fill reversed mode npath [...] nloop [...]`.
pub fn walk_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = surface(&mut r)?;
        let q = quadric(&mut r);
        let (tol, eps, defl, step) = (r.real(), r.real(), r.real(), r.real());
        let (fill, rev, mode) = (r.flag(), r.flag(), r.integer());
        let mut f = SurfFunction::with_surfaces(&s, &q);
        f.set_tolerance(tol);
        let np = r.integer();
        let mut paths = Vec::new();
        for _ in 0..np {
            paths.push(path_point(&mut r, &mut f)?);
        }
        let nl = r.integer();
        let mut loops = Vec::new();
        for _ in 0..nl {
            loops.push(interior_point(&mut r, &mut f)?);
        }
        let mut w = IntWalkIWalking::<ProbeTypes>::new(eps, defl, step, fill)?;
        if mode != 2 {
            attempt(|| w.perform(&paths, &loops, &mut f, &s, rev));
        }
        if mode == 1 {
            attempt(|| w.perform_open(&paths, &mut f, &s, rev));
        }
        walk_report(&w);
        Ok(())
    });
    end();
}

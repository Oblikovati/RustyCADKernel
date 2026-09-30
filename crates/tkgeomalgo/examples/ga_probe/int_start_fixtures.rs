//! The probe fixtures of tools/oracle/tkgeomalgo_probe_intstart.cxx that are not IntPatch
//! classes: the TheSOBTool whose arcs have been seen (GaProbeSeenTool) and the domain
//! whose Edge() is a real edge (GaProbeEdgeTopolTool). The IntPatch classes themselves
//! are the real ones of `rustycad_tkgeomalgo::int_patch`.
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertexBase, Adaptor3dTopolTool};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::int_patch::IntPatchHInterTool;
use rustycad_tkgeomalgo::int_start::{TheArc, TheSOBTool, TheTopolTool, TheVertex};
use rustycad_tkmath::gp::{Point2, Point3};
use std::cell::RefCell;

fn out_of_range() -> Failure {
    Failure::with_type("Standard_OutOfRange", "", "")
}

/// The known points `(x y z tol prm isvertex)` and segments of the seen arcs.
#[derive(Default)]
pub struct SeenTable {
    pub pts: Vec<[f64; 6]>,
    pub segs: Vec<(i32, i32)>,
    pub all: bool,
}

thread_local! {
    pub static SEEN: RefCell<SeenTable> = RefCell::new(SeenTable::default());
}

/// GaProbeSeenTool: TheSOBTool whose every arc has been seen.
pub struct ProbeSeenTool;

fn known(i: i32) -> Result<[f64; 6], Failure> {
    SEEN.with(|t| {
        let t = t.borrow();
        let slot = usize::try_from(i).ok().and_then(|i| i.checked_sub(1));
        slot.and_then(|k| t.pts.get(k))
            .copied()
            .ok_or_else(out_of_range)
    })
}

fn seg(i: i32) -> Result<(i32, i32), Failure> {
    SEEN.with(|t| {
        let t = t.borrow();
        let slot = usize::try_from(i).ok().and_then(|i| i.checked_sub(1));
        slot.and_then(|k| t.segs.get(k))
            .copied()
            .ok_or_else(out_of_range)
    })
}

impl TheSOBTool for ProbeSeenTool {
    fn parameter(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure> {
        IntPatchHInterTool::parameter(vertex, arc)
    }
    fn tolerance(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure> {
        IntPatchHInterTool::tolerance(vertex, arc)
    }
    fn has_been_seen(_: &TheArc) -> Result<bool, Failure> {
        Ok(true)
    }
    fn bounds(arc: &TheArc) -> Result<(f64, f64), Failure> {
        IntPatchHInterTool::bounds(arc)
    }
    fn nb_points(_: &TheArc) -> Result<i32, Failure> {
        Ok(SEEN.with(|t| t.borrow().pts.len() as i32))
    }
    fn value(_: &TheArc, index: i32) -> Result<(Point3, f64, f64), Failure> {
        let k = known(index)?;
        Ok((Point3::new(k[0], k[1], k[2]), k[3], k[4]))
    }
    fn is_vertex(_: &TheArc, index: i32) -> Result<bool, Failure> {
        Ok(known(index)?[5] != 0.0)
    }
    fn vertex(_: &TheArc, index: i32) -> Result<TheVertex, Failure> {
        let k = known(index)?;
        let v = Adaptor3dHVertexBase::new(Point2::new(k[4], 0.0), Orientation::Forward, k[3]);
        Ok(Handle::new(v).cast())
    }
    fn nb_segments(_: &TheArc) -> Result<i32, Failure> {
        Ok(SEEN.with(|t| t.borrow().segs.len() as i32))
    }
    fn has_first_point(_: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        let first: i32 = seg(index)?.0;
        Ok((first != 0).then_some(first))
    }
    fn has_last_point(_: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        let last: i32 = seg(index)?.1;
        Ok((last != 0).then_some(last))
    }
    fn is_all_solution(_: &TheArc) -> Result<bool, Failure> {
        Ok(SEEN.with(|t| t.borrow().all))
    }
}

/// GaProbeEdgeTopolTool: Adaptor3d_TopolTool whose Edge() is a real edge.
pub struct ProbeDomain {
    pub tool: Adaptor3dTopolTool,
    pub edge: Option<Edge>,
}

impl TheTopolTool for ProbeDomain {
    fn nb_samples_u(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples_u(&self.tool)
    }
    fn nb_samples_v(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples_v(&self.tool)
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples(&self.tool)
    }
    fn sample_point(&self, index: i32) -> Result<(Point2, Point3), Failure> {
        TheTopolTool::sample_point(&self.tool, index)
    }
    fn classify(&self, point: Point2, tol: f64, reframe: bool) -> Result<State, Failure> {
        TheTopolTool::classify(&self.tool, point, tol, reframe)
    }
    fn init(&self) {
        TheTopolTool::init(&self.tool);
    }
    fn more(&self) -> bool {
        TheTopolTool::more(&self.tool)
    }
    fn value(&self) -> Result<TheArc, Failure> {
        TheTopolTool::value(&self.tool)
    }
    fn next(&self) {
        TheTopolTool::next(&self.tool);
    }
    fn initialize_arc(&self, arc: &TheArc) -> Result<(), Failure> {
        self.tool.initialize_arc(arc)
    }
    fn init_vertex_iterator(&self) -> Result<(), Failure> {
        TheTopolTool::init_vertex_iterator(&self.tool)
    }
    fn more_vertex(&self) -> bool {
        TheTopolTool::more_vertex(&self.tool)
    }
    fn vertex(&self) -> Result<TheVertex, Failure> {
        TheTopolTool::vertex(&self.tool)
    }
    fn next_vertex(&self) -> Result<(), Failure> {
        TheTopolTool::next_vertex(&self.tool)
    }
    fn identical(&self, first: &TheVertex, second: &TheVertex) -> Result<bool, Failure> {
        TheTopolTool::identical(&self.tool, first, second)
    }
    fn edge(&self) -> Result<Option<Edge>, Failure> {
        Ok(self.edge.clone())
    }
    fn is_the_point_on(&self, point: Point2, tol: f64, reframe: bool) -> Result<bool, Failure> {
        TheTopolTool::is_the_point_on(&self.tool, point, tol, reframe)
    }
    fn domain_is_infinite(&self) -> Result<bool, Failure> {
        TheTopolTool::domain_is_infinite(&self.tool)
    }
    fn sample_pnts(&self, deflection: f64, nu_min: i32, nv_min: i32) -> Result<(), Failure> {
        TheTopolTool::sample_pnts(&self.tool, deflection, nu_min, nv_min)
    }
    fn u_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        TheTopolTool::u_parameters(&self.tool)
    }
    fn v_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        TheTopolTool::v_parameters(&self.tool)
    }
    fn is_uniform_sampling(&self) -> Result<bool, Failure> {
        TheTopolTool::is_uniform_sampling(&self.tool)
    }
}

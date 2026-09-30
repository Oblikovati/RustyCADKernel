// Rust translation of IntCurvesFace_Intersector.cxx (Perform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Perform members of `IntCurvesFace_Intersector`: the IntCurveSurface_HInter
//! intersection of the curve with the face surface (through the polyhedron and its
//! box grid when there is one), limited to the parameter range where the curve meets
//! the polyhedron box.
use super::intersector::IntCurvesFaceIntersector;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::int_curve_surface::{
    HCurve, HInter, TheHCurveTool, ThePolygonOfHInter, ThePolyhedronOfHInter,
};
use rustycad_tkgeomalgo::intf::Tool as IntfTool;
use rustycad_tkmath::bnd::{BoundSortBox, BoundingBox3};
use rustycad_tkmath::gp::Line3;

impl IntCurvesFaceIntersector {
    /// `Perform(L, ParMin, ParMax)`: the points of the line `line` on the face with a
    /// line parameter in `[par_min, par_max]` (RealLast bounds for an infinite line),
    /// sorted by parameter; with a polyhedron the range shrinks to the segments of the
    /// line in the polyhedron box (widened by 5%); not done when the intersector is not
    /// ready; `intersector.perform_line(&line, -1e100, 1e100)?`.
    pub fn perform_line(
        &mut self,
        line: &Line3,
        par_min: f64,
        par_max: f64,
    ) -> Result<(), Failure> {
        if !self.start_perform() {
            return Ok(());
        }
        let geomline: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(*line)).cast();
        let hll: HCurve = Handle::new(GeomAdaptorCurve::with_curve(&geomline)?).cast();
        let mut hics = HInter::new();
        let (mut parinf, mut parsup) = (par_min, par_max);
        if let Some(polyhedron) = &self.my_polyhedron {
            let Some(range) = line_range(line, polyhedron, (par_min, par_max)) else {
                return Ok(());
            };
            (parinf, parsup) = range;
            let polygon = ThePolygonOfHInter::with_range(&hll, parinf, parsup, 2)?;
            let grid: &mut BoundSortBox = polyhedron_grid(&mut self.my_bnd_bounding, polyhedron)?;
            hics.perform_polygon_polyhedron_grid(&hll, &polygon, &self.hsurface, polyhedron, grid)?;
        } else {
            hics.perform(&hll, &self.hsurface)?;
        }
        self.internal_call(&hics, parinf, parsup)
    }

    /// `Perform(HCu, ParMin, ParMax)`: the same for an Adaptor3d curve; with a
    /// polyhedron the curve range is clipped to `[par_min, par_max]` and sampled at
    /// its TheHCurveTool sample count; `intersector.perform_curve(&curve, 0.0, 1.0)?`.
    pub fn perform_curve(
        &mut self,
        curve: &HCurve,
        par_min: f64,
        par_max: f64,
    ) -> Result<(), Failure> {
        if !self.start_perform() {
            return Ok(());
        }
        let mut hics = HInter::new();
        let (mut parinf, mut parsup) = (par_min, par_max);
        if let Some(polyhedron) = &self.my_polyhedron {
            let Some(range) = curve_range(curve, (par_min, par_max))? else {
                return Ok(());
            };
            (parinf, parsup) = range;
            let nbs: i32 = TheHCurveTool::nb_samples(curve, parinf, parsup)?;
            let polygon = ThePolygonOfHInter::with_range(curve, parinf, parsup, nbs)?;
            let grid: &mut BoundSortBox = polyhedron_grid(&mut self.my_bnd_bounding, polyhedron)?;
            hics.perform_polygon_polyhedron_grid(
                curve,
                &polygon,
                &self.hsurface,
                polyhedron,
                grid,
            )?;
        } else {
            hics.perform(curve, &self.hsurface)?;
        }
        self.internal_call(&hics, parinf, parsup)
    }

    // Resets the results; false (not done) when the intersector is not ready.
    fn start_perform(&mut self) -> bool {
        self.done = false;
        if !self.my_ready {
            return false;
        }
        self.done = true;
        self.seq_pnt.clear();
        self.my_seq_state.clear();
        self.nbpnt = 0;
        true
    }
}

// `myBndBounding`: the box grid of the polyhedron triangles, built on first use (the
// native file is compiled with OPTIMISATION, so the grid is always used).
fn polyhedron_grid<'a>(
    slot: &'a mut Option<BoundSortBox>,
    polyhedron: &ThePolyhedronOfHInter,
) -> Result<&'a mut BoundSortBox, Failure> {
    if slot.is_none() {
        let mut grid = BoundSortBox::default();
        let components = polyhedron.components_bounding().clone();
        grid.initialize_with_enclosing(polyhedron.bounding(), components)?;
        *slot = Some(grid);
    }
    Ok(slot.as_mut().expect("grid built above"))
}

// The curve range of Perform(HCu) with a polyhedron, clipped to [par_min, par_max];
// None when shorter than 1e-9.
fn curve_range(
    curve: &HCurve,
    (par_min, par_max): (f64, f64),
) -> Result<Option<(f64, f64)>, Failure> {
    let mut parinf: f64 = TheHCurveTool::first_parameter(curve)?;
    let mut parsup: f64 = TheHCurveTool::last_parameter(curve)?;
    if parinf < par_min {
        parinf = par_min;
    }
    if parsup > par_max {
        parsup = par_max;
    }
    if parinf > (parsup - 1e-9) {
        return Ok(None);
    }
    Ok(Some((parinf, parsup)))
}

// The line parameters of Perform(L) with a polyhedron: the union of the segments of
// the line in the polyhedron box, each widened by 5% (and by 1e-10 when shorter than
// 1e-10), clipped to [par_min, par_max]; None when nothing remains.
fn line_range(
    line: &Line3,
    polyhedron: &ThePolyhedronOfHInter,
    (par_min, par_max): (f64, f64),
) -> Option<(f64, f64)> {
    let mut bnd_tool = IntfTool::new();
    let mut box_line = BoundingBox3::default();
    bnd_tool.lin_box(*line, polyhedron.bounding(), &mut box_line);
    let nb_segments: i32 = bnd_tool.nb_segments();
    if nb_segments == 0 {
        return None;
    }
    let (mut parinf, mut parsup) = (par_min, par_max);
    for nbseg in 1..=nb_segments {
        let (pinf, psup) = widened_segment(&bnd_tool, nbseg);
        if nbseg == 1 {
            (parinf, parsup) = (pinf, psup);
            continue;
        }
        if parinf > pinf {
            parinf = pinf;
        }
        if parsup < psup {
            parsup = psup;
        }
    }
    clipped(parinf, parsup, (par_min, par_max))
}

// One segment of the Intf_Tool, widened by 5% of its length (1e-10 more on each side
// when it stays shorter than 1e-10).
fn widened_segment(bnd_tool: &IntfTool, nbseg: i32) -> (f64, f64) {
    let mut pinf: f64 = bnd_tool
        .begin_param(nbseg)
        .expect("segment index within NbSegments");
    let mut psup: f64 = bnd_tool
        .end_param(nbseg)
        .expect("segment index within NbSegments");
    let pppp: f64 = 0.05 * (psup - pinf);
    pinf -= pppp;
    psup += pppp;
    if (psup - pinf) < 1e-10 {
        pinf -= 1e-10;
        psup += 1e-10;
    }
    (pinf, psup)
}

// The range clipped to [par_min, par_max]; None when it misses it or is shorter than
// 1e-9.
fn clipped(mut parinf: f64, mut parsup: f64, (par_min, par_max): (f64, f64)) -> Option<(f64, f64)> {
    if parinf > par_max || parsup < par_min {
        return None;
    }
    if parinf < par_min {
        parinf = par_min;
    }
    if parsup > par_max {
        parsup = par_max;
    }
    if parinf > (parsup - 1e-9) {
        return None;
    }
    Some((parinf, parsup))
}

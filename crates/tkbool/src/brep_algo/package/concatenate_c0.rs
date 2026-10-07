// Rust translation of BRepAlgo::ConcatenateWireC0
// (ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The single edge of a wire: its consecutive edges on the same curve merged, the
//! others concatenated in C1 then joined into one B-spline.
use super::concatenate::null_curve;
use super::same_curve::{SameCurveTolerances, same_conic};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::{self, Edge, Vertex, Wire};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_convert::{self, CompCurveToBSplineCurve, HArrayOfBSplineCurves};
use rustycad_tkgeombase::geom_lprop;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tkshhealing::shape_fix::ShapeFixShape;
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeEdge};

/// The curves of the merged edges: each curve with its range, its sense and the
/// tolerance of the vertex before each next curve.
#[derive(Default)]
struct CurveChain {
    curves: Vec<Handle<dyn GeomCurve>>,
    fpars: Vec<f64>,
    lpars: Vec<f64>,
    tols: Vec<f64>,
    is_fwd: Vec<bool>,
    cur_type: Option<CurveType>,
    first_vertex: Vertex,
}

/// The parameters on the previous periodic curve of a same curve: ordered by the
/// senses and adjusted after (or before) the previous range.
fn periodic_parameters(
    chain: &CurveChain,
    a_period: f64,
    is_same_dir: bool,
    (mut new_fpar, mut new_lpar): (f64, f64),
) -> (f64, f64) {
    if is_same_dir {
        // The first parameter must not be greater than the last one.
        while new_fpar > new_lpar {
            new_fpar -= a_period;
        }
    } else {
        // The last parameter must not be greater than the first one.
        while new_lpar > new_fpar {
            new_lpar -= a_period;
        }
        (new_fpar, new_lpar) = (new_lpar, new_fpar);
    }
    let (prev_f, prev_l) = (
        chain.fpars[chain.fpars.len() - 1],
        chain.lpars[chain.lpars.len() - 1],
    );
    if chain.is_fwd[chain.is_fwd.len() - 1] {
        // The current curve is after the previous one.
        elclib::adjust_periodic(
            prev_l,
            prev_l + a_period,
            precision::p_confusion(),
            new_fpar,
            new_lpar,
        )
    } else {
        // The current curve is before the previous one.
        elclib::adjust_periodic(
            prev_f - a_period,
            prev_f,
            precision::p_confusion(),
            new_fpar,
            new_lpar,
        )
    }
}

impl CurveChain {
    /// One edge of the wire: merged into the last curve when it lies on it, else
    /// appended (with the tolerance of the vertex between them).
    fn push(
        &mut self,
        an_edge: &Edge,
        is_fwd: bool,
        current_vertex: &Vertex,
    ) -> Result<(), Failure> {
        let (a_curve, mut fpar, mut lpar) = BRepTool::curve(an_edge)?;
        if a_curve.is_null() {
            return Ok(());
        }
        let a_ga_curve = GeomAdaptorCurve::with_curve(&a_curve)?;
        let a_type: CurveType = a_ga_curve.get_type()?;
        let a_basis_curve = a_ga_curve.curve();
        let basis = a_basis_curve.get().ok_or_else(null_curve)?;
        if basis.is_periodic() {
            (fpar, lpar) = elclib::adjust_periodic(
                basis.first_parameter(),
                basis.last_parameter(),
                precision::p_confusion(),
                fpar,
                lpar,
            );
        }
        if self.curves.is_empty() {
            self.append(&a_curve, (fpar, lpar), is_fwd, None);
            self.cur_type = Some(a_type);
            self.first_vertex = current_vertex.clone();
            return Ok(());
        }
        let Some(range) = self.same_curve(&a_curve, &a_ga_curve, a_type, (fpar, lpar))? else {
            self.append(
                &a_curve,
                (fpar, lpar),
                is_fwd,
                Some(BRepTool::vertex_tolerance(current_vertex)?),
            );
            self.cur_type = Some(a_type);
            return Ok(());
        };
        self.merge(basis, is_fwd, range)
    }

    /// The parameters on the last curve of a curve on it (the same handle, or the
    /// same conic or line), `None` otherwise.
    fn same_curve(
        &self,
        a_curve: &Handle<dyn GeomCurve>,
        a_ga_curve: &GeomAdaptorCurve,
        a_type: CurveType,
        (fpar, lpar): (f64, f64),
    ) -> Result<Option<(f64, f64)>, Failure> {
        let last = &self.curves[self.curves.len() - 1];
        let ga_prev_curve = GeomAdaptorCurve::with_curve(last)?;
        if a_curve.object_address() == last.object_address() {
            return Ok(Some((fpar, lpar)));
        }
        if Some(a_type) != self.cur_type {
            return Ok(None);
        }
        let t = SameCurveTolerances {
            lin_tol: precision::confusion(),
            ang_tol: precision::angular(),
        };
        same_conic(a_type, a_ga_curve, &ga_prev_curve, (fpar, lpar), &t)
    }

    /// The range of the last curve extended by a same curve.
    fn merge(
        &mut self,
        basis: &dyn GeomCurve,
        is_fwd: bool,
        (mut new_fpar, mut new_lpar): (f64, f64),
    ) -> Result<(), Failure> {
        let last_fwd: bool = self.is_fwd[self.is_fwd.len() - 1];
        let is_same_dir: bool = is_fwd == last_fwd;
        if basis.is_periodic() {
            (new_fpar, new_lpar) =
                periodic_parameters(self, basis.period()?, is_same_dir, (new_fpar, new_lpar));
        } else if !is_same_dir {
            // Not periodic curves in opposite directions.
            (new_fpar, new_lpar) = (new_lpar, new_fpar);
        }
        if last_fwd {
            let n = self.lpars.len();
            self.lpars[n - 1] = new_lpar;
        } else {
            let n = self.fpars.len();
            self.fpars[n - 1] = new_fpar;
        }
        Ok(())
    }

    /// A new curve.
    fn append(
        &mut self,
        a_curve: &Handle<dyn GeomCurve>,
        (fpar, lpar): (f64, f64),
        is_fwd: bool,
        tol: Option<f64>,
    ) {
        self.curves.push(a_curve.clone());
        self.fpars.push(fpar);
        self.lpars.push(lpar);
        self.is_fwd.push(is_fwd);
        if let Some(tol) = tol {
            self.tols.push(tol);
        }
    }
}

/// The basis of a trimmed curve, else the curve.
fn untrimmed(c: &Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    let t: Handle<GeomTrimmedCurve> = c.cast();
    t.get().map_or_else(|| c.clone(), |t| t.basis_curve())
}

/// An array error as a native exception.
fn array_failure(e: impl std::fmt::Debug) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepAlgo: {e:?}; expected an index in the array bounds"),
        "NCollection_Array1",
    )
}

/// The wire with 3d curves fixed by ShapeFix_Shape at the confusion precision.
fn fixed_wire(a_wire: &Wire) -> Result<Wire, Failure> {
    brep_lib::build_curves3d_default(a_wire.as_ref())?;
    let fixer = ShapeFixShape::new(a_wire.as_ref().clone());
    fixer.set_precision(precision::confusion());
    fixer.set_max_tolerance(precision::confusion());
    fixer.perform()?;
    topods::wire(&fixer.shape())
}

/// `BRepAlgo::ConcatenateWireC0(Wire)`: the edge of the curves of the wire, the
/// consecutive same curves merged, the others concatenated in C1 and joined within
/// five times the greatest vertex tolerance; a null edge without curve;
/// `brep_algo::concatenate_wire_c0(&wire)?`.
pub fn concatenate_wire_c0(a_wire: &Wire) -> Result<Edge, Failure> {
    let the_wire = fixed_wire(a_wire)?;
    let mut chain = CurveChain::default();
    let mut wexp = WireExplorer::with_wire(&the_wire)?;
    while wexp.more() {
        let an_edge = wexp.current().clone();
        let is_fwd: bool = wexp.orientation()? != Orientation::Reversed;
        let current_vertex = wexp.current_vertex().clone();
        chain.push(&an_edge, is_fwd, &current_vertex)?;
        wexp.next()?;
    }
    let last_vertex: Vertex = wexp.current_vertex().clone();
    chain.tols.push(BRepTool::vertex_tolerance(&last_vertex)?);
    let is_reverse: bool = chain.is_fwd.first().is_some_and(|fwd| !fwd);
    let (mut first_final, mut last_final) = if is_reverse {
        (last_vertex.clone(), chain.first_vertex.clone())
    } else {
        (chain.first_vertex.clone(), last_vertex.clone())
    };
    first_final.set_orientation(Orientation::Forward);
    last_final.set_orientation(Orientation::Reversed);
    if chain.curves.is_empty() {
        return Ok(Edge::default());
    }
    let mut res_edge: Edge = if chain.curves.len() > 1 {
        concatenated_edge(
            &mut chain,
            &last_vertex,
            is_reverse,
            (&first_final, &last_final),
        )?
    } else {
        let basis = untrimmed(&chain.curves[0]);
        let a_copy_curve: Handle<dyn GeomCurve> =
            basis.get().ok_or_else(null_curve)?.copy()?.cast();
        BRepLibMakeEdge::from_curve_vertices_params(
            &a_copy_curve,
            &first_final,
            &last_final,
            chain.fpars[0],
            chain.lpars[0],
        )?
        .edge()?
    };
    if is_reverse {
        res_edge.reverse();
    }
    Ok(res_edge)
}

/// The B-splines of the curves (C1, reversed against the wire) and five times the
/// tolerances of the vertices after each.
fn chain_bsplines(
    chain: &mut CurveChain,
) -> Result<
    (
        Array1<'static, Handle<GeomBSplineCurve>>,
        Array1<'static, f64>,
    ),
    Failure,
> {
    let nb_curve = chain.curves.len() as i32;
    let mut tab: Array1<'static, Handle<GeomBSplineCurve>> =
        Array1::with_bounds(0, nb_curve - 1).map_err(array_failure)?;
    let mut tabtolvertex: Array1<'static, f64> =
        Array1::with_bounds(0, nb_curve - 1).map_err(array_failure)?;
    for i in 1..=nb_curve {
        let k = (i - 1) as usize;
        chain.curves[k] = untrimmed(&chain.curves[k]);
        let a_tr_curve: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(
            &chain.curves[k],
            chain.fpars[k],
            chain.lpars[k],
            true,
            true,
        )?)
        .cast();
        let mut bs =
            geom_convert::curve_to_bspline_curve(&a_tr_curve, ParameterisationType::TgtThetaOver2)?;
        geom_convert::c0_bspline_to_c1_bspline_curve(&mut bs, precision::confusion(), 1.0e-7)?;
        if !chain.is_fwd[k] {
            bs.get().ok_or_else(null_curve)?.reverse()?;
        }
        tab[i - 1] = bs;
        if i > 1 {
            tabtolvertex[i - 2] = chain.tols[(i - 2) as usize] * 5.;
        }
    }
    tabtolvertex[nb_curve - 1] = chain.tols[chain.tols.len() - 1] * 5.;
    Ok((tab, tabtolvertex))
}

/// The edge of the C1 concatenation of the curves (closed in G1 on a closed wire),
/// its pieces joined into one B-spline.
fn concatenated_edge(
    chain: &mut CurveChain,
    last_vertex: &Vertex,
    is_reverse: bool,
    (first_final, last_final): (&Vertex, &Vertex),
) -> Result<Edge, Failure> {
    let (mut tab, tabtolvertex) = chain_bsplines(chain)?;
    let nb_curve = chain.curves.len() as i32;
    let (mut closed_flag, mut closed_tolerance) = (false, 0.);
    if chain.first_vertex.is_same(last_vertex) {
        let (c0, cn): (Handle<dyn GeomCurve>, Handle<dyn GeomCurve>) =
            (tab[0].clone().cast(), tab[nb_curve - 1].clone().cast());
        let u0 = c0.get().ok_or_else(null_curve)?.first_parameter();
        let un = cn.get().ok_or_else(null_curve)?.last_parameter();
        if geom_lprop::continuity_with_tolerances(
            &c0,
            &cn,
            u0,
            un,
            false,
            false,
            precision::confusion(),
            precision::angular(),
        )? >= Continuity::G1
        {
            closed_flag = true;
            closed_tolerance = BRepTool::vertex_tolerance(&chain.first_vertex)?;
        }
    }
    let mut concatcurve: HArrayOfBSplineCurves = Handle::default();
    let mut array_of_indices: Handle<HArray1<i32>> = Handle::default();
    geom_convert::concat_c1(
        &mut tab,
        &tabtolvertex,
        &mut array_of_indices,
        &mut concatcurve,
        &mut closed_flag,
        closed_tolerance,
    )?;
    let curves = concatcurve.get().ok_or_else(null_curve)?;
    let (lower, upper) = (curves.array1().lower(), curves.array1().upper());
    if upper - lower + 1 > 1 {
        let max_tol_ver: f64 = chain
            .tols
            .iter()
            .fold(precision::confusion(), |m, t| if *t > m { *t } else { m })
            * 5.;
        let first: Handle<dyn GeomBoundedCurve> = curves
            .array1()
            .value(lower)
            .map_err(array_failure)?
            .clone()
            .cast();
        let mut concat =
            CompCurveToBSplineCurve::with_curve(&first, ParameterisationType::TgtThetaOver2)?;
        for i in lower + 1..=upper {
            let next: Handle<dyn GeomBoundedCurve> = curves
                .array1()
                .value(i)
                .map_err(array_failure)?
                .clone()
                .cast();
            concat.add(&next, max_tol_ver, true, true, 0)?;
        }
        *curves
            .change_array1()
            .change_value(lower)
            .map_err(array_failure)? = concat.bspline_curve();
    }
    let result = curves.array1().value(lower).map_err(array_failure)?.clone();
    let bs = result.get().ok_or_else(null_curve)?;
    if is_reverse {
        bs.reverse()?;
    }
    let (f, l) = (bs.first_parameter(), bs.last_parameter());
    BRepLibMakeEdge::from_curve_vertices_params(
        &result.clone().cast(),
        first_final,
        last_final,
        f,
        l,
    )?
    .edge()
}

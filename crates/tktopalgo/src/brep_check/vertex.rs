// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Vertex.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Vertex`: the checker of a vertex, whose point representations must lie
//! within its tolerance of its point.
use super::explore::holds_same;
use super::result::{BRepCheckResult, ResultBase, SharedStatusList, add_status};
use super::status::BRepCheckStatus;
use rustycad_tkbrep::brep::access::{edge_tshape, face_tshape, vertex_tshape};
use rustycad_tkbrep::brep::{
    BRepCurveRepresentation, BRepGCurve, BRepPointRepresentation, BRepTVertex, BRepTool,
};
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Vertex, edge, face, vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

/// `BRepCheck_Vertex`: checks that the point representations of a vertex on the
/// curves of an edge, or on the surface of a face, lie within the tolerance.
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_check::{BRepCheckResult, BRepCheckStatus, BRepCheckVertex};
/// let mut v = Vertex::default();
/// BRepBuilder::default().make_vertex_at(&mut v, Point3::new(0.0, 0.0, 0.0), 1e-7).unwrap();
/// let checker = BRepCheckVertex::new(&v).unwrap();
/// assert_eq!(checker.status().unwrap().first(), Some(&BRepCheckStatus::NoError));
/// ```
pub struct BRepCheckVertex {
    transient: StandardTransient,
    base: ResultBase,
}
impl_transient!(BRepCheckVertex, "BRepCheck_Vertex", dyn BRepCheckResult, transient,
    interfaces: [dyn BRepCheckResult]);

/// The frame of one InContext check: the vertex point, the squared tolerance, the list.
struct VertexCheck<'a> {
    point: Point3,
    tol2: f64,
    list: &'a SharedStatusList,
    tvertex: &'a BRepTVertex,
}

impl VertexCheck<'_> {
    /// Adds `status` when `control`, moved by `location`, is beyond the tolerance.
    fn control(&self, mut control: Point3, location: &Location, status: BRepCheckStatus) {
        control.transform(location.transformation());
        if self.point.square_distance(&control) > self.tol2 {
            add_status(self.list, status);
        }
    }

    /// The point representations of the vertex.
    fn points(&self) -> Vec<Handle<dyn BRepPointRepresentation>> {
        self.tvertex.points()
    }
}

impl BRepCheckVertex {
    /// `BRepCheck_Vertex(V)`: `Init(V)`, so the minimum check is done;
    /// `BRepCheckVertex::new(&vertex)?`.
    pub fn new(vertex: &Vertex) -> Result<Self, Failure> {
        let checker = Self {
            transient: StandardTransient::default(),
            base: ResultBase::new(),
        };
        checker.init(vertex)?;
        Ok(checker)
    }

    /// `Tolerance()`: the square root of 1.05 times the largest squared distance from
    /// the vertex point to its representations (the tolerance squared at least). A
    /// point representation on a null curve keeps the previous control point, and
    /// every control point is moved by its representation location, as natively;
    /// `checker.tolerance()?`.
    pub fn tolerance(&self) -> Result<f64, Failure> {
        let shape = self.base.shape();
        let tvertex = vertex_tshape(&shape, "BRepCheck_Vertex::Tolerance")?;
        let point: Point3 = tvertex.pnt();
        let mut tol: f64 = BRepTool::vertex_tolerance(&vertex(&shape)?)?;
        tol *= tol;
        let mut control: Point3 = point;
        for representation in tvertex.points() {
            let Some(pr) = representation.get() else {
                continue;
            };
            control = representation_point(pr, control)?;
            control.transform(pr.location().transformation());
            let distance: f64 = point.square_distance(&control);
            if distance > tol {
                tol = distance;
            }
        }
        Ok((tol * 1.05).sqrt())
    }

    /// InContext in an edge: the representations on each curve of the edge.
    fn check_on_edge(&self, context: &Edge, list: &SharedStatusList) -> Result<(), Failure> {
        let shape = self.base.shape();
        let (found, multiple) = find_on_edge(context, &shape)?;
        let tol: f64 =
            BRepTool::vertex_tolerance(&vertex(&shape)?)?.max(BRepTool::edge_tolerance(context)?);
        let tvertex = vertex_tshape(&shape, "BRepCheck_Vertex::InContext")?;
        let check = VertexCheck {
            point: tvertex.pnt(),
            tol2: tol * tol,
            list,
            tvertex,
        };
        let edge_location = context.location();
        let tedge = edge_tshape(context, "BRepCheck_Vertex::InContext")?;
        for cr in tedge.curves() {
            let Some(r) = cr.get() else {
                continue;
            };
            let location = (edge_location * &r.location())?.predivided(shape.location())?;
            if r.is_curve3d() {
                check_curve3d(&check, &cr, &location, (found, multiple))?;
            } else if r.is_curve_on_surface() {
                check_curve_on_surface(&check, r, &location)?;
            }
        }
        super::result::close_status_list(list);
        Ok(())
    }

    /// InContext in a face: the representations on the surface of the face.
    fn check_on_face(&self, context: &Face, list: &SharedStatusList) -> Result<(), Failure> {
        let shape = self.base.shape();
        let tface = face_tshape(context, "BRepCheck_Vertex::InContext")?;
        let surface = tface.surface();
        let location = (context.location() * &tface.location())?.predivided(shape.location())?;
        let tol: f64 =
            BRepTool::vertex_tolerance(&vertex(&shape)?)?.max(BRepTool::face_tolerance(context)?);
        let tvertex = vertex_tshape(&shape, "BRepCheck_Vertex::InContext")?;
        let check = VertexCheck {
            point: tvertex.pnt(),
            tol2: tol * tol,
            list,
            tvertex,
        };
        for representation in check.points() {
            let Some(pr) = representation.get() else {
                continue;
            };
            if pr.is_point_on_surface_at(&surface, &location) {
                let control = geom_surface(&surface)?.value(pr.parameter(), pr.parameter2()?)?;
                check.control(control, &location, BRepCheckStatus::InvalidPointOnSurface);
            }
        }
        super::result::close_status_list(list);
        Ok(())
    }
}

/// The occurrence of `shape` in the FORWARD edge: the one with a FORWARD or REVERSED
/// orientation when there are several, and whether it is both FORWARD and REVERSED.
fn find_on_edge(context: &Edge, shape: &Shape) -> Result<(Orientation, bool), Failure> {
    let mut found: Option<Shape> = None;
    let mut multiple: bool = false;
    let forward = context.oriented(Orientation::Forward);
    let mut iterator = ShapeIterator::new(&forward, true, true);
    while let Ok(value) = iterator.value() {
        let vf = value.clone();
        iterator.next();
        if !vf.is_same(shape) {
            continue;
        }
        let Some(current) = found.as_ref() else {
            found = Some(vf);
            continue;
        };
        let (a, b) = (current.orientation(), vf.orientation());
        if matches!(
            (a, b),
            (Orientation::Forward, Orientation::Reversed)
                | (Orientation::Reversed, Orientation::Forward)
        ) {
            multiple = true;
        }
        let current_oriented = matches!(a, Orientation::Forward | Orientation::Reversed);
        let new_oriented = matches!(b, Orientation::Forward | Orientation::Reversed);
        if !current_oriented && new_oriented {
            found = Some(vf);
        }
    }
    // "VFind is not null for sure": the explorer found the vertex in the edge.
    let orientation = found.map_or(Orientation::Forward, |v| v.orientation());
    Ok((orientation, multiple))
}

/// The points on a 3d curve representation, and the ends the vertex orientation names.
fn check_curve3d(
    check: &VertexCheck<'_>,
    cr: &Handle<dyn BRepCurveRepresentation>,
    location: &Location,
    (orientation, multiple): (Orientation, bool),
) -> Result<(), Failure> {
    let r = cr.get().expect("checked representation");
    let curve = r.curve3d()?;
    let Some(c) = curve.get() else {
        return Ok(());
    };
    for representation in check.points() {
        let Some(pr) = representation.get() else {
            continue;
        };
        if pr.is_point_on_curve_at(&curve, location) {
            let control = c.value(pr.parameter())?;
            check.control(control, location, BRepCheckStatus::InvalidPointOnCurve);
        }
    }
    if !matches!(orientation, Orientation::Forward | Orientation::Reversed) {
        return Ok(());
    }
    let gcurve = cr.cast::<dyn BRepGCurve>();
    let gc = gcurve.get().ok_or_else(|| {
        super::errors::null_object("BRepCheck_Vertex::InContext: 3d curve is not a BRep_GCurve")
    })?;
    if orientation == Orientation::Forward || multiple {
        check.control(
            c.value(gc.first())?,
            location,
            BRepCheckStatus::InvalidPointOnCurve,
        );
    }
    if orientation == Orientation::Reversed || multiple {
        check.control(
            c.value(gc.last())?,
            location,
            BRepCheckStatus::InvalidPointOnCurve,
        );
    }
    Ok(())
}

/// The points on the pcurves of a curve on surface representation.
fn check_curve_on_surface(
    check: &VertexCheck<'_>,
    r: &dyn BRepCurveRepresentation,
    location: &Location,
) -> Result<(), Failure> {
    let surface = r.surface()?;
    let pcurve = r.pcurve()?;
    let pcurve2 = if r.is_curve_on_closed_surface() {
        r.pcurve2()?
    } else {
        Handle::default()
    };
    for representation in check.points() {
        let Some(pr) = representation.get() else {
            continue;
        };
        for pc in [&pcurve, &pcurve2] {
            if pc.is_null() || !pr.is_point_on_curve_on_surface_at(pc, &surface, location) {
                continue;
            }
            let p2d = geom2d_curve(pc)?.value(pr.parameter())?;
            let control = geom_surface(&surface)?.value(p2d.x(), p2d.y())?;
            check.control(
                control,
                location,
                BRepCheckStatus::InvalidPointOnCurveOnSurface,
            );
        }
    }
    Ok(())
}

/// The control point of one representation in `Tolerance()`; a point on a null
/// curve keeps `previous`.
fn representation_point(
    pr: &dyn BRepPointRepresentation,
    previous: Point3,
) -> Result<Point3, Failure> {
    if pr.is_point_on_curve() {
        let curve = pr.curve()?;
        return match curve.get() {
            Some(c) => c.value(pr.parameter()),
            None => Ok(previous),
        };
    }
    if pr.is_point_on_curve_on_surface() {
        let uv = geom2d_curve(&pr.pcurve()?)?.value(pr.parameter())?;
        return geom_surface(&pr.surface()?)?.value(uv.x(), uv.y());
    }
    if pr.is_point_on_surface() {
        return geom_surface(&pr.surface()?)?.value(pr.parameter(), pr.parameter2()?);
    }
    Ok(previous)
}

/// The surface of a handle; Standard_NullObject for a null one (a crash natively).
fn geom_surface(
    surface: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
) -> Result<&dyn rustycad_tkg3d::geom::GeomSurface, Failure> {
    surface
        .get()
        .ok_or_else(|| super::errors::null_object("BRepCheck_Vertex: null surface"))
}

/// The pcurve of a handle; Standard_NullObject for a null one (a crash natively).
fn geom2d_curve(
    curve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) -> Result<&dyn rustycad_tkg2d::geom2d::Geom2dCurve, Failure> {
    curve
        .get()
        .ok_or_else(|| super::errors::null_object("BRepCheck_Vertex: null pcurve"))
}

impl BRepCheckResult for BRepCheckVertex {
    fn result_base(&self) -> &ResultBase {
        &self.base
    }

    /// `InContext(S)`: `SubshapeNotInShape` when the vertex is not in `context`, else
    /// the point checks on the edge or face `context` (other types record nothing).
    fn in_context(&self, context: &Shape) -> Result<(), Failure> {
        let Some(list) = self.base.bind_context_list(context) else {
            return Ok(());
        };
        let shape = self.base.shape();
        if !holds_same(context, ShapeEnum::Vertex, &shape)? {
            add_status(&list, BRepCheckStatus::SubshapeNotInShape);
            return Ok(());
        }
        match context.shape_type()? {
            ShapeEnum::Edge => self.check_on_edge(&edge(context)?, &list),
            ShapeEnum::Face => self.check_on_face(&face(context)?, &list),
            _ => Ok(()),
        }
    }

    /// `Minimum()`: the vertex itself has no defect (`NoError`).
    fn minimum(&self) -> Result<(), Failure> {
        if self.base.is_minimum() {
            return Ok(());
        }
        let list = self.base.bind_own_list();
        list.write().append(BRepCheckStatus::NoError);
        self.base.set_minimum(true);
        Ok(())
    }

    /// `Blind()`: marks the vertex checked (the native body is commented out).
    fn blind(&self) {
        self.base.set_blind(true);
    }
}

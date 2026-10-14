//! AIS attribute, type, signature, exclusion, bad edge and C0 regularity filters and
//! AIS_GraphicTool, the twin of tools/oracle/tkv3d_probe_ais_attribute_filters.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomCircle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Axis3, Cylinder3, Direction3, Point3, Sphere3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
};
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::select_mgr::{
    SelectMgrEntityOwner, SelectMgrEntityOwnerSource, SelectMgrFilter, SelectMgrSelectableObject,
};
use rustycad_tkv3d::std_select::StdSelectBRepOwner;

const KINDS: [AisKindOfInteractive; 7] = [
    AisKindOfInteractive::None,
    AisKindOfInteractive::Datum,
    AisKindOfInteractive::Shape,
    AisKindOfInteractive::Object,
    AisKindOfInteractive::Relation,
    AisKindOfInteractive::Dimension,
    AisKindOfInteractive::LightSource,
];
const ATTRIBUTES: [AisTypeOfAttribute; 16] = [
    AisTypeOfAttribute::Line,
    AisTypeOfAttribute::Dimension,
    AisTypeOfAttribute::Wire,
    AisTypeOfAttribute::Plane,
    AisTypeOfAttribute::Vector,
    AisTypeOfAttribute::UIso,
    AisTypeOfAttribute::VIso,
    AisTypeOfAttribute::Free,
    AisTypeOfAttribute::UnFree,
    AisTypeOfAttribute::Section,
    AisTypeOfAttribute::Hidden,
    AisTypeOfAttribute::Seen,
    AisTypeOfAttribute::FaceBoundary,
    AisTypeOfAttribute::FirstAxis,
    AisTypeOfAttribute::SecondAxis,
    AisTypeOfAttribute::ThirdAxis,
];

fn kind(value: i32) -> AisKindOfInteractive {
    KINDS[value.clamp(0, 6) as usize]
}

fn object(kind: i32) -> Handle<dyn SelectMgrSelectableObject> {
    match kind {
        0 => AisPoint::new(&Handle::new(GeomCartesianPoint::new(1., 2., 3.)).cast())
            .unwrap()
            .cast(),
        1 => {
            let axes = Axis2::with_x_direction(
                Point3::new(0., 0., 0.),
                Direction3::new(0., 0., 1.).unwrap(),
                Direction3::new(1., 0., 0.).unwrap(),
            )
            .unwrap();
            AisCircle::new(&Handle::new(GeomCircle::new(axes, 2.).unwrap())).cast()
        }
        2 => {
            let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(0., 0., 0.)).unwrap();
            AisColoredShape::new(&Shape::from(vertex.vertex().unwrap())).cast()
        }
        _ => Handle::default(),
    }
}

fn shape(kind: i32) -> Result<Shape, Failure> {
    let ax = Axis3::new(
        Point3::new(0., 0., 0.),
        Direction3::new(0., 0., 1.)?,
        Direction3::new(1., 0., 0.)?,
    )?;
    Ok(match kind {
        0 => Shape::from(
            BRepBuilderAPIMakeFace::from_cylinder_bounds(
                Cylinder3::new(ax, 2.)?,
                (0., 2. * std::f64::consts::PI, 0., 1.),
            )?
            .face()?,
        ),
        1 => Shape::from(BRepBuilderAPIMakeFace::from_sphere(Sphere3::new(ax, 1.)?)?.face()?),
        _ => Shape::from(
            BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))?
                .edge()?,
        ),
    })
}

fn sub_shapes(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .collect()
}

fn brep_owner(shape: &Shape) -> Handle<dyn SelectMgrEntityOwnerSource> {
    StdSelectBRepOwner::with_shape(shape, 0, false).into_cast()
}

fn list(values: &[i32]) {
    int(values.len() as i32);
    for value in values {
        int(*value);
    }
}

/// `v3_ais_attr_filter Op Object Color Width Values...`.
pub fn attribute_filter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (op, object_kind, color) = (r.integer(), r.integer(), r.integer());
    let width = r.real();
    let selectable = object(object_kind);
    if let Some(object) = selectable.get() {
        if color >= 0 {
            object
                .set_color(Color::named(NamedColor::from_index(color).unwrap()))
                .unwrap();
        }
        if width > 0. {
            object.set_width(width).unwrap();
        }
    }
    let owner: Handle<dyn SelectMgrEntityOwnerSource> =
        SelectMgrEntityOwner::with_selectable(&selectable, 0).into_cast();
    match op {
        0 => flag(AisTypeFilter::new(kind(r.integer())).is_ok(&owner).unwrap()),
        1 => {
            let (k, sig) = (r.integer(), r.integer());
            flag(AisSignatureFilter::new(kind(k), sig).is_ok(&owner).unwrap());
        }
        2 => {
            let (mode, col, wid) = (r.integer(), r.integer(), r.real());
            let filter = match mode {
                0 => AisAttributeFilter::default(),
                1 => AisAttributeFilter::with_color(NamedColor::from_index(col).unwrap()),
                _ => AisAttributeFilter::with_width(wid),
            };
            for _ in 0..r.integer() {
                match r.integer() {
                    0 => filter.set_color(NamedColor::from_index(r.integer()).unwrap()),
                    1 => filter.set_width(r.real()),
                    2 => filter.unset_color(),
                    _ => filter.unset_width(),
                }
            }
            flag(filter.has_color());
            flag(filter.has_width());
            flag(filter.is_ok(&owner).unwrap());
        }
        3 => {
            let (ctor, k, sig, on) = (r.integer(), r.integer(), r.integer(), r.flag());
            let filter = match ctor {
                0 => AisExclusionFilter::new(on),
                1 => AisExclusionFilter::with_type(kind(k), on),
                _ => AisExclusionFilter::with_signature(kind(k), sig, on),
            };
            for _ in 0..r.integer() {
                let (code, k, s) = (r.integer(), kind(r.integer()), r.integer());
                match code {
                    0 => flag(filter.add(k)),
                    1 => flag(filter.add_signature(k, s)),
                    2 => flag(filter.remove(k)),
                    3 => flag(filter.remove_signature(k, s)),
                    4 => filter.clear(),
                    _ => filter.set_exclusion_flag(s != 0),
                }
            }
            flag(filter.is_exclusion_flag_on());
            flag(filter.is_ok(&owner).unwrap());
            for k in KINDS {
                flag(filter.is_stored(k));
                list(&filter.list_of_signature(k));
            }
            list(&filter.list_of_stored_types());
        }
        4 => {
            let drawer = Handle::new(Prs3dDrawer::new());
            drawer.setup_own_defaults().unwrap();
            let attribute = ATTRIBUTES[r.integer().clamp(0, 15) as usize];
            attempt(|| AisGraphicTool::get_line_color(&drawer, attribute).map(|c| int(c.index())));
            attempt(|| {
                AisGraphicTool::get_line_color_value(&drawer, attribute).map(|c| {
                    real(c.red());
                    real(c.green());
                    real(c.blue());
                })
            });
            attempt(|| AisGraphicTool::get_line_width(&drawer, attribute).map(real));
            attempt(|| AisGraphicTool::get_line_type(&drawer, attribute).map(|t| int(t as i32)));
            attempt(|| {
                AisGraphicTool::get_line_att(&drawer, attribute).map(|(n, w, t)| {
                    int(n.index());
                    real(w);
                    int(t as i32);
                })
            });
            attempt(|| AisGraphicTool::get_interior_color(&drawer).map(|c| int(c.index())));
            attempt(|| AisGraphicTool::get_material(&drawer).map(|m| int(m.name() as i32)));
        }
        5 => {
            let shape = shape(r.integer()).unwrap();
            let filter = AisBadEdgeFilter::default();
            let edges = sub_shapes(&shape, ShapeEnum::Edge);
            for _ in 0..r.integer() {
                let (code, a, b) = (r.integer(), r.integer(), r.integer());
                match code {
                    0 => filter.set_contour(a),
                    1 => {
                        filter.add_edge(&topods::edge(&edges[a as usize % edges.len()]).unwrap(), b)
                    }
                    _ => filter.remove_edges(a),
                }
            }
            flag(filter.acts_on(ShapeEnum::Edge));
            flag(filter.acts_on(ShapeEnum::Face));
            flag(filter.is_ok(&owner).unwrap());
            for edge in &edges {
                flag(filter.is_ok(&brep_owner(edge)).unwrap());
            }
        }
        6 => {
            let shape = shape(r.integer()).unwrap();
            let mut filter = None;
            attempt(|| AisC0RegularityFilter::new(&shape).map(|f| filter = Some(f)));
            if let Some(filter) = filter {
                flag(filter.acts_on(ShapeEnum::Edge));
                flag(filter.acts_on(ShapeEnum::Vertex));
                flag(filter.is_ok(&owner).unwrap());
                for kind in [ShapeEnum::Edge, ShapeEnum::Face] {
                    for sub in sub_shapes(&shape, kind) {
                        flag(filter.is_ok(&brep_owner(&sub)).unwrap());
                    }
                }
            }
        }
        _ => {}
    }
    end();
}

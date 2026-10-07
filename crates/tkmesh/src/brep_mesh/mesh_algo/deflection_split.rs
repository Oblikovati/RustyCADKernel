// Rust translation of the triangle splitting of BRepMesh_DelaunayDeflectionControlMeshAlgo.hxx
// (splitTriangleGeometry and its helpers) at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::base::BaseMeshAlgoData;
use super::delaunay_insertion::DelaunayInsertionHooks;
use crate::brep_mesh::{
    BRepMeshDelaun, BRepMeshOrientedEdge, BRepMeshTriangle, DegreeOfFreedom, GeomTool,
    RangeSplitter,
};
use rustycad_tkernel::ncollection::{List, Map, PackedMap};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeombase::geom_lib::norm_estim;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector2, Vector3, Xy, Xyz};

/// `TriangleNodeInfo`: a node of the examined triangle.
#[derive(Clone, Copy, Default)]
struct TriangleNodeInfo {
    point_2d: Xy,
    point: Xyz,
    is_frontier_link: bool,
}

/// The deviation measure of a candidate point.
enum Deviation {
    /// `NormalDeviation`: distance to the triangle plane through `point`.
    Normal { point: Point3, normal: Vector3 },
    /// `LineDeviation`: deflection from the segment between two nodes.
    Line { first: Point3, last: Point3 },
}

impl Deviation {
    fn square_deviation(&self, point: Point3) -> Result<f64, Failure> {
        match self {
            Deviation::Normal {
                point: reference,
                normal,
            } => {
                let deflection = normal.dot(Vector3::between(*reference, point)).abs();
                Ok(deflection * deflection)
            }
            Deviation::Line { first, last } => {
                GeomTool::square_deflection_of_segment(*first, *last, point)
            }
        }
    }
}

/// The state of the deflection control kept across and within its passes.
pub(super) struct DeflectionPass {
    pub(super) max_sq_deflection: f64,
    sq_min_size: f64,
    pub(super) all_degenerated: bool,
    couples: Map<BRepMeshOrientedEdge>,
    pub(super) control_nodes: List<Point2>,
}

impl DeflectionPass {
    pub(super) fn new(sq_min_size: f64, couples: Map<BRepMeshOrientedEdge>) -> Self {
        Self {
            max_sq_deflection: -1.0,
            sq_min_size,
            all_degenerated: false,
            couples,
            control_nodes: crate::brep_mesh::range_splitter::new_surface_nodes(),
        }
    }

    /// The reset at the start of a pass; false when the domain has no triangle.
    pub(super) fn start(&mut self, data: &BaseMeshAlgoData) -> bool {
        self.max_sq_deflection = -1.0;
        self.all_degenerated = true;
        self.control_nodes = crate::brep_mesh::range_splitter::new_surface_nodes();
        data.get_structure().elements_of_domain().extent() >= 1
    }

    /// Every domain triangle in packed-map order through `splitTriangleGeometry`.
    pub(super) fn split_domain<A: DelaunayInsertionHooks + ?Sized>(
        &mut self,
        algo: &A,
        mesher: &mut BRepMeshDelaun,
    ) -> Result<(), Failure> {
        let structure = algo.base_data().get_structure().clone();
        for triangle_id in structure.elements_of_domain().iter() {
            let triangle = structure.get_element(triangle_id)?;
            self.split_triangle_geometry(algo, mesher, &triangle)?;
        }
        Ok(())
    }

    /// `splitTriangleGeometry`: the center against the triangle plane, then the links.
    fn split_triangle_geometry<A: DelaunayInsertionHooks + ?Sized>(
        &mut self,
        algo: &A,
        mesher: &mut BRepMeshDelaun,
        triangle: &BRepMeshTriangle,
    ) -> Result<(), Failure> {
        if triangle.movability() == DegreeOfFreedom::Deleted {
            return Ok(());
        }
        let nodes = algo.base_data().get_structure().element_nodes(triangle)?;
        let info = triangle_info(algo, triangle, &nodes)?;
        let Some(normal) = triangle_normal(&info)? else {
            return Ok(());
        };
        self.all_degenerated = false;
        let center = (info[0].point_2d + info[1].point_2d + info[2].point_2d) / 3.0;
        let plane = Deviation::Normal {
            point: Point3::from(info[0].point),
            normal,
        };
        self.use_point(algo, mesher, center, &plane)?;
        self.split_links(algo, mesher, &info, &nodes)
    }

    /// `splitLinks`: the middle of each non-frontier link not yet examined, kept when
    /// it deviates and passes the minimal size and angular checks.
    fn split_links<A: DelaunayInsertionHooks + ?Sized>(
        &mut self,
        algo: &A,
        mesher: &mut BRepMeshDelaun,
        info: &[TriangleNodeInfo; 3],
        nodes: &[i32; 3],
    ) -> Result<(), Failure> {
        for i in 0..3 {
            let j = (i + 1) % 3;
            let couple = BRepMeshOrientedEdge::new(nodes[i].min(nodes[j]), nodes[i].max(nodes[j]));
            if info[i].is_frontier_link || !self.couples.add(couple) {
                continue;
            }
            let middle = (info[i].point_2d + info[j].point_2d) / 2.0;
            let segment = Deviation::Line {
                first: Point3::from(info[i].point),
                last: Point3::from(info[j].point),
            };
            if self.use_point(algo, mesher, middle, &segment)?
                || reject_split_for_min_size(algo, self.sq_min_size, &info[i], &info[j], middle)?
                || link_ends_within_angle(algo, &info[i], &info[j])?
            {
                continue;
            }
            self.control_nodes.append(Point2::from(middle));
        }
        Ok(())
    }

    /// `usePoint`: the point is queued when its deviation fails the deflection check.
    fn use_point<A: DelaunayInsertionHooks + ?Sized>(
        &mut self,
        algo: &A,
        mesher: &mut BRepMeshDelaun,
        point_2d: Xy,
        deviation: &Deviation,
    ) -> Result<bool, Failure> {
        let point = point_3d(algo, point_2d)?;
        let sq_deviation = deviation.square_deviation(point)?;
        if self.check_deflection(algo, mesher, point_2d, point, sq_deviation)? {
            return Ok(false);
        }
        self.control_nodes.append(Point2::from(point_2d));
        Ok(true)
    }

    /// `checkDeflectionOfPointAndUpdateCache`: records the largest deviation; true when
    /// it is below the face deflection or the point is too close to existing nodes.
    fn check_deflection<A: DelaunayInsertionHooks + ?Sized>(
        &mut self,
        algo: &A,
        mesher: &mut BRepMeshDelaun,
        point_2d: Xy,
        point: Point3,
        sq_deflection: f64,
    ) -> Result<bool, Failure> {
        if sq_deflection > self.max_sq_deflection {
            self.max_sq_deflection = sq_deflection;
        }
        let deflection = algo.base_data().get_dface().get_deflection();
        if sq_deflection < deflection * deflection {
            return Ok(true);
        }
        self.reject_by_min_size(algo, mesher, point_2d, point)
    }

    /// `rejectByMinSize`: whether a node of a triangle whose circumcircle contains the
    /// point lies within MinSize of it.
    fn reject_by_min_size<A: DelaunayInsertionHooks + ?Sized>(
        &self,
        algo: &A,
        mesher: &mut BRepMeshDelaun,
        point_2d: Xy,
        point: Point3,
    ) -> Result<bool, Failure> {
        let splitter = algo.insertion().get_range_splitter().range_state();
        let scaled = splitter.scale(Point2::from(point_2d), true).xy();
        let circles: Vec<i32> = mesher
            .circles_mut()
            .select(&scaled)?
            .iter()
            .copied()
            .collect();
        let data = algo.base_data();
        let mut used = PackedMap::<i32>::default();
        for triangle_id in circles {
            let triangle = data.get_structure().get_element(triangle_id)?;
            for node in data.get_structure().element_nodes(&triangle)? {
                if used.add(node)
                    && point.square_distance(&node_point(data, node)?) < self.sq_min_size
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

/// `getTriangleInfo`: the surface parameters, 3d point and frontier flag of each node.
fn triangle_info<A: DelaunayInsertionHooks + ?Sized>(
    algo: &A,
    triangle: &BRepMeshTriangle,
    nodes: &[i32; 3],
) -> Result<[TriangleNodeInfo; 3], Failure> {
    let data = algo.base_data();
    let splitter = algo.insertion().get_range_splitter().range_state();
    let mut info = [TriangleNodeInfo::default(); 3];
    for i in 0..3 {
        let vertex = data.get_structure().get_node(nodes[i])?;
        info[i].point_2d = splitter.scale(Point2::from(*vertex.coord()), false).xy();
        info[i].point = node_point(data, nodes[i])?.xyz();
        let link = data.get_structure().get_link(triangle.edges[i])?;
        info[i].is_frontier_link = link.movability() == DegreeOfFreedom::Frontier;
    }
    Ok(info)
}

/// The 3d point of a structure node (`myNodesMap->Value(Location3d)`).
fn node_point(data: &BaseMeshAlgoData, node: i32) -> Result<Point3, Failure> {
    let location = data.get_structure().get_node(node)?.location_3d();
    usize::try_from(location)
        .ok()
        .and_then(|index| data.get_nodes_map().get(index).copied())
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!("DeflectionControl: Location3d {location} of node {node}; expected an index below {}", data.get_nodes_map().len()),
            )
        })
}

/// `computeTriangleGeometry`: the unit normal of a non-degenerate triangle (3d links
/// longer than 1e-6, parametric area above 1e-9, normal above gp::Resolution()).
fn triangle_normal(info: &[TriangleNodeInfo; 3]) -> Result<Option<Vector3>, Failure> {
    let mut links = [Vector3::default(); 3];
    for i in 0..3 {
        links[i] = Vector3::from(info[(i + 1) % 3].point - info[i].point);
        if links[i].square_magnitude() < 1.0e-12 {
            return Ok(None);
        }
    }
    let link_1 = Vector2::from(info[1].point_2d - info[0].point_2d);
    let link_2 = Vector2::from(info[2].point_2d - info[1].point_2d);
    if link_1.crossed(link_2).abs().partial_cmp(&1.0e-9) != Some(std::cmp::Ordering::Greater) {
        return Ok(None);
    }
    let normal = links[0].crossed(links[1]);
    if normal
        .square_magnitude()
        .partial_cmp(&rustycad_tkmath::gp::resolution())
        != Some(std::cmp::Ordering::Greater)
    {
        return Ok(None);
    }
    Ok(Some(normal.normalized()?))
}

/// `getPoint3d`: the face surface at the parameters.
fn point_3d<A: DelaunayInsertionHooks + ?Sized>(algo: &A, point_2d: Xy) -> Result<Point3, Failure> {
    algo.base_data()
        .get_dface()
        .get_surface()
        .d0(point_2d.x(), point_2d.y())
}

/// `rejectSplitLinksForMinSize`: a half link shorter than MinSize.
fn reject_split_for_min_size<A: DelaunayInsertionHooks + ?Sized>(
    algo: &A,
    sq_min_size: f64,
    first: &TriangleNodeInfo,
    second: &TriangleNodeInfo,
    middle: Xy,
) -> Result<bool, Failure> {
    let point = point_3d(algo, middle)?.xyz();
    Ok((first.point - point).square_modulus() < sq_min_size
        || (second.point - point).square_modulus() < sq_min_size)
}

/// `checkLinkEndsForAngularDeviation`: true (keep the link) unless both end normals
/// exist and differ by more than AngleInterior.
fn link_ends_within_angle<A: DelaunayInsertionHooks + ?Sized>(
    algo: &A,
    first: &TriangleNodeInfo,
    second: &TriangleNodeInfo,
) -> Result<bool, Failure> {
    let surface = algo
        .base_data()
        .get_dface()
        .get_surface()
        .transformed_surface()
        .geom_surface_original();
    let (mut normal_1, mut normal_2) = (Direction3::default(), Direction3::default());
    let tolerance = precision::confusion();
    if norm_estim(
        &surface,
        Point2::from(first.point_2d),
        tolerance,
        &mut normal_1,
    )? == 0
        && norm_estim(
            &surface,
            Point2::from(second.point_2d),
            tolerance,
            &mut normal_2,
        )? == 0
        && normal_1.angle(&normal_2) > algo.base_data().get_parameters().angle_interior
    {
        return Ok(false);
    }
    Ok(true)
}

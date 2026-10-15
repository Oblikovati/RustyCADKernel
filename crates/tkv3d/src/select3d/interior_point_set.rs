//! `Select3D_InteriorSensitivePointSet`: the interior of a polygon split into planar convex
//! parts (sensitive polys) kept in a BVH tree.
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase, Select3DSensitivePoly,
    Select3DSensitiveSet, Select3DSensitiveSetBase, Select3DTypeOfSensitivity,
    select3d_set_build_bvh, select3d_set_dump_json, select3d_set_matches,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::ncollection::{Array1, Vec3};
use rustycad_tkernel::precision::confusion;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped};
use std::io;

/// The plane of a planar part (`Select3D_Plane`): `ax + by + cz + d`, unset at first.
#[derive(Clone, Copy, Default)]
struct PartPlane {
    plane: [f64; 4],
    is_initialized: bool,
}

impl PartPlane {
    /// `Contains`: the signed plane value below `Precision::Confusion` (no absolute value natively).
    fn contains(&self, point: &Point3) -> bool {
        let [a, b, c, d] = self.plane;
        self.is_initialized && a * point.x() + b * point.y() + c * point.z() + d < confusion()
    }
    /// `MakePlane`: the plane of three points (an unnormalized normal).
    fn make(&mut self, p1: &Point3, p2: &Point3, p3: &Point3) {
        let (v1, v2) = (p2.xyz() - p1.xyz(), p3.xyz() - p1.xyz());
        let dir = v1.crossed(v2);
        let d = dir.dot(p1.xyz().reversed());
        (self.plane, self.is_initialized) = ([dir.x(), dir.y(), dir.z(), d], true);
    }
}

/// The planar parts of the point sequence, as the native constructor splits them.
fn planar_parts(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    points: &[Point3],
) -> Result<Vec<Handle<Select3DSensitivePoly>>, Failure> {
    let mut parts = Vec::new();
    let mut plane = PartPlane::default();
    let upper = points.len() as i32 - 1;
    let (mut start, mut end) = (0i32, 0i32);
    let part = |from: i32, to: i32, bvh: bool| -> Result<Handle<Select3DSensitivePoly>, Failure> {
        Ok(Handle::new(Select3DSensitivePoly::new(
            owner,
            &points[from as usize..=to as usize],
            bvh,
        )?))
    };
    for iter in 0..=upper {
        let p3 = points[iter as usize];
        let (p1, p2) = if iter >= 2 {
            (points[iter as usize - 2], points[iter as usize - 1])
        } else {
            (Point3::default(), Point3::default())
        };
        if iter - start == 2 && !plane.is_initialized {
            plane.make(&p1, &p2, &p3);
            (start, end) = (iter - 2, iter);
            if end == upper {
                parts.push(part(start, end, false)?);
            }
        } else if plane.is_initialized {
            let angle = (p1.xyz() - p2.xyz()).dot(p3.xyz() - p2.xyz());
            if !plane.contains(&p3) || angle > confusion() {
                parts.push(part(start, iter - 1, true)?);
                (start, end) = (iter, iter);
                plane.is_initialized = false;
            } else {
                end += 1;
                if end == upper {
                    parts.push(part(start, end, true)?);
                }
            }
        }
    }
    Ok(parts)
}

/// The mutable state: the BVH order of the parts.
struct PointSetState {
    polygons_idxs: Vec<i32>,
}

/// The interior of a polygon as planar parts.
///
/// Example: `let set = Select3DInteriorSensitivePointSet::new(&owner, &points)?;`.
pub struct Select3DInteriorSensitivePointSet {
    transient: StandardTransient,
    entity: Select3DSensitiveEntityBase,
    set: Select3DSensitiveSetBase,
    planar_polygons: Vec<Handle<Select3DSensitivePoly>>,
    state: SharedState<PointSetState>,
    cog: Point3,
    bnd_box: Select3DBndBox3d,
}
rustycad_tkernel::impl_transient!(Select3DInteriorSensitivePointSet, "Select3D_InteriorSensitivePointSet", dyn Select3DSensitiveSet, transient,
    interfaces: [dyn Select3DSensitiveEntity, dyn Select3DSensitiveSet]);

impl Select3DInteriorSensitivePointSet {
    /// The set of the points: the planar parts, the center and the box; no point gives a
    /// division by zero natively (a NaN center here). Example: `Select3DInteriorSensitivePointSet::new(&owner, &points)?`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        points: &[Point3],
    ) -> Result<Self, Failure> {
        let mut sum = Xyz::new(0.0, 0.0, 0.0);
        let mut bnd_box = Select3DBndBox3d::default();
        for point in points {
            sum += point.xyz();
            bnd_box.add(&Vec3::new(point.x(), point.y(), point.z()));
        }
        let planar_polygons = planar_parts(owner, points)?;
        let polygons_idxs = (0..planar_polygons.len() as i32).collect();
        Ok(Self {
            transient: StandardTransient::default(),
            entity: Select3DSensitiveEntityBase::new(owner),
            set: Select3DSensitiveSetBase::default(),
            planar_polygons,
            state: SharedState::new(PointSetState { polygons_idxs }),
            cog: Point3::from(sum / points.len() as f64),
            bnd_box,
        })
    }
    /// `GetPoints`: the points of the parts without the shared last points; the array has the
    /// size of all the part points, its tail left at the origin as natively. Example: `set.get_points()`.
    pub fn get_points(&self) -> Vec<Point3> {
        let size: i32 = self
            .planar_polygons
            .iter()
            .map(|p| p.core.nb_sub_elements())
            .sum();
        let mut output = vec![Point3::default(); size.max(0) as usize];
        let mut index = 0;
        let count = self.planar_polygons.len();
        for (polyg_idx, polygon) in self.planar_polygons.iter().enumerate() {
            let points = polygon.points_3d();
            let kept = if polyg_idx + 1 < count {
                points.len().saturating_sub(1)
            } else {
                points.len()
            };
            for point in points.into_iter().take(kept) {
                output[index] = point;
                index += 1;
            }
        }
        output
    }
    /// The part at a BVH index.
    fn part(&self, idx: i32) -> Option<Handle<Select3DSensitivePoly>> {
        let polyg_idx = *self.state.read().polygons_idxs.get(idx as usize)?;
        self.planar_polygons.get(polyg_idx as usize).cloned()
    }
}

impl Select3DSensitiveEntity for Select3DInteriorSensitivePointSet {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.entity
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        select3d_set_matches(self, manager, pick_result, false)
    }
    fn nb_sub_elements(&self) -> i32 {
        self.planar_polygons.len() as i32
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.bnd_box
    }
    fn center_of_geometry(&self) -> Point3 {
        self.cog
    }
    fn bvh(&self) -> Result<(), Failure> {
        select3d_set_build_bvh(self)
    }
    fn to_build_bvh(&self) -> bool {
        self.set.is_dirty()
    }
    fn clear(&self) {}
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_InteriorSensitivePointSet")?;
        base_class(stream, depth, &|s, d| select3d_set_dump_json(self, s, d))?;
        let bounds = self.bnd_box;
        field_dumped(stream, depth, "&myBndBox", &|s, _| bounds.dump_json(s))
    }
}

impl Select3DSensitiveSet for Select3DInteriorSensitivePointSet {
    fn set_base(&self) -> &Select3DSensitiveSetBase {
        &self.set
    }
    fn size(&self) -> i32 {
        self.planar_polygons.len() as i32
    }
    fn element_box(&self, idx: i32) -> Select3DBndBox3d {
        self.part(idx)
            .map(|p| p.core.bounding_box())
            .unwrap_or_default()
    }
    /// `Center`: natively the coordinate `theAxis - 1` of the part center, so axis 0 raises
    /// `Standard_OutOfRange` and the axes 1 and 2 read X and Y.
    fn center(&self, idx: i32, axis: i32) -> Result<f64, Failure> {
        let cog = self
            .part(idx)
            .map(|p| p.core.center_of_geometry())
            .unwrap_or_default();
        cog.coord(axis - 1)
    }
    fn swap(&self, idx1: i32, idx2: i32) {
        self.state
            .write()
            .polygons_idxs
            .swap(idx1 as usize, idx2 as usize);
    }
    /// `overlapsElement`: the interior polygon test of the part points.
    fn overlaps_element(
        &self,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        _is_full_inside: bool,
    ) -> Result<bool, Failure> {
        let Some(part) = self.part(elem_idx) else {
            return Ok(false);
        };
        let points = part.points_3d();
        manager.overlaps_polygon(
            &Array1::from_shared(&points, 1),
            Select3DTypeOfSensitivity::Interior as i32,
            pick_result,
        )
    }
    fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        let mut dummy = SelectBasicsPickResult::default();
        self.overlaps_element(&mut dummy, manager, elem_idx, is_full_inside)
    }
    fn distance_to_cog(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
    ) -> Result<f64, Failure> {
        manager.dist_to_geometry_center(&self.cog)
    }
}

/// The protected fields of `Select3D_InteriorSensitivePointSet`, read only.
impl Select3DInteriorSensitivePointSet {
    /// `myPlanarPolygons`: the planar parts; `set.planar_polygons().len()`.
    pub fn planar_polygons(&self) -> &[Handle<Select3DSensitivePoly>] {
        &self.planar_polygons
    }
    /// `myPolygonsIdxs`: the BVH order of the parts; `set.polygons_idxs()`.
    pub fn polygons_idxs(&self) -> Vec<i32> {
        self.state.read().polygons_idxs.clone()
    }
}

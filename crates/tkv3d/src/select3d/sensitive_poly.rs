//! `Select3D_SensitivePoly`: a sensitive polyline (its segments in a BVH tree) or a filled
//! polygon. The behaviour is in [`Select3DSensitivePolyCore`], which the poly and its
//! subclass `Select3D_SensitiveCurve` embed; [`impl_select3d_poly`] writes their trait
//! implementations.
use super::point_data::Select3DPointData;
use super::{
    Select3DBndBox3d, Select3DSensitiveEntityBase, Select3DSensitiveSet, Select3DSensitiveSetBase,
    Select3DTypeOfSensitivity, select3d_set_dump_json, select3d_set_matches,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::{Array1, Vec3};
use rustycad_tkernel::precision::confusion;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkmath::bvh::{BvhBox, std_max, std_min};
use rustycad_tkmath::elclib::{circle_d1, circle_value};
use rustycad_tkmath::gp::{Circle3, Point3, Xyz, resolution};
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::f64::consts::PI;
use std::io;

/// The mutable data of a poly.
#[derive(Clone)]
pub struct Select3DPolyData {
    /// `myPolyg`.
    pub polyg: Select3DPointData,
    /// `myCOG`.
    pub cog: Point3,
    /// `mySegmentIndexes` (null without BVH).
    pub segment_indexes: Option<Vec<i32>>,
    /// `myBndBox`.
    pub bnd_box: Select3DBndBox3d,
    /// `mySensType`.
    pub sens_type: Select3DTypeOfSensitivity,
    /// `myIsComputed`.
    pub is_computed: bool,
}

/// The state of a poly: the entity and set bases and the poly data.
pub struct Select3DSensitivePolyCore {
    /// The entity base.
    pub entity: Select3DSensitiveEntityBase,
    /// The set base.
    pub set: Select3DSensitiveSetBase,
    /// The poly data.
    pub data: SharedState<Select3DPolyData>,
}

/// `GetCircleNbPoints`: 2n+1 points (2 more for a filled sector), 1 for a null radius.
fn circle_nb_points(circle: &Circle3, nb_pnts: i32, u1: f64, u2: f64, is_filled: bool) -> i32 {
    if nb_pnts <= 0 {
        return 0;
    }
    if circle.radius() <= confusion() {
        return 1;
    }
    let is_sector = is_filled && ((u2 - u1).abs() - 2.0 * PI).abs() > resolution();
    2 * nb_pnts + 1 + if is_sector { 2 } else { 0 }
}

/// `initCircle`: the points of an arc with the mid tangent points (and the center for a
/// filled sector).
fn init_circle(
    polygon: &mut Select3DPointData,
    circle: &Circle3,
    u1: f64,
    u2: f64,
    is_filled: bool,
    nb_pnts: i32,
) -> Result<(), Failure> {
    let step = (u2 - u1) / f64::from(nb_pnts);
    let radius = circle.radius();
    let (mut index, mut cur_u) = (0, u1);
    let is_sector = (u2 - u1 - 2.0 * PI).abs() > resolution();
    if is_sector && is_filled {
        polygon.set_pnt(index, &circle.location())?;
        index += 1;
    }
    for _ in 1..=nb_pnts {
        let (p1, v1) = circle_d1(cur_u, circle.position(), radius);
        polygon.set_pnt(index, &p1)?;
        let v1 = v1.normalized()?;
        let p2 = Point3::from(p1.xyz() + *v1.xyz() * (step * 0.5).tan() * radius);
        polygon.set_pnt(index + 1, &p2)?;
        index += 2;
        cur_u += step;
    }
    polygon.set_pnt(index, &circle_value(u2, circle.position(), radius))?;
    if is_sector && is_filled {
        polygon.set_pnt(index + 1, &circle.location())?;
    }
    Ok(())
}

/// The segment indexes `0..n-1` of a BVH-enabled poly of `nb_pnts` points.
fn segment_indexes(bvh_enabled: bool, nb_pnts: i32) -> Option<Vec<i32>> {
    bvh_enabled.then(|| (0..(nb_pnts - 1).max(0)).collect())
}

impl Select3DSensitivePolyCore {
    /// The core of a poly of the given points (boundary sensitivity); no point raises
    /// `Standard_ConstructionError`. Example: `Select3DSensitivePolyCore::from_points(&owner, &points, true)?`.
    pub fn from_points(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        points: &[Point3],
        bvh_enabled: bool,
    ) -> Result<Self, Failure> {
        let mut polyg = Select3DPointData::new(points.len() as i32)?;
        let mut sum = Xyz::new(0.0, 0.0, 0.0);
        let mut bnd_box = Select3DBndBox3d::default();
        for (index, point) in points.iter().enumerate() {
            sum += point.xyz();
            bnd_box.add(&Vec3::new(point.x(), point.y(), point.z()));
            polyg.set_pnt(index as i32, point)?;
        }
        let size = polyg.size();
        let data = Select3DPolyData {
            polyg,
            cog: Point3::from(sum / f64::from(size)),
            segment_indexes: segment_indexes(bvh_enabled, size),
            bnd_box,
            sens_type: Select3DTypeOfSensitivity::Boundary,
            is_computed: true,
        };
        Ok(Self::with_data(owner, data))
    }
    /// The core of a poly of `nb_pnts` points to fill; `Select3DSensitivePolyCore::with_size(&owner, true, 6)?`.
    pub fn with_size(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        bvh_enabled: bool,
        nb_pnts: i32,
    ) -> Result<Self, Failure> {
        let data = Select3DPolyData {
            polyg: Select3DPointData::new(nb_pnts)?,
            cog: Point3::new(real_last(), real_last(), real_last()),
            segment_indexes: segment_indexes(bvh_enabled, nb_pnts),
            bnd_box: Select3DBndBox3d::default(),
            sens_type: Select3DTypeOfSensitivity::Boundary,
            is_computed: false,
        };
        Ok(Self::with_data(owner, data))
    }
    /// The core of an arc of a circle (a filled sector or a boundary of sensitivity 6).
    /// Example: `Select3DSensitivePolyCore::circle(&owner, &circle, 0.0, PI, false, 12)?`.
    pub fn circle(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        circle: &Circle3,
        u1: f64,
        u2: f64,
        is_filled: bool,
        nb_pnts: i32,
    ) -> Result<Self, Failure> {
        let core = Self::with_size(
            owner,
            !is_filled,
            circle_nb_points(circle, nb_pnts, u1, u2, is_filled),
        )?;
        {
            let mut data = core.data.write();
            data.sens_type = if is_filled {
                Select3DTypeOfSensitivity::Interior
            } else {
                Select3DTypeOfSensitivity::Boundary
            };
            if data.polyg.size() != 1 {
                init_circle(
                    &mut data.polyg,
                    circle,
                    std_min(u1, u2),
                    std_max(u1, u2),
                    is_filled,
                    nb_pnts,
                )?;
            } else {
                data.polyg.set_pnt(0, &circle.position().location())?;
            }
        }
        if !is_filled {
            core.entity.fields.write().sensitivity_factor = 6;
        }
        Ok(core)
    }
    /// The core with its data.
    fn with_data(owner: &Handle<dyn SelectMgrEntityOwnerSource>, data: Select3DPolyData) -> Self {
        Self {
            entity: Select3DSensitiveEntityBase::new(owner),
            set: Select3DSensitiveSetBase::default(),
            data: SharedState::new(data),
        }
    }
    /// `Points3D`: the points in double precision (natively a new array from index 1).
    /// Example: `core.points_3d()`.
    pub fn points_3d(&self) -> Vec<Point3> {
        self.data
            .read()
            .polyg
            .points()
            .iter()
            .map(|p| p.to_point())
            .collect()
    }
    /// `ArrayBounds`: `(0, size - 1)`; `core.array_bounds()`.
    pub fn array_bounds(&self) -> (i32, i32) {
        (0, self.data.read().polyg.size() - 1)
    }
    /// `GetPoint3d`: the point, or the origin out of range; `core.get_point_3d(0)`.
    pub fn get_point_3d(&self, pnt_idx: i32) -> Point3 {
        self.data.read().polyg.pnt3d(pnt_idx).unwrap_or_default()
    }
    /// `Matches`: the BVH traversal of the segments for a boundary, the polygon test for an interior.
    pub fn matches_poly<S: Select3DSensitiveSet + ?Sized>(
        &self,
        set: &S,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let sens_type = self.data.read().sens_type;
        match sens_type {
            Select3DTypeOfSensitivity::Boundary => {
                select3d_set_matches(set, manager, pick_result, false)
            }
            Select3DTypeOfSensitivity::Interior => self.matches_interior(manager, pick_result),
        }
    }
    /// The interior test of `Matches`.
    fn matches_interior(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let points = self.points_3d();
        let array = Array1::from_shared(&points, 1);
        let interior = Select3DTypeOfSensitivity::Interior as i32;
        if !manager.is_overlap_allowed() {
            if manager.get_active_selection_type() == SelectMgrSelectionType::Polyline {
                let mut dummy = SelectBasicsPickResult::default();
                return manager.overlaps_polygon(&array, interior, &mut dummy);
            }
            for p in &points {
                if !manager.overlaps_point_only(p)? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
        if !manager.overlaps_polygon(&array, interior, pick_result)? {
            return Ok(false);
        }
        pick_result.set_dist_to_geom_center(self.distance_to_cog(manager)?);
        Ok(true)
    }
    /// `BoundingBox`: the box of the points, computed once.
    pub fn bounding_box(&self) -> Select3DBndBox3d {
        let mut data = self.data.write();
        if data.bnd_box.is_valid() {
            return data.bnd_box;
        }
        let mut bounds = Select3DBndBox3d::default();
        for p in data.polyg.points() {
            bounds.add(&Vec3::new(f64::from(p.x), f64::from(p.y), f64::from(p.z)));
        }
        data.bnd_box = bounds;
        bounds
    }
    /// `Size`: the number of segments, -1 without BVH.
    pub fn size(&self) -> i32 {
        self.data
            .read()
            .segment_indexes
            .as_ref()
            .map_or(-1, |s| s.len() as i32)
    }
    /// The two points of the segment at a BVH index.
    fn segment(&self, idx: i32) -> Option<(Point3, Point3)> {
        let data = self.data.read();
        let segment = *data.segment_indexes.as_ref()?.get(idx as usize)?;
        Some((
            data.polyg.pnt3d(segment).ok()?,
            data.polyg.pnt3d(segment + 1).ok()?,
        ))
    }
    /// `Box`: the box of a segment, the point box (RealLast) without BVH.
    pub fn element_box(&self, idx: i32) -> Select3DBndBox3d {
        let Some((a, b)) = self.segment(idx) else {
            let last = Vec3::new(real_last(), real_last(), real_last());
            return BvhBox::from_corners(last, last);
        };
        let min = Vec3::new(
            std_min(a.x(), b.x()),
            std_min(a.y(), b.y()),
            std_min(a.z(), b.z()),
        );
        let max = Vec3::new(
            std_max(a.x(), b.x()),
            std_max(a.y(), b.y()),
            std_max(a.z(), b.z()),
        );
        BvhBox::from_corners(min, max)
    }
    /// `Center`: the center of the segment box along an axis, RealLast without BVH.
    pub fn center(&self, idx: i32, axis: i32) -> Result<f64, Failure> {
        if self.data.read().segment_indexes.is_none() {
            return Ok(real_last());
        }
        let bounds = self.element_box(idx);
        let center = (*bounds.corner_min() + *bounds.corner_max()) * 0.5;
        Ok(match axis {
            0 => center.x,
            1 => center.y,
            _ => center.z,
        })
    }
    /// `Swap`: exchanges two segment indexes.
    pub fn swap(&self, idx1: i32, idx2: i32) {
        if let Some(indexes) = self.data.write().segment_indexes.as_mut() {
            indexes.swap(idx1 as usize, idx2 as usize);
        }
    }
    /// `overlapsElement`: the segment overlaps (always for a full-inside node).
    pub fn overlaps_element(
        &self,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if self.data.read().segment_indexes.is_none() {
            return Ok(false);
        }
        if is_full_inside {
            return Ok(true);
        }
        match self.segment(elem_idx) {
            Some((a, b)) => manager.overlaps_segment(&a, &b, pick_result),
            None => Ok(false),
        }
    }
    /// `elementIsInside`: the segment (polyline) or both its points are inside.
    pub fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if is_full_inside {
            return Ok(true);
        }
        let Some((a, b)) = self.segment(elem_idx) else {
            return Ok(false);
        };
        if manager.get_active_selection_type() == SelectMgrSelectionType::Polyline {
            let mut dummy = SelectBasicsPickResult::default();
            return manager.overlaps_segment(&a, &b, &mut dummy);
        }
        Ok(manager.overlaps_point_only(&a)? && manager.overlaps_point_only(&b)?)
    }
    /// The center of geometry, computed once from the single-precision points.
    pub fn center_of_geometry(&self) -> Point3 {
        let mut data = self.data.write();
        if !data.is_computed {
            let mut center = Xyz::new(0.0, 0.0, 0.0);
            for p in data.polyg.points() {
                center += p.to_xyz();
            }
            data.cog = Point3::from(center / f64::from(data.polyg.size()));
            data.is_computed = true;
        }
        data.cog
    }
    /// `distanceToCOG`; `core.distance_to_cog(&manager)`.
    pub fn distance_to_cog(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
    ) -> Result<f64, Failure> {
        manager.dist_to_geometry_center(&self.center_of_geometry())
    }
    /// `NbSubElements`: the number of points.
    pub fn nb_sub_elements(&self) -> i32 {
        self.data.read().polyg.size()
    }
    /// `Select3D_SensitivePoly::DumpJson`.
    pub fn dump_json<S: Select3DSensitiveSet + ?Sized>(
        &self,
        set: &S,
        stream: &mut DumpStream,
        depth: i32,
    ) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitivePoly")?;
        base_class(stream, depth, &|s, d| select3d_set_dump_json(set, s, d))?;
        let data = self.data.read().clone();
        let bounds = data.bnd_box;
        field_dumped(stream, depth, "&myBndBox", &|s, _| bounds.dump_json(s))?;
        field_integer(stream, "myIsComputed", i32::from(data.is_computed))
    }
}

/// The `Select3DSensitiveEntity` and `Select3DSensitiveSet` implementations of a poly type
/// with a `core: Select3DSensitivePolyCore` field; the caller adds `get_connected` and
/// `dump_json` through the closures. Example: `impl_select3d_poly!(Select3DSensitivePoly, |s| ..., |s, st, d| ...);`.
#[macro_export]
macro_rules! impl_select3d_poly {
    ($type:ty, $connected:expr) => {
        impl $crate::select3d::Select3DSensitiveEntity for $type {
            fn sensitive_base(&self) -> &$crate::select3d::Select3DSensitiveEntityBase {
                &self.core.entity
            }
            fn matches(
                &self,
                manager: &dyn $crate::select_basics::SelectBasicsSelectingVolumeManager,
                pick_result: &mut $crate::select_basics::SelectBasicsPickResult,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.core.matches_poly(self, manager, pick_result)
            }
            fn nb_sub_elements(&self) -> i32 {
                self.core.nb_sub_elements()
            }
            fn bounding_box(&self) -> $crate::select3d::Select3DBndBox3d {
                self.core.bounding_box()
            }
            fn center_of_geometry(&self) -> rustycad_tkmath::gp::Point3 {
                self.core.center_of_geometry()
            }
            fn get_connected(
                &self,
            ) -> Result<
                rustycad_tkernel::standard::Handle<dyn $crate::select3d::Select3DSensitiveEntity>,
                rustycad_tkernel::standard::Failure,
            > {
                ($connected)(self)
            }
            fn bvh(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
                $crate::select3d::select3d_set_build_bvh(self)
            }
            fn to_build_bvh(&self) -> bool {
                self.core.set.is_dirty()
            }
            fn clear(&self) {}
            fn dump_json(
                &self,
                stream: &mut rustycad_tkernel::standard::dump::DumpStream,
                depth: i32,
            ) -> std::io::Result<()> {
                self.core.dump_json(self, stream, depth)
            }
        }
        impl $crate::select3d::Select3DSensitiveSet for $type {
            fn set_base(&self) -> &$crate::select3d::Select3DSensitiveSetBase {
                &self.core.set
            }
            fn size(&self) -> i32 {
                self.core.size()
            }
            fn element_box(&self, idx: i32) -> $crate::select3d::Select3DBndBox3d {
                self.core.element_box(idx)
            }
            fn center(
                &self,
                idx: i32,
                axis: i32,
            ) -> Result<f64, rustycad_tkernel::standard::Failure> {
                self.core.center(idx, axis)
            }
            fn swap(&self, idx1: i32, idx2: i32) {
                self.core.swap(idx1, idx2);
            }
            fn overlaps_element(
                &self,
                pick_result: &mut $crate::select_basics::SelectBasicsPickResult,
                manager: &dyn $crate::select_basics::SelectBasicsSelectingVolumeManager,
                elem_idx: i32,
                is_full_inside: bool,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.core
                    .overlaps_element(pick_result, manager, elem_idx, is_full_inside)
            }
            fn element_is_inside(
                &self,
                manager: &dyn $crate::select_basics::SelectBasicsSelectingVolumeManager,
                elem_idx: i32,
                is_full_inside: bool,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.core
                    .element_is_inside(manager, elem_idx, is_full_inside)
            }
            fn distance_to_cog(
                &self,
                manager: &dyn $crate::select_basics::SelectBasicsSelectingVolumeManager,
            ) -> Result<f64, Failure> {
                self.core.distance_to_cog(manager)
            }
        }
    };
}

/// A sensitive polyline or filled polygon.
///
/// Example: `let poly = Select3DSensitivePoly::new(&owner, &points, true)?; poly.nb_sub_elements();`.
pub struct Select3DSensitivePoly {
    transient: rustycad_tkernel::standard::StandardTransient,
    /// The poly state.
    pub core: Select3DSensitivePolyCore,
}
rustycad_tkernel::impl_transient!(Select3DSensitivePoly, "Select3D_SensitivePoly", dyn Select3DSensitiveSet, transient,
    interfaces: [dyn super::Select3DSensitiveEntity, dyn Select3DSensitiveSet]);
impl_select3d_poly!(Select3DSensitivePoly, |_: &Select3DSensitivePoly| Ok(
    Handle::default()
));

impl Select3DSensitivePoly {
    /// `Select3D_SensitivePoly(owner, points, isBVHEnabled)`; no point raises
    /// `Standard_ConstructionError`. Example: `Select3DSensitivePoly::new(&owner, &points, true)?`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        points: &[Point3],
        bvh_enabled: bool,
    ) -> Result<Self, Failure> {
        Ok(Self::of(Select3DSensitivePolyCore::from_points(
            owner,
            points,
            bvh_enabled,
        )?))
    }
    /// `Select3D_SensitivePoly(owner, isBVHEnabled, nbPnts)`: `nb_pnts` points to fill.
    /// Example: `Select3DSensitivePoly::with_size(&owner, true, 6)?`.
    pub fn with_size(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        bvh_enabled: bool,
        nb_pnts: i32,
    ) -> Result<Self, Failure> {
        Ok(Self::of(Select3DSensitivePolyCore::with_size(
            owner,
            bvh_enabled,
            nb_pnts,
        )?))
    }
    /// `Select3D_SensitivePoly(owner, circle, U1, U2, isFilled, nbPnts)`: an arc.
    /// Example: `Select3DSensitivePoly::circle(&owner, &circle, 0.0, PI, false, 12)?`.
    pub fn circle(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        circle: &Circle3,
        u1: f64,
        u2: f64,
        is_filled: bool,
        nb_pnts: i32,
    ) -> Result<Self, Failure> {
        Ok(Self::of(Select3DSensitivePolyCore::circle(
            owner, circle, u1, u2, is_filled, nb_pnts,
        )?))
    }
    /// A poly of a core.
    fn of(core: Select3DSensitivePolyCore) -> Self {
        Self {
            transient: rustycad_tkernel::standard::StandardTransient::default(),
            core,
        }
    }
    /// `Points3D`; `poly.points_3d()`.
    pub fn points_3d(&self) -> Vec<Point3> {
        self.core.points_3d()
    }
    /// `ArrayBounds`; `poly.array_bounds()`.
    pub fn array_bounds(&self) -> (i32, i32) {
        self.core.array_bounds()
    }
    /// `GetPoint3d`; `poly.get_point_3d(0)`.
    pub fn get_point_3d(&self, pnt_idx: i32) -> Point3 {
        self.core.get_point_3d(pnt_idx)
    }
}

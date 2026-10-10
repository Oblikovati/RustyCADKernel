//! Shared named-fixture guard: abstract geometry must never be called by base tests.
macro_rules! reject_abstract_geometry {
    () => {
        fn build(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn is_scalable(&self) -> bool {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn scale_and_transform(
            &self,
            _scale_factor: i32,
            _trsf: &rustycad_tkmath::gp::GeneralTransform3,
            _builder: &rustycad_tkernel::standard::Handle<
                rustycad_tkv3d::select_mgr::SelectMgrFrustumBuilder,
            >,
        ) -> Result<
            rustycad_tkernel::standard::Handle<
                dyn rustycad_tkv3d::select_mgr::SelectMgrBaseIntersector,
            >,
            rustycad_tkernel::standard::Failure,
        > {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn copy_with_builder(
            &self,
            _builder: &rustycad_tkernel::standard::Handle<
                rustycad_tkv3d::select_mgr::SelectMgrFrustumBuilder,
            >,
        ) -> Result<
            rustycad_tkernel::standard::Handle<
                dyn rustycad_tkv3d::select_mgr::SelectMgrBaseIntersector,
            >,
            rustycad_tkernel::standard::Failure,
        > {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_box_pick(
            &self,
            _min: &rustycad_tkernel::ncollection::Vec3<f64>,
            _max: &rustycad_tkernel::ncollection::Vec3<f64>,
            _clip: &rustycad_tkv3d::select_mgr::SelectMgrViewClipRange,
            _pick: &mut rustycad_tkv3d::select_basics::SelectBasicsPickResult,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_box(
            &self,
            _min: &rustycad_tkernel::ncollection::Vec3<f64>,
            _max: &rustycad_tkernel::ncollection::Vec3<f64>,
            _inside: Option<&mut bool>,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_point_pick(
            &self,
            _point: &rustycad_tkmath::gp::Point3,
            _clip: &rustycad_tkv3d::select_mgr::SelectMgrViewClipRange,
            _pick: &mut rustycad_tkv3d::select_basics::SelectBasicsPickResult,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_point(
            &self,
            _point: &rustycad_tkmath::gp::Point3,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_polygon(
            &self,
            _points: &[rustycad_tkmath::gp::Point3],
            _sens_type: i32,
            _clip: &rustycad_tkv3d::select_mgr::SelectMgrViewClipRange,
            _pick: &mut rustycad_tkv3d::select_basics::SelectBasicsPickResult,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_segment(
            &self,
            _p1: &rustycad_tkmath::gp::Point3,
            _p2: &rustycad_tkmath::gp::Point3,
            _clip: &rustycad_tkv3d::select_mgr::SelectMgrViewClipRange,
            _pick: &mut rustycad_tkv3d::select_basics::SelectBasicsPickResult,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_triangle(
            &self,
            _p1: &rustycad_tkmath::gp::Point3,
            _p2: &rustycad_tkmath::gp::Point3,
            _p3: &rustycad_tkmath::gp::Point3,
            _sens_type: i32,
            _clip: &rustycad_tkv3d::select_mgr::SelectMgrViewClipRange,
            _pick: &mut rustycad_tkv3d::select_basics::SelectBasicsPickResult,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_sphere(
            &self,
            _center: &rustycad_tkmath::gp::Point3,
            _radius: f64,
            _inside: Option<&mut bool>,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_sphere_pick(
            &self,
            _center: &rustycad_tkmath::gp::Point3,
            _radius: f64,
            _clip: &rustycad_tkv3d::select_mgr::SelectMgrViewClipRange,
            _pick: &mut rustycad_tkv3d::select_basics::SelectBasicsPickResult,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_cylinder_pick(
            &self,
            _bottom_rad: f64,
            _top_rad: f64,
            _height: f64,
            _trsf: &rustycad_tkmath::gp::Transform3,
            _is_hollow: bool,
            _clip: &rustycad_tkv3d::select_mgr::SelectMgrViewClipRange,
            _pick: &mut rustycad_tkv3d::select_basics::SelectBasicsPickResult,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_cylinder(
            &self,
            _bottom_rad: f64,
            _top_rad: f64,
            _height: f64,
            _trsf: &rustycad_tkmath::gp::Transform3,
            _is_hollow: bool,
            _inside: Option<&mut bool>,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_circle_pick(
            &self,
            _radius: f64,
            _trsf: &rustycad_tkmath::gp::Transform3,
            _is_filled: bool,
            _clip: &rustycad_tkv3d::select_mgr::SelectMgrViewClipRange,
            _pick: &mut rustycad_tkv3d::select_basics::SelectBasicsPickResult,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
        fn overlaps_circle(
            &self,
            _radius: f64,
            _trsf: &rustycad_tkmath::gp::Transform3,
            _is_filled: bool,
            _inside: Option<&mut bool>,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            panic!("BaseDefaultsIntersector: unexpected abstract geometry call")
        }
    };
}
pub(crate) use reject_abstract_geometry;

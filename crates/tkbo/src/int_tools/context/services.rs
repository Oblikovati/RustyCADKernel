use super::*;
impl Context {
    /// Returns a cached face classifier; reversed orientation shares the same service; `context.fclass2d(&face)?`.
    pub fn fclass2d(&self, face: &Face) -> Result<Cached<FClass2d>, Failure> {
        self.classifiers.get(face, || {
            let forward = Face::unchecked(face.oriented(Orientation::Forward));
            FClass2d::new(&forward, BRepTool::face_tolerance(&forward)?)
        })
    }
    /// Returns the face-restricted adaptor; `context.surface_adaptor(&face)?`.
    pub fn surface_adaptor(&self, face: &Face) -> Result<Cached<BRepAdaptorSurface>, Failure> {
        self.surfaces
            .get(face, || BRepAdaptorSurface::with_face(face, true))
    }
    /// Reads the cached adaptor's bounds; `context.uv_bounds(&face)?`.
    pub fn uv_bounds(&self, face: &Face) -> Result<(f64, f64, f64, f64), Failure> {
        let service = self.surface_adaptor(face)?;
        let adaptor = service.read();
        Ok((
            adaptor.first_u_parameter()?,
            adaptor.last_u_parameter()?,
            adaptor.first_v_parameter()?,
            adaptor.last_v_parameter()?,
        ))
    }
    /// Returns default 3-by-3 localization settings; `context.surface_data(&face)?`.
    pub fn surface_data(&self, face: &Face) -> Result<Cached<SurfaceRangeLocalizeData>, Failure> {
        self.localization.get(face, || {
            Ok(SurfaceRangeLocalizeData::new(
                3,
                3,
                10. * rustycad_tkernel::precision::p_confusion(),
                10. * rustycad_tkernel::precision::p_confusion(),
            ))
        })
    }
    /// Returns a cached bounding box, including native shape tolerances; `context.bnd_box(&shape)?`.
    pub fn bnd_box(&self, shape: &Shape) -> Result<Cached<BoundingBox3>, Failure> {
        self.boxes.get(shape, || {
            let mut bounds = BoundingBox3::default();
            BRepBndLib::add(shape, &mut bounds, true)?;
            Ok(bounds)
        })
    }
    /// Checks open bounding-box directions; `context.is_infinite_face(&face)?`.
    pub fn is_infinite_face(&self, face: &Face) -> Result<bool, Failure> {
        let service = self.bnd_box(face)?;
        let b = service.read();
        Ok(b.is_open_xmax()
            || b.is_open_xmin()
            || b.is_open_ymax()
            || b.is_open_ymin()
            || b.is_open_zmax()
            || b.is_open_zmin())
    }
    /// Caches the first gap requested for a shape; `context.obb(&shape, 1e-7)?`.
    pub fn obb(&self, shape: &Shape, gap: f64) -> Result<Cached<OrientedBoundingBox>, Failure> {
        self.oriented_boxes.get(shape, || {
            let mut bounds = OrientedBoundingBox::default();
            BRepBndLib::add_obb(shape, &mut bounds, true, false, true)?;
            bounds.enlarge(gap);
            Ok(bounds)
        })
    }
    /// Returns a cached loaded solid classifier; `context.solid_classifier(&solid)?`.
    pub fn solid_classifier(
        &self,
        solid: &Solid,
    ) -> Result<Cached<BRepClass3dSolidClassifier>, Failure> {
        self.solids
            .get(solid, || BRepClass3dSolidClassifier::with_shape(solid))
    }
}

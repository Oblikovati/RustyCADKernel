//! Exact native frustum dump fields; instance addresses retain their native meaning.
use super::{SelectMgrBaseFrustumBase, SelectMgrFrustumCore};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::Vector3;
use rustycad_tkservice::json_dump::{
    class_begin, field_dumped, field_integer, field_pointer, field_real,
};
use std::io;

impl<const N: usize> SelectMgrFrustumCore<N> {
    fn dump_frustum_planes(
        &self,
        stream: &mut DumpStream,
        depth: i32,
        is_ortho: bool,
    ) -> io::Result<()> {
        for index in (0..N + 1).step_by(self.inc_factor(is_ortho)) {
            let plane = Vector3::from(self.planes[index]);
            field_dumped(stream, depth, "&aPlane", &|s, _| plane.dump_json(s))?;
            field_real(
                stream,
                "myMaxVertsProjections[aPlaneIdx]",
                self.max_verts_projections[index],
            )?;
            field_real(
                stream,
                "myMinVertsProjections[aPlaneIdx]",
                self.min_verts_projections[index],
            )?;
        }
        Ok(())
    }
    fn dump_frustum_ortho(&self, stream: &mut DumpStream) -> io::Result<()> {
        for index in 0..3 {
            field_real(
                stream,
                "aMaxOrthoVertsProjections",
                self.max_ortho_verts_projections[index],
            )?;
            field_real(
                stream,
                "aMinOrthoVertsProjections",
                self.min_ortho_verts_projections[index],
            )?;
        }
        Ok(())
    }
    /// Dump the protected arrays and associated frustum state; `core.dump_json(&base, true, &mut stream, -1)?`.
    pub fn dump_json(
        &self,
        base: &SelectMgrBaseFrustumBase,
        is_ortho: bool,
        stream: &mut DumpStream,
        depth: i32,
    ) -> io::Result<()> {
        // The native template inherits its static get_type_name from BaseFrustum.
        class_begin(stream, "SelectMgr_BaseFrustum")?;
        self.dump_frustum_planes(stream, depth, is_ortho)?;
        for vertex in &self.vertices {
            field_dumped(stream, depth, "&aVertex", &|s, _| vertex.dump_json(s))?;
        }
        field_integer(stream, "myPixelTolerance", *base.pixel_tolerance.read())?;
        field_pointer(
            stream,
            "myBuilder",
            base.builder.read().object_address() as usize,
        )?;
        field_pointer(
            stream,
            "myCamera",
            base.intersector.camera.read().object_address() as usize,
        )?;
        self.dump_frustum_ortho(stream)?;
        for edge in &self.edge_dirs {
            field_dumped(stream, depth, "&anEdgeDir", &|s, _| {
                Vector3::from(*edge).dump_json(s)
            })?;
        }
        Ok(())
    }
}

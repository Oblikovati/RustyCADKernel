//! The primitive array classes (`Graphic3d_ArrayOfPoints` ... `Graphic3d_ArrayOfPolygons`):
//! each one is the base array of its type with its constructors; the members are those of
//! [`Graphic3dArrayOfPrimitives`], reached through `Deref`.
use super::{
    GRAPHIC3D_ARRAY_FLAGS_BOUND_COLOR as BOUND_COLOR,
    GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR as VERTEX_COLOR,
    GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL as VERTEX_NORMAL,
    GRAPHIC3D_ARRAY_FLAGS_VERTEX_TEXEL as VERTEX_TEXEL, Graphic3dArrayFlags,
    Graphic3dArrayOfPrimitives, Graphic3dTypeOfPrimitiveArray as Kind,
};
use rustycad_tkernel::standard::{Failure, Handle};

/// `flag` when `on`, else none.
fn flag_if(on: bool, flag: Graphic3dArrayFlags) -> Graphic3dArrayFlags {
    if on { flag } else { 0 }
}

/// Declares an array class that embeds the base array of its type.
macro_rules! array_class {
    ($ty:ident, $name:literal) => {
        #[doc = concat!("The native `", $name, "`: the base array of its type.")]
        pub struct $ty {
            base: Graphic3dArrayOfPrimitives,
        }
        rustycad_tkernel::impl_transient!($ty, $name, Graphic3dArrayOfPrimitives, base);
        impl std::ops::Deref for $ty {
            type Target = Graphic3dArrayOfPrimitives;
            fn deref(&self) -> &Graphic3dArrayOfPrimitives {
                &self.base
            }
        }
    };
}
array_class!(Graphic3dArrayOfPoints, "Graphic3d_ArrayOfPoints");
array_class!(Graphic3dArrayOfSegments, "Graphic3d_ArrayOfSegments");
array_class!(Graphic3dArrayOfPolylines, "Graphic3d_ArrayOfPolylines");
array_class!(Graphic3dArrayOfTriangles, "Graphic3d_ArrayOfTriangles");
array_class!(
    Graphic3dArrayOfTriangleStrips,
    "Graphic3d_ArrayOfTriangleStrips"
);
array_class!(
    Graphic3dArrayOfTriangleFans,
    "Graphic3d_ArrayOfTriangleFans"
);
array_class!(Graphic3dArrayOfQuadrangles, "Graphic3d_ArrayOfQuadrangles");
array_class!(
    Graphic3dArrayOfQuadrangleStrips,
    "Graphic3d_ArrayOfQuadrangleStrips"
);
array_class!(Graphic3dArrayOfPolygons, "Graphic3d_ArrayOfPolygons");

/// `CreateArray`: wraps the base array in the class of its type, returned as the base type.
pub(crate) fn create_class(
    kind: Kind,
    base: Graphic3dArrayOfPrimitives,
) -> Result<Handle<Graphic3dArrayOfPrimitives>, Failure> {
    let handle: Handle<Graphic3dArrayOfPrimitives> = match kind {
        Kind::Points => Handle::new(Graphic3dArrayOfPoints { base }).cast(),
        Kind::Segments => Handle::new(Graphic3dArrayOfSegments { base }).cast(),
        Kind::Polylines => Handle::new(Graphic3dArrayOfPolylines { base }).cast(),
        Kind::Triangles => Handle::new(Graphic3dArrayOfTriangles { base }).cast(),
        Kind::Trianglestrips => Handle::new(Graphic3dArrayOfTriangleStrips { base }).cast(),
        Kind::Trianglefans => Handle::new(Graphic3dArrayOfTriangleFans { base }).cast(),
        Kind::Quadrangles => Handle::new(Graphic3dArrayOfQuadrangles { base }).cast(),
        Kind::Quadranglestrips => Handle::new(Graphic3dArrayOfQuadrangleStrips { base }).cast(),
        Kind::Polygons => Handle::new(Graphic3dArrayOfPolygons { base }).cast(),
        _ => Handle::new(base),
    };
    Ok(handle)
}

impl Graphic3dArrayOfPoints {
    /// Points with flags; `Graphic3dArrayOfPoints::new(10, 0)?`.
    pub fn new(max_vertexs: i32, flags: Graphic3dArrayFlags) -> Result<Self, Failure> {
        Ok(Self {
            base: Graphic3dArrayOfPrimitives::new(Kind::Points, max_vertexs, 0, 0, flags)?,
        })
    }
    /// Points with optional colors and normals; `Graphic3dArrayOfPoints::with_options(10, true, false)?`.
    pub fn with_options(
        max_vertexs: i32,
        has_colors: bool,
        has_normals: bool,
    ) -> Result<Self, Failure> {
        Self::new(
            max_vertexs,
            flag_if(has_colors, VERTEX_COLOR) | flag_if(has_normals, VERTEX_NORMAL),
        )
    }
}

impl Graphic3dArrayOfSegments {
    /// Segments with flags; `Graphic3dArrayOfSegments::new(4, 6, 0)?`.
    pub fn new(
        max_vertexs: i32,
        max_edges: i32,
        flags: Graphic3dArrayFlags,
    ) -> Result<Self, Failure> {
        Ok(Self {
            base: Graphic3dArrayOfPrimitives::new(
                Kind::Segments,
                max_vertexs,
                0,
                max_edges,
                flags,
            )?,
        })
    }
    /// Segments with optional colors; `Graphic3dArrayOfSegments::with_options(4, 0, false)?`.
    pub fn with_options(
        max_vertexs: i32,
        max_edges: i32,
        has_colors: bool,
    ) -> Result<Self, Failure> {
        Self::new(max_vertexs, max_edges, flag_if(has_colors, VERTEX_COLOR))
    }
}

impl Graphic3dArrayOfPolylines {
    /// Polylines with flags; `Graphic3dArrayOfPolylines::new(6, 2, 0, 0)?`.
    pub fn new(
        max_vertexs: i32,
        max_bounds: i32,
        max_edges: i32,
        flags: Graphic3dArrayFlags,
    ) -> Result<Self, Failure> {
        Ok(Self {
            base: Graphic3dArrayOfPrimitives::new(
                Kind::Polylines,
                max_vertexs,
                max_bounds,
                max_edges,
                flags,
            )?,
        })
    }
    /// Polylines with optional vertex and bound colors; `Graphic3dArrayOfPolylines::with_options(6, 2, 0, false, true)?`.
    pub fn with_options(
        max_vertexs: i32,
        max_bounds: i32,
        max_edges: i32,
        has_colors: bool,
        has_bound_colors: bool,
    ) -> Result<Self, Failure> {
        Self::new(
            max_vertexs,
            max_bounds,
            max_edges,
            flag_if(has_colors, VERTEX_COLOR) | flag_if(has_bound_colors, BOUND_COLOR),
        )
    }
}

/// The vertex flags of the normal, color and texel options.
fn vertex_flags(has_normals: bool, has_colors: bool, has_texels: bool) -> Graphic3dArrayFlags {
    flag_if(has_normals, VERTEX_NORMAL)
        | flag_if(has_colors, VERTEX_COLOR)
        | flag_if(has_texels, VERTEX_TEXEL)
}

impl Graphic3dArrayOfTriangles {
    /// Triangles with flags; `Graphic3dArrayOfTriangles::new(3, 0, 0)?`.
    pub fn new(
        max_vertexs: i32,
        max_edges: i32,
        flags: Graphic3dArrayFlags,
    ) -> Result<Self, Failure> {
        Ok(Self {
            base: Graphic3dArrayOfPrimitives::new(
                Kind::Triangles,
                max_vertexs,
                0,
                max_edges,
                flags,
            )?,
        })
    }
    /// Triangles with optional normals, colors and texels; `Graphic3dArrayOfTriangles::with_options(3, 0, true, false, false)?`.
    pub fn with_options(
        max_vertexs: i32,
        max_edges: i32,
        has_normals: bool,
        has_colors: bool,
        has_texels: bool,
    ) -> Result<Self, Failure> {
        Self::new(
            max_vertexs,
            max_edges,
            vertex_flags(has_normals, has_colors, has_texels),
        )
    }
}

impl Graphic3dArrayOfQuadrangles {
    /// Quadrangles with flags; `Graphic3dArrayOfQuadrangles::new(4, 0, 0)?`.
    pub fn new(
        max_vertexs: i32,
        max_edges: i32,
        flags: Graphic3dArrayFlags,
    ) -> Result<Self, Failure> {
        Ok(Self {
            base: Graphic3dArrayOfPrimitives::new(
                Kind::Quadrangles,
                max_vertexs,
                0,
                max_edges,
                flags,
            )?,
        })
    }
    /// Quadrangles with optional normals, colors and texels; `Graphic3dArrayOfQuadrangles::with_options(4, 0, true, false, false)?`.
    pub fn with_options(
        max_vertexs: i32,
        max_edges: i32,
        has_normals: bool,
        has_colors: bool,
        has_texels: bool,
    ) -> Result<Self, Failure> {
        Self::new(
            max_vertexs,
            max_edges,
            vertex_flags(has_normals, has_colors, has_texels),
        )
    }
}

/// The options of the strip, fan and polygon classes: vertex normals, colors and texels, and
/// bound colors.
#[derive(Clone, Copy, Debug, Default)]
pub struct Graphic3dArrayOptions {
    /// Per-vertex normals.
    pub has_normals: bool,
    /// Per-vertex colors.
    pub has_colors: bool,
    /// Per-bound (strip, fan, polygon) colors.
    pub has_bound_colors: bool,
    /// Per-vertex texels.
    pub has_texels: bool,
}
impl Graphic3dArrayOptions {
    fn flags(self) -> Graphic3dArrayFlags {
        vertex_flags(self.has_normals, self.has_colors, self.has_texels)
            | flag_if(self.has_bound_colors, BOUND_COLOR)
    }
}

/// The bounded classes: `new(maxVertexs, maxBounds, flags)` and `with_options`.
macro_rules! bounded_class {
    ($ty:ident, $kind:ident) => {
        impl $ty {
            /// The array with flags; `new(8, 2, 0)?`.
            pub fn new(
                max_vertexs: i32,
                max_bounds: i32,
                flags: Graphic3dArrayFlags,
            ) -> Result<Self, Failure> {
                Ok(Self {
                    base: Graphic3dArrayOfPrimitives::new(
                        Kind::$kind,
                        max_vertexs,
                        max_bounds,
                        0,
                        flags,
                    )?,
                })
            }
            /// The array with options; `with_options(8, 2, Graphic3dArrayOptions::default())?`.
            pub fn with_options(
                max_vertexs: i32,
                max_bounds: i32,
                options: Graphic3dArrayOptions,
            ) -> Result<Self, Failure> {
                Self::new(max_vertexs, max_bounds, options.flags())
            }
        }
    };
}
bounded_class!(Graphic3dArrayOfTriangleStrips, Trianglestrips);
bounded_class!(Graphic3dArrayOfTriangleFans, Trianglefans);
bounded_class!(Graphic3dArrayOfQuadrangleStrips, Quadranglestrips);

impl Graphic3dArrayOfPolygons {
    /// Polygons with flags; `Graphic3dArrayOfPolygons::new(8, 2, 0, 0)?`.
    pub fn new(
        max_vertexs: i32,
        max_bounds: i32,
        max_edges: i32,
        flags: Graphic3dArrayFlags,
    ) -> Result<Self, Failure> {
        Ok(Self {
            base: Graphic3dArrayOfPrimitives::new(
                Kind::Polygons,
                max_vertexs,
                max_bounds,
                max_edges,
                flags,
            )?,
        })
    }
    /// Polygons with options; `Graphic3dArrayOfPolygons::with_options(8, 2, 0, Graphic3dArrayOptions::default())?`.
    pub fn with_options(
        max_vertexs: i32,
        max_bounds: i32,
        max_edges: i32,
        options: Graphic3dArrayOptions,
    ) -> Result<Self, Failure> {
        Self::new(max_vertexs, max_bounds, max_edges, options.flags())
    }
}

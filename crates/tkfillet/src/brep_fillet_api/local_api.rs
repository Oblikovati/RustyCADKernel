// Rust translation of the members BRepFilletAPI_MakeFillet and BRepFilletAPI_MakeChamfer
// share word for word (BRepFilletAPI_MakeFillet.cxx, BRepFilletAPI_MakeChamfer.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The members the 3d fillet and chamfer algorithms share: `Build`, the history
//! (`Generated`, `Modified`, `IsDeleted`), `Builder` and the contour interface
//! (`BRepFilletAPI_LocalOperation`), all forwarded to their ChFi3d builder.
use rustycad_tkbool::top_ope_brep_build::TopOpeBRepBuildHBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

/// The faces of the result added to the map (`creation of the Map` of `Build`).
pub(super) fn map_faces(shape: &Shape, map: &mut MapOfShape) -> Result<(), Failure> {
    let mut ex = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while ex.more() {
        map.add(ex.current().clone());
        ex.next()?;
    }
    Ok(())
}

/// `Modified(F)`: the splits of the face out of, in and on the result, in this order.
pub(super) fn modified_splits(
    builder: &Handle<TopOpeBRepBuildHBuilder>,
    f: &Shape,
    out: &mut List<Shape>,
) {
    out.clear();
    for st in [State::Out, State::In, State::On] {
        if builder.is_split(f, st) {
            for s in builder.splits(f, st).iter() {
                out.append(s.clone());
            }
        }
    }
}

/// `IsDeleted(F)`: the face is neither a face of the result nor split.
pub(super) fn is_deleted(
    builder: &Handle<TopOpeBRepBuildHBuilder>,
    map: &MapOfShape,
    f: &Shape,
) -> bool {
    !(map.contains(f)
        || builder.is_split(f, State::Out)
        || builder.is_split(f, State::In)
        || builder.is_split(f, State::On))
}

/// Implements `BRepBuilderAPI_MakeShape` and `BRepFilletAPI_LocalOperation` for an
/// algorithm with the fields `base`, `my_builder` (a ChFi3d builder) and `my_map`.
macro_rules! impl_local_api {
    ($class:ty) => {
        impl $class {
            /// `Builder()`: the builder of the result; `api.builder()`.
            pub fn builder(
                &self,
            ) -> rustycad_tkernel::standard::Handle<
                rustycad_tkbool::top_ope_brep_build::TopOpeBRepBuildHBuilder,
            > {
                self.my_builder.builder_base().builder()
            }

            /// The spine of the contour `i` (Standard_NullObject without one).
            fn contour_spine(
                &self,
                i: i32,
            ) -> Result<crate::chfi_ds::SpineHandle, rustycad_tkernel::standard::Failure> {
                self.my_builder.builder_base().value(i)?.ok_or_else(|| {
                    rustycad_tkernel::standard::Failure::with_message(
                        "Standard_NullObject",
                        "BRepFilletAPI: contour without spine",
                    )
                })
            }
        }

        impl rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape for $class {
            fn make_shape_base(
                &self,
            ) -> &rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase {
                &self.base
            }
            fn make_shape_base_mut(
                &mut self,
            ) -> &mut rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase {
                &mut self.base
            }
            /// `Build()`: the builder computed; when done, the result and the map of its
            /// faces.
            fn build(
                &mut self,
                _range: &rustycad_tkernel::message::ProgressRange<'_>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.my_builder.compute()?;
                if self.my_builder.builder_base().is_done() {
                    self.base.set_done();
                    self.base.my_shape = self.my_builder.builder_base().shape()?;
                    super::local_api::map_faces(&self.base.my_shape, &mut self.my_map)?;
                }
                Ok(())
            }
            /// `Generated(EorV)`: the faces generated from an edge or a vertex.
            fn generated(
                &mut self,
                eor_v: &rustycad_tkbrep::topods::Shape,
            ) -> Result<
                &rustycad_tkernel::ncollection::List<rustycad_tkbrep::topods::Shape>,
                rustycad_tkernel::standard::Failure,
            > {
                self.my_builder.builder_base_mut().generated(eor_v)
            }
            /// `Modified(F)`: the splits of the face.
            fn modified(
                &mut self,
                f: &rustycad_tkbrep::topods::Shape,
            ) -> Result<
                &rustycad_tkernel::ncollection::List<rustycad_tkbrep::topods::Shape>,
                rustycad_tkernel::standard::Failure,
            > {
                let b = self.builder();
                super::local_api::modified_splits(&b, f, &mut self.base.my_generated);
                Ok(&self.base.my_generated)
            }
            /// `IsDeleted(F)`.
            fn is_deleted(
                &mut self,
                f: &rustycad_tkbrep::topods::Shape,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                Ok(super::local_api::is_deleted(
                    &self.builder(),
                    &self.my_map,
                    f,
                ))
            }
        }

        impl super::local_operation::BRepFilletAPILocalOperation for $class {
            fn add(
                &mut self,
                e: &rustycad_tkbrep::topods::Edge,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.my_builder.add(e)
            }
            fn reset_contour(
                &mut self,
                ic: i32,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.my_builder.reset_contour(ic)
            }
            fn nb_contours(&self) -> i32 {
                self.my_builder.builder_base().nb_elements()
            }
            fn contour(&self, e: &rustycad_tkbrep::topods::Edge) -> i32 {
                self.my_builder.builder_base().contains(e).unwrap_or(0)
            }
            fn nb_edges(&self, i: i32) -> Result<i32, rustycad_tkernel::standard::Failure> {
                Ok(crate::chfi_ds::lock(&self.contour_spine(i)?).nb_edges())
            }
            fn edge(
                &self,
                i: i32,
                j: i32,
            ) -> Result<rustycad_tkbrep::topods::Edge, rustycad_tkernel::standard::Failure> {
                Ok(crate::chfi_ds::lock(&self.contour_spine(i)?)
                    .edges(j)?
                    .clone())
            }
            fn remove(
                &mut self,
                e: &rustycad_tkbrep::topods::Edge,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.my_builder.builder_base_mut().remove(e)
            }
            fn length(&self, ic: i32) -> Result<f64, rustycad_tkernel::standard::Failure> {
                self.my_builder.builder_base().length(ic)
            }
            fn first_vertex(
                &self,
                ic: i32,
            ) -> Result<rustycad_tkbrep::topods::Vertex, rustycad_tkernel::standard::Failure> {
                self.my_builder.builder_base().first_vertex(ic)
            }
            fn last_vertex(
                &self,
                ic: i32,
            ) -> Result<rustycad_tkbrep::topods::Vertex, rustycad_tkernel::standard::Failure> {
                self.my_builder.builder_base().last_vertex(ic)
            }
            fn abscissa(
                &self,
                ic: i32,
                v: &rustycad_tkbrep::topods::Vertex,
            ) -> Result<f64, rustycad_tkernel::standard::Failure> {
                self.my_builder.builder_base().abscissa(ic, v)
            }
            fn relative_abscissa(
                &self,
                ic: i32,
                v: &rustycad_tkbrep::topods::Vertex,
            ) -> Result<f64, rustycad_tkernel::standard::Failure> {
                self.my_builder.builder_base().relative_abscissa(ic, v)
            }
            fn closed_and_tangent(
                &self,
                ic: i32,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.my_builder.builder_base().closed_and_tangent(ic)
            }
            fn closed(&self, ic: i32) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.my_builder.builder_base().closed(ic)
            }
            /// `Reset()`: not done, the builder reset, the map of the faces cleared.
            fn reset(&mut self) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.base.set_not_done();
                self.my_builder.builder_base_mut().reset()?;
                self.my_map.clear(false);
                Ok(())
            }
            fn simulate(&mut self, ic: i32) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.my_builder.simulate(ic)
            }
            fn nb_surf(&self, ic: i32) -> Result<i32, rustycad_tkernel::standard::Failure> {
                Ok(self.my_builder.nb_surf(ic))
            }
            fn sect(
                &self,
                ic: i32,
                is: i32,
            ) -> Result<
                rustycad_tkernel::standard::Handle<
                    rustycad_tkernel::ncollection::HArray1<crate::chfi_ds::CircSection>,
                >,
                rustycad_tkernel::standard::Failure,
            > {
                self.my_builder.sect(ic, is)
            }
        }
    };
}
pub(super) use impl_local_api;

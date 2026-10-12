// Reuse the genuine native recorder and concrete Compute fixture; no copied shading algorithm.
#include <Prs3d_ShadingAspect.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#define main martensite_native_compute_matrix_entry
#include "martensite_textured_shape_compute.cxx"
#undef main
int main() {
  V3DisplayTrace trace;
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  const occ::handle<Prs3d_Presentation> prs = new Prs3d_Presentation(viewer->StructureManager());
  NativeTexturedCompute object(native_textured_face(1));
  configure_textured_compute(object,0); object.SetTextureFileName("0");
  // Each fresh process has a deliberately invalid texture directory, provoking the real constructor failure.
  object.build(prs,3);
  const bool originalAspect = prs->Groups().Value(1)->Aspects() == object.Attributes()->ShadingAspect()->Aspect();
  std::cout << "MARTENSITE_COMPUTE_FAILURE " << prs->Groups().Size() << ' '
    << !object.aspectNull() << ' ' << object.textureNull() << ' ' << originalAspect << '\n';
  return 0;
}

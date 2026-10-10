// Paired with tests/support/homogeneous_arrays.rs, using native Vec4 input attributes.
#include <Graphic3d_Buffer.hxx>
#include <array>
static std::array<float,3> homogeneous_weights(const std::string& mode) {
  const auto suffix = mode.substr(mode.rfind('-') + 1);
  if (suffix == "unit") return {1,1,1};
  if (suffix == "double") return {2,2,2};
  if (suffix == "infinite") return {1,0,0};
  if (suffix == "negative") return {1,1,-1};
  if (suffix == "behind") return {1,1,-2};
  if (suffix == "mixed" || suffix == "clipped") return {.5f,2,1};
  throw Standard_Failure(("received " + mode + "; expected a homogeneous position mode").c_str());
}
static occ::handle<Graphic3d_Buffer> homogeneous_buffer(const std::string& mode) {
  occ::handle<Graphic3d_Buffer> buffer = new Graphic3d_Buffer(Graphic3d_Buffer::DefaultAllocator());
  const Graphic3d_Attribute attribute = {Graphic3d_TOA_POS, Graphic3d_TOD_VEC4};
  if (!buffer->Init(3, &attribute, 1)) throw Standard_Failure("homogeneous buffer allocation failed");
  const auto weights = homogeneous_weights(mode);
  auto vertices = reinterpret_cast<NCollection_Vec4<float>*>(buffer->ChangeData());
  vertices[0] = NCollection_Vec4<float>(-8,-8,0,weights[0]);
  vertices[1] = NCollection_Vec4<float>(8,-8,0,weights[1]);
  vertices[2] = NCollection_Vec4<float>(0,8,0,weights[2]);
  return buffer;
}
static void homogeneous_geometry(const occ::handle<Graphic3d_Group>& group, const std::string& mode) {
  group->Clear(false);
  auto aspects = unlit_aspects();
  aspects->SetMarkerType(Aspect_TOM_PLUS); aspects->SetMarkerScale(2);
  group->SetGroupPrimitivesAspect(aspects);
  auto kind = mode.rfind("hom-line-",0)==0 ? Graphic3d_TOPA_POLYLINES :
              mode.rfind("hom-point-",0)==0 ? Graphic3d_TOPA_POINTS : Graphic3d_TOPA_TRIANGLES;
  group->AddPrimitiveArray(kind, nullptr, homogeneous_buffer(mode), nullptr, true);
}
static void homogeneous_scene(const occ::handle<Graphic3d_Structure>& structure,
                              const occ::handle<V3d_View>& view, const std::string& mode) {
  homogeneous_geometry(structure->Groups().First(), mode);
  if (mode.find("clipped") != std::string::npos) {
    occ::handle<Graphic3d_SequenceOfHClipPlane> planes = new Graphic3d_SequenceOfHClipPlane();
    planes->Append(clip_plane(1,0,-1)); view->SetClipPlanes(planes);
  }
  structure->CalculateBoundBox();
}

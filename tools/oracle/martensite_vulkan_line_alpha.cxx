#include <Graphic3d_ClipPlane.hxx>
#include <Graphic3d_SequenceOfHClipPlane.hxx>
// Native OpenGl_PrimitiveArray forced MaskBlend, opaque-pass order and depth at 3d097a0.
static void alpha_line_style(const occ::handle<Graphic3d_Aspects>& style,
                             const std::string& mode) {
  style->SetInteriorColor(Quantity_ColorRGBA(Quantity_Color(1.,0.,0.,Quantity_TOC_RGB),
      mode == "line-alpha-zero" ? 0.f : 0.25f));
  Graphic3d_AlphaMode alpha = Graphic3d_AlphaMode_MaskBlend;
  if (mode == "line-alpha-opaque") alpha = Graphic3d_AlphaMode_Opaque;
  if (mode == "line-alpha-auto") alpha = Graphic3d_AlphaMode_BlendAuto;
  if (mode == "line-alpha-blend") alpha = Graphic3d_AlphaMode_Blend;
  if (mode == "line-alpha-mask") alpha = Graphic3d_AlphaMode_Mask;
  style->SetAlphaMode(alpha, 0.9f);
  style->SetLineWidth(mode == "line-alpha-wide" ? 3.f : 1.f);
  style->SetLineType(mode == "line-alpha-dash" ? Aspect_TOL_DASH : Aspect_TOL_SOLID);
}
static void alpha_line_vertices(const occ::handle<Graphic3d_ArrayOfPrimitives>& array,
                                const std::string& mode, bool colored) {
  const uint8_t colors[3][4] = {{255,0,0,64},{0,255,0,128},{255,255,255,0}};
  // Fractional Y isolates alpha behavior from known integer-boundary raster row differences.
  const float ys[3] = {-4.125f,0.375f,4.625f};
  const float zs[3] = {2.f,0.f,-2.f};
  for (int segment = 0; segment < 3; ++segment) {
    for (float x : {-8.f,8.f}) {
      int index = array->AddVertex(x, mode == "line-alpha-depth" ? 0.375f : ys[segment],
                                   mode == "line-alpha-depth" ? zs[segment] : 0.f);
      if (!colored) continue;
      const auto& c = colors[segment];
      array->SetVertexColor(index, NCollection_Vec4<uint8_t>(c[0],c[1],c[2],c[3]));
    }
  }
}
static occ::handle<Graphic3d_ArrayOfPrimitives> alpha_line_array(const std::string& mode) {
  bool colored = mode == "line-alpha-vertices" || mode == "line-alpha-depth";
  auto flags = Graphic3d_ArrayFlags(colored ? Graphic3d_ArrayFlags_VertexColor : 0);
  occ::handle<Graphic3d_ArrayOfPrimitives> array =
      new MartensiteOracleBoundArray(Graphic3d_TOPA_SEGMENTS, 6, 0, 0, flags);
  alpha_line_vertices(array, mode, colored);
  return array;
}
static void alpha_line_clipping(const occ::handle<Graphic3d_Structure>& structure,
                                const std::string& mode) {
  if (mode != "line-alpha-clipped") return;
  occ::handle<Graphic3d_SequenceOfHClipPlane> planes = new Graphic3d_SequenceOfHClipPlane;
  planes->Append(new Graphic3d_ClipPlane(NCollection_Vec4<double>(1.,0.,0.,0.)));
  structure->SetClipPlanes(planes);
}
static void alpha_line_scene(const occ::handle<Graphic3d_Structure>& structure,
                             const std::string& mode) {
  structure->Clear();
  auto group = structure->NewGroup();
  auto style = unlit_aspects();
  alpha_line_style(style, mode);
  group->SetGroupPrimitivesAspect(style);
  group->AddPrimitiveArray(alpha_line_array(mode));
  alpha_line_clipping(structure, mode);
}

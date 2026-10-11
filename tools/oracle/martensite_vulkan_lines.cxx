// Native line assembly and shader stipple at OCCT 3d097a0.
#include "martensite_vulkan_bound_array.hxx"
#include <array>
#include "martensite_vulkan_line_alpha.cxx"
static occ::handle<Graphic3d_ArrayOfPrimitives>
line_vertices(const std::string &mode) {
  const bool segments = mode.rfind("line-segments", 0) == 0;
  const bool indexed = mode.find("indexed") != std::string::npos;
  const bool colored = mode == "line-vertex-colors" || mode == "line-gradient";
  const auto flags =
      Graphic3d_ArrayFlags(Graphic3d_ArrayFlags_BoundColor |
                           (colored ? Graphic3d_ArrayFlags_VertexColor : 0));
  occ::handle<Graphic3d_ArrayOfPrimitives> array =
      new MartensiteOracleBoundArray(segments ? Graphic3d_TOPA_SEGMENTS
                                              : Graphic3d_TOPA_POLYLINES,
                                     8, 2, indexed ? 8 : 0, flags);
  const std::array<std::array<float, 3>, 4> points = {
      {{-8, -6, 0}, {-2, -6, 0}, {-2, 6, 0}, {-8, 2, 0}}};
  for (float shift : {0.f, 10.f})
    for (const auto &p : points)
      array->AddVertex(p[0] + shift, p[1], p[2]);
  array->AddBound(4, 1, 0, 0);
  array->AddBound(4, 0, 1, 0);
  for (int i = 1; i <= 8; ++i) {
    if (indexed)
      array->AddEdge(i);
    if (colored) {
      const float colors[4][3] = {{1, 1, 0}, {1, 0, 1}, {0, 1, 1}, {1, 1, 1}};
      const auto &c = colors[mode == "line-gradient" ? (i - 1) % 4 : 0];
      array->SetVertexColor(i, c[0], c[1], c[2]);
    }
    if (mode == "line-depth") {
      const auto p = array->Vertice(i);
      const float depth[4] = {0, 2, -1, 3};
      array->SetVertice(i, float(p.X()), float(p.Y()), depth[(i - 1) % 4]);
    }
  }
  return array;
}
static void configure_line_style(const occ::handle<Graphic3d_Aspects> &aspects,
                                 const std::string &mode) {
  aspects->SetLineWidth(mode == "line-wide"         ? 3.f
                        : mode == "line-fractional" ? 2.5f
                                                    : 1.f);
  Aspect_TypeOfLine kind = Aspect_TOL_SOLID;
  if (mode == "line-dash")
    kind = Aspect_TOL_DASH;
  else if (mode == "line-dot")
    kind = Aspect_TOL_DOT;
  else if (mode == "line-dotdash")
    kind = Aspect_TOL_DOTDASH;
  else if (mode == "line-empty")
    kind = Aspect_TOL_EMPTY;
  aspects->SetLineType(kind);
  if (mode == "line-custom") {
    aspects->SetLinePattern(0x8421);
    aspects->SetLineStippleFactor(3);
  }
}
static void line_scene(const occ::handle<Graphic3d_Structure> &structure,
                       const std::string &mode) {
  if (mode.rfind("line-alpha-", 0) == 0) {
    alpha_line_scene(structure, mode);
    return;
  }
  structure->Clear();
  auto group = structure->NewGroup();
  auto aspects = unlit_aspects();
  configure_line_style(aspects, mode);
  group->SetGroupPrimitivesAspect(aspects);
  group->AddPrimitiveArray(line_vertices(mode));
}

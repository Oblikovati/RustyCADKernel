// Surface assembly oracle uses native arrays and per-bound colors (OCCT
// 3d097a0).
#include "martensite_vulkan_bound_array.hxx"
#include <Graphic3d_ArrayOfPrimitives.hxx>
#include <array>
#include <vector>
static Graphic3d_TypeOfPrimitiveArray surface_kind(const std::string &mode) {
  if (mode == "surface-bound-colors")
    return Graphic3d_TOPA_TRIANGLES;
  if (mode == "surface-fan" || mode == "surface-fan-indexed")
    return Graphic3d_TOPA_TRIANGLEFANS;
  return Graphic3d_TOPA_TRIANGLESTRIPS;
}
static std::vector<std::array<float, 3>>
surface_points(Graphic3d_TypeOfPrimitiveArray kind) {
  std::vector<std::array<float, 3>> points = {{-8, -6, 0}, {-2, -6, 0}};
  if (kind == Graphic3d_TOPA_TRIANGLES)
    points.push_back({-5, 6, 0});
  else if (kind == Graphic3d_TOPA_TRIANGLEFANS) {
    points.push_back({-2, 6, 0});
    points.push_back({-8, 6, 0});
  } else {
    points.push_back({-8, 6, 0});
    points.push_back({-2, 6, 0});
  }
  const auto count = points.size();
  for (size_t i = 0; i < count; ++i) {
    auto point = points[i];
    point[0] += 10;
    points.push_back(point);
  }
  return points;
}
static occ::handle<Graphic3d_ArrayOfPrimitives>
surface_vertices(const std::string &mode) {
  const auto kind = surface_kind(mode);
  const auto points = surface_points(kind);
  const int count = static_cast<int>(points.size());
  const bool indexed = mode != "surface-strip" && mode != "surface-fan";
  const bool colored = mode == "surface-vertex-colors";
  const auto flags =
      Graphic3d_ArrayFlags(Graphic3d_ArrayFlags_BoundColor |
                           (colored ? Graphic3d_ArrayFlags_VertexColor : 0));
  occ::handle<Graphic3d_ArrayOfPrimitives> array =
      new MartensiteOracleBoundArray(kind, count, 2, indexed ? count : 0,
                                     flags);
  for (const auto &p : points)
    array->AddVertex(p[0], p[1], p[2]);
  array->AddBound(count / 2, 1, 0, 0);
  array->AddBound(count / 2, 0, 1, 0);
  for (int i = 1; i <= count; ++i) {
    if (indexed)
      array->AddEdge(i);
    if (colored)
      array->SetVertexColor(i, 1, 1, 0);
  }
  return array;
}

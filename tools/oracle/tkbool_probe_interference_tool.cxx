// Factory/aliasing oracle for TopOpeBRepDS_InterferenceTool at OCCT 3d097a0.
#include <TopOpeBRepDS_InterferenceTool.hxx>
#include <TopOpeBRepDS_CurvePointInterference.hxx>
#include <TopOpeBRepDS_EdgeVertexInterference.hxx>
#include <TopOpeBRepDS_FaceEdgeInterference.hxx>
#include <TopOpeBRepDS_SurfaceCurveInterference.hxx>
#include <Geom2d_Line.hxx>
using BlInterferenceTool = TopOpeBRepDS_InterferenceTool;

static void bl_interference_fields(const occ::handle<TopOpeBRepDS_Interference>& interference) {
  ga_text(interference->DynamicType()->Name());
  ga_int(interference->SupportType()); ga_int(interference->Support());
  ga_int(interference->GeometryType()); ga_int(interference->Geometry());
  bl_transition(interference->Transition());
}

static void bl_interference_curves(const TopOpeBRepDS_Transition& transition, int support, int geometry, double parameter) {
  auto edge = BlInterferenceTool::MakeEdgeInterference(transition, TopOpeBRepDS_EDGE, support, TopOpeBRepDS_VERTEX, geometry, parameter);
  auto curve = BlInterferenceTool::MakeCurveInterference(transition, TopOpeBRepDS_CURVE, support, TopOpeBRepDS_POINT, geometry, parameter);
  bl_interference_fields(edge); ga_real(BlInterferenceTool::Parameter(edge));
  bl_interference_fields(curve); ga_real(BlInterferenceTool::Parameter(curve));
  auto copy = BlInterferenceTool::DuplicateCurvePointInterference(curve);
  bl_interference_fields(copy); ga_flag(copy != curve);
  auto alias = curve;
  BlInterferenceTool::Parameter(curve, parameter + 1.0);
  ga_real(BlInterferenceTool::Parameter(alias)); ga_real(BlInterferenceTool::Parameter(copy));
}

static void bl_interference_surfaces(const TopOpeBRepDS_Transition& transition, int support, int geometry, bool hasCurve) {
  occ::handle<Geom2d_Curve> pcurve;
  if (hasCurve) pcurve = new Geom2d_Line(gp_Pnt2d(1, 2), gp_Dir2d(1, 0));
  auto face = BlInterferenceTool::MakeFaceCurveInterference(transition, support, geometry, pcurve);
  bl_interference_fields(face);
  auto typed = occ::down_cast<TopOpeBRepDS_SurfaceCurveInterference>(face);
  ga_flag(typed->PCurve() == pcurve); ga_flag(typed->PCurve().IsNull());
  if (!pcurve.IsNull()) ga_pnt2d(typed->PCurve()->Value(0.5));
  bl_interference_fields(BlInterferenceTool::MakeSolidSurfaceInterference(transition, support, geometry));
}

static void bl_interference_shapes(const TopOpeBRepDS_Transition& transition, int support, int geometry,
                                   double parameter, TopOpeBRepDS_Config config, bool bound) {
  auto edge = BlInterferenceTool::MakeEdgeVertexInterference(transition, support, geometry, bound, config, parameter);
  bl_interference_fields(edge);
  auto vertex = occ::down_cast<TopOpeBRepDS_EdgeVertexInterference>(edge);
  ga_flag(vertex->GBound()); ga_int(vertex->Config()); ga_real(vertex->Parameter());
  auto face = BlInterferenceTool::MakeFaceEdgeInterference(transition, support, geometry, bound, config);
  bl_interference_fields(face);
  auto typed = occ::down_cast<TopOpeBRepDS_FaceEdgeInterference>(face);
  ga_flag(typed->GBound()); ga_int(typed->Config());
}

static void bl_interference_tool_case(std::istream& input) {
  GaReader reader{input};
  TopOpeBRepDS_Transition transition((TopAbs_Orientation)reader.integer());
  transition.IndexBefore(11); transition.IndexAfter(13);
  const int support = reader.integer(), geometry = reader.integer();
  const double parameter = reader.real();
  const auto config = (TopOpeBRepDS_Config)reader.integer();
  const bool bound = reader.integer() != 0;
  bl_interference_curves(transition, support, geometry, parameter);
  bl_interference_surfaces(transition, support, geometry, bound);
  bl_interference_shapes(transition, support, geometry, parameter, config, bound); ga_end();
}

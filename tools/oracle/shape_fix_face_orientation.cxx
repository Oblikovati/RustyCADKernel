#include <Precision.hxx>
#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Tool.hxx>
#include <BRepTopAdaptor_FClass2d.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <NCollection_DataMap.hxx>
#include <NCollection_List.hxx>
#include <ShapeFix_Face.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_MsgRegistrator.hxx>
#include <TopAbs_Orientation.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Wire.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TCollection_AsciiString.hxx>
#include <gp_Pln.hxx>
#include <gp_Sphere.hxx>
#include <gp_Cylinder.hxx>
#include <iostream>
#include <string>

static TopoDS_Wire square(double x0, double y0, double x1, double y1, bool reverse = false) {
  BRepBuilderAPI_MakePolygon polygon;
  if (reverse) {
    polygon.Add(gp_Pnt(x0, y0, 0)); polygon.Add(gp_Pnt(x0, y1, 0));
    polygon.Add(gp_Pnt(x1, y1, 0)); polygon.Add(gp_Pnt(x1, y0, 0));
  } else {
    polygon.Add(gp_Pnt(x0, y0, 0)); polygon.Add(gp_Pnt(x1, y0, 0));
    polygon.Add(gp_Pnt(x1, y1, 0)); polygon.Add(gp_Pnt(x0, y1, 0));
  }
  polygon.Close();
  return polygon.Wire();
}

static TopoDS_Face face_with_wire(const gp_Pln& plane, const TopoDS_Wire& wire) {
  TopoDS_Face face;
  BRep_Builder builder;
  occ::handle<Geom_Surface> surface = new Geom_Plane(plane);
  builder.MakeFace(face, surface, Precision::Confusion());
  builder.Add(face, wire);
  return face;
}

static void print_case(const char* name, TopoDS_Face face, bool mode = true) {
  ShapeFix_Face fixer(face);
  fixer.FixAddNaturalBoundMode() = mode ? 1 : 0;
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher> map;
  bool changed = fixer.FixOrientation(map);
  int wires = 0, forward = 0, reversed = 0, internal = 0, mappedChildren = 0;
  for (decltype(map)::Iterator it(map); it.More(); it.Next()) mappedChildren += it.Value().Extent();
  for (TopExp_Explorer it(fixer.Face(), TopAbs_WIRE); it.More(); it.Next()) {
    ++wires;
    TopAbs_Orientation orientation = it.Current().Orientation();
    forward += orientation == TopAbs_FORWARD;
    reversed += orientation == TopAbs_REVERSED;
    internal += orientation == TopAbs_INTERNAL;
  }
  std::cout << name << " changed=" << changed << " map=" << map.Extent()
            << " wires=" << wires << " forward=" << forward
            << " reversed=" << reversed << " internal=" << internal
            << " mapped_children=" << mappedChildren << '\n';
}

static void print_periodic_shift_case() {
  const gp_Cylinder cylinder(gp_Ax3(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 1.0);
  occ::handle<Geom_Surface> surface = new Geom_CylindricalSurface(cylinder);
  BRepBuilderAPI_MakeFace leftPatch(surface, 5.8, 6.4, 1.0, 4.0, Precision::Confusion());
  BRepBuilderAPI_MakeFace rightPatch(surface, 0.1, 0.5, 1.0, 4.0, Precision::Confusion());
  TopoDS_Wire leftWire = TopoDS::Wire(TopExp_Explorer(leftPatch.Face(), TopAbs_WIRE).Current());
  TopoDS_Wire rightWire = TopoDS::Wire(TopExp_Explorer(rightPatch.Face(), TopAbs_WIRE).Current());
  TopoDS_Face face;
  BRep_Builder builder;
  builder.MakeFace(face, surface, Precision::Confusion());
  builder.Add(face, leftWire);
  builder.Add(face, rightWire);
  std::cout << "periodic_edge_counts="
            << TopExp_Explorer(leftWire, TopAbs_EDGE).More() << ":";
  int leftCount = 0, rightCount = 0;
  for (TopExp_Explorer it(leftWire, TopAbs_EDGE); it.More(); it.Next()) ++leftCount;
  for (TopExp_Explorer it(rightWire, TopAbs_EDGE); it.More(); it.Next()) ++rightCount;
  std::cout << leftCount << ',' << rightCount << '\n';
  TopoDS_Wire sourceWire = leftWire;
  TopoDS_Face single;
  builder.MakeFace(single, surface, Precision::Confusion());
  builder.Add(single, sourceWire);
  BRepTopAdaptor_FClass2d classifier(single, Precision::PConfusion());
  std::cout << "periodic_states infinite=" << static_cast<int>(classifier.PerformInfinitePoint());
  for (TopoDS_Iterator edgeIt(rightWire); edgeIt.More(); edgeIt.Next()) {
    double edgeFirst = 0, edgeLast = 0;
    occ::handle<Geom2d_Curve> candidate = BRep_Tool::CurveOnSurface(
      TopoDS::Edge(edgeIt.Value()), face, edgeFirst, edgeLast);
    gp_Pnt2d sample = candidate->Value((edgeFirst + edgeLast) / 2);
    std::cout << " " << sample.X() << "," << sample.Y() << "=" << static_cast<int>(classifier.Perform(sample, false));
  }
  std::cout << '\n';
  TopoDS_Face secondSingle;
  builder.MakeFace(secondSingle, surface, Precision::Confusion());
  builder.Add(secondSingle, rightWire);
  BRepTopAdaptor_FClass2d secondClassifier(secondSingle, Precision::PConfusion());
  BRepBuilderAPI_MakeFace shiftedPatch(surface, 0.1 + 2 * M_PI, 0.5 + 2 * M_PI, 1.0, 4.0, Precision::Confusion());
  TopoDS_Wire shiftedRight = TopoDS::Wire(TopExp_Explorer(shiftedPatch.Face(), TopAbs_WIRE).Current());
  TopoDS_Face shiftedSingle;
  builder.MakeFace(shiftedSingle, surface, Precision::Confusion());
  builder.Add(shiftedSingle, shiftedRight);
  BRepTopAdaptor_FClass2d shiftedClassifier(shiftedSingle, Precision::PConfusion());
  std::cout << "periodic_shifted_states infinite=" << static_cast<int>(shiftedClassifier.PerformInfinitePoint());
  for (TopoDS_Iterator edgeIt(leftWire); edgeIt.More(); edgeIt.Next()) {
    double edgeFirst = 0, edgeLast = 0;
    occ::handle<Geom2d_Curve> candidate = BRep_Tool::CurveOnSurface(TopoDS::Edge(edgeIt.Value()), face, edgeFirst, edgeLast);
    gp_Pnt2d sample = candidate->Value((edgeFirst + edgeLast) / 2);
    std::cout << " " << sample.X() << "," << sample.Y() << "=" << static_cast<int>(shiftedClassifier.Perform(sample, false));
  }
  std::cout << '\n';
  std::cout << "periodic_reverse_states infinite=" << static_cast<int>(secondClassifier.PerformInfinitePoint());
  for (TopoDS_Iterator edgeIt(leftWire); edgeIt.More(); edgeIt.Next()) {
    double edgeFirst = 0, edgeLast = 0;
    occ::handle<Geom2d_Curve> candidate = BRep_Tool::CurveOnSurface(
      TopoDS::Edge(edgeIt.Value()), face, edgeFirst, edgeLast);
    gp_Pnt2d sample = candidate->Value((edgeFirst + edgeLast) / 2);
    std::cout << " " << sample.X() << "," << sample.Y() << "=" << static_cast<int>(secondClassifier.Perform(sample, false));
  }
  std::cout << '\n';
  TopoDS_Edge edge = TopoDS::Edge(TopExp_Explorer(rightWire, TopAbs_EDGE).Current());
  double first = 0, last = 0;
  occ::handle<Geom2d_Curve> pcurve = BRep_Tool::CurveOnSurface(edge, face, first, last);
  const double before = pcurve->Value((first + last) / 2).X();
  ShapeFix_Face fixer(face);
  occ::handle<ShapeExtend_MsgRegistrator> messages = new ShapeExtend_MsgRegistrator();
  fixer.SetMsgRegistrator(messages);
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher> map;
  const bool changed = fixer.FixOrientation(map);
  int forward = 0, reversed = 0;
  int edgeForward = 0, edgeReversed = 0;
  for (TopExp_Explorer it(fixer.Face(), TopAbs_WIRE); it.More(); it.Next()) {
    forward += it.Current().Orientation() == TopAbs_FORWARD;
    reversed += it.Current().Orientation() == TopAbs_REVERSED;
  }
  for (TopExp_Explorer it(fixer.Face(), TopAbs_EDGE); it.More(); it.Next()) {
    edgeForward += it.Current().Orientation() == TopAbs_FORWARD;
    edgeReversed += it.Current().Orientation() == TopAbs_REVERSED;
  }
  pcurve = BRep_Tool::CurveOnSurface(edge, face, first, last);
  const double after = pcurve->Value((first + last) / 2).X();
  std::cout << "periodic_messages";
  for (NCollection_DataMap<TopoDS_Shape, NCollection_List<Message_Msg>, TopTools_ShapeMapHasher>::Iterator it(messages->MapShape()); it.More(); it.Next()) {
    TCollection_AsciiString text(it.Value().First().Original(), '?');
    std::cout << " " << text.ToCString() << ":L=" << it.Key().IsSame(leftWire) << ":R=" << it.Key().IsSame(rightWire);
  }
  std::cout << '\n';
  std::cout << "periodic_shift changed=" << changed << " pcurve_mid_before=" << before
            << " pcurve_mid_after=" << after << " map=" << map.Extent()
            << " forward=" << forward << " reversed=" << reversed
            << " warning_shapes=" << messages->MapShape().Extent()
            << " edge_forward=" << edgeForward << " edge_reversed=" << edgeReversed << '\n';
}

static void print_shared_seam_reversal_case() {
  const gp_Cylinder cylinder(gp_Ax3(), 1.0);
  occ::handle<Geom_Surface> surface = new Geom_CylindricalSurface(cylinder);
  BRepBuilderAPI_MakeFace patch(surface, 0.0, 2.0 * M_PI, 1.0, 4.0, Precision::Confusion());
  TopoDS_Wire wire = TopoDS::Wire(TopExp_Explorer(patch.Face(), TopAbs_WIRE).Current());
  TopoDS_Wire reversed = wire;
  reversed.Reverse();
  TopoDS_Face face;
  BRep_Builder builder;
  builder.MakeFace(face, surface, Precision::Confusion());
  builder.Add(face, wire);
  builder.Add(face, reversed);

  ShapeFix_Face fixer(face);
  fixer.FixAddNaturalBoundMode() = 0;
  const bool changed = fixer.FixOrientation();
  std::cout << "shared_seam_reversal changed=" << changed;
  for (TopExp_Explorer wi(fixer.Face(), TopAbs_WIRE); wi.More(); wi.Next()) {
    TopoDS_Edge edge = TopoDS::Edge(TopExp_Explorer(wi.Current(), TopAbs_EDGE).Current());
    double first = 0, last = 0;
    const auto curve = BRep_Tool::CurveOnSurface(edge, fixer.Face(), first, last);
    std::cout << " p=" << curve->Value((first + last) * 0.5).X();
  }
  std::cout << '\n';
}

static void print_context_case(const gp_Pln& plane) {
  TopoDS_Face original = face_with_wire(plane, square(20, 20, 24, 24));
  TopoDS_Face replacement = face_with_wire(plane, square(0, 0, 4, 4, true));
  occ::handle<ShapeBuild_ReShape> context = new ShapeBuild_ReShape();
  context->Replace(original, replacement);
  ShapeFix_Face fixer(original);
  fixer.SetContext(context);
  const bool changed = fixer.FixOrientation();
  const TopoDS_Shape applied = context->Apply(original);
  std::cout << "context changed=" << changed
            << " result_matches_context=" << applied.IsSame(fixer.Face())
            << " result_forward=" << (fixer.Face().Orientation() == TopAbs_FORWARD) << '\n';
}

static void print_sphere_natural_bound_case() {
  const gp_Sphere sphere(gp_Ax3(), 2.0);
  occ::handle<Geom_Surface> surface = new Geom_SphericalSurface(sphere);
  BRepBuilderAPI_MakeFace patch(surface, 0.2, 1.0, 0.1, 0.5, Precision::Confusion());
  TopoDS_Wire wire = TopoDS::Wire(TopExp_Explorer(patch.Face(), TopAbs_WIRE).Current());
  wire.Orientation(TopAbs_REVERSED);
  TopoDS_Face face;
  BRep_Builder builder;
  builder.MakeFace(face, surface, Precision::Confusion());
  builder.Add(face, wire);
  ShapeFix_Face fixer(face);
  fixer.FixAddNaturalBoundMode() = 1;
  const bool changed = fixer.FixOrientation();
  TopoDS_Iterator children(fixer.Face(), false);
  std::cout << "sphere_natural_bound changed=" << changed
            << " wire_orientation=" << static_cast<int>(children.Value().Orientation()) << '\n';
}

int main() {
  const gp_Pln plane(gp_Ax3(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
  print_case("single_ccw", face_with_wire(plane, square(0, 0, 4, 4)));
  print_case("single_cw", face_with_wire(plane, square(0, 0, 4, 4, true)));

  TopoDS_Face nested = face_with_wire(plane, square(0, 0, 10, 10));
  BRep_Builder builder;
  builder.Add(nested, square(2, 2, 8, 8));
  builder.Add(nested, square(3, 3, 4, 4));
  print_case("nested", nested);

  TopoDS_Face disjoint = face_with_wire(plane, square(0, 0, 2, 2));
  builder.Add(disjoint, square(5, 0, 7, 2));
  print_case("disjoint", disjoint);

  TopoDS_Face reversed = face_with_wire(plane, square(0, 0, 4, 4));
  reversed.Reverse();
  print_case("reversed_face", reversed);

  TopoDS_Face tiny = face_with_wire(plane, square(0, 0, 4, 4, true));
  TopoDS_Edge shortEdge = BRepBuilderAPI_MakeEdge(gp_Lin(gp_Pnt(6, 6, 0), gp_Dir(1, 0, 0)), 0, 1);
  builder.Range(shortEdge, 0, 1.e-8, true);
  BRepBuilderAPI_MakeWire tinyWire;
  tinyWire.Add(shortEdge);
  builder.Add(tiny, tinyWire.Wire());
  print_case("tiny_wire_removed", tiny);

  TopoDS_Face empty = face_with_wire(plane, square(0, 0, 4, 4, true));
  TopoDS_Wire emptyWire;
  builder.MakeWire(emptyWire);
  builder.Add(empty, emptyWire);
  print_case("empty_wire_removed", empty);

  TopoDS_Face internal = face_with_wire(plane, square(0, 0, 4, 4, true));
  TopoDS_Wire internalWire = square(6, 6, 8, 8);
  internalWire.Orientation(TopAbs_INTERNAL);
  builder.Add(internal, internalWire);
  print_case("internal_child_preserved", internal);
  print_periodic_shift_case();
  print_shared_seam_reversal_case();
  print_context_case(plane);
  print_sphere_natural_bound_case();
  return 0;
}

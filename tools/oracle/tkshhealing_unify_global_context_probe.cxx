#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepLib.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRep_Tool.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRep_Builder.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pnt.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Lin2d.hxx>
#include <Geom_Surface.hxx>
#include <Geom2d_Curve.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Vec.hxx>

#include <iostream>
#include <iomanip>

static void addLine(BRepBuilderAPI_MakeWire& wire, const gp_Pnt& first, const gp_Pnt& last) {
  wire.Add(BRepBuilderAPI_MakeEdge(first, last));
}

static TopoDS_Face makeSplitSide(double x0, double x1, bool sharedIsRight) {
  const double ySplit = 0.5;
  BRepBuilderAPI_MakeWire wire;
  if (sharedIsRight) {
    addLine(wire, gp_Pnt(x0, 0, 0), gp_Pnt(x1, 0, 0));
    addLine(wire, gp_Pnt(x1, 0, 0), gp_Pnt(x1, ySplit, 0));
    addLine(wire, gp_Pnt(x1, ySplit, 0), gp_Pnt(x1, 1, 0));
    addLine(wire, gp_Pnt(x1, 1, 0), gp_Pnt(x0, 1, 0));
    addLine(wire, gp_Pnt(x0, 1, 0), gp_Pnt(x0, 0, 0));
  } else {
    addLine(wire, gp_Pnt(x0, 0, 0), gp_Pnt(x1, 0, 0));
    addLine(wire, gp_Pnt(x1, 0, 0), gp_Pnt(x1, 1, 0));
    addLine(wire, gp_Pnt(x1, 1, 0), gp_Pnt(x0, 1, 0));
    addLine(wire, gp_Pnt(x0, 1, 0), gp_Pnt(x0, ySplit, 0));
    addLine(wire, gp_Pnt(x0, ySplit, 0), gp_Pnt(x0, 0, 0));
  }
  return BRepBuilderAPI_MakeFace(wire.Wire(), true);
}

static TopoDS_Shape makeSewnPair() {
  BRepBuilderAPI_Sewing sewing;
  sewing.Add(makeSplitSide(0, 1, true));
  sewing.Add(makeSplitSide(1, 2, false));
  sewing.Perform();
  return sewing.SewedShape();
}

static TopoDS_Edge makeCylinderPCurve(const occ::handle<Geom_Surface>& surface,
                                      gp_Pnt2d origin,
                                      gp_Dir2d direction,
                                      double first,
                                      double last) {
  occ::handle<Geom2d_Curve> pcurve = new Geom2d_Line(gp_Lin2d(origin, direction));
  return BRepBuilderAPI_MakeEdge(pcurve, surface, first, last);
}

static TopoDS_Face makeSplitCylinderPatch() {
  const gp_Ax3 axis(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1));
  occ::handle<Geom_Surface> surface = new Geom_CylindricalSurface(gp_Cylinder(axis, 2.0));
  BRepBuilderAPI_MakeWire wire;
  wire.Add(makeCylinderPCurve(surface, gp_Pnt2d(0, 0), gp_Dir2d(1, 0), 0, M_PI));
  wire.Add(makeCylinderPCurve(surface, gp_Pnt2d(M_PI, 0), gp_Dir2d(0, 1), 0, 2.5));
  wire.Add(makeCylinderPCurve(surface, gp_Pnt2d(M_PI, 2.5), gp_Dir2d(0, 1), 0, 2.5));
  wire.Add(makeCylinderPCurve(surface, gp_Pnt2d(M_PI, 5), gp_Dir2d(-1, 0), 0, M_PI));
  wire.Add(makeCylinderPCurve(surface, gp_Pnt2d(0, 5), gp_Dir2d(0, -1), 0, 2.5));
  wire.Add(makeCylinderPCurve(surface, gp_Pnt2d(0, 2.5), gp_Dir2d(0, -1), 0, 2.5));
  for (TopExp_Explorer edge(wire.Wire(), TopAbs_EDGE); edge.More(); edge.Next()) {
    BRepLib::BuildCurve3d(TopoDS::Edge(edge.Current()));
  }
  return BRepBuilderAPI_MakeFace(surface, wire.Wire(), true);
}

static void emitSplitCylinderPcurves(const TopoDS_Face& face) {
  ShapeUpgrade_UnifySameDomain unify(face, true, false, false);
  unify.Build();
  const TopoDS_Shape result = unify.Shape();
  for (TopExp_Explorer item(result, TopAbs_EDGE); item.More(); item.Next()) {
    const TopoDS_Edge edge = TopoDS::Edge(item.Current());
    if (BRepAdaptor_Curve(edge).GetType() != GeomAbs_Line) continue;
    double first = 0.0, last = 0.0;
    const occ::handle<Geom2d_Curve> pcurve = BRep_Tool::CurveOnSurface(edge, face, first, last);
    if (pcurve.IsNull()) continue;
    const gp_Pnt2d middle = pcurve->Value((first + last) * 0.5);
    std::cout << std::setprecision(17) << "cylinder_patch_pcurve "
              << Geom2dAdaptor_Curve(pcurve).GetType() << ' '
              << first << ' ' << last << ' ' << middle.X() << ' ' << middle.Y() << '\n';
    return;
  }
}

static void emit(const char* name, const TopoDS_Shape& shape) {
  ShapeUpgrade_UnifySameDomain unify(shape, true, false, false);
  unify.Build();
  const TopoDS_Shape result = unify.Shape();
  int faces = 0, wires = 0, edges = 0, vertices = 0;
  for (TopExp_Explorer item(result, TopAbs_FACE); item.More(); item.Next()) ++faces;
  for (TopExp_Explorer item(result, TopAbs_WIRE); item.More(); item.Next()) ++wires;
  for (TopExp_Explorer item(result, TopAbs_EDGE); item.More(); item.Next()) ++edges;
  for (TopExp_Explorer item(result, TopAbs_VERTEX); item.More(); item.Next()) ++vertices;
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> uniqueEdges;
  TopExp::MapShapes(result, TopAbs_EDGE, uniqueEdges);
  std::cout << name << " faces " << faces << " wires " << wires << " edges " << edges
            << " unique_edges " << uniqueEdges.Extent() << " vertices " << vertices << " valid "
            << BRepCheck_Analyzer(result, true).IsValid() << '\n';
}

int main() {
  TopoDS_Face single = makeSplitSide(0, 2, true);
  emit("single_face_compound", single);
  const TopoDS_Shape sewn = makeSewnPair();
  emit("two_face_shell", sewn);
  TopoDS_Compound compound;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.Add(compound, makeSplitSide(0, 1, true));
  builder.Add(compound, makeSplitSide(1, 2, false));
  emit("two_face_compound", compound);
  BRepPrimAPI_MakePrism prism(single, gp_Vec(0, 0, 1));
  emit("prism_solid", prism.Shape());
  BRepPrimAPI_MakeCylinder cylinder(2.0, 5.0);
  emit("cylinder_solid", cylinder.Shape());
  const TopoDS_Face splitCylinder = makeSplitCylinderPatch();
  std::cout << "cylinder_patch_input_valid " << BRepCheck_Analyzer(splitCylinder, true).IsValid()
            << '\n';
  emit("cylinder_patch_split_lines", splitCylinder);
  emitSplitCylinderPcurves(splitCylinder);
}

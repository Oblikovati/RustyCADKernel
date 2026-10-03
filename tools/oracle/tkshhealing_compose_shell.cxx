#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <NCollection_HArray2.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeExtend_CompositeSurface.hxx>
#include <ShapeFix_ComposeShell.hxx>
#include <ShapeFix_WireSegment.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS.hxx>
#include <ShapeAnalysis_TransferParameters.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Lin2d.hxx>
#include <TopLoc_Location.hxx>
#include <iostream>

class ComposeProbe : public ShapeFix_ComposeShell
{
public:
  using ShapeFix_ComposeShell::LoadWires;
  using ShapeFix_ComposeShell::ComputeCode;
  using ShapeFix_ComposeShell::BreakWires;
};

static void runInit()
{
  BRepBuilderAPI_MakeWire wire;
  gp_Pnt points[] = {gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0), gp_Pnt(2, 2, 0), gp_Pnt(0, 2, 0)};
  for (int i = 0; i < 4; ++i)
    wire.Add(BRepBuilderAPI_MakeEdge(points[i], points[(i + 1) % 4]).Edge());
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), wire.Wire(), true).Face();
  Handle(NCollection_HArray2<Handle(Geom_Surface)>) patches =
    new NCollection_HArray2<Handle(Geom_Surface)>(1, 1, 1, 1);
  patches->SetValue(1, 1, new Geom_Plane(gp_Pln()));
  Handle(ShapeExtend_CompositeSurface) grid = new ShapeExtend_CompositeSurface(patches);
  ShapeFix_ComposeShell tool;
  tool.Init(grid, TopLoc_Location(), face, 1.0e-7);
  std::cout << "init " << tool.ClosedMode() << ' '
            << tool.Status(ShapeExtend_DONE1) << ' ' << tool.Result().IsNull() << '\n';
}

static void runLoadWires()
{
  BRepBuilderAPI_MakeWire wire;
  gp_Pnt points[] = {gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0), gp_Pnt(2, 2, 0), gp_Pnt(0, 2, 0)};
  for (int i = 0; i < 4; ++i)
    wire.Add(BRepBuilderAPI_MakeEdge(points[i], points[(i + 1) % 4]).Edge());
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), wire.Wire(), true).Face();
  Handle(NCollection_HArray2<Handle(Geom_Surface)>) patches =
    new NCollection_HArray2<Handle(Geom_Surface)>(1, 1, 1, 1);
  patches->SetValue(1, 1, new Geom_Plane(gp_Pln()));
  Handle(ShapeExtend_CompositeSurface) grid = new ShapeExtend_CompositeSurface(patches);
  ComposeProbe tool;
  tool.SetContext(new ShapeBuild_ReShape);
  tool.Init(grid, TopLoc_Location(), face, 1.0e-7);
  NCollection_Sequence<ShapeFix_WireSegment> segments;
  tool.LoadWires(segments);
  std::cout << "load_wires " << segments.Length() << ' ' << segments(1).NbEdges() << ' '
            << (segments(1).Orientation() == TopAbs_REVERSED ? "reversed" : "other") << '\n';
}

static void runMixedLoadWires()
{
  BRepBuilderAPI_MakeWire wire;
  gp_Pnt points[] = {gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0), gp_Pnt(2, 2, 0), gp_Pnt(0, 2, 0)};
  for (int i = 0; i < 4; ++i)
  {
    BRepBuilderAPI_MakeEdge made(points[i], points[(i + 1) % 4]);
    TopoDS_Shape edge = made.Shape();
    if (i == 0)
      edge.Orientation(TopAbs_INTERNAL);
    wire.Add(TopoDS::Edge(edge));
  }
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), wire.Wire(), true).Face();
  Handle(NCollection_HArray2<Handle(Geom_Surface)>) patches =
    new NCollection_HArray2<Handle(Geom_Surface)>(1, 1, 1, 1);
  patches->SetValue(1, 1, new Geom_Plane(gp_Pln()));
  Handle(ShapeExtend_CompositeSurface) grid = new ShapeExtend_CompositeSurface(patches);
  ComposeProbe tool;
  tool.SetContext(new ShapeBuild_ReShape);
  tool.Init(grid, TopLoc_Location(), face, 1.0e-7);
  NCollection_Sequence<ShapeFix_WireSegment> segments;
  tool.LoadWires(segments);
  std::cout << "load_wires_mixed " << segments.Length();
  for (int i = 1; i <= segments.Length(); ++i)
  {
    const char* orientation = segments(i).Orientation() == TopAbs_INTERNAL ? "internal" :
                              segments(i).Orientation() == TopAbs_REVERSED ? "reversed" : "other";
    std::cout << ' ' << segments(i).NbEdges() << ' ' << orientation;
  }
  std::cout << '\n';
}

static void runSplitEdges()
{
  BRepBuilderAPI_MakeWire wire;
  gp_Pnt points[] = {gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0), gp_Pnt(2, 2, 0), gp_Pnt(0, 2, 0)};
  for (int i = 0; i < 4; ++i)
    wire.Add(BRepBuilderAPI_MakeEdge(points[i], points[(i + 1) % 4]).Edge());
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), wire.Wire(), true).Face();
  Handle(NCollection_HArray2<Handle(Geom_Surface)>) patches =
    new NCollection_HArray2<Handle(Geom_Surface)>(1, 2, 1, 1);
  Handle(Geom_Surface) plane = new Geom_Plane(gp_Pln());
  patches->SetValue(1, 1, new Geom_RectangularTrimmedSurface(plane, 0., 1., 0., 2., true, true));
  patches->SetValue(2, 1, new Geom_RectangularTrimmedSurface(plane, 1., 3., 0., 2., true, true));
  Handle(ShapeExtend_CompositeSurface) grid = new ShapeExtend_CompositeSurface(patches);
  ShapeFix_ComposeShell tool;
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  tool.SetContext(context);
  tool.Init(grid, TopLoc_Location(), face, 1.0e-7);
  tool.SplitEdges();
  std::cout << "split_edges";
  TopoDS_Shape sourceWire = wire.Shape();
  for (TopoDS_Iterator it(sourceWire); it.More(); it.Next())
  {
    TopoDS_Shape mapped = context->Apply(it.Value());
    std::cout << ' ' << mapped.ShapeType();
    if (mapped.ShapeType() == TopAbs_WIRE)
    {
      int count = 0;
      for (TopoDS_Iterator edgeIt(mapped); edgeIt.More(); edgeIt.Next())
        ++count;
      std::cout << ':' << count;
    }
  }
  std::cout << '\n';
}

static void runBreakWires()
{
  BRep_Builder builder;
  TopoDS_Vertex vertices[4];
  gp_Pnt points[] = {gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0), gp_Pnt(2, 2, 0), gp_Pnt(0, 2, 0)};
  for (int i = 0; i < 4; ++i)
    builder.MakeVertex(vertices[i], points[i], 1.0e-7);
  TopoDS_Edge boundary[4];
  for (int i = 0; i < 4; ++i) {
    builder.MakeEdge(boundary[i]);
    builder.Add(boundary[i], vertices[i]);
    TopoDS_Vertex last = vertices[(i + 1) % 4];
    last.Orientation(TopAbs_REVERSED);
    builder.Add(boundary[i], last);
  }
  TopoDS_Edge diagonal;
  builder.MakeEdge(diagonal);
  builder.Add(diagonal, vertices[1]);
  TopoDS_Vertex diagonalEnd = vertices[3];
  diagonalEnd.Orientation(TopAbs_REVERSED);
  builder.Add(diagonal, diagonalEnd);
  diagonal.Orientation(TopAbs_EXTERNAL);

  ShapeFix_WireSegment boundarySegment;
  for (int i = 0; i < 4; ++i) {
    boundarySegment.AddEdge(0, boundary[i]);
    boundarySegment.DefineIUMin(i + 1, i + 3);
  }
  ShapeFix_WireSegment cutSegment;
  cutSegment.AddEdge(0, diagonal);
  cutSegment.Orientation(TopAbs_EXTERNAL);
  NCollection_Sequence<ShapeFix_WireSegment> segments;
  segments.Append(boundarySegment);
  segments.Append(cutSegment);
  ComposeProbe tool;
  tool.BreakWires(segments);
  std::cout << "break_wires " << segments.Length();
  for (int i = 1; i <= segments.Length(); ++i) {
    int umin, umax, vmin, vmax;
    const ShapeFix_WireSegment& segment = segments(i);
    std::cout << ' ' << segment.NbEdges() << ':' << segment.Orientation() << ':';
    for (int j = 1; j <= segment.NbEdges(); ++j) {
      segment.GetPatchIndex(j, umin, umax, vmin, vmax);
      if (j > 1) std::cout << ',';
      std::cout << umin;
    }
  }
  std::cout << '\n';
}

static void runInternalExternalBreakWires()
{
  BRep_Builder builder;
  TopoDS_Vertex vertices[3];
  for (int i = 0; i < 3; ++i)
    builder.MakeVertex(vertices[i], gp_Pnt(i, 0, 0), 1.0e-7);
  TopoDS_Edge edges[2];
  for (int i = 0; i < 2; ++i) {
    builder.MakeEdge(edges[i]);
    builder.Add(edges[i], vertices[i]);
    TopoDS_Vertex last = vertices[i + 1];
    last.Orientation(TopAbs_REVERSED);
    builder.Add(edges[i], last);
  }
  edges[0].Orientation(TopAbs_EXTERNAL);
  ShapeFix_WireSegment segment;
  segment.AddEdge(0, edges[0]);
  segment.AddEdge(0, edges[1]);
  segment.Orientation(TopAbs_INTERNAL);
  NCollection_Sequence<ShapeFix_WireSegment> segments;
  segments.Append(segment);
  ComposeProbe tool;
  tool.BreakWires(segments);
  std::cout << "break_wires_internal_external " << segments.Length();
  for (int i = 1; i <= segments.Length(); ++i)
    std::cout << ' ' << segments(i).NbEdges() << ':' << segments(i).Orientation();
  std::cout << '\n';
}

static void runComputeCode()
{
  BRepBuilderAPI_MakeWire wire;
  gp_Pnt points[] = {gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0), gp_Pnt(2, 2, 0), gp_Pnt(0, 2, 0)};
  for (int i = 0; i < 4; ++i)
    wire.Add(BRepBuilderAPI_MakeEdge(points[i], points[(i + 1) % 4]).Edge());
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), wire.Wire(), true).Face();
  Handle(NCollection_HArray2<Handle(Geom_Surface)>) patches =
    new NCollection_HArray2<Handle(Geom_Surface)>(1, 1, 1, 1);
  patches->SetValue(1, 1, new Geom_Plane(gp_Pln()));
  Handle(ShapeExtend_CompositeSurface) grid = new ShapeExtend_CompositeSurface(patches);
  ComposeProbe tool;
  tool.SetContext(new ShapeBuild_ReShape);
  tool.Init(grid, TopLoc_Location(), face, 1.0e-7);
  NCollection_Sequence<ShapeFix_WireSegment> segments;
  tool.LoadWires(segments);
  gp_Lin2d line(gp_Pnt2d(1, 0), gp_Dir2d(0, 1));
  std::cout << "compute_code "
            << tool.ComputeCode(segments(1).WireData(), line, 1, 1, 0.25, 0.75) << '\n';
  std::cout << "compute_code_right "
            << tool.ComputeCode(segments(1).WireData(), line, 1, 1, 1.25, 1.75) << '\n';
  gp_Lin2d tangent(gp_Pnt2d(0, 0), gp_Dir2d(1, 0));
  std::cout << "compute_code_tangent "
            << tool.ComputeCode(segments(1).WireData(), tangent, 1, 1, 0.25, 0.75) << '\n';
  tool.ClosedMode() = true;
  std::cout << "compute_code_closed_mode "
            << tool.ComputeCode(segments(1).WireData(), line, 1, 1, 0., 2.) << '\n';
}

static void runPatchLimits()
{
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)).Edge();
  ShapeFix_WireSegment segment;
  segment.AddEdge(0, edge);
  segment.DefineIUMin(1, 2);
  segment.DefineIUMax(1, 2);
  segment.DefineIVMin(1, 3);
  int umin, umax, vmin, vmax;
  segment.GetPatchIndex(1, umin, umax, vmin, vmax);
  std::cout << "patch_limits " << umin << ' ' << umax << ' ' << vmin << ' ' << vmax << '\n';
}

int main()
{
  ShapeFix_ComposeShell tool;
  std::cout << "default " << tool.ClosedMode() << ' '
            << tool.Status(ShapeExtend_DONE1) << ' ' << tool.Result().IsNull() << ' '
            << tool.GetTransferParamTool()->DynamicType()->Name() << '\n';
  tool.ClosedMode() = true;
  std::cout << "closed_mode " << tool.ClosedMode() << '\n';
  tool.SetTransferParamTool(new ShapeAnalysis_TransferParameters);
  std::cout << "transfer_type " << tool.GetTransferParamTool()->DynamicType()->Name() << '\n';
  runInit();
  runLoadWires();
  runMixedLoadWires();
  runComputeCode();
  runBreakWires();
  runInternalExternalBreakWires();
  runPatchLimits();
  runSplitEdges();
}

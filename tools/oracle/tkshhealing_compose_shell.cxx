#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <Geom_Plane.hxx>
#include <NCollection_HArray2.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeExtend_CompositeSurface.hxx>
#include <ShapeFix_ComposeShell.hxx>
#include <ShapeFix_WireSegment.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeAnalysis_TransferParameters.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <TopLoc_Location.hxx>
#include <iostream>

class ComposeProbe : public ShapeFix_ComposeShell
{
public:
  using ShapeFix_ComposeShell::LoadWires;
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
}

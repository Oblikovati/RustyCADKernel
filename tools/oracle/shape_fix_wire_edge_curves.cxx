// ShapeFix_Wire::FixEdgeCurves orchestration at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <gp_Circ.hxx>
#include <gp_Sphere.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <ShapeBuild_Edge.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeFix_Wire.hxx>
#include <ShapeExtend.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <iostream>

struct Fixture { TopoDS_Wire wire; TopoDS_Face face; TopoDS_Edge edge; };

Fixture cylinder_edge_without_pcurve()
{
  Fixture fixture;
  occ::handle<Geom_CylindricalSurface> surface = new Geom_CylindricalSurface(gp_Ax3(), 1.0);
  fixture.face = BRepBuilderAPI_MakeFace(surface, 0.0, 6.283185307179586, 0.0, 2.0, 1e-7).Face();
  fixture.edge = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(1, 0, 1)).Edge();
  fixture.wire = BRepBuilderAPI_MakeWire(fixture.edge).Wire();
  return fixture;
}

void unloaded_case()
{
  ShapeFix_Wire fixer;
  std::cout << "unloaded " << fixer.FixEdgeCurves() << ' '
            << fixer.StatusEdgeCurves(ShapeExtend_OK) << '\n';
}

void explicit_add_pcurve_case()
{
  Fixture fixture = cylinder_edge_without_pcurve();
  ShapeFix_Wire fixer(fixture.wire, fixture.face, 1e-7);
  fixer.FixReversed2dMode() = 0;
  fixer.FixRemovePCurveMode() = 0;
  fixer.FixAddPCurveMode() = 1;
  fixer.FixRemoveCurve3dMode() = 0;
  fixer.FixAddCurve3dMode() = 0;
  fixer.FixSeamMode() = 0;
  fixer.FixShiftedMode() = 0;
  fixer.FixSameParameterMode() = 0;
  fixer.FixVertexToleranceMode() = 0;
  bool fixed = fixer.FixEdgeCurves();
  std::cout << "add-pcurve " << fixed << ' '
            << fixer.StatusEdgeCurves(ShapeExtend_DONE3) << ' '
            << fixer.StatusEdgeCurves(ShapeExtend_FAIL3) << ' '
            << ShapeAnalysis_Edge().HasPCurve(fixture.edge, fixture.face) << '\n';
}

void default_modes_case()
{
  Fixture fixture = cylinder_edge_without_pcurve();
  ShapeFix_Wire fixer(fixture.wire, fixture.face, 1e-7);
  bool fixed = fixer.FixEdgeCurves();
  std::cout << "default-modes " << fixed;
  for (int index = 1; index <= 8; ++index)
  {
    std::cout << ' ' << fixer.StatusEdgeCurves(static_cast<ShapeExtend_Status>(index));
  }
  for (int index = 10; index <= 17; ++index)
  {
    std::cout << ' ' << fixer.StatusEdgeCurves(static_cast<ShapeExtend_Status>(index));
  }
  std::cout << ' ' << ShapeAnalysis_Edge().HasPCurve(fixture.edge, fixture.face) << '\n';
}

void spherical_pole_split_case()
{
  gp_Sphere sphere(gp_Ax3(), 1.0);
  occ::handle<Geom_Surface> surface = new Geom_SphericalSurface(sphere);
  TopoDS_Face face = BRepBuilderAPI_MakeFace(
    surface, 0.0, 6.283185307179586, -1.570796326794897, 1.570796326794897, 1e-7).Face();
  gp_Ax2 circleFrame(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0), gp_Dir(0, 0, 1));
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Circ(circleFrame, 1.0), -0.5, 0.5).Edge();
  TopoDS_Wire wire = BRepBuilderAPI_MakeWire(edge).Wire();
  ShapeFix_Wire fixer(wire, face, 1e-7);
  fixer.SetContext(new ShapeBuild_ReShape());
  fixer.FixReversed2dMode() = 0;
  fixer.FixRemovePCurveMode() = 0;
  fixer.FixAddPCurveMode() = 1;
  fixer.FixRemoveCurve3dMode() = 0;
  fixer.FixAddCurve3dMode() = 0;
  fixer.FixSeamMode() = 0;
  fixer.FixShiftedMode() = 0;
  fixer.FixSameParameterMode() = 0;
  fixer.FixVertexToleranceMode() = 0;
  bool fixed = fixer.FixEdgeCurves();
  std::cout << "sphere-pole " << fixed << ' ' << fixer.WireData()->NbEdges() << ' '
            << fixer.StatusEdgeCurves(ShapeExtend_DONE3) << ' '
            << fixer.StatusEdgeCurves(ShapeExtend_FAIL3);
  for (int index = 1; index <= fixer.WireData()->NbEdges(); ++index)
  {
    double first, last;
    BRep_Tool::CurveOnSurface(fixer.WireData()->Edge(index), face, first, last);
    std::cout << ' ' << first << ' ' << last;
  }
  std::cout << '\n';
}

void incomplete_edge_case()
{
  Fixture fixture = cylinder_edge_without_pcurve();
  ShapeBuild_Edge().RemoveCurve3d(fixture.edge);
  ShapeFix_Wire fixer;
  fixer.Load(fixture.wire);
  fixer.FixReversed2dMode() = 0;
  fixer.FixRemovePCurveMode() = 0;
  fixer.FixAddPCurveMode() = 0;
  fixer.FixRemoveCurve3dMode() = 0;
  fixer.FixAddCurve3dMode() = 1;
  fixer.FixSeamMode() = 0;
  fixer.FixShiftedMode() = 0;
  fixer.FixSameParameterMode() = 0;
  fixer.FixVertexToleranceMode() = 0;
  bool fixed = fixer.FixEdgeCurves();
  std::cout << "incomplete-edge " << fixed << ' ' << fixer.WireData()->NbEdges() << ' '
            << fixer.StatusEdgeCurves(ShapeExtend_DONE5) << ' '
            << fixer.StatusEdgeCurves(ShapeExtend_FAIL5) << '\n';
}

int main()
{
  unloaded_case();
  explicit_add_pcurve_case();
  default_modes_case();
  incomplete_edge_case();
  spherical_pole_split_case();
}

// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 ShapeFix_Edge curve repair oracle.
#include <BRepLib_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <ShapeBuild_Edge.hxx>
#include <ShapeFix_Edge.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <iostream>

struct Fixture { TopoDS_Edge edge; TopoDS_Face face; };
Fixture cylinderFixture(bool reversed)
{
  Fixture result;
  result.edge=BRepLib_MakeEdge(gp_Pnt(1,0,0),gp_Pnt(1,0,1)).Edge();
  occ::handle<Geom_Surface> surface=new Geom_CylindricalSurface(gp_Ax3(),1);
  occ::handle<Geom2d_Curve> curve=new Geom2d_Line(gp_Pnt2d(0,reversed?1:0),gp_Dir2d(0,reversed?-1:1));
  BRep_Builder builder;
  builder.MakeFace(result.face,surface,1e-7);
  builder.UpdateEdge(result.edge,curve,result.face,1e-7);
  builder.Range(result.edge,result.face,0,1);
  return result;
}

void removalCases()
{
  Fixture fixture=cylinderFixture(false);
  ShapeFix_Edge fixer;
  std::cout << "retained " << fixer.FixRemovePCurve(fixture.edge,fixture.face) << ' '
            << fixer.FixRemoveCurve3d(fixture.edge) << '\n';
  TopoDS_Vertex first=ShapeAnalysis_Edge().FirstVertex(fixture.edge);
  BRep_Builder().UpdateVertex(first,gp_Pnt(9,0,0),1e-7);
  const bool removedPCurve=fixer.FixRemovePCurve(fixture.edge,fixture.face);
  std::cout << "removed-pcurve " << removedPCurve << ' ' << fixer.Status(ShapeExtend_OK) << '\n';
  const bool removedCurve=fixer.FixRemoveCurve3d(fixture.edge);
  std::cout << "removed-curve " << removedCurve << ' ' << fixer.Status(ShapeExtend_OK) << '\n';
}

void reconstructionCases()
{
  Fixture fixture=cylinderFixture(false);
  ShapeFix_Edge fixer;
  std::cout << "existing " << fixer.FixAddCurve3d(fixture.edge) << '\n';
  ShapeBuild_Edge().RemoveCurve3d(fixture.edge);
  const bool added=fixer.FixAddCurve3d(fixture.edge);
  std::cout << "added " << added << ' ' << fixer.Status(ShapeExtend_DONE1) << '\n';
  ShapeBuild_Edge().RemoveCurve3d(fixture.edge);
  ShapeBuild_Edge().RemovePCurve(fixture.edge,fixture.face);
  const bool failed=fixer.FixAddCurve3d(fixture.edge);
  std::cout << "missing " << failed << ' ' << fixer.Status(ShapeExtend_FAIL1) << '\n';
}

void reversalCases()
{
  Fixture fixture=cylinderFixture(true);
  ShapeFix_Edge fixer;
  const bool reversed=fixer.FixReversed2d(fixture.edge,fixture.face);
  double first,last;
  occ::handle<Geom2d_Curve> curve=BRep_Tool::CurveOnSurface(fixture.edge,fixture.face,first,last);
  std::cout << "reversed " << reversed << ' ' << fixer.Status(ShapeExtend_DONE1) << ' '
            << first << ' ' << last << ' ' << BRep_Tool::SameRange(fixture.edge) << ' '
            << BRep_Tool::SameParameter(fixture.edge) << '\n';
  std::cout << "repeat " << fixer.FixReversed2d(fixture.edge,fixture.face) << '\n';
  ShapeBuild_Edge().RemovePCurve(fixture.edge,fixture.face);
  const bool missing=fixer.FixReversed2d(fixture.edge,fixture.face);
  std::cout << "missing-reversal " << missing << ' ' << fixer.Status(ShapeExtend_FAIL1) << '\n';
}

int main() { removalCases(); reconstructionCases(); reversalCases(); }

#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_Offset.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <BRep_Builder.hxx>
#include <GProp_GProps.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <gp_Pln.hxx>
#include <gp_Ax3.hxx>

#include <iomanip>
#include <iostream>

static int Count(const TopoDS_Shape& theShape, const TopAbs_ShapeEnum theType)
{
  int aCount = 0;
  for (TopExp_Explorer anExplorer(theShape, theType); anExplorer.More(); anExplorer.Next())
  {
    ++aCount;
  }
  return aCount;
}

int main()
{
  const TopoDS_Face aSource = BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3()), 0.0, 1.0, 0.0, 2.0).Face();
  BRepOffset_Offset anOffset(aSource, 0.5);
  const TopoDS_Face& anImage = anOffset.Face();
  GProp_GProps aProperties;
  BRepGProp::SurfaceProperties(anImage, aProperties, false, false);
  std::cout << std::setprecision(17)
            << "status " << static_cast<int>(anOffset.Status())
            << " initial " << (anOffset.InitialShape().IsSame(aSource) ? 1 : 0)
            << " edges " << Count(anImage, TopAbs_EDGE)
            << " vertices " << Count(anImage, TopAbs_VERTEX)
            << " area " << aProperties.Mass()
            << " valid " << (BRepCheck_Analyzer(anImage, true, false, false).IsValid() ? 1 : 0)
            << " tolerance " << BRep_Tool::Tolerance(anImage)
            << " generated " << (!anOffset.Generated(TopoDS_Shape(TopExp_Explorer(aSource, TopAbs_EDGE).Current())).IsNull() ? 1 : 0)
            << '\n';
  return 0;
}

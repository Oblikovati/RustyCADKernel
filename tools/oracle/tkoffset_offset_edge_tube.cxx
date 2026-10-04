#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_Offset.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Pnt.hxx>

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

static int EdgeIndex(const TopoDS_Shape& theShape, const TopoDS_Shape& theEdge)
{
  int anIndex = 0;
  for (TopExp_Explorer anExplorer(theShape, TopAbs_EDGE); anExplorer.More(); anExplorer.Next())
  {
    if (anExplorer.Current().IsSame(theEdge))
    {
      return anIndex;
    }
    ++anIndex;
  }
  return -1;
}

int main(int argc, char** argv)
{
  const double anOffset = argc > 1 ? std::stod(argv[1]) : 1.0;
  const TopoDS_Edge anEdge = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(0, 0, 10));
  BRepOffset_Offset anImage;
  anImage.Init(anEdge, anOffset);
  const TopoDS_Face& aFace = anImage.Face();
  TopoDS_Vertex aFirst;
  TopoDS_Vertex aLast;
  TopExp::Vertices(anEdge, aFirst, aLast);
  GProp_GProps aProperties;
  BRepGProp::SurfaceProperties(aFace, aProperties, false, false);
  std::cout << std::setprecision(17)
            << "offset " << anOffset
            << " status " << static_cast<int>(anImage.Status())
            << " initial " << (anImage.InitialShape().IsSame(anEdge) ? 1 : 0)
            << " edges " << Count(aFace, TopAbs_EDGE)
            << " vertices " << Count(aFace, TopAbs_VERTEX)
            << " area " << aProperties.Mass()
            << " valid " << (BRepCheck_Analyzer(aFace, true, false, false).IsValid() ? 1 : 0)
            << " orientation " << static_cast<int>(aFace.Orientation())
            << " tolerance " << BRep_Tool::Tolerance(aFace)
            << " generated_first " << (anImage.Generated(aFirst).IsNull() ? 0 : 1)
            << " generated_last " << (anImage.Generated(aLast).IsNull() ? 0 : 1)
            << " generated_edge " << (anImage.Generated(anEdge).IsNull() ? 0 : 1)
            << " generated_first_index " << EdgeIndex(aFace, anImage.Generated(aFirst))
            << " generated_last_index " << EdgeIndex(aFace, anImage.Generated(aLast))
            << " generated_edge_index " << EdgeIndex(aFace, anImage.Generated(anEdge))
            << '\n';
  return 0;
}

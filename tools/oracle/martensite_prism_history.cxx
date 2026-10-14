// SPDX-License-Identifier: GPL-2.0-only
// Pinned OCCT3d097a0: actual finite MakePrism role/history observations, not region certification.
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <TopExp.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <iostream>
static TopoDS_Face Rectangle()
{
  BRepBuilderAPI_MakePolygon polygon;
  polygon.Add(gp_Pnt(0,0,0)); polygon.Add(gp_Pnt(4,0,0));
  polygon.Add(gp_Pnt(4,3,0)); polygon.Add(gp_Pnt(0,3,0));
  polygon.Close();
  return BRepBuilderAPI_MakeFace(polygon.Wire()).Face();
}
static void CaptureBase(BRepPrimAPI_MakePrism& maker,const TopoDS_Shape& base,int index)
{
  // Copy the reusable native generated list before another history query can overwrite it.
  const NCollection_List<TopoDS_Shape> generated=maker.Generated(base);
  const TopoDS_Shape first=maker.FirstShape(base),last=maker.LastShape(base);
  std::cout<<"member "<<index<<' '<<base.ShapeType()<<' '<<generated.Size();
  for (const TopoDS_Shape& shape:generated) std::cout<<' '<<shape.ShapeType();
  std::cout<<' '<<first.ShapeType()<<' '<<last.ShapeType()<<' '<<maker.IsDeleted(base)<<'\n';
}
static void RunCase(bool reverse)
{
  TopoDS_Face input=Rectangle(); if (reverse) input.Reverse();
  BRepBuilderAPI_Copy copier(input,true,true);
  const TopoDS_Shape base=copier.Shape();
  BRepPrimAPI_MakePrism maker(base,gp_Vec(0,0,2),false,true);
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> inputs,outputs;
  TopExp::MapShapes(base,inputs); TopExp::MapShapes(maker.Shape(),outputs);
  std::cout<<"case "<<reverse<<' '<<inputs.Extent()<<' '<<outputs.Extent()<<'\n';
  for (int index=1;index<=inputs.Extent();++index) CaptureBase(maker,inputs(index),index);
}
int main()
{
  RunCase(false);
  RunCase(true);
  return 0;
}

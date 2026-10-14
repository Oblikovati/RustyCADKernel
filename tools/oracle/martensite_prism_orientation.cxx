// SPDX-License-Identifier: GPL-2.0-only
// Actual native role images versus cumulative output occurrences; missing matches are not repaired.
#include "martensite_prism_fixture.hxx"
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <TopoDS_Iterator.hxx>
#include <vector>
#include <iostream>
#include <gp_Trsf.hxx>
#include <TopLoc_Location.hxx>
static void Collect(const TopoDS_Shape& shape,std::vector<TopoDS_Shape>& found)
{
  for (const TopoDS_Shape& previous:found) if (previous.IsEqual(shape)) return;
  found.push_back(shape);
  for (TopoDS_Iterator children(shape,true,true);children.More();children.Next())
    Collect(children.Value(),found);
}
static void Image(const char* role,const TopoDS_Shape& image,const std::vector<TopoDS_Shape>& outputs)
{
  int same=0,equal=0;
  for (const TopoDS_Shape& output:outputs) {
    same+=output.IsSame(image); equal+=output.IsEqual(image);
  }
  std::cout<<' '<<role<<' '<<image.ShapeType()<<' '<<image.Orientation()<<' '<<same<<' '<<equal;
}
static void Member(BRepPrimAPI_MakePrism& maker,const TopoDS_Shape& source,int index,const std::vector<TopoDS_Shape>& outputs)
{
  std::cout<<"member "<<index<<' '<<source.ShapeType()<<' '<<source.Orientation();
  const NCollection_List<TopoDS_Shape> generated=maker.Generated(source);
  for (const TopoDS_Shape& image:generated) Image("generated",image,outputs);
  Image("first",maker.FirstShape(source),outputs);
  Image("last",maker.LastShape(source),outputs);
  std::cout<<'\n';
}
static void Run(bool reversed,double offset)
{
  TopoDS_Face original=Rectangle(); if (reversed) original.Reverse();
  gp_Trsf translation; translation.SetTranslation(gp_Vec(offset,0,0));
  original.Location(TopLoc_Location(translation));
  BRepBuilderAPI_Copy copier(original,true,true);
  const TopoDS_Shape base=copier.Shape();
  BRepPrimAPI_MakePrism maker(base,gp_Vec(0,0,2),false,true);
  std::vector<TopoDS_Shape> inputs,outputs;
  Collect(base,inputs); Collect(maker.Shape(),outputs);
  std::cout<<"case "<<reversed<<' '<<offset<<' '<<inputs.size()<<' '<<outputs.size()<<'\n';
  for (size_t index=0;index<inputs.size();++index) Member(maker,inputs[index],int(index+1),outputs);
}
int main()
{
  Run(false,0); Run(true,0);
  Run(false,10); Run(true,10);
  return 0;
}

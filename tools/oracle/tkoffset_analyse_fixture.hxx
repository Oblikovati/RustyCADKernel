#pragma once
#include <BRepOffset_Analyse.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <iostream>
#include <iomanip>
#include <vector>
#include <Geom_BezierSurface.hxx>
#include <Geom2d_Line.hxx>
#include <TopoDS_Wire.hxx>
#include <NCollection_Array2.hxx>
TopoDS_Shape spline_pair(double slope){
 BRep_Builder builder;TopoDS_Compound pair;builder.MakeCompound(pair);
 auto edge=BRepBuilderAPI_MakeEdge(gp_Pnt(),gp_Pnt(1,0,0)).Edge();
 for(int side=0;side<2;side++){
  NCollection_Array2<gp_Pnt> poles(1,2,1,2);
  for(int u=1;u<=2;u++)for(int v=1;v<=2;v++)poles(u,v)=gp_Pnt(u-1,v-1,side*slope*(u-1));
  Handle(Geom_Surface) surface=new Geom_BezierSurface(poles);
  TopoDS_Face face;builder.MakeFace(face,surface,1e-7);
  Handle(Geom2d_Curve) curve=new Geom2d_Line(gp_Ax2d(gp_Pnt2d(),gp_Dir2d(1,0)));
  builder.UpdateEdge(edge,curve,face,1e-7);builder.Range(edge,face,0.,1.);
  TopoDS_Wire wire;builder.MakeWire(wire);builder.Add(wire,edge);builder.Add(face,wire);builder.Add(pair,face);
 }
 return pair;
}

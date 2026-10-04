#pragma once
#include <ChFi3d.hxx>
#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <iostream>
TopoDS_Face face(const TopoDS_Edge& edge,gp_Dir normal,int reversed,bool missing=false){
 BRep_Builder builder;TopoDS_Face face;Handle(Geom_Surface) surface=new Geom_Plane(gp_Ax3(gp_Pnt(),normal,gp_Dir(1,0,0)));if(missing)surface=new Geom_CylindricalSurface(gp_Ax3(),2.);builder.MakeFace(face,surface,1e-7);TopoDS_Wire wire;builder.MakeWire(wire);builder.Add(wire,edge);builder.Add(face,wire);if(reversed)face.Reverse();return face;
}

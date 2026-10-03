// ShapeFix_Wire state delegation at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <ShapeFix_Wire.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeAnalysis_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Edge.hxx>
#include <Geom_Plane.hxx>
#include <gp_Pln.hxx>
#include <iostream>
static void print(const char* name,ShapeFix_Wire& fixer) {
  std::cout<<name<<' '<<fixer.Precision()<<' '<<fixer.Analyzer()->Precision()<<' '
    <<fixer.MinTolerance()<<' '<<fixer.MaxTolerance()<<' '<<fixer.NbEdges()<<' '
    <<fixer.IsLoaded()<<' '<<fixer.IsReady()<<' '<<fixer.Face().IsNull()<<' '
    <<fixer.ClosedWireMode()<<'\n';
}
int main() {
  ShapeFix_Wire fixer; print("default",fixer);
  fixer.SetPrecision(0.25); print("precision",fixer);
  fixer.ClosedWireMode()=false; print("open",fixer);
  TopoDS_Wire wire; BRep_Builder().MakeWire(wire);
  BRep_Builder().Add(wire,BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(1,0,0)).Edge());
  TopoDS_Face face=BRepBuilderAPI_MakeFace(gp_Pln()).Face();
  ShapeFix_Wire configured(wire,face,0.125); print("constructor",configured);
  fixer.Init(wire,face,0.5); print("init",fixer);
  occ::handle<ShapeAnalysis_Wire> analyzer=new ShapeAnalysis_Wire;
  analyzer->SetPrecision(0.75); fixer.Init(analyzer); print("shared",fixer);
  fixer.SetPrecision(0.375); print("shared_updated",fixer);
  occ::handle<Geom_Surface> surface=new Geom_Plane(gp_Pln());
  fixer.SetSurface(surface); print("surface",fixer);
  fixer.SetSurface(surface,TopLoc_Location()); print("located",fixer);
  occ::handle<ShapeAnalysis_Surface> analysis=new ShapeAnalysis_Surface(surface);
  fixer.SetFace(face,analysis);
  std::cout<<"face_analysis "<<fixer.Face().IsSame(face)<<' '<<(fixer.Analyzer()->Surface()==analysis)<<'\n';
  occ::handle<ShapeAnalysis_Surface> replacement=new ShapeAnalysis_Surface(surface);
  fixer.SetSurface(replacement);
  std::cout<<"surface_analysis "<<fixer.Face().IsSame(face)<<' '<<(fixer.Analyzer()->Surface()==replacement)<<'\n';
}

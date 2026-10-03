// Native sequence overload at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Plane.hxx>
#include <NCollection_HArray2.hxx>
#include <ShapeExtend_CompositeSurface.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeFix_ComposeShell.hxx>
#include <ShapeFix_WireSegment.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Pln.hxx>
#include <iomanip>
#include <iostream>
class SequenceProbe : public ShapeFix_ComposeShell {
public: using ShapeFix_ComposeShell::LoadWires; using ShapeFix_ComposeShell::SplitByLine;
};
static TopoDS_Wire rectangle(double y) {
  BRepBuilderAPI_MakeWire wire;
  gp_Pnt points[]={gp_Pnt(0.,y,0.),gp_Pnt(2.,y,0.),gp_Pnt(2.,y+2.,0.),gp_Pnt(0.,y+2.,0.)};
  for(int i=0;i<4;++i) wire.Add(BRepBuilderAPI_MakeEdge(points[i],points[(i+1)%4]).Edge());
  return wire.Wire();
}
static void printEdge(const ShapeFix_WireSegment& wire,int index,const TopoDS_Face& face) {
  int umin,umax,vmin,vmax; wire.GetPatchIndex(index,umin,umax,vmin,vmax);
  double first,last; auto edge=wire.Edge(index); auto curve=BRep_Tool::CurveOnSurface(edge,face,first,last);
  gp_Pnt2d start=curve->Value(first),end=curve->Value(last);
  double cf,cl; bool spatial=!BRep_Tool::Curve(edge,cf,cl).IsNull();
  std::cout << ' ' << umin << ' ' << umax << ' ' << vmin << ' ' << vmax << ' ' << first << ' ' << last
    << ' ' << start.X() << ' ' << start.Y() << ' ' << end.X() << ' ' << end.Y()
    << ' ' << spatial << ' ' << BRep_Tool::SameParameter(edge) << ' ' << BRep_Tool::SameRange(edge);
}
static void inspect(const char* name,int mode) {
  TopoDS_Face face=BRepBuilderAPI_MakeFace(gp_Pln(),rectangle(0.),true).Face();
  if(mode==6) BRep_Builder().Add(face,rectangle(3.));
  occ::handle<NCollection_HArray2<occ::handle<Geom_Surface>>> patches=new NCollection_HArray2<occ::handle<Geom_Surface>>(1,1,1,1);
  patches->SetValue(1,1,new Geom_Plane(gp_Pln()));
  occ::handle<ShapeExtend_CompositeSurface> grid=new ShapeExtend_CompositeSurface(patches);
  SequenceProbe probe; probe.SetContext(new ShapeBuild_ReShape); probe.Init(grid,TopLoc_Location(),face,1.e-7);
  probe.ClosedMode()=mode==4;
  NCollection_Sequence<ShapeFix_WireSegment> wires; probe.LoadWires(wires);
  if(mode==5) wires(1).WireData()->Remove(3);
  bool alongU=mode!=1; double position=mode==2 ? 3. : mode==3 ? 0. : 1.;
  gp_Lin2d line(alongU ? gp_Pnt2d(position,0.) : gp_Pnt2d(0.,position),alongU ? gp_Dir2d(0.,1.) : gp_Dir2d(1.,0.));
  probe.SplitByLine(wires,line,alongU,1);
  std::cout << name << ' ' << wires.Length() << ' ' << probe.Status(ShapeExtend_FAIL4);
  for(int i=1;i<=wires.Length();++i) {
    std::cout << ' ' << int(wires(i).Orientation()) << ' ' << wires(i).NbEdges();
    for(int j=1;j<=wires(i).NbEdges();++j) printEdge(wires(i),j,face);
  }
  std::cout << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  const char* names[]={"vertical","horizontal","outside","tangent","closed","open","disjoint"};
  for(int mode=0;mode<7;++mode) inspect(names[mode],mode);
}

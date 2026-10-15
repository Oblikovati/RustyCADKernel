// The shapes and the result printer of tools/oracle/tkfeat_probe.cxx; the Rust twin is
// crates/tkfeat/examples/ft_probe/shapes.rs.
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Builder.hxx>
#include <GProp_GProps.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Ax2.hxx>
#include <gp_Pln.hxx>

// The box of the corner (x, y, z) and the sizes.
TopoDS_Shape ft_box(double x,double y,double z,double dx,double dy,double dz){
 return BRepPrimAPI_MakeBox(gp_Pnt(x,y,z),dx,dy,dz).Shape();}
// The basis of a request: 0 the box [0, 10]^3, 1 the cylinder of radius 5 and height 10
// on the axis (5, 5), 2 the plates [0, 10]^2 x [0, 4] and [0, 10]^2 x [6, 10] in a
// compound.
TopoDS_Shape ft_basis(int kind){
 if(kind==1)return BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(5.,5.,0.),gp_Dir(0.,0.,1.)),5.,10.).Shape();
 if(kind==2){
  BRep_Builder b;TopoDS_Compound c;b.MakeCompound(c);
  b.Add(c,ft_box(0.,0.,0.,10.,10.,4.));b.Add(c,ft_box(0.,0.,6.,10.,10.,4.));return c;}
 return ft_box(0.,0.,0.,10.,10.,10.);}
// The planar face bounded by the closed polygon through the points.
TopoDS_Face ft_polygon_face(const std::vector<gp_Pnt>& pts){
 BRepBuilderAPI_MakePolygon poly;for(const gp_Pnt& p:pts)poly.Add(p);poly.Close();
 return BRepBuilderAPI_MakeFace(poly.Wire(),true).Face();}
// The open polygon wire through the points.
TopoDS_Wire ft_polygon_wire(const std::vector<gp_Pnt>& pts){
 BRepBuilderAPI_MakePolygon poly;for(const gp_Pnt& p:pts)poly.Add(p);return poly.Wire();}
// The rectangle [x0, x1] x [y0, y1] at the height z.
TopoDS_Face ft_rect(double x0,double y0,double x1,double y1,double z){
 return ft_polygon_face({gp_Pnt(x0,y0,z),gp_Pnt(x1,y0,z),gp_Pnt(x1,y1,z),gp_Pnt(x0,y1,z)});}
// The first planar face of the shape at the height z with a vertical normal.
TopoDS_Face ft_face_at(const TopoDS_Shape& s,double z){
 for(TopExp_Explorer ex(s,TopAbs_FACE);ex.More();ex.Next()){
  BRepAdaptor_Surface a(TopoDS::Face(ex.Current()));
  if(a.GetType()!=GeomAbs_Plane)continue;
  gp_Pln p=a.Plane();
  if(p.Location().Z()==z&&std::abs(p.Axis().Direction().Z())==1.)return TopoDS::Face(ex.Current());}
 return TopoDS_Face();}
// The number of sub-shapes of a kind met by an explorer.
int ft_count(const TopoDS_Shape& s,TopAbs_ShapeEnum k){
 int n=0;for(TopExp_Explorer ex(s,k);ex.More();ex.Next())n++;return n;}
// A shape: its face and edge counts, volume, area and validity.
void ft_shape(const TopoDS_Shape& s){
 ga_int(ft_count(s,TopAbs_FACE));ga_int(ft_count(s,TopAbs_EDGE));
 GProp_GProps v;BRepGProp::VolumeProperties(s,v);ga_real(v.Mass());
 GProp_GProps a;BRepGProp::SurfaceProperties(s,a);ga_real(a.Mass());
 ga_flag(BRepCheck_Analyzer(s).IsValid());}
// A built algorithm: done, then its shape when done.
template<class Api> void ft_result(Api& api){
 ga_flag(api.IsDone());
 if(api.IsDone())ft_shape(api.Shape());}

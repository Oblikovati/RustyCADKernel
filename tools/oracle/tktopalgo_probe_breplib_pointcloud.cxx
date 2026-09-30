// BRepLib_PointCloudShape probe operation over shapes read by tp_shape
// (tktopalgo_probe_shape.cxx); the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_lib_point_cloud.rs.
#include <BRepLib_PointCloudShape.hxx>
#include <BRep_Tool.hxx>
#include <ElSLib.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Poly_Triangulation.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
// The cloud prints each point as it comes: point, normal, UV and the index of its face
// in the face map of the shape.
class TpPointCloud : public BRepLib_PointCloudShape {
public:
 TpPointCloud(const TopoDS_Shape& s,double tol):BRepLib_PointCloudShape(s,tol){TopExp::MapShapes(s,TopAbs_FACE,myFaces);}
protected:
 void addPoint(const gp_Pnt& p,const gp_Vec& n,const gp_Pnt2d& uv,const TopoDS_Shape& f) override{
  ga_pnt(p);ga_xyz(n.XYZ());ga_pnt2d(uv);ga_int(myFaces.FindIndex(f));}
private:
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> myFaces;
};
// Gives the triangulation of each plane, cylinder or sphere face UV nodes (the ElSLib
// parameters of its nodes), so ComputeNormals takes the surface normals.
void tp_cloud_uv_nodes(const TopoDS_Shape& s){
 for(TopExp_Explorer e(s,TopAbs_FACE);e.More();e.Next()){
  const TopoDS_Face& f=TopoDS::Face(e.Current());TopLoc_Location l;
  occ::handle<Poly_Triangulation> t=BRep_Tool::Triangulation(f,l);occ::handle<Geom_Surface> g=BRep_Tool::Surface(f);
  if(t.IsNull()||g.IsNull())continue;
  GeomAdaptor_Surface a(g);GeomAbs_SurfaceType k=a.GetType();
  if(k!=GeomAbs_Plane&&k!=GeomAbs_Cylinder&&k!=GeomAbs_Sphere)continue;
  t->AddUVNodes();
  for(int i=1;i<=t->NbNodes();i++){double u=0,v=0;gp_Pnt p=t->Node(i);
   if(k==GeomAbs_Plane)ElSLib::Parameters(a.Plane(),p,u,v);
   else if(k==GeomAbs_Cylinder)ElSLib::Parameters(a.Cylinder(),p,u,v);
   else ElSLib::Parameters(a.Sphere(),p,u,v);
   t->SetUVNode(i,gp_Pnt2d(u,v));}}}
// `shape tol dist uv n (op arg)...`: ops 0 NbPointsByDensity(arg), 1 GeneratePointsByDensity(arg),
// 2 NbPointsByTriangulation, 3 GeneratePointsByTriangulation, 4 SetDistance(arg),
// 5 SetTolerance(arg); prints each result (and the points of the generators), then the
// tolerance and the distance, or the exception.
void tp_blp_pointcloud_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);double tol=r.real(),dist=r.real();if(r.flag())tp_cloud_uv_nodes(s.shape);
  TpPointCloud c(s.shape,tol);c.SetDistance(dist);int n=r.integer();
  for(int i=0;i<n;i++){int op=r.integer();double arg=r.real();
   if(op==0)ga_int(c.NbPointsByDensity(arg));else if(op==1)ga_flag(c.GeneratePointsByDensity(arg));
   else if(op==2)ga_int(c.NbPointsByTriangulation());else if(op==3)ga_flag(c.GeneratePointsByTriangulation());
   else if(op==4)c.SetDistance(arg);else c.SetTolerance(arg);}
  ga_real(c.Tolerance());ga_real(c.GetDistance()))
 ga_end();}

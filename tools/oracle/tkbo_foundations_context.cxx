// IntTools_Context: cache identity and mutation, projection invalidation, real faces and solids.
#include <IntTools_Context.hxx>
#include <IntTools_SurfaceRangeLocalizeData.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <GeomAPI_ProjectPointOnSurf.hxx>
#include <Geom2dHatch_Hatcher.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Bnd_OBB.hxx>
void bo_context_face(IntTools_Context& c,const TopoDS_Face& f) {
 TopoDS_Face reversed=f;reversed.Orientation(TopAbs_REVERSED);
 bo_int(&c.FClass2d(f)==&c.FClass2d(reversed));bo_int(&c.SurfaceAdaptor(f)==&c.SurfaceAdaptor(reversed));
 double a,b,d,e;c.UVBounds(f,a,b,d,e);for(double x:{a,b,d,e})bo_real(x);
 auto& local=c.SurfaceData(f);local.SetGridDeflection(0.125);bo_int(&local==&c.SurfaceData(reversed));
 bo_int(local.GetNbSampleU());bo_int(local.GetNbSampleV());bo_real(local.GetMinRangeU());bo_real(local.GetMinRangeV());bo_real(local.GetGridDeflection());
 bo_int(c.IsInfiniteFace(f));
}
void bo_context_classification(IntTools_Context& c,const TopoDS_Face& f,const std::vector<double>& v) {
 gp_Pnt2d uv(v[2],v[3]);gp_Pnt p(v[2],v[3],v[4]);bo_int(c.StatePointFace(f,uv));bo_int(c.IsPointInFace(f,uv));bo_int(c.IsPointInOnFace(f,uv));
 bo_int(c.IsPointInFace(p,f,v[5]));bo_int(c.IsValidPointForFace(p,f,v[5]));bo_int(c.IsValidPointForFaces(p,f,f,v[5]));
 bo_int(c.ProjPS(f).IsDone());c.SetPOnSProjectionTolerance(1e-9);bo_int(c.ProjPS(f).IsDone());bo_int(c.IsValidPointForFace(p,f,v[5]));
}
void bo_context_hatching(IntTools_Context& c,const TopoDS_Face& f) {
 auto& h=c.Hatcher(f);bo_int(&h==&c.Hatcher(f));bo_real(h.Confusion2d());bo_real(h.Confusion3d());
 bo_real(h.Intersector().ConfusionTolerance());bo_real(h.Intersector().TangencyTolerance());bo_int(h.KeepPoints());bo_int(h.KeepSegments());
}
void bo_context_projection(IntTools_Context& c,const gp_Pnt& p) {
 TopoDS_Edge e=BRepBuilderAPI_MakeEdge(gp_Pnt(0.,0.,0.),gp_Pnt(2.,3.,4.));auto& pr=c.ProjPC(e);bo_int(&pr==&c.ProjPC(e));
 double f,l;auto curve=BRep_Tool::Curve(e,f,l);auto& other=c.ProjPT(curve);bo_int(&other==&c.ProjPT(curve));
 for(auto* projector:{&pr,&other}){projector->Perform(p);bo_int(projector->NbPoints());if(projector->NbPoints()>0){bo_real(projector->LowerDistance());bo_real(projector->LowerDistanceParameter());}}
}
void bo_context_solids(IntTools_Context& c,const gp_Pnt& p) {
 TopoDS_Solid solid=BRepPrimAPI_MakeBox(2.,3.,4.).Solid();auto& classifier=c.SolidClassifier(solid);bo_int(&classifier==&c.SolidClassifier(solid));
 classifier.Perform(p,1e-7);bo_int(classifier.State());auto& box=c.BndBox(solid);bo_int(&box==&c.BndBox(solid));
 for(const auto& point:{box.CornerMin(),box.CornerMax()}){bo_real(point.X());bo_real(point.Y());bo_real(point.Z());}
 auto& obb=c.OBB(solid,0.1);bo_int(&obb==&c.OBB(solid,5.));bo_real(obb.XHSize());bo_real(obb.YHSize());bo_real(obb.ZHSize());
}
void bo_context_case(const std::vector<double>& v) {
 try {IntTools_Context c;TopoDS_Face f=bo_fclass_face(int(v[0]),v[1]);bo_context_face(c,f);bo_context_classification(c,f,v);bo_context_hatching(c,f);
  gp_Pnt p(v[2],v[3],v[4]);bo_context_projection(c,p);bo_context_solids(c,p);
  IntTools_Context allocated(new NCollection_IncAllocator(4096));bo_context_face(allocated,f);bo_context_classification(allocated,f,v);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}

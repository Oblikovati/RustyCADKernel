// BRepClass3d probe operations over shapes read by tp_shape (tktopalgo_probe_shape.cxx);
// the Rust twin is crates/tktopalgo/examples/tp_probe/brep_class3d.rs. States and
// transitions are printed as integers, faces and shells as their record index (IsSame;
// -1 for a null shape, -2 for another shape), edges and vertices of the explorer map
// as their map index.
#include <BRepAdaptor_Surface.hxx>
#include <BRepClass3d.hxx>
#include <BRepClass3d_BndBoxTree.hxx>
#include <BRepClass3d_Intersector3d.hxx>
#include <BRepClass3d_SClassifier.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepClass3d_SolidExplorer.hxx>
#include <IntCurvesFace_Intersector.hxx>
#include <gp_Lin.hxx>
#include <gp_Vec.hxx>
template<class S> int tp_index_of(const std::vector<S>& shapes,const TopoDS_Shape& x){
 if(x.IsNull())return -1;for(size_t k=0;k<shapes.size();++k)if(shapes[k].IsSame(x))return int(k);return -2;}
// Makes the protected ClassifyUVPoint reachable.
struct TpExplorer:BRepClass3d_SolidExplorer{
 using BRepClass3d_SolidExplorer::BRepClass3d_SolidExplorer;using BRepClass3d_SolidExplorer::ClassifyUVPoint;};
void tp_state(const TpShape& s,const BRepClass3d_SClassifier& c){
 ga_int(c.State());ga_flag(c.Rejected());ga_flag(c.IsOnAFace());ga_int(tp_index_of(s.faces,c.Face()));}
// `shape mode tol n (x y z)...`: a solid classifier (mode 0 SolidClassifier(S) then
// Perform per point, 1 SolidClassifier(S, P, Tol) per point, 2 Load(S) on an empty
// classifier then Perform, 3 no shape at all, 4 PerformInfinitePoint then Perform).
void tp_bc3_classify_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);int mode=r.integer();double tol=r.real();int n=r.integer();
  if(mode==1){for(int i=0;i<n;++i){gp_Pnt p=r.pnt();GA_TRY(BRepClass3d_SolidClassifier c(s.shape,p,tol);tp_state(s,c))}}
  else{BRepClass3d_SolidClassifier c;if(mode==0||mode==4)c.Load(s.shape);else if(mode==2){c.Load(s.shape);c.Load(s.shape);}
   if(mode==4){GA_TRY(c.PerformInfinitePoint(tol);tp_state(s,c))}
   for(int i=0;i<n;++i){gp_Pnt p=r.pnt();GA_TRY(c.Perform(p,tol);tp_state(s,c))}
   c.Destroy();GA_TRY(c.PerformInfinitePoint(tol);tp_state(s,c))})
 ga_end();}
// `shape tol mode`: PerformInfinitePoint of a SolidClassifier (mode 0) or of an
// SClassifier on an explorer (mode 1, twice), then OuterShell of the shape as a solid.
void tp_bc3_infinite_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);double tol=r.real();int mode=r.integer();
  if(mode==0){BRepClass3d_SolidClassifier c(s.shape);GA_TRY(c.PerformInfinitePoint(tol);tp_state(s,c))}
  else{BRepClass3d_SolidExplorer e(s.shape);BRepClass3d_SClassifier c;tp_state(s,c);
   GA_TRY(c.PerformInfinitePoint(e,tol);tp_state(s,c))GA_TRY(c.PerformInfinitePoint(e,tol);tp_state(s,c))}
  if(s.shape.ShapeType()==TopAbs_SOLID){GA_TRY(ga_int(tp_index_of(s.shells,BRepClass3d::OuterShell(TopoDS::Solid(s.shape)))))})
 ga_end();}
// `shape tol n (x y z)...`: the SClassifier(S, P, Tol) constructor on one explorer, and
// its face segment index after each point.
void tp_bc3_sclassifier_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);double tol=r.real();BRepClass3d_SolidExplorer e(s.shape);int n=r.integer();
  for(int i=0;i<n;++i){gp_Pnt p=r.pnt();GA_TRY(BRepClass3d_SClassifier c(e,p,tol);tp_state(s,c);ga_int(e.GetFaceSegmentIndex()))})
 ga_end();}
// `shape x y z n`: Segment, then n OtherSegment calls: flag, line, parameter, index.
void tp_bc3_segment_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);gp_Pnt p=r.pnt();int n=r.integer();BRepClass3d_SolidExplorer e(s.shape);
  for(int i=0;i<=n;++i){gp_Lin l(gp_Pnt(7,8,9),gp_Dir(0,1,0));double par=-5.;
   GA_TRY(int flag=i==0?e.Segment(p,l,par):e.OtherSegment(p,l,par);ga_int(flag);ga_pnt(l.Location());ga_dir(l.Direction());
    ga_real(par);ga_int(e.GetFaceSegmentIndex());ga_flag(e.Reject(p)))})
 ga_end();}
// The explorer members: rejection, map, shells and faces (with their intersector boxes),
// the members past the last shell and face, a foreign face, Destroy, then a default
// explorer (no shape).
void tp_bc3_explorer_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);BRepClass3d_SolidExplorer e(s.shape);ga_flag(e.Reject(gp_Pnt()));ga_int(e.GetMapEV().Extent());
  for(int i=1;i<=e.GetMapEV().Extent();++i)ga_int(e.GetMapEV()(i).ShapeType());
  ga_flag(e.GetShape().IsSame(s.shape));ga_flag(e.GetTree().IsEmpty());
  for(e.InitShell();e.MoreShell();e.NextShell()){ga_int(tp_index_of(s.shells,e.CurrentShell()));
   ga_flag(e.RejectShell(gp_Lin()));
   for(e.InitFace();e.MoreFace();e.NextFace()){TopoDS_Face f=e.CurrentFace();ga_int(tp_index_of(s.faces,f));ga_int(f.Orientation());
    ga_flag(e.RejectFace(gp_Lin()));GA_TRY(IntCurvesFace_Intersector& x=e.Intersector(f);ga_int(x.Face().Orientation());
     Bnd_Box b=x.Bounding();ga_flag(b.IsVoid());if(!b.IsVoid()){double x0,y0,z0,x1,y1,z1;b.Get(x0,y0,z0,x1,y1,z1);ga_real(x0);ga_real(z1);})}
   GA_TRY(ga_int(tp_index_of(s.faces,e.CurrentFace())))}
  GA_TRY(ga_int(tp_index_of(s.shells,e.CurrentShell())))GA_TRY(e.InitFace();ga_flag(e.MoreFace()))
  TopoDS_Face foreign;BRep_Builder B;B.MakeFace(foreign);GA_TRY(e.Intersector(foreign);ga_int(0))
  e.Destroy();for(const TopoDS_Face& f:s.faces)GA_TRY(e.Intersector(f);ga_int(1))
  e.InitShape(s.shape);ga_int(e.GetFaceSegmentIndex());for(const TopoDS_Face& f:s.faces)GA_TRY(e.Intersector(f);ga_int(2))
  e.DumpSegment(gp_Pnt(),gp_Lin(),1.,TopAbs_IN);
  BRepClass3d_SolidExplorer d;ga_flag(d.Reject(gp_Pnt()));ga_int(d.GetFaceSegmentIndex());ga_flag(d.GetShape().IsNull());
  gp_Lin l(gp_Pnt(1,2,3),gp_Dir(0,0,1));double par=-1.;ga_int(d.Segment(gp_Pnt(),l,par));ga_real(par);ga_pnt(l.Location());
  ga_flag(d.Reject(gp_Pnt()));d.InitShell();ga_flag(d.MoreShell()))
 ga_end();}
// `shape face param mode`: the static FindAPointInTheFace overloads on the face.
void tp_bc3_findpoint_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);const TopoDS_Face& f=s.faces.at(r.integer());double param=r.real();
  gp_Pnt p(1,2,3);double u=4,v=5;gp_Vec d1u(6,7,8),d1v(9,10,11);
  GA_TRY(ga_flag(BRepClass3d_SolidExplorer::FindAPointInTheFace(f,p,u,v,param,d1u,d1v));ga_pnt(p);ga_real(u);ga_real(v);
   ga_xyz(d1u.XYZ());ga_xyz(d1v.XYZ()))
  gp_Pnt q(1,2,3);GA_TRY(ga_flag(BRepClass3d_SolidExplorer::FindAPointInTheFace(f,q,param));ga_pnt(q))
  gp_Pnt q2;double u2=-1,v2=-1;GA_TRY(ga_flag(BRepClass3d_SolidExplorer::FindAPointInTheFace(f,q2,u2,v2,param));ga_pnt(q2);ga_real(u2);ga_real(v2))
  gp_Pnt q3;double u3=-1,v3=-1;GA_TRY(ga_flag(BRepClass3d_SolidExplorer::FindAPointInTheFace(f,q3,u3,v3));ga_pnt(q3);ga_real(u3);ga_real(v3))
  gp_Pnt q4;GA_TRY(ga_flag(BRepClass3d_SolidExplorer::FindAPointInTheFace(f,q4));ga_pnt(q4))
  double u5=-1,v5=-1;GA_TRY(ga_flag(BRepClass3d_SolidExplorer::FindAPointInTheFace(f,u5,v5));ga_real(u5);ga_real(v5)))
 ga_end();}
// `shape face u v index n`: PointInTheFace(F, P, u, v, Param, Index) n times from the
// given (u, v) and index (the point starts at the surface point of (u, v) + (0, 0,
// mode)), then the surface overload with D1 on the unrestricted surface bounds.
void tp_bc3_pointinface_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);TopoDS_Face f=s.faces.at(r.integer());double u=r.real(),v=r.real();int index=r.integer();int n=r.integer();
  double lift=r.real();BRepClass3d_SolidExplorer e(s.shape);double param=0.5;
  BRepAdaptor_Surface a(f,true);gp_Pnt p;GA_TRY(p=a.Value(u,v).Translated(gp_Vec(0,0,lift)))
  for(int i=0;i<n;++i){GA_TRY(ga_flag(e.PointInTheFace(f,p,u,v,param,index));ga_pnt(p);ga_real(u);ga_real(v);ga_real(param);ga_int(index))}
  occ::handle<BRepAdaptor_Surface> h=new BRepAdaptor_Surface(f,false);gp_Vec d1u(1,0,0),d1v(0,1,0);int k=1;
  GA_TRY(ga_flag(e.PointInTheFace(f,p,u,v,param,k,h,h->FirstUParameter(),h->FirstVParameter(),h->LastUParameter(),h->LastVParameter(),d1u,d1v));
   ga_pnt(p);ga_real(u);ga_real(v);ga_int(k);ga_xyz(d1u.XYZ());ga_xyz(d1v.XYZ()))
  gp_Pnt p2=p;double u2=u,v2=v;int k2=3;
  GA_TRY(ga_flag(e.PointInTheFace(f,p2,u2,v2,param,k2,h,0.,0.,1.,1.));ga_pnt(p2);ga_real(u2);ga_real(v2);ga_int(k2)))
 ga_end();}
// `shape face n (u v)...`: the protected ClassifyUVPoint on the face's intersector and
// restricted surface.
void tp_bc3_uvpoint_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);TopoDS_Face f=s.faces.at(r.integer());TpExplorer e(s.shape);
  occ::handle<BRepAdaptor_Surface> h=new BRepAdaptor_Surface(f,true);int n=r.integer();
  for(int i=0;i<n;++i){gp_Pnt2d q=r.pnt2d();GA_TRY(ga_int(e.ClassifyUVPoint(e.Intersector(f),h,q)))})
 ga_end();}
// `shape mode fields`: the tree selectors on the explorer's tree: mode 0 point x y z,
// mode 1 line P D par (then its edge and vertex parameters, and out-of-range indices).
void tp_bc3_tree_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);int mode=r.integer();BRepClass3d_SolidExplorer e(s.shape);
  const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& m=e.GetMapEV();
  if(mode==0){gp_Pnt p=r.pnt();BRepClass3d_BndBoxTreeSelectorPoint sel(m);sel.SetCurrentPoint(p);
   GA_TRY(ga_int(e.GetTree().Select(sel));ga_flag(sel.Stop()))
   for(int i=1;i<=m.Extent()+1;++i){GA_TRY(ga_flag(sel.Accept(i)))}
   GA_TRY(ga_flag(sel.Accept(0)))
   Bnd_Box b;b.Update(-1,-1,-1,1,1,1);ga_flag(sel.Reject(b));}
  else{gp_Pnt p=r.pnt();gp_Dir d=r.dir();double par=r.real();BRepClass3d_BndBoxTreeSelectorLine sel(m);
   GA_TRY(sel.SetCurrentLine(gp_Lin(p,d),par);ga_int(e.GetTree().Select(sel));ga_flag(sel.IsCorrect());
    ga_int(sel.GetNbEdgeParam());ga_int(sel.GetNbVertParam());
    for(int i=0;i<=sel.GetNbEdgeParam()+1;++i){GA_TRY(TopoDS_Edge x;double a,b;sel.GetEdgeParam(i,x,a,b);ga_int(m.FindIndex(x));ga_real(a);ga_real(b))}
    for(int i=0;i<=sel.GetNbVertParam()+1;++i){GA_TRY(TopoDS_Vertex x;double a;sel.GetVertParam(i,x,a);ga_int(m.FindIndex(x));ga_real(a))}
    sel.ClearResults();ga_flag(sel.IsCorrect());ga_int(sel.GetNbEdgeParam());
    Bnd_Box b;b.Update(-1,-1,-1,1,1,1);ga_flag(sel.Reject(b)))}
 ) ga_end();}
// `shape face n (P D prm tol)...`: one Intersector3d on the face for several lines (the
// transition only once a point is kept: it is uninitialized natively before).
void tp_bc3_inter3d_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);TopoDS_Face f=s.faces.at(r.integer());int n=r.integer();BRepClass3d_Intersector3d x;
  ga_flag(x.IsDone());ga_flag(x.HasAPoint());ga_int(x.State());
  for(int i=0;i<n;++i){gp_Pnt p=r.pnt();gp_Dir d=r.dir();double prm=r.real(),tol=r.real();
   GA_TRY(x.Perform(gp_Lin(p,d),prm,tol,f);ga_flag(x.IsDone());ga_flag(x.HasAPoint());ga_real(x.UParameter());ga_real(x.VParameter());
    ga_real(x.WParameter());ga_pnt(x.Pnt());if(x.HasAPoint())ga_int(x.Transition());ga_int(x.State());ga_int(tp_index_of(s.faces,x.Face()));
    ga_int(x.Face().IsNull()?-1:x.Face().Orientation()))})
 ga_end();}

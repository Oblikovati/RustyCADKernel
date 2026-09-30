// BRepGProp probe operations over shapes read by tp_shape (tktopalgo_probe_shape.cxx): the
// package functions and the Cinert, Sinert, Vinert, VinertGK, MeshCinert and MeshProps
// frameworks. The Rust twin is crates/tktopalgo/examples/tp_probe/brep_gprop.rs; the face
// tool, domain, Gauss integrator and integrand functions are in
// tktopalgo_probe_brepgprop_tools.cxx. Edges and faces are their record index (IsSame).
#include <BRepGProp.hxx>
#include <BRepGProp_Cinert.hxx>
#include <BRepGProp_Domain.hxx>
#include <BRepGProp_EdgeTool.hxx>
#include <BRepGProp_Face.hxx>
#include <BRepGProp_MeshCinert.hxx>
#include <BRepGProp_MeshProps.hxx>
#include <BRepGProp_Sinert.hxx>
#include <BRepGProp_Vinert.hxx>
#include <BRepGProp_VinertGK.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <GProp_GProps.hxx>
#include <GProp_PrincipalProps.hxx>
#include <gp_Pln.hxx>
// A framework: mass, centre, matrix, static moments, the moment about a fixed axis,
// then the principal properties (or their exception).
void tp_gprops(const GProp_GProps& p){
 ga_real(p.Mass());ga_pnt(p.CentreOfMass());gp_Mat m=p.MatrixOfInertia();
 for(int i=1;i<=3;++i)for(int j=1;j<=3;++j)ga_real(m(i,j));
 double ix,iy,iz;p.StaticMoments(ix,iy,iz);ga_real(ix);ga_real(iy);ga_real(iz);
 ga_real(p.MomentOfInertia(gp_Ax1(gp_Pnt(1,2,3),gp_Dir(1,1,1))));
 GA_TRY(GProp_PrincipalProps pp=p.PrincipalProperties();double a,b,c;pp.Moments(a,b,c);ga_real(a);ga_real(b);ga_real(c);
  ga_xyz(pp.FirstAxisOfInertia().XYZ());ga_xyz(pp.SecondAxisOfInertia().XYZ());ga_xyz(pp.ThirdAxisOfInertia().XYZ());
  pp.RadiusOfGyration(a,b,c);ga_real(a);ga_real(b);ga_real(c);ga_flag(pp.HasSymmetryAxis());ga_flag(pp.HasSymmetryPoint()))}
// `P N` of a plane.
gp_Pln tp_plane(GaReader& r){gp_Pnt p=r.pnt();return gp_Pln(p,r.dir());}
// `shape mode flags...`: mode 0 LinearProperties(S, P, skip, tri), 1 SurfaceProperties(S, P,
// skip, tri), 2 SurfaceProperties(S, P, eps, skip), 3 VolumeProperties(S, P, closed, skip,
// tri), 4 VolumeProperties(S, P, eps, closed, skip), 5 VolumePropertiesGK(S, P, eps, closed,
// span, cg, i, skip), 6 the same with a plane `P N` first; the error, then the framework.
void tp_bgp_package_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);int mode=r.integer();GProp_GProps p;
  if(mode==0){bool a=r.flag();BRepGProp::LinearProperties(s.shape,p,a,r.flag());}
  else if(mode==1){bool a=r.flag();BRepGProp::SurfaceProperties(s.shape,p,a,r.flag());}
  else if(mode==2){double e=r.real();ga_real(BRepGProp::SurfaceProperties(s.shape,p,e,r.flag()));}
  else if(mode==3){bool a=r.flag(),b=r.flag();BRepGProp::VolumeProperties(s.shape,p,a,b,r.flag());}
  else if(mode==4){double e=r.real();bool a=r.flag();ga_real(BRepGProp::VolumeProperties(s.shape,p,e,a,r.flag()));}
  else if(mode==5){double e=r.real();bool a=r.flag(),b=r.flag(),c=r.flag(),d=r.flag();
   ga_real(BRepGProp::VolumePropertiesGK(s.shape,p,e,a,b,c,d,r.flag()));}
  else{gp_Pln pl=tp_plane(r);double e=r.real();bool a=r.flag(),b=r.flag(),c=r.flag(),d=r.flag();
   ga_real(BRepGProp::VolumePropertiesGK(s.shape,p,pl,e,a,b,c,d,r.flag()));}
  tp_gprops(p))
 ga_end();}
// `shape edge x y z`: the edge tool on BRepAdaptor_Curve(E) (range, order, CN intervals,
// value and D1 at the middle), Cinert(C, loc) and a default Cinert performed after
// SetLocation.
void tp_bgp_cinert_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);const TopoDS_Edge& e=s.edges.at(r.integer());gp_Pnt loc=r.pnt();BRepAdaptor_Curve c(e);
  double f=BRepGProp_EdgeTool::FirstParameter(c),l=BRepGProp_EdgeTool::LastParameter(c);ga_real(f);ga_real(l);
  ga_int(BRepGProp_EdgeTool::IntegrationOrder(c));int n=BRepGProp_EdgeTool::NbIntervals(c,GeomAbs_CN);ga_int(n);
  NCollection_Array1<double> t(1,n+1);BRepGProp_EdgeTool::Intervals(c,t,GeomAbs_CN);for(double x:t)ga_real(x);
  ga_pnt(BRepGProp_EdgeTool::Value(c,0.5*(f+l)));gp_Pnt p;gp_Vec v;BRepGProp_EdgeTool::D1(c,0.5*(f+l),p,v);ga_pnt(p);ga_xyz(v.XYZ());
  BRepGProp_Cinert a(c,loc);tp_gprops(a);BRepGProp_Cinert b;b.SetLocation(loc);b.Perform(c);tp_gprops(b))
 ga_end();}
// The deprecated overloads, outside the GA_TRY macro so that the pragma applies.
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wdeprecated-declarations"
void tp_prepare_polygon_into(const TopoDS_Edge& e,occ::handle<NCollection_HArray1<gp_Pnt>>& h){
 BRepGProp_MeshCinert::PreparePolygon(e,h);}
void tp_u_knots_into(const BRepGProp_Face& f,double a,double b,occ::handle<NCollection_HArray1<double>>& h){f.GetUKnots(a,b,h);}
void tp_t_knots_into(const BRepGProp_Face& f,double a,double b,occ::handle<NCollection_HArray1<double>>& h){f.GetTKnots(a,b,h);}
#pragma GCC diagnostic pop
// `shape edge x y z`: PreparePolygon(E) (null, or its points) and the MeshCinert of them.
void tp_bgp_mesh_cinert_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);const TopoDS_Edge& e=s.edges.at(r.integer());gp_Pnt loc=r.pnt();
  occ::handle<NCollection_HArray1<gp_Pnt>> nodes=BRepGProp_MeshCinert::PreparePolygon(e);ga_flag(nodes.IsNull());
  if(!nodes.IsNull()){ga_int(nodes->Lower());ga_int(nodes->Length());for(const gp_Pnt& x:nodes->Array1())ga_pnt(x);
   BRepGProp_MeshCinert m;m.SetLocation(loc);m.Perform(nodes->Array1());tp_gprops(m);}
  occ::handle<NCollection_HArray1<gp_Pnt>> again;tp_prepare_polygon_into(e,again);ga_flag(again.IsNull()))
 ga_end();}
// `lower n (x y z)... x y z`: MeshCinert::Perform on the nodes indexed from `lower`.
void tp_bgp_mesh_nodes_case(std::istream& input){
 GaReader r{input};int lower=r.integer(),n=r.integer();NCollection_Array1<gp_Pnt> nodes(lower,lower+n-1);
 for(int i=0;i<n;++i)nodes(lower+i)=r.pnt();gp_Pnt loc=r.pnt();
 GA_TRY(BRepGProp_MeshCinert m;m.SetLocation(loc);m.Perform(nodes);tp_gprops(m))
 ga_end();}
// `kind args`: a transformation: 0 translation D, 1 rotation P D angle, 2 scale P s,
// 3 mirror about a point P, 4 rotation then scale P D angle s.
gp_Trsf tp_bgp_trsf(GaReader& r){
 int kind=r.integer();gp_Trsf t;
 if(kind==0){t.SetTranslation(gp_Vec(r.xyz()));}
 else if(kind==1){gp_Pnt p=r.pnt();gp_Dir d=r.dir();t.SetRotation(gp_Ax1(p,d),r.real());}
 else if(kind==2){gp_Pnt p=r.pnt();t.SetScale(p,r.real());}
 else if(kind==3){t.SetMirror(r.pnt());}
 else{gp_Pnt p=r.pnt();gp_Dir d=r.dir();t.SetRotation(gp_Ax1(p,d),r.real());gp_Trsf s;s.SetScale(p,r.real());t.PreMultiply(s);}
 return t;}
// `shape face type ori located [trsf] x y z`: MeshProps(type) on the face triangulation,
// with the location of `trsf` (Perform(mesh, loc, ori)) or without (Perform(mesh, ori)).
void tp_bgp_mesh_props_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);const TopoDS_Face& f=s.faces.at(r.integer());
  BRepGProp_MeshProps m(BRepGProp_MeshProps::BRepGProp_MeshObjType(r.integer()));
  TopAbs_Orientation o=TopAbs_Orientation(r.integer());bool located=r.flag();TopLoc_Location l;
  if(located)l=TopLoc_Location(tp_bgp_trsf(r));m.SetLocation(r.pnt());ga_int(m.GetMeshObjType());
  TopLoc_Location dummy;const occ::handle<Poly_Triangulation>& mesh=BRep_Tool::Triangulation(f,dummy);
  if(located)m.Perform(mesh,l,o);else m.Perform(mesh,o);tp_gprops(m))
 ga_end();}
// `p1 p2 p3 apex volume n (l1 l2 w)...`: CalculateProps into zeroed sums.
void tp_bgp_mesh_calc_case(std::istream& input){
 GaReader r{input};gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt(),apex=r.pnt();bool volume=r.flag();int n=r.integer();
 std::vector<double> rule(3*n);for(double& x:rule)x=r.real();double sums[10]={0,0,0,0,0,0,0,0,0,0};
 GA_TRY(BRepGProp_MeshProps::CalculateProps(a,b,c,apex,volume,sums,n,rule.data());for(double x:sums)ga_real(x))
 ga_end();}
// One boundary of the region: `0` the location, `1 x y z` a point, `2 P N` a plane.
struct TpBound{int kind;gp_Pnt point;gp_Pln plane;};
TpBound tp_bound(GaReader& r){TpBound b{r.integer(),gp_Pnt(),gp_Pln()};
 if(b.kind==1)b.point=r.pnt();else if(b.kind==2)b.plane=tp_plane(r);return b;}
// `shape n (face mode) x y z eps`: one Sinert, the first step a constructor (mode 0 (S,
// loc), 1 (S, D, loc), 2 (S, loc, eps), 3 (S, D, loc, eps)) or a Perform (mode 4 (S), 5
// (S, D), 6 (S, eps), 7 (S, D, eps)) and the next steps Performs; the returned error, the
// epsilon and the framework after each step.
void tp_bgp_sinert_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);int n=r.integer();std::vector<std::pair<int,int>> steps;
  for(int i=0;i<n;++i){int f=r.integer();steps.emplace_back(f,r.integer());}gp_Pnt loc=r.pnt();double eps=r.real();
  std::unique_ptr<BRepGProp_Sinert> g;
  for(auto [k,mode]:steps){BRepGProp_Face F(s.faces.at(k));BRepGProp_Domain D(s.faces.at(k));
   GA_TRY(if(mode==0)g.reset(new BRepGProp_Sinert(F,loc));else if(mode==1)g.reset(new BRepGProp_Sinert(F,D,loc));
    else if(mode==2)g.reset(new BRepGProp_Sinert(F,loc,eps));else if(mode==3)g.reset(new BRepGProp_Sinert(F,D,loc,eps));
    else{if(!g){g.reset(new BRepGProp_Sinert());g->SetLocation(loc);}
     if(mode==4)g->Perform(F);else if(mode==5)g->Perform(F,D);else if(mode==6)ga_real(g->Perform(F,eps));else ga_real(g->Perform(F,D,eps));}
    ga_real(g->GetEpsilon());tp_gprops(*g))})
 ga_end();}
// One Vinert step: `mode` bit 0 adaptive (eps), bit 1 with the domain, bit 2 a
// constructor; the bound `b`.
void tp_vinert_step(std::unique_ptr<BRepGProp_Vinert>& g,BRepGProp_Face& F,BRepGProp_Domain& D,int mode,const TpBound& b,
                    const gp_Pnt& loc,double eps){
 bool adaptive=mode&1,domain=mode&2,ctor=mode&4;
 if(ctor){
  if(b.kind==0){if(domain){if(adaptive)g.reset(new BRepGProp_Vinert(F,D,loc,eps));else g.reset(new BRepGProp_Vinert(F,D,loc));}
   else if(adaptive)g.reset(new BRepGProp_Vinert(F,loc,eps));else g.reset(new BRepGProp_Vinert(F,loc));}
  else if(b.kind==1){if(domain){if(adaptive)g.reset(new BRepGProp_Vinert(F,D,b.point,loc,eps));else g.reset(new BRepGProp_Vinert(F,D,b.point,loc));}
   else if(adaptive)g.reset(new BRepGProp_Vinert(F,b.point,loc,eps));else g.reset(new BRepGProp_Vinert(F,b.point,loc));}
  else{if(domain){if(adaptive)g.reset(new BRepGProp_Vinert(F,D,b.plane,loc,eps));else g.reset(new BRepGProp_Vinert(F,D,b.plane,loc));}
   else if(adaptive)g.reset(new BRepGProp_Vinert(F,b.plane,loc,eps));else g.reset(new BRepGProp_Vinert(F,b.plane,loc));}
  return;}
 if(!g){g.reset(new BRepGProp_Vinert());g->SetLocation(loc);}
 if(b.kind==0){if(domain){if(adaptive)ga_real(g->Perform(F,D,eps));else g->Perform(F,D);}
  else if(adaptive)ga_real(g->Perform(F,eps));else g->Perform(F);}
 else if(b.kind==1){if(domain){if(adaptive)ga_real(g->Perform(F,D,b.point,eps));else g->Perform(F,D,b.point);}
  else if(adaptive)ga_real(g->Perform(F,b.point,eps));else g->Perform(F,b.point);}
 else{if(domain){if(adaptive)ga_real(g->Perform(F,D,b.plane,eps));else g->Perform(F,D,b.plane);}
  else if(adaptive)ga_real(g->Perform(F,b.plane,eps));else g->Perform(F,b.plane);}}
// `shape n (face mode bound) x y z eps`: one Vinert over the steps (tp_vinert_step); the
// returned error, the epsilon and the framework after each step.
void tp_bgp_vinert_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);int n=r.integer();std::vector<std::tuple<int,int,TpBound>> steps;
  for(int i=0;i<n;++i){int f=r.integer(),mode=r.integer();steps.emplace_back(f,mode,tp_bound(r));}
  gp_Pnt loc=r.pnt();double eps=r.real();std::unique_ptr<BRepGProp_Vinert> g;
  for(auto& [k,mode,b]:steps){BRepGProp_Face F(s.faces.at(k));BRepGProp_Domain D(s.faces.at(k));
   GA_TRY(tp_vinert_step(g,F,D,mode,b,loc,eps);ga_real(g->GetEpsilon());tp_gprops(*g))})
 ga_end();}
// One VinertGK step: `mode` bit 0 with the domain, bit 1 a constructor; the bound `b`.
void tp_vinert_gk_step(std::unique_ptr<BRepGProp_VinertGK>& g,BRepGProp_Face& F,BRepGProp_Domain& D,int mode,const TpBound& b,
                       const gp_Pnt& loc,double tol,bool cg,bool in){
 bool domain=mode&1,ctor=mode&2;
 if(ctor){
  if(b.kind==0){if(domain)g.reset(new BRepGProp_VinertGK(F,D,loc,tol,cg,in));else g.reset(new BRepGProp_VinertGK(F,loc,tol,cg,in));}
  else if(b.kind==1){if(domain)g.reset(new BRepGProp_VinertGK(F,D,b.point,loc,tol,cg,in));else g.reset(new BRepGProp_VinertGK(F,b.point,loc,tol,cg,in));}
  else{if(domain)g.reset(new BRepGProp_VinertGK(F,D,b.plane,loc,tol,cg,in));else g.reset(new BRepGProp_VinertGK(F,b.plane,loc,tol,cg,in));}
  return;}
 if(!g){g.reset(new BRepGProp_VinertGK());g->SetLocation(loc);}
 if(b.kind==0){if(domain)ga_real(g->Perform(F,D,tol,cg,in));else ga_real(g->Perform(F,tol,cg,in));}
 else if(b.kind==1){if(domain)ga_real(g->Perform(F,D,b.point,tol,cg,in));else ga_real(g->Perform(F,b.point,tol,cg,in));}
 else{if(domain)ga_real(g->Perform(F,D,b.plane,tol,cg,in));else ga_real(g->Perform(F,b.plane,tol,cg,in));}}
// `shape n (face mode bound) x y z tol cg i span`: one VinertGK over the steps (face
// tools with IsUseSpan `span`); the returned error, GetErrorReached and the framework.
void tp_bgp_vinert_gk_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);int n=r.integer();std::vector<std::tuple<int,int,TpBound>> steps;
  for(int i=0;i<n;++i){int f=r.integer(),mode=r.integer();steps.emplace_back(f,mode,tp_bound(r));}
  gp_Pnt loc=r.pnt();double tol=r.real();bool cg=r.flag(),in=r.flag(),span=r.flag();std::unique_ptr<BRepGProp_VinertGK> g;
  for(auto& [k,mode,b]:steps){BRepGProp_Face F(s.faces.at(k),span);BRepGProp_Domain D(s.faces.at(k));
   GA_TRY(tp_vinert_gk_step(g,F,D,mode,b,loc,tol,cg,in);ga_real(g->GetErrorReached());tp_gprops(*g))})
 ga_end();}

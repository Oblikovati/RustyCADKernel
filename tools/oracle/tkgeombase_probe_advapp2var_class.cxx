// TKGeomBase oracle: the AdvApp2Var classes (tkgeombase_probe.cxx) - Context, Node, Iso,
// Patch, Network, Framework and ApproxAFunc2Var - over the functions of
// tkgeombase_probe_advapp2var_eval.cxx. The Rust twin is
// crates/tkgeombase/examples/gb_probe/adv_class.rs.
#include <AdvApp2Var_ApproxAFunc2Var.hxx>
#include <AdvApp2Var_Context.hxx>
#include <AdvApp2Var_Criterion.hxx>
#include <AdvApp2Var_Framework.hxx>
#include <AdvApp2Var_Iso.hxx>
#include <AdvApp2Var_Network.hxx>
#include <AdvApp2Var_Node.hxx>
#include <AdvApp2Var_Patch.hxx>
#include <AdvApprox_DichoCutting.hxx>
#include <AdvApprox_PrefAndRec.hxx>
#include <Geom_BSplineSurface.hxx>
// The tolerances of `n` subspaces of dimension `d`: 1e-3*k*d, frontier (k, j) = tol*(1+j/10).
occ::handle<NCollection_HArray1<double>> gb_aa_tol(int n,int d){
 if(n<=0)return nullptr;occ::handle<NCollection_HArray1<double>> t=new NCollection_HArray1<double>(1,n);
 for(int k=1;k<=n;++k)t->SetValue(k,1.e-3*k*d);return t;
}
occ::handle<NCollection_HArray2<double>> gb_aa_tof(int n,int d){
 if(n<=0)return nullptr;occ::handle<NCollection_HArray2<double>> t=new NCollection_HArray2<double>(1,n,1,4);
 for(int k=1;k<=n;++k)for(int j=1;j<=4;++j)t->SetValue(k,j,1.e-3*k*d*(1.+j/10.));return t;
}
void gb_aa_harray(const occ::handle<NCollection_HArray1<double>>& a){
 if(a.IsNull()){gb_int(-1);return;}gb_int(a->Length());for(int i=a->Lower();i<=a->Upper();++i)gb_real(a->Value(i));
}
void gb_aa_harray2(const occ::handle<NCollection_HArray2<double>>& a){
 if(a.IsNull()){gb_int(-1);return;}gb_int(a->ColLength());gb_int(a->RowLength());
 for(int i=a->LowerRow();i<=a->UpperRow();++i)for(int j=a->LowerCol();j<=a->UpperCol();++j)gb_real(a->Value(i,j));
}
// The first `n` values of a table (the rest is uninitialised natively).
void gb_aa_head(const occ::handle<NCollection_HArray1<double>>& a,int n){for(int i=0;i<n;++i)gb_real(a->Value(a->Lower()+i));}
// A Gauss table of `n` points: for an even count mmapptt_ leaves the zero-root row
// unwritten (uninitialised natively), so it is skipped.
void gb_aa_gauss(const occ::handle<NCollection_HArray1<double>>& a,int n){
 gb_int(a->Length());for(int k=0;k<a->Length();++k)if(n%2==1||k%(n/2+1)!=0)gb_real(a->Value(a->Lower()+k));
}
// aa_context: ifav iu iv nlimu nlimv iprecis nb1 nb2 nb3.
void gb_aa_context_case(std::istream& in){
 GbReader r{in};int f=r.integer(),iu=r.integer(),iv=r.integer(),nu=r.integer(),nv=r.integer(),p=r.integer();
 int n1=r.integer(),n2=r.integer(),n3=r.integer();
 try{
  AdvApp2Var_Context c(f,iu,iv,nu,nv,p,n1,n2,n3,gb_aa_tol(n1,1),gb_aa_tol(n2,2),gb_aa_tol(n3,3),gb_aa_tof(n1,1),
   gb_aa_tof(n2,2),gb_aa_tof(n3,3));
  gb_int(c.TotalDimension());gb_int(c.TotalNumberSSP());gb_int(c.FavorIso());gb_int(c.UOrder());gb_int(c.VOrder());
  gb_int(c.ULimit());gb_int(c.VLimit());gb_int(c.UJacDeg());gb_int(c.VJacDeg());gb_aa_harray(c.UJacMax());
  gb_aa_harray(c.VJacMax());gb_int(c.URoots()->Length());gb_int(c.VRoots()->Length());
  gb_aa_head(c.URoots(),c.URoots()->Length()-(iu>-1?2:0));gb_aa_head(c.VRoots(),c.VRoots()->Length()-(iv>-1?2:0));
  const int pu=c.URoots()->Length()-(iu>-1?2:0),pv=c.VRoots()->Length()-(iv>-1?2:0);
  gb_aa_gauss(c.UGauss(),pu);gb_aa_gauss(c.VGauss(),pv);gb_aa_harray(c.IToler());gb_aa_harray2(c.FToler());
  gb_aa_harray2(c.CToler());
  AdvApp2Var_Context copy;copy=c;gb_int(copy.ULimit());gb_aa_harray(copy.IToler());
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// The one-patch grid of AdvApp2Var_ApproxAFunc2Var::InitGrid, with `cuts` (direction, value)
// applied to the network and the framework.
struct GbAaGrid{
 AdvApp2Var_Network net;AdvApp2Var_Framework frame;
 GbAaGrid(int iu,int iv,const double* d){
  NCollection_Sequence<occ::handle<AdvApp2Var_Patch>> p;p.Append(new AdvApp2Var_Patch(d[0],d[1],d[2],d[3],iu,iv));
  NCollection_Sequence<double> us,vs;us.Append(d[0]);us.Append(d[1]);vs.Append(d[2]);vs.Append(d[3]);
  net=AdvApp2Var_Network(p,us,vs);
  NCollection_Sequence<occ::handle<AdvApp2Var_Node>> bag;
  bag.Append(new AdvApp2Var_Node(gp_XY(d[0],d[2]),iu,iv));bag.Append(new AdvApp2Var_Node(gp_XY(d[1],d[2]),iu,iv));
  bag.Append(new AdvApp2Var_Node(gp_XY(d[0],d[3]),iu,iv));bag.Append(new AdvApp2Var_Node(gp_XY(d[1],d[3]),iu,iv));
  NCollection_Sequence<occ::handle<AdvApp2Var_Iso>> bu,bv;
  bu.Append(new AdvApp2Var_Iso(GeomAbs_IsoV,d[2],d[0],d[1],d[2],d[3],1,iu,iv));
  bu.Append(new AdvApp2Var_Iso(GeomAbs_IsoV,d[3],d[0],d[1],d[2],d[3],2,iu,iv));
  bv.Append(new AdvApp2Var_Iso(GeomAbs_IsoU,d[0],d[0],d[1],d[2],d[3],3,iu,iv));
  bv.Append(new AdvApp2Var_Iso(GeomAbs_IsoU,d[1],d[0],d[1],d[2],d[3],4,iu,iv));
  NCollection_Sequence<NCollection_Sequence<occ::handle<AdvApp2Var_Iso>>> su,sv;su.Append(bu);sv.Append(bv);
  frame=AdvApp2Var_Framework(bag,su,sv);
 }
 void cut(GbReader& r){
  const int n=r.integer();
  for(int i=0;i<n;++i){const int dir=r.integer();const double x=r.real();
   if(dir==0){net.UpdateInU(x);frame.UpdateInU(x);}else{net.UpdateInV(x);frame.UpdateInV(x);}}
 }
};
void gb_aa_iso(const AdvApp2Var_Iso& i){
 gb_int(i.Type());gb_real(i.Constante());gb_real(i.T0());gb_real(i.T1());gb_real(i.U0());gb_real(i.U1());gb_real(i.V0());
 gb_real(i.V1());gb_int(i.UOrder());gb_int(i.VOrder());gb_int(i.Position());gb_flag(i.IsApproximated());gb_flag(i.HasResult());
}
// aa_grid: iu iv u0 u1 v0 v1 cuts: the network and framework topology after the cuts.
void gb_aa_grid_case(std::istream& in){
 GbReader r{in};int iu=r.integer(),iv=r.integer();double d[4];for(double& x:d)x=r.real();
 try{
  GbAaGrid g(iu,iv,d);g.cut(r);AdvApp2Var_Network& n=g.net;AdvApp2Var_Framework& f=g.frame;
  gb_int(n.NbPatch());gb_int(n.NbPatchInU());gb_int(n.NbPatchInV());
  for(int i=1;i<=n.NbPatchInU()+1;++i)gb_real(n.UParameter(i));for(int j=1;j<=n.NbPatchInV()+1;++j)gb_real(n.VParameter(j));
  for(int j=1;j<=n.NbPatchInV();++j)for(int i=1;i<=n.NbPatchInU();++i){const AdvApp2Var_Patch& p=n.Patch(i,j);
   gb_real(p.U0());gb_real(p.U1());gb_real(p.V0());gb_real(p.V1());gb_int(p.UOrder());gb_int(p.VOrder());}
  int k=0;gb_flag(n.FirstNotApprox(k));gb_int(k);
  for(int i=1;i<=n.NbPatch();++i)n(i).ChangeNbCoeff(i+2,(2*i)%7+1);
  int cu=0,cv=0;n.SameDegree(iu,iv,cu,cv);gb_int(cu);gb_int(cv);
  for(int i=1;i<=n.NbPatch();++i){gb_int(n(i).NbCoeffInU());gb_int(n(i).NbCoeffInV());}
  const int nodes=(n.NbPatchInU()+1)*(n.NbPatchInV()+1);
  for(int i=1;i<=nodes;++i){gp_XY c=f.Node(i)->Coord();gb_xy(c);gb_int(f.Node(i)->UOrder());}
  for(int j=1;j<=n.NbPatchInV()+1;++j)for(int i=1;i<=n.NbPatchInU();++i){
   gb_aa_iso(f.IsoV(n.UParameter(i),n.UParameter(i+1),n.VParameter(j)));
   gb_int(f.FirstNode(GeomAbs_IsoV,j,i));gb_int(f.LastNode(GeomAbs_IsoV,j,i));}
  for(int i=1;i<=n.NbPatchInU()+1;++i)for(int j=1;j<=n.NbPatchInV();++j){
   gb_aa_iso(f.IsoU(n.UParameter(i),n.VParameter(j),n.VParameter(j+1)));
   gb_int(f.FirstNode(GeomAbs_IsoU,i,j));gb_int(f.LastNode(GeomAbs_IsoU,i,j));}
  gb_xy(f.Node(n.UParameter(1),n.VParameter(n.NbPatchInV()+1))->Coord());
  int i1=0,i2=0;occ::handle<AdvApp2Var_Iso> first=f.FirstNotApprox(i1,i2);gb_flag(!first.IsNull());gb_int(i1);gb_int(i2);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// A criterion of the probe: 2 * max error + 1e-4 * coefficient count, satisfied under MaxValue.
class GbAaCriterion : public AdvApp2Var_Criterion{
 public:
 GbAaCriterion(double m,int type,int rep){myMaxValue=m;myType=AdvApp2Var_CriterionType(type);
  myRepartition=AdvApp2Var_CriterionRepartition(rep);}
 void Value(AdvApp2Var_Patch& p,const AdvApp2Var_Context&) const override{
  p.SetCritValue(2.*p.MaxErrors()->Value(1)+1.e-4*p.NbCoeffInU()*p.NbCoeffInV());}
 bool IsSatisfied(const AdvApp2Var_Patch& p) const override{return p.CritValue()<myMaxValue;}
};
void gb_aa_patch(const AdvApp2Var_Patch& p,const AdvApp2Var_Context& c,const GbAaCriterion& crit){
 gb_flag(p.IsDiscretised());gb_flag(p.IsApproximated());gb_flag(p.HasResult());gb_int(p.CutSense());
 gb_int(p.CutSense(crit,3));gb_int(p.NbCoeffInU());gb_int(p.NbCoeffInV());
 if(!p.HasResult())return;
 gb_aa_harray(p.MaxErrors());gb_aa_harray(p.AverageErrors());gb_aa_harray2(p.IsoErrors());
 gb_aa_head(p.Coefficients(1,c),c.ULimit()*c.VLimit()*3);
 occ::handle<NCollection_HArray2<gp_Pnt>> poles=p.Poles(1,c);gb_int(poles->ColLength());gb_int(poles->RowLength());
 for(int i=poles->LowerRow();i<=poles->UpperRow();++i)for(int j=poles->LowerCol();j<=poles->UpperCol();++j)gb_pnt(poles->Value(i,j));
 GB_TRY(p.Coefficients(2,c));
}
// aa_patch: function ifav iu iv ndu ndv iprecis u0 u1 v0 v1 numdec critmax cuts: the isos of
// the grid approximated without cutting, then each patch discretised and approximated.
void gb_aa_patch_case(std::istream& in){
 GbReader r{in};GbAaFunction fn(r.integer());int f=r.integer(),iu=r.integer(),iv=r.integer(),nu=r.integer(),nv=r.integer();
 int pr=r.integer();double d[4];for(double& x:d)x=r.real();int numdec=r.integer();GbAaCriterion crit(r.real(),1,1);
 try{
  AdvApp2Var_Context c(f,iu,iv,nu,nv,pr,0,0,1,nullptr,nullptr,gb_aa_tol(1,3),nullptr,nullptr,gb_aa_tof(1,3));
  GbAaGrid g(iu,iv,d);g.cut(r);AdvApp2Var_Framework& fr=g.frame;AdvApp2Var_Node n1(iu,iv),n2(iu,iv);int i1=0,i2=0;
  for(occ::handle<AdvApp2Var_Iso> iso=fr.FirstNotApprox(i1,i2);!iso.IsNull();iso=fr.FirstNotApprox(i1,i2)){
   const int a=fr.FirstNode(iso->Type(),i1,i2),b=fr.LastNode(iso->Type(),i1,i2);n1=*fr.Node(a);n2=*fr.Node(b);
   iso->MakeApprox(c,d[0],d[1],d[2],d[3],fn,n1,n2);gb_aa_iso(*iso);gb_int(iso->NbCoeff());
   // An iso without result leaves null tables that Discretise dereferences natively.
   if(!iso->HasResult()){gb_int(-9);gb_end();return;}
   gb_aa_harray(iso->Polynom());gb_aa_harray(iso->SomTab());gb_aa_harray(iso->DifTab());gb_aa_harray2(iso->MaxErrors());
   gb_aa_harray2(iso->MoyErrors());iso->OverwriteApprox();fr.ChangeIso(i1,i2,iso);*fr.Node(a)=n1;*fr.Node(b)=n2;
   for(int u=0;u<=iu;++u)for(int v=0;v<=iv;++v){gb_pnt(n1.Point(u,v));gb_real(n1.Error(u,v));gb_pnt(n2.Point(u,v));}
  }
  for(int i=1;i<=g.net.NbPatch();++i){AdvApp2Var_Patch& p=g.net(i);p.Discretise(c,fr,fn);
   if(p.IsDiscretised()){p.MakeApprox(c,fr,numdec);if(p.HasResult()){crit.Value(p,c);gb_real(p.CritValue());}}
   gb_aa_patch(p,c,crit);}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// FNV-1a of a text, to compare the Dump output.
uint64_t gb_aa_fnv(const std::string& s){uint64_t h=1469598103934665603ull;for(unsigned char ch:s){h^=ch;h*=1099511628211ull;}return h;}
// aa_approx: function favor contU contV iprecis maxdegU maxdegV maxpatch u0 u1 v0 v1 cutkind critkind critrep critmax
// (favor: GeomAbs_IsoType; cont: GeomAbs_Shape; cutkind 0 dicho, 1 pref-and-rec; critkind 0 none, 1 absolute, 2 relative).
void gb_aa_approx_case(std::istream& in){
 GbReader r{in};GbAaFunction fn(r.integer());int fav=r.integer(),cu=r.integer(),cv=r.integer(),pr=r.integer();
 int mu=r.integer(),mv=r.integer(),mp=r.integer();double d[4];for(double& x:d)x=r.real();
 int cut=r.integer(),ck=r.integer(),cr=r.integer();double cm=r.real();
 NCollection_Array1<double> rec(1,1),pref(1,2);rec(1)=0.5*(d[0]+d[1]);pref(1)=0.3;pref(2)=0.7;
 AdvApprox_DichoCutting dicho;AdvApprox_PrefAndRec pr_cut(rec,pref,5.);AdvApprox_Cutting& ch=cut==0?(AdvApprox_Cutting&)dicho:pr_cut;
 GbAaCriterion crit(cm,ck==2?1:0,cr);
 try{
  std::unique_ptr<AdvApp2Var_ApproxAFunc2Var> a;
  if(ck==0)a.reset(new AdvApp2Var_ApproxAFunc2Var(0,0,1,nullptr,nullptr,gb_aa_tol(1,3),nullptr,nullptr,gb_aa_tof(1,3),
   d[0],d[1],d[2],d[3],GeomAbs_IsoType(fav),GeomAbs_Shape(cu),GeomAbs_Shape(cv),pr,mu,mv,mp,fn,ch,ch));
  else a.reset(new AdvApp2Var_ApproxAFunc2Var(0,0,1,nullptr,nullptr,gb_aa_tol(1,3),nullptr,nullptr,gb_aa_tof(1,3),
   d[0],d[1],d[2],d[3],GeomAbs_IsoType(fav),GeomAbs_Shape(cu),GeomAbs_Shape(cv),pr,mu,mv,mp,fn,crit,ch,ch));
  gb_flag(a->IsDone());gb_flag(a->HasResult());gb_int(a->UDegree());gb_int(a->VDegree());
  for(int k=1;k<=3;++k)gb_int(a->NumSubSpaces(k));
  gb_real(a->MaxError(3,1));gb_real(a->AverageError(3,1));gb_real(a->UFrontError(3,1));gb_real(a->VFrontError(3,1));
  gb_real(a->CritError(3,1));gb_flag(a->MaxError(1).IsNull());gb_flag(a->VFrontError(2).IsNull());
  GB_TRY(a->MaxError(4));GB_TRY(a->AverageError(0));GB_TRY(a->UFrontError(3,2));GB_TRY(a->CritError(1,1));
  occ::handle<Geom_BSplineSurface> s=a->Surface(1);gb_int(s->UDegree());gb_int(s->VDegree());
  gb_int(s->NbUKnots());gb_int(s->NbVKnots());
  for(int i=1;i<=s->NbUKnots();++i){gb_real(s->UKnot(i));gb_int(s->UMultiplicity(i));}
  for(int i=1;i<=s->NbVKnots();++i){gb_real(s->VKnot(i));gb_int(s->VMultiplicity(i));}
  gb_int(s->NbUPoles());gb_int(s->NbVPoles());
  for(int i=1;i<=s->NbUPoles();++i)for(int j=1;j<=s->NbVPoles();++j)gb_pnt(s->Pole(i,j));
  std::ostringstream o;a->Dump(o);std::cout<<gb_aa_fnv(o.str())<<' ';
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}

// ShapeExtend_CompositeSurface probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/composite_surface.rs.
//
// `sh_composite program nu nv (surface)... ctor ... npts (u v)... prec trsf ... newjoints`:
// the patches are surfaces of the shape program's table, rows (U) first. ctor 0 `param`
// and 2 `param` build the grid with ShapeExtend_CompositeSurface(grid, param) or Init;
// ctor 1 and 3 `ulower nu (u)... vlower nv (v)...` with the joint values. Prints the
// constant members, the joints and bounds, then for each global point: the located cell,
// the patch (as its index in the surface table), the local/global mappings, the
// global-to-local transformation (applied to the point), the evaluators and Value(pnt).
// Then SetUJointValues/SetVJointValues with `newjoints` (`ulower nu (u)... vlower nv (v)...`),
// SetUFirstValue/SetVFirstValue, CheckConnectivity(prec), Copy() and Transform(trsf)
// (trsf as in tkshhealing_probe_complexcurve.cxx), each followed by the points again.
#include <ShapeExtend_CompositeSurface.hxx>
#include <gp_Trsf2d.hxx>
typedef NCollection_HArray2<occ::handle<Geom_Surface>> ShGrid;
NCollection_Array1<double> sh_cs_joints(GaReader& r){
 int lower=r.integer();int n=r.integer();NCollection_Array1<double> a(lower,lower+n-1);
 for(int i=lower;i<lower+n;++i)a(i)=r.real();return a;}
int sh_cs_surface_index(const TpShapes& t,const occ::handle<Geom_Surface>& s){
 for(size_t i=0;i<t.surfaces.size();++i)if(t.surfaces[i]==s)return int(i);return s.IsNull()?-1:-2;}
void sh_cs_array(const occ::handle<NCollection_HArray1<double>>& a){
 if(a.IsNull()){ga_int(-1);return;}ga_int(a->Lower());ga_int(a->Length());for(int i=a->Lower();i<=a->Upper();++i)ga_real(a->Value(i));}
void sh_cs_state(const occ::handle<ShapeExtend_CompositeSurface>& cs){
 ga_int(cs->NbUPatches());ga_int(cs->NbVPatches());sh_cs_array(cs->UJointValues());sh_cs_array(cs->VJointValues());
 double u1,u2,v1,v2;cs->Bounds(u1,u2,v1,v2);ga_real(u1);ga_real(u2);ga_real(v1);ga_real(v2);
 ga_flag(cs->IsUClosed());ga_flag(cs->IsVClosed());ga_flag(cs->IsUPeriodic());ga_flag(cs->IsVPeriodic());
 ga_int(cs->Continuity());ga_flag(cs->IsCNu(0));ga_flag(cs->IsCNu(1));ga_flag(cs->IsCNv(-1));ga_flag(cs->IsCNv(2));
 ga_real(cs->UReversedParameter(0.3));ga_real(cs->VReversedParameter(-0.7));
 ga_flag(cs->UIso(0.5).IsNull());ga_flag(cs->VIso(0.5).IsNull());
 GA_TRY(ga_real(cs->UJointValue(0)))GA_TRY(ga_real(cs->VJointValue(cs->NbVPatches()+2)))
 GA_TRY(ga_flag(cs->Patch(0,1).IsNull()))}
void sh_cs_vec(const gp_Vec& v){ga_xyz(v.XYZ());}
void sh_cs_cell(const occ::handle<ShapeExtend_CompositeSurface>& cs,int i,int j,const gp_Pnt2d& uv){
 GA_TRY(gp_Pnt2d l=cs->GlobalToLocal(i,j,uv);ga_pnt2d(l);ga_pnt2d(cs->LocalToGlobal(i,j,l));
  ga_real(cs->UGlobalToLocal(i,j,uv.X()));ga_real(cs->VGlobalToLocal(i,j,uv.Y()));
  ga_real(cs->ULocalToGlobal(i,j,l.X()));ga_real(cs->VLocalToGlobal(i,j,l.Y())))
 GA_TRY(double uf=0.;gp_Trsf2d T;ga_flag(cs->GlobalToLocalTransformation(i,j,uf,T));ga_real(uf);ga_int(T.Form());
  gp_XY p=uv.XY();T.Transforms(p);ga_xy(p))}
void sh_cs_point(const TpShapes& t,const occ::handle<ShapeExtend_CompositeSurface>& cs,double u,double v){
 int i=0,j=0;GA_TRY(cs->LocateUVPoint(gp_Pnt2d(u,v),i,j);ga_int(i);ga_int(j))
 GA_TRY(ga_int(sh_cs_surface_index(t,cs->Patch(u,v))))
 sh_cs_cell(cs,i,j,gp_Pnt2d(u,v));
 GA_TRY(ga_pnt(cs->Geom_Surface::Value(u,v)))
 GA_TRY(Geom_Surface::ResD1 d=cs->EvalD1(u,v);ga_pnt(d.Point);sh_cs_vec(d.D1U);sh_cs_vec(d.D1V))
 GA_TRY(Geom_Surface::ResD2 d=cs->EvalD2(u,v);sh_cs_vec(d.D2U);sh_cs_vec(d.D2V);sh_cs_vec(d.D2UV))
 GA_TRY(Geom_Surface::ResD3 d=cs->EvalD3(u,v);sh_cs_vec(d.D3U);sh_cs_vec(d.D3V);sh_cs_vec(d.D3UUV);sh_cs_vec(d.D3UVV))
 GA_TRY(sh_cs_vec(cs->EvalDN(u,v,1,1)))GA_TRY(sh_cs_vec(cs->EvalDN(u,v,0,0)))
 GA_TRY(ga_pnt(cs->Value(gp_Pnt2d(u,v))))}
void sh_cs_points(const TpShapes& t,const occ::handle<ShapeExtend_CompositeSurface>& cs,const std::vector<double>& pts){
 for(size_t k=0;k+1<pts.size();k+=2)sh_cs_point(t,cs,pts[k],pts[k+1]);}
occ::handle<ShapeExtend_CompositeSurface> sh_cs_build(GaReader& r,const occ::handle<ShGrid>& grid){
 int ctor=r.integer();occ::handle<ShapeExtend_CompositeSurface> cs;
 if(ctor==0)cs=new ShapeExtend_CompositeSurface(grid,ShapeExtend_Parametrisation(r.integer()));
 else if(ctor==2){cs=new ShapeExtend_CompositeSurface();ga_flag(cs->Init(grid,ShapeExtend_Parametrisation(r.integer())));}
 else{NCollection_Array1<double> uj=sh_cs_joints(r),vj=sh_cs_joints(r);
  if(ctor==1)cs=new ShapeExtend_CompositeSurface(grid,uj,vj);
  else{cs=new ShapeExtend_CompositeSurface();ga_flag(cs->Init(grid,uj,vj));}}
 return cs;}
void sh_composite_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);int nu=r.integer(),nv=r.integer();
 occ::handle<ShGrid> grid=new ShGrid(1,nu,1,nv);
 for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)grid->SetValue(i,j,t.surfaces.at(r.integer()));
 occ::handle<ShapeExtend_CompositeSurface> cs=sh_cs_build(r,grid);
 std::vector<double> pts=r.reals();double prec=r.real();gp_Trsf T=sh_cc_trsf(r);
 NCollection_Array1<double> uj=sh_cs_joints(r),vj=sh_cs_joints(r);
 sh_cs_state(cs);sh_cs_points(t,cs,pts);
 ga_flag(cs->SetUJointValues(uj));ga_flag(cs->SetVJointValues(vj));sh_cs_state(cs);
 cs->SetUFirstValue(-1.5);cs->SetVFirstValue(2.);sh_cs_array(cs->UJointValues());sh_cs_points(t,cs,pts);
 ga_flag(cs->CheckConnectivity(prec));ga_flag(cs->IsUClosed());ga_flag(cs->IsVClosed());
 occ::handle<ShapeExtend_CompositeSurface> copy=occ::down_cast<ShapeExtend_CompositeSurface>(cs->Copy());
 sh_cs_state(copy);sh_cs_points(t,copy,pts);
 cs->Transform(T);sh_cs_points(t,cs,pts);sh_cs_points(t,copy,pts);
 occ::handle<ShapeExtend_CompositeSurface> empty=new ShapeExtend_CompositeSurface();
 ga_flag(empty->Patches().IsNull());ga_flag(empty->UJointValues().IsNull());
 ga_flag(occ::down_cast<ShapeExtend_CompositeSurface>(empty->Copy())->Patches().IsNull());
 empty->Transform(T);empty->SetUFirstValue(1.);ga_flag(empty->Init(occ::handle<ShGrid>(),ShapeExtend_Natural));
 ga_end();}

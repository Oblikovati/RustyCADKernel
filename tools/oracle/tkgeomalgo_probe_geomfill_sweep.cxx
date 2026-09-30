// GeomFill sweep probe operations: the GeomFill class, the arc convertors and the
// tensor (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom_fill_sweep.rs. The trihedron, location,
// section and sweep operations follow in tkgeomalgo_probe_geomfill_*.cxx.
#include <GeomFill.hxx>
#include <GeomFill_PolynomialConvertor.hxx>
#include <GeomFill_QuasiAngularConvertor.hxx>
#include <GeomFill_Tensor.hxx>
#include <Convert_ParameterisationType.hxx>
#include <math_Matrix.hxx>
#include <math_Vector.hxx>
void ga_gfs_pnts(const NCollection_Array1<gp_Pnt>& a){for(int i=a.Lower();i<=a.Upper();++i)ga_pnt(a(i));}
void ga_gfs_vecs(const NCollection_Array1<gp_Vec>& a){for(int i=a.Lower();i<=a.Upper();++i)ga_xyz(a(i).XYZ());}
void ga_gfs_reals(const NCollection_Array1<double>& a){for(int i=a.Lower();i<=a.Upper();++i)ga_real(a(i));}
gp_Vec ga_gfs_vec(GaReader& r){return gp_Vec(r.xyz());}
// `max_ang conv lower n min_ang radius ang_tol sp_tol`: GetShape, Knots and Mults on an
// array `lower..lower+n-1`, GetMinimalWeights on `1..NbPoles` and GetTolerance.
void ga_gfs_shape_case(std::istream& input){
 GaReader r{input};double max_ang=r.real();Convert_ParameterisationType conv=Convert_ParameterisationType(r.integer());
 int lower=r.integer(),n=r.integer();double min_ang=r.real(),radius=r.real(),ang_tol=r.real(),sp_tol=r.real();
 int np=0,nk=0,deg=0;GeomFill::GetShape(max_ang,np,nk,deg,conv);ga_int(np);ga_int(nk);ga_int(deg);ga_int(int(conv));
 NCollection_Array1<double> k(lower,lower+n-1);k.Init(-1);GA_TRY(GeomFill::Knots(conv,k));ga_gfs_reals(k);
 NCollection_Array1<int> m(lower,lower+n-1);m.Init(-1);GA_TRY(GeomFill::Mults(conv,m));for(int i=m.Lower();i<=m.Upper();++i)ga_int(m(i));
 NCollection_Array1<double> w(1,np>0?np:1);w.Init(-1);GA_TRY(GeomFill::GetMinimalWeights(conv,min_ang,max_ang,w);ga_int(w.Lower());ga_int(w.Upper());ga_gfs_reals(w));
 GA_TRY(double t=GeomFill::GetTolerance(conv,min_ang,radius,ang_tol,sp_tol);ga_real(t));
 ga_end();
}
// `conv n lower` then the circle data: D0, D1 and D2 GetCircle on arrays
// `lower..lower+n-1`, each printed after the call (or its exception).
void ga_gfs_circle_case(std::istream& input){
 GaReader r{input};Convert_ParameterisationType conv=Convert_ParameterisationType(r.integer());int n=r.integer(),lower=r.integer();
 gp_Vec ns1=ga_gfs_vec(r),ns2=ga_gfs_vec(r),nplan=ga_gfs_vec(r);gp_Pnt pts1=r.pnt(),pts2=r.pnt();double rayon=r.real();gp_Pnt center=r.pnt();
 gp_Vec dn1w=ga_gfs_vec(r),dn2w=ga_gfs_vec(r),dnplan=ga_gfs_vec(r),tang1=ga_gfs_vec(r),tang2=ga_gfs_vec(r);double drayon=r.real();gp_Vec dcenter=ga_gfs_vec(r);
 gp_Vec d2n1w=ga_gfs_vec(r),d2n2w=ga_gfs_vec(r),d2nplan=ga_gfs_vec(r),dtang1=ga_gfs_vec(r),dtang2=ga_gfs_vec(r);double d2rayon=r.real();gp_Vec d2center=ga_gfs_vec(r);
 int u=lower+n-1;
 {NCollection_Array1<gp_Pnt> p(lower,u);NCollection_Array1<double> w(lower,u);p.Init(gp_Pnt(-1,-1,-1));w.Init(-1);
  GA_TRY(GeomFill::GetCircle(conv,ns1,ns2,nplan,pts1,pts2,rayon,center,p,w));ga_gfs_pnts(p);ga_gfs_reals(w);}
 {NCollection_Array1<gp_Pnt> p(lower,u);NCollection_Array1<gp_Vec> dp(lower,u);NCollection_Array1<double> w(lower,u),dw(lower,u);
  p.Init(gp_Pnt(-1,-1,-1));dp.Init(gp_Vec(-1,-1,-1));w.Init(-1);dw.Init(-1);
  GA_TRY(bool ok=GeomFill::GetCircle(conv,ns1,ns2,dn1w,dn2w,nplan,dnplan,pts1,pts2,tang1,tang2,rayon,drayon,center,dcenter,p,dp,w,dw);ga_flag(ok));
  ga_gfs_pnts(p);ga_gfs_vecs(dp);ga_gfs_reals(w);ga_gfs_reals(dw);}
 {NCollection_Array1<gp_Pnt> p(lower,u);NCollection_Array1<gp_Vec> dp(lower,u),d2p(lower,u);NCollection_Array1<double> w(lower,u),dw(lower,u),d2w(lower,u);
  p.Init(gp_Pnt(-1,-1,-1));dp.Init(gp_Vec(-1,-1,-1));d2p.Init(gp_Vec(-1,-1,-1));w.Init(-1);dw.Init(-1);d2w.Init(-1);
  GA_TRY(bool ok=GeomFill::GetCircle(conv,ns1,ns2,dn1w,dn2w,d2n1w,d2n2w,nplan,dnplan,d2nplan,pts1,pts2,tang1,tang2,dtang1,dtang2,
   rayon,drayon,d2rayon,center,dcenter,d2center,p,dp,d2p,w,dw,d2w);ga_flag(ok));
  ga_gfs_pnts(p);ga_gfs_vecs(dp);ga_gfs_vecs(d2p);ga_gfs_reals(w);ga_gfs_reals(dw);ga_gfs_reals(d2w);}
 ga_end();
}
// `kind(0 polynomial, 1 quasi-angular) init n lower` then the arc and its two
// derivatives: Initialized, then the three Section members on arrays
// `lower..lower+n-1`.
void ga_gfs_convert_case(std::istream& input){
 GaReader r{input};int kind=r.integer();bool init=r.flag();int n=r.integer(),lower=r.integer(),u=lower+n-1;
 gp_Pnt fp=r.pnt();gp_Vec dfp=ga_gfs_vec(r),d2fp=ga_gfs_vec(r);gp_Pnt c=r.pnt();gp_Vec dc=ga_gfs_vec(r),d2c=ga_gfs_vec(r);
 gp_Vec dir=ga_gfs_vec(r),ddir=ga_gfs_vec(r),d2dir=ga_gfs_vec(r);double a=r.real(),da=r.real(),d2a=r.real();
 NCollection_Array1<gp_Pnt> p(lower,u);NCollection_Array1<gp_Vec> dp(lower,u),d2p(lower,u);NCollection_Array1<double> w(lower,u),dw(lower,u),d2w(lower,u);
 auto reset=[&]{p.Init(gp_Pnt(-1,-1,-1));dp.Init(gp_Vec(-1,-1,-1));d2p.Init(gp_Vec(-1,-1,-1));w.Init(-1);dw.Init(-1);d2w.Init(-1);};
 if(kind==0){GeomFill_PolynomialConvertor pc;ga_flag(pc.Initialized());if(init)pc.Init();ga_flag(pc.Initialized());
  reset();GA_TRY(pc.Section(fp,c,dir,a,p));ga_gfs_pnts(p);
  reset();GA_TRY(pc.Section(fp,dfp,c,dc,dir,ddir,a,da,p,dp));ga_gfs_pnts(p);ga_gfs_vecs(dp);
  reset();GA_TRY(pc.Section(fp,dfp,d2fp,c,dc,d2c,dir,ddir,d2dir,a,da,d2a,p,dp,d2p));ga_gfs_pnts(p);ga_gfs_vecs(dp);ga_gfs_vecs(d2p);}
 else{GeomFill_QuasiAngularConvertor qc;ga_flag(qc.Initialized());if(init)qc.Init();ga_flag(qc.Initialized());
  reset();GA_TRY(qc.Section(fp,c,dir,a,p,w));ga_gfs_pnts(p);ga_gfs_reals(w);
  reset();GA_TRY(qc.Section(fp,dfp,c,dc,dir,ddir,a,da,p,dp,w,dw));ga_gfs_pnts(p);ga_gfs_vecs(dp);ga_gfs_reals(w);ga_gfs_reals(dw);
  reset();GA_TRY(qc.Section(fp,dfp,d2fp,c,dc,d2c,dir,ddir,d2dir,a,da,d2a,p,dp,d2p,w,dw,d2w));
  ga_gfs_pnts(p);ga_gfs_vecs(dp);ga_gfs_vecs(d2p);ga_gfs_reals(w);ga_gfs_reals(dw);ga_gfs_reals(d2w);}
 ga_end();
}
// `nrow ncol nmat init n (row col mat value)... nvec v...`: builds a tensor, sets
// entries, reads them back (with out-of-range reads) and multiplies by a vector.
void ga_gfs_tensor_case(std::istream& input){
 GaReader r{input};int nr=r.integer(),nc=r.integer(),nm=r.integer();double init=r.real();
 GeomFill_Tensor t(nr,nc,nm);t.Init(init);int n=r.integer();
 for(int k=0;k<n;++k){int i=r.integer(),j=r.integer(),m=r.integer();double v=r.real();GA_TRY(t.ChangeValue(i,j,m)=v);}
 for(int i=0;i<=nr+1;++i)for(int j=1;j<=nc;++j)for(int m=1;m<=nm;++m)GA_TRY(ga_real(t.Value(i,j,m)));
 std::vector<double> v=r.reals();math_Vector right(1,int(v.size()));for(int k=1;k<=right.Length();++k)right(k)=v[k-1];
 math_Matrix prod(1,nr,1,nc,0.0);GA_TRY(t.Multiply(right,prod);for(int i=1;i<=nr;++i)for(int j=1;j<=nc;++j)ga_real(prod(i,j)));
 ga_end();
}

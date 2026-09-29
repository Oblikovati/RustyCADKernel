// TKGeomBase oracle: the AdvApp2Var_MathBase kernels, part 1 (tkgeombase_probe.cxx).
// Every request gives the sizes, then the input arrays in the Fortran layout; outputs
// start filled with 7.5 so untouched entries compare too. The Rust twin is
// crates/tkgeombase/examples/gb_probe/adv_math.rs.
#include <AdvApp2Var_MathBase.hxx>
#include <AdvApp2Var_ApproxF2var.hxx>
#include <AdvApp2Var_SysBase.hxx>
#include <algorithm>
#include <cmath>
std::vector<double> gb_aa_reals(GbReader& r,int n){std::vector<double> v(std::max(n,0));for(double& x:v)x=r.real();return v;}
std::vector<double> gb_aa_fill(int n){return std::vector<double>(std::max(n,1),7.5);}
void gb_aa_print(const std::vector<double>& v){for(double x:v)gb_real(x);}
// aa_norms: n v[n] w[n] x k -> mzsnorm_, msc_, pow__di, mmeps1_, mmveps3_.
void gb_aa_norms_case(std::istream& in){
 GbReader r{in};int n=r.integer();auto v=gb_aa_reals(r,n),w=gb_aa_reals(r,n);double x=r.real();int k=r.integer();
 gb_real(AdvApp2Var_MathBase::mzsnorm_(&n,v.data()));gb_real(AdvApp2Var_MathBase::msc_(&n,v.data(),w.data()));
 gb_real(AdvApp2Var_MathBase::pow__di(&x,&k));
 double e1=0,e3=0;AdvApp2Var_MathBase::mmeps1_(&e1);AdvApp2Var_MathBase::mmveps3_(&e3);gb_real(e1);gb_real(e3);gb_end();
}
// aa_mmwprcs: eps1 eps2 eps3 eps4 niter1 niter2 -> the new mmeps1_, mmveps3_.
void gb_aa_mmwprcs_case(std::istream& in){
 GbReader r{in};double e[4];for(double& x:e)x=r.real();int n1=r.integer(),n2=r.integer();
 AdvApp2Var_MathBase::mmwprcs_(&e[0],&e[1],&e[2],&e[3],&n1,&n2);
 double e1=0,e3=0;AdvApp2Var_MathBase::mmeps1_(&e1);AdvApp2Var_MathBase::mmveps3_(&e3);gb_real(e1);gb_real(e3);gb_end();
}
// aa_mmapcmp: ndim ncofmx ncoeff crvold[ncofmx*ndim].
void gb_aa_mmapcmp_case(std::istream& in){
 GbReader r{in};int ndim=r.integer(),ncofmx=r.integer(),ncoeff=r.integer();auto old=gb_aa_reals(r,ncofmx*ndim);
 auto out=gb_aa_fill(((ncoeff-1)/2+1)*2*ndim);AdvApp2Var_MathBase::mmapcmp_(&ndim,&ncofmx,&ncoeff,old.data(),out.data());
 gb_aa_print(out);gb_end();
}
// aa_mmdrc11: iordre ndimen ncoeff courbe[ncoeff*ndimen].
void gb_aa_mmdrc11_case(std::istream& in){
 GbReader r{in};int iordre=r.integer(),ndimen=r.integer(),ncoeff=r.integer();auto c=gb_aa_reals(r,ncoeff*ndimen);
 auto pts=gb_aa_fill(2*(iordre+1)*ndimen),fact=gb_aa_fill(iordre+1);
 AdvApp2Var_MathBase::mmdrc11_(&iordre,&ndimen,&ncoeff,c.data(),pts.data(),fact.data());gb_aa_print(pts);gb_aa_print(fact);gb_end();
}
// aa_mmfmca8: ndimen ncoefu ncoefv ndimax ncfumx inplace tabini[ndimen*ncoefu*ncoefv].
void gb_aa_mmfmca8_case(std::istream& in){
 GbReader r{in};int nd=r.integer(),nu=r.integer(),nv=r.integer(),ndx=r.integer(),nux=r.integer();bool inplace=r.flag();
 auto ini=gb_aa_reals(r,nd*nu*nv);auto res=gb_aa_fill(ndx*nux*nv);
 if(inplace){std::copy(ini.begin(),ini.end(),res.begin());AdvApp2Var_MathBase::mmfmca8_(&nd,&nu,&nv,&ndx,&nux,&nv,res.data(),res.data());}
 else AdvApp2Var_MathBase::mmfmca8_(&nd,&nu,&nv,&ndx,&nux,&nv,ini.data(),res.data());
 gb_aa_print(res);gb_end();
}
// aa_mmfmca9: ndimax ncfumx ndimen ncoefu ncoefv inplace tabini[ndimax*ncfumx*ncoefv].
void gb_aa_mmfmca9_case(std::istream& in){
 GbReader r{in};int ndx=r.integer(),nux=r.integer(),nd=r.integer(),nu=r.integer(),nv=r.integer();bool inplace=r.flag();
 auto ini=gb_aa_reals(r,ndx*nux*nv);
 if(inplace){AdvApp2Var_MathBase::mmfmca9_(&ndx,&nux,&nv,&nd,&nu,&nv,ini.data(),ini.data());gb_aa_print(ini);}
 else{auto res=gb_aa_fill(nd*nu*nv);AdvApp2Var_MathBase::mmfmca9_(&ndx,&nux,&nv,&nd,&nu,&nv,ini.data(),res.data());gb_aa_print(res);}
 gb_end();
}
// aa_mmfmcb5: isenmsc ndimax ncf1mx ncoeff ncf2mx ndimen courb1[ndimax*ncf1mx] courb2[ncf2mx*ndimen].
void gb_aa_mmfmcb5_case(std::istream& in){
 GbReader r{in};int is=r.integer(),ndx=r.integer(),n1=r.integer(),nc=r.integer(),n2=r.integer(),nd=r.integer();
 auto c1=gb_aa_reals(r,ndx*n1),c2=gb_aa_reals(r,n2*nd);int ier=0;
 AdvApp2Var_MathBase::mmfmcb5_(&is,&ndx,&n1,c1.data(),&nc,&n2,&nd,c2.data(),&ier);gb_int(ier);gb_aa_print(c1);gb_aa_print(c2);gb_end();
}
// aa_mmfmtb1: maxsz1 isize1 jsize1 maxsz2 table1[maxsz1*jsize1].
void gb_aa_mmfmtb1_case(std::istream& in){
 GbReader r{in};int m1=r.integer(),i1=r.integer(),j1=r.integer(),m2=r.integer();auto t1=gb_aa_reals(r,m1*j1);
 auto t2=gb_aa_fill(m2*i1);int i2=-1,j2=-1,ier=-1;
 AdvApp2Var_MathBase::mmfmtb1_(&m1,t1.data(),&i1,&j1,&m2,t2.data(),&i2,&j2,&ier);gb_int(i2);gb_int(j2);gb_int(ier);gb_aa_print(t2);gb_end();
}
// aa_mmfmcar: ndimen ncofmx ncoefu ncoefv u1 u2 v1 v2 patold[ndimen*ncofmx*ncoefv].
void gb_aa_mmfmcar_case(std::istream& in){
 GbReader r{in};int nd=r.integer(),nx=r.integer(),nu=r.integer(),nv=r.integer();double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
 auto old=gb_aa_reals(r,nd*nx*nv);auto neu=gb_aa_fill(nd*nx*nv);int ier=-1;
 AdvApp2Var_MathBase::mmfmcar_(&nd,&nx,&nu,&nv,old.data(),&u1,&u2,&v1,&v2,neu.data(),&ier);gb_int(ier);gb_aa_print(neu);gb_end();
}
// aa_mmcglc1: ndimax ndimen ncoeff t0 t1 eps courbe[ndimax*ncoeff].
void gb_aa_mmcglc1_case(std::istream& in){
 GbReader r{in};int ndx=r.integer(),nd=r.integer(),nc=r.integer();double t0=r.real(),t1=r.real(),eps=r.real();
 auto c=gb_aa_reals(r,ndx*nc);double len=7.5,err=7.5;int ier=-1;
 AdvApp2Var_MathBase::mmcglc1_(&ndx,&nd,&nc,c.data(),&t0,&t1,&eps,&len,&err,&ier);gb_real(len);gb_real(err);gb_int(ier);gb_end();
}
// aa_mmbulld: nbcoln nblign numcle table[nblign*nbcoln].
void gb_aa_mmbulld_case(std::istream& in){
 GbReader r{in};int nc=r.integer(),nl=r.integer(),key=r.integer();auto t=gb_aa_reals(r,nc*nl);
 AdvApp2Var_MathBase::mmbulld_(&nc,&nl,t.data(),&key);gb_aa_print(t);gb_end();
}
// aa_mvsheld: n is icle table[is*n].
void gb_aa_mvsheld_case(std::istream& in){
 GbReader r{in};int n=r.integer(),is=r.integer(),key=r.integer();auto t=gb_aa_reals(r,n*is);
 AdvApp2Var_MathBase::mvsheld_(&n,&is,t.data(),&key);gb_aa_print(t);gb_end();
}
// aa_mmcdriv: ndimen ncoeff ideriv courbe[ndimen*ncoeff].
void gb_aa_mmcdriv_case(std::istream& in){
 GbReader r{in};int nd=r.integer(),nc=r.integer(),id=r.integer();auto c=gb_aa_reals(r,nd*nc);auto d=gb_aa_fill(nd*nc);int ncf=-1;
 AdvApp2Var_MathBase::mmcdriv_(&nd,&nc,c.data(),&id,&ncf,d.data());gb_int(ncf);gb_aa_print(d);gb_end();
}
// aa_mmdrvck: ncoeff ndimen ideriv t courbe[ndimen*ncoeff].
void gb_aa_mmdrvck_case(std::istream& in){
 GbReader r{in};int nc=r.integer(),nd=r.integer(),id=r.integer();double t=r.real();auto c=gb_aa_reals(r,nd*nc);auto p=gb_aa_fill(nd);
 AdvApp2Var_MathBase::mmdrvck_(&nc,&nd,c.data(),&id,&t,p.data());gb_aa_print(p);gb_end();
}
// aa_mmcvctx: ndimen ncofmx nderiv ctrtes[ndimen*2*(nderiv+1)].
void gb_aa_mmcvctx_case(std::istream& in){
 GbReader r{in};int nd=r.integer(),nx=r.integer(),nv=r.integer();auto c=gb_aa_reals(r,nd*2*(nv+1));
 auto res=gb_aa_fill(nx*nd),aux=gb_aa_fill((nv+1+nd)*(nv+1)),x=gb_aa_fill((nv+1)*nd);int ier=-1;
 AdvApp2Var_MathBase::mmcvctx_(&nd,&nx,&nv,c.data(),res.data(),aux.data(),x.data(),&ier);gb_int(ier);gb_aa_print(res);gb_end();
}
// aa_mdsptpt: n p1[n] p2[n] (n above 100: smaller sizes dereference null natively).
void gb_aa_mdsptpt_case(std::istream& in){
 GbReader r{in};int n=r.integer();auto p1=gb_aa_reals(r,n),p2=gb_aa_reals(r,n);double d=7.5;
 AdvApp2Var_MathBase::mdsptpt_(&n,p1.data(),p2.data(),&d);gb_real(d);gb_end();
}
// aa_mmaperx: ncofmx ndimen ncoeff iordre ncfnew crvjac[ncofmx*ndimen].
void gb_aa_mmaperx_case(std::istream& in){
 GbReader r{in};int nx=r.integer(),nd=r.integer(),nc=r.integer(),io=r.integer(),nn=r.integer();auto c=gb_aa_reals(r,nx*nd);
 auto y=gb_aa_fill(nd);double e=7.5;int ier=-1;
 AdvApp2Var_MathBase::mmaperx_(&nx,&nd,&nc,&io,c.data(),&nn,y.data(),&e,&ier);gb_aa_print(y);gb_real(e);gb_int(ier);gb_end();
}
// aa_mmtrpjj: ncofmx ndimen ncoeff eps iordre crvlgd[ncofmx*ndimen].
void gb_aa_mmtrpjj_case(std::istream& in){
 GbReader r{in};int nx=r.integer(),nd=r.integer(),nc=r.integer();double eps=r.real();int io=r.integer();auto c=gb_aa_reals(r,nx*nd);
 auto y=gb_aa_fill(nd);double e=7.5;int nn=-1;
 AdvApp2Var_MathBase::mmtrpjj_(&nx,&nd,&nc,&eps,&io,c.data(),y.data(),&e,&nn);gb_aa_print(y);gb_real(e);gb_int(nn);gb_end();
}
// The integrands of aa_mmgaus1: 0 clears iercod, 1 leaves it, 2 fails beyond 0.5.
int gb_aa_integrand_kind=0;
int gb_aa_integrand(int* n,double* x,double* f,int* ier){
 const double t=*x;const double v[3]={t*t*t-2.*t,std::sin(3.*t),std::exp(-t)};
 for(int i=0;i<*n;++i)f[i]=v[i%3]+i;
 if(gb_aa_integrand_kind==0)*ier=0;
 if(gb_aa_integrand_kind==2)*ier=t>0.5?5:0;
 return 0;
}
// aa_mmgaus1: ndimf k xd xf kind.
void gb_aa_mmgaus1_case(std::istream& in){
 GbReader r{in};int nf=r.integer(),k=r.integer();double xd=r.real(),xf=r.real();gb_aa_integrand_kind=r.integer();
 auto s1=gb_aa_fill(nf),s2=gb_aa_fill(nf),s=gb_aa_fill(nf);int niter=-1,ier=-7;
 AdvApp2Var_MathBase::mmgaus1_(&nf,gb_aa_integrand,&k,&xd,&xf,s1.data(),s2.data(),s.data(),&niter,&ier);
 gb_aa_print(s);gb_int(niter);gb_int(ier);gb_end();
}

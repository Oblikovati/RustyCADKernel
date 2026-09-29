// TKGeomBase oracle: the AdvApp2Var_ApproxF2var patch and iso kernels (tkgeombase_probe.cxx),
// with the helpers of tkgeombase_probe_advapp2var_f2var.cxx. The Rust twin is
// crates/tkgeombase/examples/gb_probe/adv_f2var2.rs.
// aa_mma2ce1: function numdec iordru iordrv nbpntu nbpntv ndjacu ndjacv ndguli ndgvli ndminu ndminv
// eps u0 u1 v0 v1: the square sampled by mma2ds1_ (isofav 2), then approximated (ndimen 3).
void gb_aa_mma2ce1_case(std::istream& in){
 GbReader r{in};GbAaFunction f(r.integer());int nd=3,ns=1,ndse=3,numdec=r.integer(),iu=r.integer(),iv=r.integer();
 int nu=r.integer(),nv=r.integer(),ju=r.integer(),jv=r.integer(),gu=r.integer(),gv=r.integer(),mu=r.integer(),mv=r.integer();
 double eps=r.real(),uv[4];for(double& x:uv)x=r.real();
 auto ur=gb_aa_fill(nu),vr=gb_aa_fill(nv);AdvApp2Var_ApproxF2var::mma2roo_(&nu,&nv,ur.data(),vr.data());
 GbAaSquare t(nu,nv,nd,0.);auto fp=gb_aa_fill(nd*std::max(nu,nv)),tt=gb_aa_fill(std::max(nu,nv));int ier=-1,iso=2;
 AdvApp2Var_ApproxF2var::mma2ds1_(&nd,uv,uv+2,f,&nu,&nv,ur.data(),vr.data(),&iso,t.soso.data(),t.diso.data(),t.sodi.data(),
  t.didi.data(),fp.data(),tt.data(),&ier);
 auto pj=gb_aa_fill((ju+1)*(jv+1)*nd);double emax=7.5,emoy=7.5;int du=-1,dv=-1,ity=-1;ier=-9;
 AdvApp2Var_ApproxF2var::mma2ce1_(&numdec,&nd,&ns,&ndse,&mu,&mv,&gu,&gv,&ju,&jv,&iu,&iv,&nu,&nv,&eps,t.soso.data(),
  t.diso.data(),t.sodi.data(),t.didi.data(),pj.data(),&emax,&emoy,&du,&dv,&ity,&ier);
 gb_int(ier);gb_int(ity);gb_int(du);gb_int(dv);gb_real(emax);gb_real(emoy);gb_aa_print(pj);gb_end();
}
// aa_mma2fnc: function isofav tconst u0 u1 v0 v1 iordre ideriv nbroot ndgjac ncflim nbcrmx eps (ndimen 3).
void gb_aa_mma2fnc_case(std::istream& in){
 GbReader r{in};GbAaFunction f(r.integer());int iso=r.integer();double tc=r.real(),uv[4];for(double& x:uv)x=r.real();
 int io=r.integer(),id=r.integer(),nb=r.integer(),ndg=r.integer(),ncf=r.integer(),mx=r.integer();double eps=r.real();
 int nd=3,ns=1,ndse=3,nbe=-1,ier=-9;auto roots=gb_aa_fill(nb),other=gb_aa_fill(nb);
 AdvApp2Var_ApproxF2var::mma2roo_(&nb,&nb,roots.data(),other.data());
 std::vector<int> ncoef(mx,-1);auto crb=gb_aa_fill(ncf*nd*mx),som=gb_aa_fill((nb/2+1)*nd*mx),dif=gb_aa_fill((nb/2+1)*nd*mx);
 auto c1=gb_aa_fill(nd*(io+2)*mx),c2=gb_aa_fill(nd*(io+2)*mx),tab=gb_aa_fill(mx+1),emax=gb_aa_fill(mx),emoy=gb_aa_fill(mx);
 AdvApp2Var_ApproxF2var::mma2fnc_(&nd,&ns,&ndse,uv,f,&tc,&iso,&nb,roots.data(),&io,&id,&ndg,&mx,&ncf,&eps,ncoef.data(),crb.data(),
  &nbe,som.data(),dif.data(),c1.data(),c2.data(),tab.data(),emax.data(),emoy.data(),&ier);
 gb_int(ier);gb_int(nbe);for(int n:ncoef)gb_int(n);gb_aa_print(tab);gb_aa_print(emax);gb_aa_print(emoy);gb_aa_print(crb);
 gb_aa_print(som);gb_aa_print(dif);gb_aa_print(c1);gb_aa_print(c2);gb_end();
}
// aa_mma2fx6: ncfmxu ncfmxv ndimen nbupat nbvpat iordru iordrv epsapr epsfro[8] ncoefu[] ncoefv[] errmax[] patcan (one subspace).
void gb_aa_mma2fx6_case(std::istream& in){
 GbReader r{in};int mu=r.integer(),mv=r.integer(),nd=r.integer(),pu=r.integer(),pv=r.integer(),iu=r.integer(),iv=r.integer();
 int ns=1,ndse=nd;double eps=r.real();auto fro=gb_aa_reals(r,8);std::vector<int> cu(pu*pv),cv(pu*pv);
 for(int& n:cu)n=r.integer();for(int& n:cv)n=r.integer();auto em=gb_aa_reals(r,pu*pv);auto pat=gb_aa_reals(r,mu*mv*nd*pu*pv);
 AdvApp2Var_ApproxF2var::mma2fx6_(&mu,&mv,&nd,&ns,&ndse,&pu,&pv,&iu,&iv,&eps,fro.data(),pat.data(),em.data(),cu.data(),cv.data());
 for(int n:cu)gb_int(n);for(int n:cv)gb_int(n);gb_end();
}

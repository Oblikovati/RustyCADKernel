// TKGeomBase oracle: the AdvApp2Var_ApproxF2var kernels (tkgeombase_probe.cxx), with the
// helpers of tkgeombase_probe_advapp2var_math.cxx and the functions of
// tkgeombase_probe_advapp2var_eval.cxx. The Rust twin is
// crates/tkgeombase/examples/gb_probe/adv_f2var.rs.
// aa_mma1her: iordre.
void gb_aa_mma1her_case(std::istream& in){
 GbReader r{in};int io=r.integer();auto h=gb_aa_fill((2*io+2)*(2*io+2));int ier=-1;
 AdvApp2Var_ApproxF2var::mma1her_(&io,h.data(),&ier);gb_int(ier);gb_aa_print(h);gb_end();
}
// aa_mma2roo: nbpntu nbpntv.
void gb_aa_mma2roo_case(std::istream& in){
 GbReader r{in};int nu=r.integer(),nv=r.integer();auto u=gb_aa_fill(nu),v=gb_aa_fill(nv);
 AdvApp2Var_ApproxF2var::mma2roo_(&nu,&nv,u.data(),v.data());gb_aa_print(u);gb_aa_print(v);gb_end();
}
// aa_mma2jmx: ndgjac iordre.
void gb_aa_mma2jmx_case(std::istream& in){
 GbReader r{in};int nd=r.integer(),io=r.integer();auto x=gb_aa_fill(nd-2*(io+1)+1);
 AdvApp2Var_ApproxF2var::mma2jmx_(&nd,&io,x.data());gb_aa_print(x);gb_end();
}
// aa_mmapptt: ndgjac nbpnts jordre.
void gb_aa_mmapptt_case(std::istream& in){
 GbReader r{in};int nd=r.integer(),np=r.integer(),jo=r.integer();auto c=gb_aa_fill((np/2+1)*(nd+1));int ier=-1;
 AdvApp2Var_ApproxF2var::mmapptt_(&nd,&np,&jo,c.data(),&ier);gb_int(ier);gb_aa_print(c);gb_end();
}
// aa_mma2ac1: ndimen mxujac mxvjac iordru iordrv contr1..4[ndimen*(iordru+2)*(iordrv+2)] patjac.
void gb_aa_mma2ac1_case(std::istream& in){
 GbReader r{in};int nd=r.integer(),mu=r.integer(),mv=r.integer(),iu=r.integer(),iv=r.integer();
 const int nc=nd*(iu+2)*(iv+2);auto c1=gb_aa_reals(r,nc),c2=gb_aa_reals(r,nc),c3=gb_aa_reals(r,nc),c4=gb_aa_reals(r,nc);
 auto pat=gb_aa_reals(r,(mu+1)*(mv+1)*nd);auto uh=gb_aa_fill((2*iu+2)*(2*iu+2)),vh=gb_aa_fill((2*iv+2)*(2*iv+2));int ier=0;
 AdvApp2Var_ApproxF2var::mma1her_(&iu,uh.data(),&ier);AdvApp2Var_ApproxF2var::mma1her_(&iv,vh.data(),&ier);
 AdvApp2Var_ApproxF2var::mma2ac1_(&nd,&mu,&mv,&iu,&iv,c1.data(),c2.data(),c3.data(),c4.data(),uh.data(),vh.data(),pat.data());
 gb_aa_print(pat);gb_end();
}
// aa_mma2ac2/aa_mma2ac3: ndimen mxujac mxvjac iordr nclim ncf1[iordr+1] crb1[nclim*ndimen*(iordr+1)] ncf2 crb2 patjac.
void gb_aa_mma2ac23(std::istream& in,bool v){
 GbReader r{in};int nd=r.integer(),mu=r.integer(),mv=r.integer(),io=r.integer(),nl=r.integer();
 std::vector<int> n1(io+1),n2(io+1);for(int& n:n1)n=r.integer();auto b1=gb_aa_reals(r,nl*nd*(io+1));
 for(int& n:n2)n=r.integer();auto b2=gb_aa_reals(r,nl*nd*(io+1));auto pat=gb_aa_reals(r,(mu+1)*(mv+1)*nd);
 auto h=gb_aa_fill((2*io+2)*(2*io+2));int ier=0;AdvApp2Var_ApproxF2var::mma1her_(&io,h.data(),&ier);
 if(v)AdvApp2Var_ApproxF2var::mma2ac2_(&nd,&mu,&mv,&io,&nl,n1.data(),b1.data(),n2.data(),b2.data(),h.data(),pat.data());
 else AdvApp2Var_ApproxF2var::mma2ac3_(&nd,&mu,&mv,&io,&nl,n1.data(),b1.data(),n2.data(),b2.data(),h.data(),pat.data());
 gb_aa_print(pat);gb_end();
}
void gb_aa_mma2ac2_case(std::istream& in){gb_aa_mma2ac23(in,true);}
void gb_aa_mma2ac3_case(std::istream& in){gb_aa_mma2ac23(in,false);}
// aa_mma2can: ncfmxu ncfmxv ndimen iordru iordrv ncoefu ncoefv patjac[ncoefu*ncoefv*ndimen].
void gb_aa_mma2can_case(std::istream& in){
 GbReader r{in};int mu=r.integer(),mv=r.integer(),nd=r.integer(),iu=r.integer(),iv=r.integer(),nu=r.integer(),nv=r.integer();
 auto pj=gb_aa_reals(r,nu*nv*nd);auto aux=gb_aa_fill(2*nu*nv*nd),can=gb_aa_fill(mu*mv*nd);int ier=-1;
 AdvApp2Var_ApproxF2var::mma2can_(&mu,&mv,&nd,&iu,&iv,&nu,&nv,pj.data(),aux.data(),can.data(),&ier);
 gb_int(ier);gb_aa_print(can);gb_aa_print(aux);gb_end();
}
// The four square tables of a (nbpntu, nbpntv, ndimen) discretisation, filled with `value`.
struct GbAaSquare{
 std::vector<double> soso,diso,sodi,didi;
 GbAaSquare(int nu,int nv,int nd,double value):soso((nu/2+1)*(nv/2+1)*nd,value),diso(std::max(nu/2*(nv/2)*nd,1),value),
  sodi(std::max(nu/2*(nv/2)*nd,1),value),didi((nu/2+1)*(nv/2+1)*nd,value){}
 void print()const{gb_aa_print(soso);gb_aa_print(diso);gb_aa_print(sodi);gb_aa_print(didi);}
};
// aa_mma2cdi: ndimen nbpntu nbpntv iordru iordrv contr1..4 sotbu1 sotbu2 ditbu1 ditbu2 sotbv1 sotbv2 ditbv1 ditbv2.
void gb_aa_mma2cdi_case(std::istream& in){
 GbReader r{in};int nd=r.integer(),nu=r.integer(),nv=r.integer(),iu=r.integer(),iv=r.integer();
 const int nc=nd*(iu+2)*(iv+2),su=(nv/2+1)*nd*(iu+1),sv=(nu/2+1)*nd*(iv+1);
 std::vector<std::vector<double>> a;for(int k=0;k<4;++k)a.push_back(gb_aa_reals(r,nc));
 for(int k=0;k<4;++k)a.push_back(gb_aa_reals(r,su));for(int k=0;k<4;++k)a.push_back(gb_aa_reals(r,sv));
 for(auto& v:a)if(v.empty())v.assign(1,0.);
 auto ur=gb_aa_fill(nu),vr=gb_aa_fill(nv);AdvApp2Var_ApproxF2var::mma2roo_(&nu,&nv,ur.data(),vr.data());
 GbAaSquare t(nu,nv,nd,7.5);int ier=-1;
 AdvApp2Var_ApproxF2var::mma2cdi_(&nd,&nu,ur.data(),&nv,vr.data(),&iu,&iv,a[0].data(),a[1].data(),a[2].data(),a[3].data(),
  a[4].data(),a[5].data(),a[6].data(),a[7].data(),a[8].data(),a[9].data(),a[10].data(),a[11].data(),
  t.soso.data(),t.sodi.data(),t.diso.data(),t.didi.data(),&ier);
 gb_int(ier);t.print();gb_end();
}
// aa_mma2ds1: nbpntu nbpntv isofav u0 u1 v0 v1 function (ndimen 3).
void gb_aa_mma2ds1_case(std::istream& in){
 GbReader r{in};int nd=3,nu=r.integer(),nv=r.integer(),iso=r.integer();double uv[4];for(double& x:uv)x=r.real();
 GbAaFunction f(r.integer());auto ur=gb_aa_fill(nu),vr=gb_aa_fill(nv);AdvApp2Var_ApproxF2var::mma2roo_(&nu,&nv,ur.data(),vr.data());
 GbAaSquare t(nu,nv,nd,0.25);auto fp=gb_aa_fill(nd*std::max(nu,nv)),tt=gb_aa_fill(std::max(nu,nv));int ier=-1;
 AdvApp2Var_ApproxF2var::mma2ds1_(&nd,uv,uv+2,f,&nu,&nv,ur.data(),vr.data(),&iso,t.soso.data(),t.diso.data(),t.sodi.data(),
  t.didi.data(),fp.data(),tt.data(),&ier);
 gb_int(ier);t.print();gb_end();
}

// TKGeomBase oracle: the AdvApp2Var_MathBase kernels, part 2 (tkgeombase_probe.cxx), with
// the helpers of tkgeombase_probe_advapp2var_math.cxx. The Rust twin is
// crates/tkgeombase/examples/gb_probe/adv_math2.rs.
// aa_mmhjcan: ndimen ncourb orcont ncflim ncftab[ncourb] tdecop[ncourb+1] tcbold[ndimen*ncflim*ncourb].
void gb_aa_mmhjcan_case(std::istream& in){
 GbReader r{in};int nd=r.integer(),nk=r.integer(),oc=r.integer(),nl=r.integer();
 std::vector<int> ncf(std::max(nk,1));for(int& n:ncf)n=r.integer();auto t=gb_aa_reals(r,nk+1),old=gb_aa_reals(r,nd*nl*nk);
 auto neu=gb_aa_fill(nd*nl*nk);int ier=-1;
 AdvApp2Var_MathBase::mmhjcan_(&nd,&nk,ncf.data(),&oc,&nl,old.data(),t.data(),neu.data(),&ier);gb_int(ier);gb_aa_print(neu);gb_end();
}
// aa_mminltt: ncolmx nlgnmx nbrcol nbrlgn table[ncolmx*nlgnmx] m, then m lines of nbrcol values.
void gb_aa_mminltt_case(std::istream& in){
 GbReader r{in};int nc=r.integer(),nl=r.integer(),ncol=r.integer(),used=r.integer();auto t=gb_aa_reals(r,nc*nl);int m=r.integer();
 for(int i=0;i<m;++i){auto line=gb_aa_reals(r,ncol);double eps=0.;int ier=-1;
  AdvApp2Var_MathBase::mminltt_(&nc,&nl,t.data(),&ncol,&used,line.data(),&eps,&ier);gb_int(used);gb_int(ier);}
 gb_aa_print(t);gb_end();
}
// aa_mmjacan: ideriv ndeg poljac[(ndeg/2+1)*2].
void gb_aa_mmjacan_case(std::istream& in){
 GbReader r{in};int id=r.integer(),nd=r.integer();auto p=gb_aa_reals(r,(nd/2+1)*2);auto c=gb_aa_fill(nd+1);
 AdvApp2Var_MathBase::mmjacan_(&id,&nd,p.data(),c.data());gb_aa_print(c);gb_end();
}
// aa_mmjaccv: ncoef ndim ider crvlgd[ncoef*ndim].
void gb_aa_mmjaccv_case(std::istream& in){
 GbReader r{in};int nc=r.integer(),nd=r.integer(),id=r.integer();auto l=gb_aa_reals(r,nc*nd);
 auto aux=gb_aa_fill(((nc-1)/2+1)*2),c=gb_aa_fill(nc*nd);
 AdvApp2Var_MathBase::mmjaccv_(&nc,&nd,&id,l.data(),aux.data(),c.data());gb_aa_print(c);gb_end();
}
// aa_mmpobas: t iordre ncoeff nderiv.
void gb_aa_mmpobas_case(std::istream& in){
 GbReader r{in};double t=r.real();int io=r.integer(),nc=r.integer(),nv=r.integer();auto v=gb_aa_fill(nc*(nv+1));int ier=-1;
 AdvApp2Var_MathBase::mmpobas_(&t,&io,&nc,&nv,v.data(),&ier);gb_int(ier);gb_aa_print(v);gb_end();
}
// aa_mmpocrb: ndimax ncoeff ndim t courbe[ndimax*ncoeff].
void gb_aa_mmpocrb_case(std::istream& in){
 GbReader r{in};int nx=r.integer(),nc=r.integer(),nd=r.integer();double t=r.real();auto c=gb_aa_reals(r,nx*nc);auto p=gb_aa_fill(nd);
 AdvApp2Var_MathBase::mmpocrb_(&nx,&nc,c.data(),&nd,&t,p.data());gb_aa_print(p);gb_end();
}
// aa_mmmpocur: ncofmx ndim ndeg t courbe[ncofmx*ndim].
void gb_aa_mmmpocur_case(std::istream& in){
 GbReader r{in};int nx=r.integer(),nd=r.integer(),ng=r.integer();double t=r.real();auto c=gb_aa_reals(r,nx*nd);auto p=gb_aa_fill(nd);
 AdvApp2Var_MathBase::mmmpocur_(&nx,&nd,&ng,c.data(),&t,p.data());gb_aa_print(p);gb_end();
}
// aa_mmposui: dimmat aposit[2*dimmat].
void gb_aa_mmposui_case(std::istream& in){
 GbReader r{in};int n=r.integer();std::vector<int> a(2*n);for(int& x:a)x=r.integer();
 std::vector<int> p(std::max(a[2*n-1],1),-9);int ier=-1,nstoc=a[2*n-1];
 AdvApp2Var_MathBase::mmposui_(&n,&nstoc,a.data(),p.data(),&ier);gb_int(ier);for(int x:p)gb_int(x);gb_end();
}
// The system of aa_mmresol, built from its sizes by exact small-integer formulas: H of
// order h with `bw` entries left of each diagonal, G with g rows of up to 5 entries
// from column i, and the dense profile of G H^-1 G^T.
struct GbAaResol{
 int h,g,bw,hn=0,gn=0,mn=0;std::vector<double> mh,mg,vh,vg;std::vector<int> hpos,hsui,gpos,msui,mpos;
 void build(){
  for(int i=1;i<=h;++i){int c=std::min(i-1,bw);for(int j=i-c;j<=i;++j)mh.push_back(i==j?2.*bw+1.+i*0.001:-1./(i-j+1.));
   hn+=c+1;hpos.push_back(c);hpos.push_back(hn);vh.push_back(1.+0.5*i);}
  for(int i=1;i<=g;++i){int c=std::min(5,h-i+1);for(int j=i;j<i+c;++j)mg.push_back(j==i?3.:1./(i+j));
   gn+=c;gpos.push_back(c);gpos.push_back(gn);gpos.push_back(i);vg.push_back(0.25*i);}
  for(int i=1;i<=g;++i){mn+=i;mpos.push_back(i-1);mpos.push_back(mn);}
  hsui.assign(std::max(hn,1),0);msui.assign(std::max(mn,1),0);int ier=0;
  AdvApp2Var_MathBase::mmposui_(&h,&hn,hpos.data(),hsui.data(),&ier);
  if(g>0)AdvApp2Var_MathBase::mmposui_(&g,&mn,mpos.data(),msui.data(),&ier);
  if(g==0){gpos.assign(3,0);mpos.assign(2,0);mg.assign(1,0.);vg.assign(1,0.);}
 }
};
// aa_mmresol: h g bw (every scratch size above 100, as natively required).
void gb_aa_mmresol_case(std::istream& in){
 GbReader r{in};GbAaResol s;s.h=r.integer();s.g=r.integer();s.bw=r.integer();s.build();
 auto sol=gb_aa_fill(s.h);int ier=-1;
 AdvApp2Var_MathBase::mmresol_(&s.h,&s.g,&s.hn,&s.gn,&s.mn,s.mh.data(),s.mg.data(),s.vh.data(),s.vg.data(),s.hpos.data(),
  s.hsui.data(),s.gpos.data(),s.msui.data(),s.mpos.data(),sol.data(),&ier);
 gb_int(ier);gb_aa_print(sol);gb_end();
}
// aa_mmrtptt: degree.
void gb_aa_mmrtptt_case(std::istream& in){
 GbReader r{in};int n=r.integer();auto v=gb_aa_fill(n/2);AdvApp2Var_MathBase::mmrtptt_(&n,v.data());gb_aa_print(v);gb_end();
}
// aa_mmsrre2: t n table[n] eps.
void gb_aa_mmsrre2_case(std::istream& in){
 GbReader r{in};double t=r.real();int n=r.integer();auto v=gb_aa_reals(r,n);double eps=r.real();int ni=-1,it=-1,ier=-1;
 AdvApp2Var_MathBase::mmsrre2_(&t,&n,v.data(),&eps,&ni,&it,&ier);gb_int(ni);gb_int(it);gb_int(ier);gb_end();
}
// aa_mmunivt: n eps v[n].
void gb_aa_mmunivt_case(std::istream& in){
 GbReader r{in};int n=r.integer();double eps=r.real();auto v=gb_aa_reals(r,n);auto o=gb_aa_fill(n);int ier=-1;
 AdvApp2Var_MathBase::mmunivt_(&n,v.data(),o.data(),&eps,&ier);gb_int(ier);gb_aa_print(o);gb_end();
}
// aa_mmvncol: n v[n].
void gb_aa_mmvncol_case(std::istream& in){
 GbReader r{in};int n=r.integer();auto v=gb_aa_reals(r,n);v.resize(std::max(n,3),0.);auto o=gb_aa_fill(3);int ier=-1;
 AdvApp2Var_MathBase::mmvncol_(&n,v.data(),o.data(),&ier);gb_int(ier);gb_aa_print(o);gb_end();
}
// aa_mmarcin: ndimax ndim ncoeff u0 u1 crvold[ncoeff*ndim].
void gb_aa_mmarcin_case(std::istream& in){
 GbReader r{in};int nx=r.integer(),nd=r.integer(),nc=r.integer();double u0=r.real(),u1=r.real();auto o=gb_aa_reals(r,nc*nd);
 auto c=gb_aa_fill(nx*nc);int ier=-1;
 AdvApp2Var_MathBase::mmarcin_(&nx,&nd,&nc,o.data(),&u0,&u1,c.data(),&ier);gb_int(ier);gb_aa_print(c);gb_end();
}
// aa_mmcvinv: ndimax ncoef ndim curveo[ncoef*ndim].
void gb_aa_mmcvinv_case(std::istream& in){
 GbReader r{in};int nx=r.integer(),nc=r.integer(),nd=r.integer();auto o=gb_aa_reals(r,nc*nd);auto c=gb_aa_fill(nx*nc);
 AdvApp2Var_MathBase::mmcvinv_(&nx,&nc,&nd,o.data(),c.data());gb_aa_print(c);gb_end();
}
// aa_mmmrslwd: normax nordre ndim eps amat[normax*nordre] bmat[normax*ndim].
void gb_aa_mmmrslwd_case(std::istream& in){
 GbReader r{in};int nx=r.integer(),no=r.integer(),nd=r.integer();double eps=r.real();auto a=gb_aa_reals(r,nx*no),b=gb_aa_reals(r,nx*nd);
 auto aux=gb_aa_fill((no+nd)*no),x=gb_aa_fill(nx*nd);int ier=-1;
 AdvApp2Var_MathBase::mmmrslwd_(&nx,&no,&nd,a.data(),b.data(),&eps,aux.data(),x.data(),&ier);gb_int(ier);gb_aa_print(x);gb_end();
}

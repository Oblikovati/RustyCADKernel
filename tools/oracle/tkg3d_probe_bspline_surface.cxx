// TKG3d oracle: the members of Geom_BSplineSurface that the generic surface requests
// (g3_surface, g3_surface_moves, g3_surface_evalrep) do not reach (tkg3d_probe.cxx).
void g3_int_array(const NCollection_Array1<int>& a){g3_int(a.Lower());g3_int(a.Upper());for(int i=a.Lower();i<=a.Upper();++i)g3_int(a(i));}
void g3_real_array(const NCollection_Array1<double>& a){g3_int(a.Lower());g3_int(a.Upper());for(int i=a.Lower();i<=a.Upper();++i)g3_real(a(i));}
// The whole state: counts, degrees, flags, knots, multiplicities, flat knots, poles and weights.
void g3_bss_state(const occ::handle<Geom_BSplineSurface>& s){
 g3_int(s->NbUPoles());g3_int(s->NbVPoles());g3_int(s->NbUKnots());g3_int(s->NbVKnots());g3_int(s->UDegree());g3_int(s->VDegree());
 g3_flag(s->IsURational());g3_flag(s->IsVRational());g3_flag(s->IsUPeriodic());g3_flag(s->IsVPeriodic());
 g3_int(s->UKnotDistribution());g3_int(s->VKnotDistribution());g3_int(s->Continuity());
 G3_TRY(g3_int(s->FirstUKnotIndex()));G3_TRY(g3_int(s->LastUKnotIndex()));G3_TRY(g3_int(s->FirstVKnotIndex()));G3_TRY(g3_int(s->LastVKnotIndex()));
 g3_real_array(s->UKnots());g3_real_array(s->VKnots());g3_int_array(s->UMultiplicities());g3_int_array(s->VMultiplicities());
 g3_real_array(s->UKnotSequence());g3_real_array(s->VKnotSequence());
 const NCollection_Array2<gp_Pnt>& p=s->Poles();
 for(int i=p.LowerRow();i<=p.UpperRow();++i)for(int j=p.LowerCol();j<=p.UpperCol();++j){g3_pnt(p(i,j));g3_real(s->Weight(i,j));}
 g3_flag(s->Weights()!=nullptr);g3_flag(s->HasEvalRepresentation());
 double u1,u2,v1,v2;s->Bounds(u1,u2,v1,v2);g3_real(u1);g3_real(u2);g3_real(v1);g3_real(v2);
}
// The getters with indices, the deprecated copying getters and their size checks.
void g3_bss_getters(const occ::handle<Geom_BSplineSurface>& s,int i,int j){
 G3_TRY(g3_pnt(s->Pole(i,j)));G3_TRY(g3_real(s->Weight(i,j)));G3_TRY(g3_real(s->UKnot(i)));G3_TRY(g3_real(s->VKnot(j)));
 G3_TRY(g3_int(s->UMultiplicity(i)));G3_TRY(g3_int(s->VMultiplicity(j)));
 NCollection_Array2<gp_Pnt> p(1,s->NbUPoles()+(i>2?1:0),1,s->NbVPoles());G3_TRY(s->Poles(p);g3_pnt(p(1,1)));
 NCollection_Array2<double> w(1,s->NbUPoles(),1,s->NbVPoles()+(j>2?1:0));G3_TRY(s->Weights(w);g3_real(w(1,1));g3_real(w(w.UpperRow(),w.UpperCol())));
 NCollection_Array1<double> ku(1,s->NbUKnots()+(i>2?1:0));G3_TRY(s->UKnots(ku);g3_real_array(ku));
 NCollection_Array1<double> kv(1,s->NbVKnots());G3_TRY(s->VKnots(kv);g3_real_array(kv));
 NCollection_Array1<double> su(1,s->UKnotSequence().Length());G3_TRY(s->UKnotSequence(su);g3_real_array(su));
 NCollection_Array1<double> sv(1,s->VKnotSequence().Length()+(j>2?1:0));G3_TRY(s->VKnotSequence(sv);g3_real_array(sv));
 NCollection_Array1<int> mu(1,s->NbUKnots());G3_TRY(s->UMultiplicities(mu);g3_int_array(mu));
 NCollection_Array1<int> mv(1,s->NbVKnots()+(i>2?1:0));G3_TRY(s->VMultiplicities(mv);g3_int_array(mv));
}
// Local evaluation between knots, LocateU/LocateV, PeriodicNormalization, the checked isos.
void g3_bss_queries(const occ::handle<Geom_BSplineSurface>& s,G3Reader& r){
 std::vector<double> uv=r.reals();int k1=r.integer(),k2=r.integer(),k3=r.integer(),k4=r.integer();std::vector<double> tol=r.reals();
 for(size_t n=0;n+1<uv.size();n+=2){double u=uv[n],v=uv[n+1];
  G3_TRY(g3_pnt(s->LocalValue(u,v,k1,k2,k3,k4)));
  G3_TRY(gp_Pnt p;gp_Vec a,b;s->LocalD1(u,v,k1,k2,k3,k4,p,a,b);g3_pnt(p);g3_vec(a);g3_vec(b));
  G3_TRY(gp_Pnt p;gp_Vec a,b,c,d,e;s->LocalD2(u,v,k1,k2,k3,k4,p,a,b,c,d,e);g3_pnt(p);for(auto x:{a,b,c,d,e})g3_vec(x));
  G3_TRY(gp_Pnt p;gp_Vec a,b,c,d,e,f,g,h,i;s->LocalD3(u,v,k1,k2,k3,k4,p,a,b,c,d,e,f,g,h,i);g3_pnt(p);for(auto x:{a,b,c,d,e,f,g,h,i})g3_vec(x));
  for(int nu=0;nu<=2;++nu)for(int nv=0;nv<=2;++nv)G3_TRY(g3_vec(s->LocalDN(u,v,k1,k2,k3,k4,nu,nv)));
  G3_TRY(double a=u,b=v;s->PeriodicNormalization(a,b);g3_real(a);g3_real(b));
  for(double t:tol)for(int rep=0;rep<=1;++rep){
   G3_TRY(int i1=-7,i2=-7;s->LocateU(u,t,i1,i2,rep!=0);g3_int(i1);g3_int(i2));
   G3_TRY(int i1=-7,i2=-7;s->LocateV(v,t,i1,i2,rep!=0);g3_int(i1);g3_int(i2));}
  for(int check=0;check<=1;++check){
   G3_TRY(occ::handle<Geom_Curve> c=s->UIso(u,check!=0);g3_name(c);g3_curve_summary(c,{0.0,0.5,1.0}));
   G3_TRY(occ::handle<Geom_Curve> c=s->VIso(v,check!=0);g3_name(c);g3_curve_summary(c,{0.0,0.5,1.0}));}
 }
}
void g3_bspline_surface_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> base=g3_built_surface(r);if(base.IsNull())return;
 occ::handle<Geom_BSplineSurface> s=occ::down_cast<Geom_BSplineSurface>(base);int i=r.integer(),j=r.integer();double tol=r.real();
 g3_bss_state(s);g3_bss_getters(s,i,j);g3_bss_queries(s,r);g3_int(Geom_BSplineSurface::MaxDegree());
 double tu=-1,tv=-1;G3_TRY(s->Resolution(tol,tu,tv);g3_real(tu);g3_real(tv));G3_TRY(s->Resolution(2*tol,tu,tv);g3_real(tu);g3_real(tv));
 g3_dump(s,-1);g3_bss_state(occ::down_cast<Geom_BSplineSurface>(s->Copy()));std::cout<<'\n';
}
void g3_bss_array_reals(G3Reader& r,NCollection_Array1<double>& a){int lower=r.integer(),n=r.integer();a.Resize(lower,lower+n-1,false);for(int k=lower;k<lower+n;++k)a(k)=r.real();}
void g3_bss_array_ints(G3Reader& r,NCollection_Array1<int>& a){int lower=r.integer(),n=r.integer();a.Resize(lower,lower+n-1,false);for(int k=lower;k<lower+n;++k)a(k)=r.integer();}
void g3_bss_array_pnts(G3Reader& r,NCollection_Array1<gp_Pnt>& a){int lower=r.integer(),n=r.integer();a.Resize(lower,lower+n-1,false);for(int k=lower;k<lower+n;++k)a(k)=r.pnt();}
// One edit: an operation code and its arguments (see tools/tkg3d_corpus_bspline_surface.py).
void g3_bss_edit(const occ::handle<Geom_BSplineSurface>& s,G3Reader& r){
 int op=r.integer();bool u=op%2==0;NCollection_Array1<double> k(1,1),w(1,1);NCollection_Array1<int> m(1,1);NCollection_Array1<gp_Pnt> p(1,1);
 switch(op){
  case 0:s->ExchangeUV();break;
  case 2:case 3:if(u)s->SetUPeriodic();else s->SetVPeriodic();break;
  case 4:case 5:{int i=r.integer();if(u)s->SetUOrigin(i);else s->SetVOrigin(i);break;}
  case 6:case 7:if(u)s->SetUNotPeriodic();else s->SetVNotPeriodic();break;
  case 8:case 9:if(u)s->UReverse();else s->VReverse();break;
  case 10:{int a=r.integer(),b=r.integer();s->IncreaseDegree(a,b);break;}
  case 12:case 13:{g3_bss_array_reals(r,k);g3_bss_array_ints(r,m);double t=r.real();bool add=r.flag();
   if(u)s->InsertUKnots(k,m,t,add);else s->InsertVKnots(k,m,t,add);break;}
  case 14:case 15:{int i=r.integer(),mm=r.integer();double t=r.real();g3_flag(u?s->RemoveUKnot(i,mm,t):s->RemoveVKnot(i,mm,t));break;}
  case 16:case 17:{int i=r.integer(),mm=r.integer();if(u)s->IncreaseUMultiplicity(i,mm);else s->IncreaseVMultiplicity(i,mm);break;}
  case 18:case 19:{int a=r.integer(),b=r.integer(),mm=r.integer();if(u)s->IncreaseUMultiplicity(a,b,mm);else s->IncreaseVMultiplicity(a,b,mm);break;}
  case 20:case 21:{int a=r.integer(),b=r.integer(),st=r.integer();if(u)s->IncrementUMultiplicity(a,b,st);else s->IncrementVMultiplicity(a,b,st);break;}
  case 22:case 23:{double x=r.real();int mm=r.integer();double t=r.real();bool add=r.flag();if(u)s->InsertUKnot(x,mm,t,add);else s->InsertVKnot(x,mm,t,add);break;}
  case 24:case 25:{double a=r.real(),b=r.real(),c=r.real(),d=r.real(),e=r.real(),f=r.real();if(u)s->Segment(a,b,c,d,e,f);else s->CheckAndSegment(a,b,c,d,e,f);break;}
  case 26:case 27:{int i=r.integer();double x=r.real();if(u)s->SetUKnot(i,x);else s->SetVKnot(i,x);break;}
  case 28:case 29:{g3_bss_array_reals(r,k);if(u)s->SetUKnots(k);else s->SetVKnots(k);break;}
  case 30:case 31:{int i=r.integer();double x=r.real();int mm=r.integer();if(u)s->SetUKnot(i,x,mm);else s->SetVKnot(i,x,mm);break;}
  case 32:{int i=r.integer(),j=r.integer();s->SetPole(i,j,r.pnt());break;}
  case 34:{int i=r.integer(),j=r.integer();gp_Pnt q=r.pnt();s->SetPole(i,j,q,r.real());break;}
  case 36:case 37:{int i=r.integer();g3_bss_array_pnts(r,p);if(u)s->SetPoleRow(i,p);else s->SetPoleCol(i,p);break;}
  case 38:case 39:{int i=r.integer();g3_bss_array_pnts(r,p);g3_bss_array_reals(r,w);if(u)s->SetPoleRow(i,p,w);else s->SetPoleCol(i,p,w);break;}
  case 40:{int i=r.integer(),j=r.integer();s->SetWeight(i,j,r.real());break;}
  case 42:case 43:{int i=r.integer();g3_bss_array_reals(r,w);if(u)s->SetWeightRow(i,w);else s->SetWeightCol(i,w);break;}
  case 44:{double a=r.real(),b=r.real();gp_Pnt q=r.pnt();int i1=r.integer(),i2=r.integer(),j1=r.integer(),j2=r.integer(),f1=-1,f2=-1,f3=-1,f4=-1;
   s->MovePoint(a,b,q,i1,i2,j1,j2,f1,f2,f3,f4);g3_int(f1);g3_int(f2);g3_int(f3);g3_int(f4);break;}
  case 46:s->Transform(g3_trsf(r));break;
  default:throw Standard_ProgramError("unknown edit");
 }
}
// A surface, parameters, then a list of edits each followed by the state and the summary.
void g3_bspline_surface_edit_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> base=g3_built_surface(r);if(base.IsNull())return;
 occ::handle<Geom_BSplineSurface> s=occ::down_cast<Geom_BSplineSurface>(base);std::vector<double> uv=r.reals();int n=r.integer();
 for(int e=0;e<n;++e){G3_TRY(g3_bss_edit(s,r));g3_bss_state(s);g3_surface_summary(s,uv);}
 g3_dump(s,-1);std::cout<<'\n';
}

// TKG3d oracle: GeomAdaptor_TransformedCurve, Adaptor3d_HVertex and Adaptor3d_TopolTool
// (tkg3d_probe.cxx). Readers of 2D adaptors and surface adaptors come from
// tkg3d_probe_adaptor3d.cxx, the adaptor summaries from tkg3d_probe_geom_adaptor.cxx.
void a3_transformed_curve_state(const occ::handle<GeomAdaptor_TransformedCurve>& t,const std::vector<double>& u){
 g3_flag(t->Is3DCurve());g3_flag(t->IsCurveOnSurface());g3_int(t->Trsf().Form());g3_name(t->GeomCurve());
 g3_real(t->Curve().FirstParameter());g3_real(t->Curve().LastParameter());g3_flag(&t->ChangeCurve()==&t->Curve());
 a3_curve_summary(t,u);
}
// <curve adaptor> params trsf t1 t2 cos <2D adaptor> <surface adaptor>: the transformed 3D
// curve, MakeCurve, Trim, ShallowCopy, then (cos != 0) a curve on a surface, SetTrsf and Load.
void a3_transformed_curve_case(std::istream& in){
 G3Reader r{in};occ::handle<GeomAdaptor_Curve> c;
 try{c=a3_geom_curve(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 std::vector<double> u=r.reals();gp_Trsf m=g3_trsf(r);double t1=r.real(),t2=r.real();bool cos=r.flag();
 occ::handle<Adaptor2d_Curve2d> c2;occ::handle<Adaptor3d_Surface> s;if(cos&&!a3_inputs(r,c2,s))return;
 occ::handle<GeomAdaptor_TransformedCurve> t=new GeomAdaptor_TransformedCurve(c->Curve(),c->FirstParameter(),c->LastParameter(),m);
 a3_transformed_curve_state(t,u);a3_make_curve(t,u);
 G3_TRY(occ::handle<Adaptor3d_Curve> x=t->Trim(t1,t2,1e-7);a3_curve_summary(x,u));
 G3_TRY(occ::handle<Adaptor3d_Curve> x=t->ShallowCopy();a3_curve_summary(x,u));
 if(cos){occ::handle<Adaptor3d_CurveOnSurface> o;
  try{o=new Adaptor3d_CurveOnSurface(c2,s);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  if(!o.IsNull()){t->LoadCurveOnSurface(o);g3_flag(&t->CurveOnSurface()==o.get());a3_transformed_curve_state(t,u);
   G3_TRY(occ::handle<Adaptor3d_Curve> x=t->Trim(t1,t2,1e-7);a3_curve_summary(x,u));
   G3_TRY(occ::handle<Adaptor3d_Curve> x=t->ShallowCopy();a3_curve_summary(x,u));
   t->LoadCurveOnSurface(nullptr);g3_flag(t->Is3DCurve());}}
 t->SetTrsf(gp_Trsf());a3_transformed_curve_state(t,u);
 G3_TRY(t->Load(c->Curve());a3_transformed_curve_state(t,u));
 G3_TRY(t->Load(c->Curve(),t1,t2);a3_transformed_curve_state(t,u));
 G3_TRY(t->ChangeCurve().Load(c->Curve());a3_transformed_curve_state(t,u));
 G3_TRY(t->Load(occ::handle<Geom_Curve>()));
 occ::handle<GeomAdaptor_TransformedCurve> e=new GeomAdaptor_TransformedCurve();g3_flag(e->Is3DCurve());g3_int(e->Trsf().Form());
 g3_name(e->GeomCurve());G3_TRY(g3_real(e->FirstParameter()));G3_TRY(e->OffsetCurve());G3_TRY(e->Trim(0.,1.,1e-7));
 std::cout<<'\n';
}
void a3_hvertex_state(const occ::handle<Adaptor3d_HVertex>& v,const occ::handle<Adaptor2d_Curve2d>& c,const occ::handle<Adaptor3d_HVertex>& o){
 gp_Pnt2d p=v->Value();g3_real(p.X());g3_real(p.Y());g3_int(v->Orientation());g3_real(v->Resolution(c));
 G3_TRY(g3_real(v->Parameter(c)));g3_flag(v->IsSame(o));g3_flag(v->IsSame(v));
}
// P orientation resolution Q <2D adaptor>: a vertex against another one at Q and a curve.
void a3_hvertex_case(std::istream& in){
 G3Reader r{in};gp_Pnt2d p=a3_pnt2(r);int ori=r.integer();double res=r.real();gp_Pnt2d q=a3_pnt2(r);
 occ::handle<Adaptor2d_Curve2d> c;try{c=a3_adaptor2d(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 occ::handle<Adaptor3d_HVertex> v=new Adaptor3d_HVertex(p,TopAbs_Orientation(ori),res);
 occ::handle<Adaptor3d_HVertex> o=new Adaptor3d_HVertex(q,TopAbs_REVERSED,0.);
 a3_hvertex_state(v,c,o);a3_hvertex_state(o,c,v);a3_hvertex_state(new Adaptor3d_HVertex(),c,v);std::cout<<'\n';
}
// The restrictions of a tool, each with its vertices.
void a3_topol_restrictions(const occ::handle<Adaptor3d_TopolTool>& t){
 for(t->Init();t->More();t->Next()){
  occ::handle<Adaptor2d_Curve2d> c=t->Value();g3_name(c);g3_real(c->FirstParameter());g3_real(c->LastParameter());
  gp_Lin2d l=c->Line();g3_real(l.Location().X());g3_real(l.Location().Y());g3_real(l.Direction().X());g3_real(l.Direction().Y());
  g3_int(t->Orientation(c));G3_TRY(g3_real(t->Tol3d(c)));
  t->Initialize(c);occ::handle<Adaptor3d_HVertex> first;
  for(t->InitVertexIterator();t->MoreVertex();t->NextVertex()){
   occ::handle<Adaptor3d_HVertex> v=t->Vertex();if(first.IsNull())first=v;
   a3_hvertex_state(v,c,first);g3_int(t->Orientation(v));g3_flag(t->Identical(v,first));
   G3_TRY(g3_real(t->Tol3d(v)));G3_TRY(g3_pnt(t->Pnt(v)));}
  G3_TRY(t->Vertex());
 }
 G3_TRY(t->Value());
}
void a3_topol_samples(const occ::handle<Adaptor3d_TopolTool>& t){
 int nu=t->NbSamplesU(),nv=t->NbSamplesV();g3_int(nu);g3_int(nv);g3_int(t->NbSamples());
 for(int i:{1,2,nu+1,nu*nv}){gp_Pnt2d p;gp_Pnt q;G3_TRY(t->SamplePoint(i,p,q);g3_real(p.X());g3_real(p.Y());g3_pnt(q));}
}
void a3_topol_parameters(const occ::handle<Adaptor3d_TopolTool>& t){
 NCollection_Array1<double> a(1,1),b(1,1);t->UParameters(a);t->VParameters(b);
 g3_int(a.Lower());g3_int(a.Upper());for(double x:a)g3_real(x);g3_int(b.Lower());g3_int(b.Upper());for(double x:b)g3_real(x);
}
// <surface adaptor> points(x y)* tol defl numin nvmin: restrictions, vertices, classification,
// sample counts and points, SamplePnts (and BSplSamplePnts for a B-spline) and the cone apex.
void a3_topol_case(std::istream& in){
 G3Reader r{in};occ::handle<Adaptor3d_Surface> s;
 try{s=a3_surface(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 std::vector<double> pts=r.reals();double tol=r.real(),defl=r.real();int numin=r.integer(),nvmin=r.integer();
 occ::handle<Adaptor3d_TopolTool> t;
 try{t=new Adaptor3d_TopolTool(s);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 a3_topol_restrictions(t);
 for(size_t i=0;i+1<pts.size();i+=2){gp_Pnt2d p(pts[i],pts[i+1]);g3_int(t->Classify(p,tol));g3_flag(t->IsThePointOn(p,tol));}
 g3_flag(t->Has3d());g3_flag(t->DomainIsInfinite());g3_flag(t->Edge()==nullptr);G3_TRY(t->Initialize());
 G3_TRY(g3_flag(t->IsUniformSampling()));G3_TRY(a3_topol_samples(t));G3_TRY(t->ComputeSamplePoints();a3_topol_samples(t));
 G3_TRY(t->SamplePnts(defl,numin,nvmin);a3_topol_samples(t);a3_topol_parameters(t));
 if(s->GetType()==GeomAbs_BSplineSurface)G3_TRY(t->BSplSamplePnts(defl,nvmin,numin);a3_topol_samples(t);a3_topol_parameters(t));
 if(s->GetType()==GeomAbs_Cone){double u=0,v=0;Adaptor3d_TopolTool::GetConeApexParam(s->Cone(),u,v);g3_real(u);g3_real(v);}
 occ::handle<Adaptor3d_TopolTool> e=new Adaptor3d_TopolTool();e->Init();g3_flag(e->More());e->InitVertexIterator();g3_flag(e->MoreVertex());
 G3_TRY(e->Value());G3_TRY(e->Vertex());g3_int(e->Classify(gp_Pnt2d(0.,0.),tol));g3_flag(e->IsThePointOn(gp_Pnt2d(0.,0.),tol));
 g3_flag(e->DomainIsInfinite());e->Initialize(s);a3_topol_restrictions(e);std::cout<<'\n';
}
// A cone frame R angle: GetConeApexParam.
void a3_cone_apex_case(std::istream& in){
 G3Reader r{in};gp_Ax3 f=r.ax3();double radius=r.real(),angle=r.real();
 try{gp_Cone c(f,angle,radius);double u=0,v=0;Adaptor3d_TopolTool::GetConeApexParam(c,u,v);g3_real(u);g3_real(v);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}std::cout<<'\n';
}

// TKG3d oracle: surface evaluation, surface transformations, the elementary surfaces
// and the swept surfaces (tkg3d_probe.cxx).
// Summary, iso curves and dumps: <surface> uv-list iso-list iso-params.
void g3_surface_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;
 std::vector<double> uv=r.reals(),isos=r.reals(),params=r.reals();g3_name(s);g3_surface_summary(s,uv);
 g3_isos(s,isos,params);g3_dump(s,-1);g3_dump(s,1);g3_dump(s,0);std::cout<<'\n';
}
std::vector<double> g3_probe_uv;
void g3_show_surface(const occ::handle<Geom_Geometry>& g){g3_surface_summary(occ::down_cast<Geom_Surface>(g),g3_probe_uv);}
// Parameter transformations, reversal and every move: <surface> uv-list trsf moves.
void g3_surface_moves_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;g3_probe_uv=r.reals();
 gp_Trsf t=g3_trsf(r);
 for(size_t i=0;i+1<g3_probe_uv.size();i+=2){double u=g3_probe_uv[i],v=g3_probe_uv[i+1];s->TransformParameters(u,v,t);g3_real(u);g3_real(v);}
 G3_TRY(g3_gtrsf2d(s->ParametricTransformation(t)));
 G3_TRY(g3_show_surface(s->UReversed()));G3_TRY(g3_show_surface(s->VReversed()));G3_TRY(g3_show_surface(s->Copy()));
 G3_TRY(s->UReverse();g3_show_surface(s));G3_TRY(s->VReverse();g3_show_surface(s));
 g3_moves(s,r,g3_show_surface);g3_dump(s,-1);std::cout<<'\n';
}
void g3_elementary_common(const occ::handle<Geom_ElementarySurface>& e){
 g3_ax1(e->Axis());g3_pnt(e->Location());g3_ax3(e->Position());
}
void g3_coefficients10(double a1,double a2,double a3,double b1,double b2,double b3,double c1,double c2,double c3,double d){
 for(double v:{a1,a2,a3,b1,b2,b3,c1,c2,c3,d})g3_real(v);
}
// Getters, then the class setters with `a` and `b`.
void g3_elementary_specific(const occ::handle<Geom_ElementarySurface>& e,double a,double b){
 double c[10];
 if(auto p=occ::down_cast<Geom_Plane>(e)){G3_TRY(p->Coefficients(c[0],c[1],c[2],c[3]);for(int i=0;i<4;++i)g3_real(c[i]));
  g3_ax3(p->Pln().Position());p->SetPln(gp_Pln(p->Position()));}
 if(auto p=occ::down_cast<Geom_CylindricalSurface>(e)){g3_real(p->Radius());
  p->Coefficients(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9]);g3_coefficients10(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9]);
  G3_TRY(gp_Cylinder g=p->Cylinder();g3_ax3(g.Position());g3_real(g.Radius());p->SetCylinder(g));
  G3_TRY(p->SetRadius(a));g3_real(p->Radius());}
 if(auto p=occ::down_cast<Geom_ConicalSurface>(e)){g3_real(p->RefRadius());g3_real(p->SemiAngle());g3_pnt(p->Apex());
  p->Coefficients(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9]);g3_coefficients10(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9]);
  G3_TRY(gp_Cone g=p->Cone();g3_ax3(g.Position());g3_real(g.SemiAngle());p->SetCone(g));
  G3_TRY(p->SetRadius(a));G3_TRY(p->SetSemiAngle(b));g3_real(p->RefRadius());g3_real(p->SemiAngle());g3_pnt(p->Apex());}
 if(auto p=occ::down_cast<Geom_SphericalSurface>(e)){g3_real(p->Radius());g3_real(p->Area());g3_real(p->Volume());
  p->Coefficients(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9]);g3_coefficients10(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9]);
  G3_TRY(gp_Sphere g=p->Sphere();g3_ax3(g.Position());p->SetSphere(g));G3_TRY(p->SetRadius(a));g3_real(p->Radius());}
 if(auto p=occ::down_cast<Geom_ToroidalSurface>(e)){g3_real(p->MajorRadius());g3_real(p->MinorRadius());g3_real(p->Area());
  g3_real(p->Volume());NCollection_Array1<double> co(1,35),small(1,30);
  G3_TRY(p->Coefficients(co);for(int i=1;i<=35;++i)g3_real(co(i)));G3_TRY(p->Coefficients(small));
  G3_TRY(gp_Torus g=p->Torus();g3_ax3(g.Position());p->SetTorus(g));
  G3_TRY(p->SetMajorRadius(a));G3_TRY(p->SetMinorRadius(b));g3_real(p->MajorRadius());g3_real(p->MinorRadius());}
}
// Elementary members: <surface> uv-list a b axis point frame.
void g3_elementary_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;std::vector<double> uv=r.reals();
 occ::handle<Geom_ElementarySurface> e=occ::down_cast<Geom_ElementarySurface>(s);double a=r.real(),b=r.real();
 gp_Ax1 ax=r.ax1();gp_Pnt p=r.pnt();gp_Ax3 f=r.ax3();
 g3_elementary_common(e);g3_elementary_specific(e,a,b);g3_surface_summary(e,uv);
 e->SetLocation(p);g3_elementary_common(e);G3_TRY(e->SetAxis(ax));g3_elementary_common(e);
 e->SetPosition(f);g3_elementary_common(e);g3_surface_summary(e,uv);g3_dump(e,-1);std::cout<<'\n';
}
// The gp-based and the other native constructors: kind frame a b P N A B C D.
void g3_elementary_build_case(std::istream& in){
 G3Reader r{in};int kind=r.integer();gp_Ax3 f=r.ax3();double a=r.real(),b=r.real();gp_Pnt p=r.pnt();gp_Dir n=r.dir();
 double ca=r.real(),cb=r.real(),cc=r.real(),cd=r.real();occ::handle<Geom_ElementarySurface> e;
 G3_TRY(if(kind==0)e=new Geom_Plane(gp_Pln(f)));
 G3_TRY(if(kind==1)e=new Geom_CylindricalSurface(gp_Cylinder(f,a)));
 G3_TRY(if(kind==2)e=new Geom_ConicalSurface(gp_Cone(f,b,a)));
 G3_TRY(if(kind==3)e=new Geom_SphericalSurface(gp_Sphere(f,a)));
 G3_TRY(if(kind==4)e=new Geom_ToroidalSurface(gp_Torus(f,a,b)));
 G3_TRY(if(kind==5)e=new Geom_Plane(p,n));
 G3_TRY(if(kind==6)e=new Geom_Plane(ca,cb,cc,cd));
 if(!e.IsNull()){g3_elementary_common(e);g3_dump(e,-1);}
 std::cout<<'\n';
}
void g3_swept_state(const occ::handle<Geom_SweptSurface>& s,const std::vector<double>& uv,const std::vector<double>& params){
 g3_dir(s->Direction());g3_name(s->BasisCurve());g3_curve_summary(s->BasisCurve(),params);g3_int(s->Continuity());
 if(auto v=occ::down_cast<Geom_SurfaceOfRevolution>(s)){g3_ax1(v->Axis());g3_pnt(v->Location());G3_TRY(g3_ax2(v->ReferencePlane()));}
 g3_surface_summary(s,uv);g3_dump(s,-1);
}
// Swept members: <surface> uv-list params D axis P <curve>.
void g3_swept_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> surface=g3_built_surface(r);if(surface.IsNull())return;
 std::vector<double> uv=r.reals(),params=r.reals();gp_Dir d=r.dir();gp_Ax1 a=r.ax1();gp_Pnt p=r.pnt();
 occ::handle<Geom_Curve> c=g3_curve(r);occ::handle<Geom_SweptSurface> s=occ::down_cast<Geom_SweptSurface>(surface);
 g3_swept_state(s,uv,params);
 if(auto e=occ::down_cast<Geom_SurfaceOfLinearExtrusion>(s)){e->SetDirection(d);g3_swept_state(s,uv,params);
  e->SetBasisCurve(c);g3_swept_state(s,uv,params);}
 if(auto v=occ::down_cast<Geom_SurfaceOfRevolution>(s)){v->SetDirection(d);g3_swept_state(s,uv,params);
  v->SetLocation(p);g3_swept_state(s,uv,params);v->SetAxis(a);g3_swept_state(s,uv,params);
  v->SetBasisCurve(c);g3_swept_state(s,uv,params);}
 std::cout<<'\n';
}
// Evaluation representation of a swept surface: kind 0 full, 1 bounded order,
// 2 mapped order (domain flag, u1 u2 v1 v2) (su ou sv ov swap).
occ::handle<GeomEval_RepSurfaceDesc::Base> g3_surface_desc(G3Reader& r,const occ::handle<Geom_Surface>& rep){
 int kind=r.integer();
 if(kind==0){occ::handle<GeomEval_RepSurfaceDesc::Full> d=new GeomEval_RepSurfaceDesc::Full();d->Representation=rep;return d;}
 if(kind==1){occ::handle<GeomEval_RepSurfaceDesc::DerivBounded> d=new GeomEval_RepSurfaceDesc::DerivBounded();
  d->Representation=rep;d->MaxDerivOrder=size_t(r.integer());return d;}
 occ::handle<GeomEval_RepSurfaceDesc::Mapped> d=new GeomEval_RepSurfaceDesc::Mapped();d->Representation=rep;
 d->MaxDerivOrder=size_t(r.integer());bool domain=r.flag();GeomEval_RepSurfaceDesc::Domain2d dom;
 dom.UFirst=r.real();dom.ULast=r.real();dom.VFirst=r.real();dom.VLast=r.real();if(domain)d->Domain=dom;
 d->ParamMap.ScaleU=r.real();d->ParamMap.OffsetU=r.real();d->ParamMap.ScaleV=r.real();d->ParamMap.OffsetV=r.real();
 d->ParamMap.SwapUV=r.flag();return d;
}
bool g3_has_rep(const occ::handle<Geom_Surface>& s){
 if(auto e=occ::down_cast<Geom_SurfaceOfLinearExtrusion>(s))return e->HasEvalRepresentation();
 return occ::down_cast<Geom_SurfaceOfRevolution>(s)->HasEvalRepresentation();
}
void g3_set_rep(const occ::handle<Geom_Surface>& s,const occ::handle<GeomEval_RepSurfaceDesc::Base>& d){
 if(auto e=occ::down_cast<Geom_SurfaceOfLinearExtrusion>(s)){e->SetEvalRepresentation(d);return;}
 occ::down_cast<Geom_SurfaceOfRevolution>(s)->SetEvalRepresentation(d);
}
void g3_clear_rep(const occ::handle<Geom_Surface>& s){
 if(auto e=occ::down_cast<Geom_SurfaceOfLinearExtrusion>(s)){e->ClearEvalRepresentation();return;}
 occ::down_cast<Geom_SurfaceOfRevolution>(s)->ClearEvalRepresentation();
}
// <swept surface> uv-list <representation surface> <descriptor> trsf.
void g3_surface_evalrep_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;std::vector<double> uv=r.reals();
 occ::handle<Geom_Surface> rep=g3_surface(r);occ::handle<GeomEval_RepSurfaceDesc::Base> d=g3_surface_desc(r,rep);gp_Trsf t=g3_trsf(r);
 g3_flag(g3_has_rep(s));G3_TRY(g3_set_rep(s,d));g3_flag(g3_has_rep(s));g3_surface_summary(s,uv);
 occ::handle<Geom_Surface> c=occ::down_cast<Geom_Surface>(s->Copy());g3_flag(g3_has_rep(c));g3_surface_summary(c,uv);
 occ::handle<GeomEval_RepSurfaceDesc::Full> self=new GeomEval_RepSurfaceDesc::Full();self->Representation=s;
 G3_TRY(g3_set_rep(s,self));g3_flag(g3_has_rep(s));
 occ::handle<GeomEval_RepSurfaceDesc::Full> none=new GeomEval_RepSurfaceDesc::Full();G3_TRY(g3_set_rep(s,none));
 g3_clear_rep(s);g3_flag(g3_has_rep(s));G3_TRY(g3_set_rep(s,d));s->Transform(t);g3_flag(g3_has_rep(s));
 g3_surface_summary(s,uv);std::cout<<'\n';
}

// TKG3d oracle: rectangular trimmed surfaces and offset surfaces with their
// equivalent surfaces, iso approximations and osculating surfaces (tkg3d_probe.cxx).
void g3_trim_state(const occ::handle<Geom_RectangularTrimmedSurface>& t,const std::vector<double>& uv){
 g3_name(t->BasisSurface());g3_surface_summary(t,uv);g3_dump(t,-1);
}
// <trimmed surface> uv isos params u1 u2 v1 v2 usense vsense p1 p2 utrim sense trsf.
void g3_rect_trim_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;
 std::vector<double> uv=r.reals(),isos=r.reals(),params=r.reals();
 double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();bool us=r.flag(),vs=r.flag();
 double p1=r.real(),p2=r.real();bool utrim=r.flag(),sense=r.flag();gp_Trsf m=g3_trsf(r);
 occ::handle<Geom_RectangularTrimmedSurface> t=occ::down_cast<Geom_RectangularTrimmedSurface>(s);
 g3_trim_state(t,uv);G3_TRY(g3_trim_state(occ::down_cast<Geom_RectangularTrimmedSurface>(t->Copy()),uv));
 g3_isos(t,isos,params);
 G3_TRY(t->SetTrim(u1,u2,v1,v2,us,vs));g3_trim_state(t,uv);
 G3_TRY(t->SetTrim(p1,p2,utrim,sense));g3_trim_state(t,uv);
 G3_TRY(t->UReverse());g3_trim_state(t,uv);G3_TRY(t->VReverse());g3_trim_state(t,uv);
 G3_TRY(t->Transform(m));g3_trim_state(t,uv);g3_isos(t,isos,params);std::cout<<'\n';
}
void g3_offset_surface_state(const occ::handle<Geom_OffsetSurface>& o,const std::vector<double>& uv){
 g3_real(o->Offset());g3_int(o->GetBasisSurfContinuity());g3_name(o->BasisSurface());g3_flag(o->HasEvalRepresentation());
 G3_TRY(occ::handle<Geom_Surface> e=o->Surface();g3_name(e);if(!e.IsNull())g3_surface_summary(e,uv));
 g3_surface_summary(o,uv);g3_dump(o,-1);
}
// The osculating queries at each (u, v): flag, opposite flag and the surface class.
void g3_osculating(const occ::handle<Geom_OffsetSurface>& o,const std::vector<double>& uv){
 for(size_t i=0;i+1<uv.size();i+=2){
  bool t=false;occ::handle<Geom_BSplineSurface> l;
  G3_TRY(g3_flag(o->UOsculatingSurface(uv[i],uv[i+1],t,l));g3_flag(t);g3_name(l);if(!l.IsNull())g3_surface_summary(l,uv));
  t=false;l.Nullify();
  G3_TRY(g3_flag(o->VOsculatingSurface(uv[i],uv[i+1],t,l));g3_flag(t);g3_name(l);if(!l.IsNull())g3_surface_summary(l,uv));
 }
}
// <offset surface> uv isos params d notcheck trsf <basis surface>.
void g3_offset_surface_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;
 std::vector<double> uv=r.reals(),isos=r.reals(),params=r.reals();double d=r.real();bool nocheck=r.flag();gp_Trsf m=g3_trsf(r);
 occ::handle<Geom_Surface> basis;try{basis=g3_surface(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 occ::handle<Geom_OffsetSurface> o=occ::down_cast<Geom_OffsetSurface>(s);
 g3_offset_surface_state(o,uv);g3_osculating(o,uv);g3_isos(o,isos,params);
 G3_TRY(g3_offset_surface_state(occ::down_cast<Geom_OffsetSurface>(o->Copy()),uv));
 G3_TRY(o->SetOffsetValue(d));g3_offset_surface_state(o,uv);
 G3_TRY(o->UReverse());g3_offset_surface_state(o,uv);G3_TRY(o->VReverse());g3_offset_surface_state(o,uv);
 G3_TRY(o->Transform(m));g3_offset_surface_state(o,uv);g3_isos(o,isos,params);
 for(size_t i=0;i+1<uv.size();i+=2){double u=uv[i],v=uv[i+1];o->TransformParameters(u,v,m);g3_real(u);g3_real(v);}
 G3_TRY(g3_gtrsf2d(o->ParametricTransformation(m)));
 if(!basis.IsNull()){G3_TRY(o->SetBasisSurface(basis,nocheck));g3_offset_surface_state(o,uv);g3_osculating(o,uv);}
 G3_TRY(o->SetBasisSurface(o,nocheck));g3_offset_surface_state(o,uv);std::cout<<'\n';
}

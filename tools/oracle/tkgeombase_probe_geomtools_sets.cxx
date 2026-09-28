// GeomTools oracle, part 3: the three set classes, progress interruption, the debug dump
// and a custom undefined-type handler. The Rust twin is
// crates/tkgeombase/examples/gb_probe/geom_tools/sets.rs.
struct GtCurves{using Set=GeomTools_CurveSet;using H=occ::handle<Geom_Curve>;
 static H build(GbReader& r){return gt_curve(r);}static H get(const Set& s,int i){return s.Curve(i);}};
struct GtCurves2d{using Set=GeomTools_Curve2dSet;using H=occ::handle<Geom2d_Curve>;
 static H build(GbReader& r){return gt_curve2d(r);}static H get(const Set& s,int i){return s.Curve2d(i);}};
struct GtSurfaces{using Set=GeomTools_SurfaceSet;using H=occ::handle<Geom_Surface>;
 static H build(GbReader& r){return gt_surface(r);}static H get(const Set& s,int i){return s.Surface(i);}};
// `n (dup | 0 spec)...`: Add each element (dup 1 re-adds the previous one), Index, the
// out-of-range getters, Dump, Write into a precision-3 stream, then Read the text twice
// into one set (Read appends) and Clear it.
template<class K> void gt_set(GbReader& r){
 typename K::Set set;int n=r.integer(),count=0;std::vector<typename K::H> items;
 for(int i=0;i<n;++i){typename K::H h;if(r.flag())h=items.back();else h=K::build(r);
  items.push_back(h);int index=set.Add(h);count=std::max(count,index);gb_int(index);}
 for(const auto& h:items)gb_int(set.Index(h));
 gb_flag(K::get(set,0).IsNull());gb_flag(K::get(set,count+1).IsNull());
 gt_written(6,[&](std::ostream& s){set.Dump(s);});
 std::ostringstream w;w.precision(3);
 gt_text(gt_captured([&]{set.Write(w);}));gt_text(w.str());gb_int(int(w.precision()));
 typename K::Set back;
 for(int pass=0;pass<2;++pass){std::istringstream in(w.str());gt_text(gt_captured([&]{back.Read(in);}));gb_int(int(in.rdstate()));}
 for(int i=1;i<=2*count+1;++i)gt_summary(K::get(back,i));
 back.Clear();gb_flag(K::get(back,1).IsNull());}
void gb_gt_set_case(std::istream& in){
 GbReader r{in};int kind=r.integer();
 if(kind==0)gt_set<GtCurves>(r);if(kind==1)gt_set<GtCurves2d>(r);if(kind==2)gt_set<GtSurfaces>(r);gb_end();}
// Add and Index of null handles: the curve set refuses them, the others store them.
void gb_gt_set_null_case(std::istream&){
 occ::handle<Geom_Curve> c;occ::handle<Geom2d_Curve> c2;occ::handle<Geom_Surface> s;
 GeomTools_CurveSet curves;gb_int(curves.Index(c));gb_int(curves.Add(c));gb_int(curves.Index(c));gb_flag(curves.Curve(1).IsNull());
 GeomTools_Curve2dSet curves2d;gb_int(curves2d.Index(c2));gb_int(curves2d.Add(c2));gb_int(curves2d.Add(c2));gb_int(curves2d.Index(c2));
 GeomTools_SurfaceSet surfaces;gb_int(surfaces.Index(s));gb_int(surfaces.Add(s));gb_int(surfaces.Add(s));gb_int(surfaces.Index(s));
 gb_end();}
// `kind bytes`: Read of a given table text; the first six entries, then the stream.
template<class K> void gt_set_read(GbReader& r){
 std::istringstream in(gt_bytes(r));typename K::Set set;
 gt_text(gt_captured([&]{set.Read(in);}));for(int i=1;i<=6;++i)gt_summary(K::get(set,i));gt_state(in);}
void gb_gt_set_read_case(std::istream& in){
 GbReader r{in};int kind=r.integer();
 if(kind==0)gt_set_read<GtCurves>(r);if(kind==1)gt_set_read<GtCurves2d>(r);if(kind==2)gt_set_read<GtSurfaces>(r);gb_end();}
// A progress indicator that breaks once UserBreak has been asked more than `limit` times.
class GtBreaker:public Message_ProgressIndicator{
public:
 int calls=0,limit;explicit GtBreaker(int l):limit(l){}
 bool UserBreak() override{return ++calls>limit;}
 void Show(const Message_ProgressScope&,const bool) override{}
};
// `kind limit n spec...`: Write and Read of n elements under a breaking indicator.
template<class K> void gt_set_progress(GbReader& r){
 int limit=r.integer(),n=r.integer();typename K::Set set;for(int i=0;i<n;++i)set.Add(K::build(r));
 occ::handle<GtBreaker> writer=new GtBreaker(limit);std::ostringstream partial;
 set.Write(partial,writer->Start());gt_text(partial.str());gb_int(writer->calls);
 std::ostringstream full;set.Write(full);
 occ::handle<GtBreaker> reader=new GtBreaker(limit);typename K::Set back;std::istringstream in(full.str());
 back.Read(in,reader->Start());gb_int(reader->calls);
 for(int i=1;i<=n+1;++i)gt_summary(K::get(back,i));gt_state(in);}
void gb_gt_progress_case(std::istream& in){
 GbReader r{in};int kind=r.integer();
 if(kind==0)gt_set_progress<GtCurves>(r);if(kind==1)gt_set_progress<GtCurves2d>(r);if(kind==2)gt_set_progress<GtSurfaces>(r);gb_end();}
// `kind spec`: GeomTools_Dump of a curve (0), curve2d (1), surface (2), a Geom point (3)
// or a null handle (4).
void gb_gt_debug_dump_case(std::istream& in){
 GbReader r{in};int kind=r.integer();occ::handle<Standard_Transient> h;
 if(kind==0)h=gt_curve(r);if(kind==1)h=gt_curve2d(r);if(kind==2)h=gt_surface(r);if(kind==3)h=new Geom_CartesianPoint(r.pnt());
 const char* message=nullptr;std::string console=gt_captured([&]{message=GeomTools_Dump(h);});
 gt_text(message);gt_text(console);gb_end();}
// Stores the GeomEval helix (42), the Geom2dEval sine wave (43) and the ellipsoid (44).
class GtHandler:public GeomTools_UndefinedTypeHandler{
public:
 void PrintCurve(const occ::handle<Geom_Curve>& C,Standard_OStream& OS,const bool compact) const override{
  occ::handle<GeomEval_CircularHelixCurve> h=occ::down_cast<GeomEval_CircularHelixCurve>(C);
  if(h.IsNull()){GeomTools_UndefinedTypeHandler::PrintCurve(C,OS,compact);return;}
  OS<<(compact?"42 ":"Helix ")<<h->Radius()<<" "<<h->Pitch()<<"\n";}
 Standard_IStream& ReadCurve(const int ctype,Standard_IStream& IS,occ::handle<Geom_Curve>& C) const override{
  if(ctype!=42)return GeomTools_UndefinedTypeHandler::ReadCurve(ctype,IS,C);
  double radius=0.,pitch=0.;GeomTools::GetReal(IS,radius);GeomTools::GetReal(IS,pitch);
  C=new GeomEval_CircularHelixCurve(gp_Ax2(),radius,pitch);return IS;}
 void PrintCurve2d(const occ::handle<Geom2d_Curve>& C,Standard_OStream& OS,const bool compact) const override{
  occ::handle<Geom2dEval_SineWaveCurve> w=occ::down_cast<Geom2dEval_SineWaveCurve>(C);
  if(w.IsNull()){GeomTools_UndefinedTypeHandler::PrintCurve2d(C,OS,compact);return;}
  OS<<(compact?"43 ":"Wave ")<<w->Amplitude()<<" "<<w->Omega()<<" "<<w->Phase()<<"\n";}
 Standard_IStream& ReadCurve2d(const int ctype,Standard_IStream& IS,occ::handle<Geom2d_Curve>& C) const override{
  if(ctype!=43)return GeomTools_UndefinedTypeHandler::ReadCurve2d(ctype,IS,C);
  double a=0.,o=0.,p=0.;GeomTools::GetReal(IS,a);GeomTools::GetReal(IS,o);GeomTools::GetReal(IS,p);
  C=new Geom2dEval_SineWaveCurve(gp_Ax2d(),a,o,p);return IS;}
 void PrintSurface(const occ::handle<Geom_Surface>& S,Standard_OStream& OS,const bool compact) const override{
  occ::handle<GeomEval_EllipsoidSurface> e=occ::down_cast<GeomEval_EllipsoidSurface>(S);
  if(e.IsNull()){GeomTools_UndefinedTypeHandler::PrintSurface(S,OS,compact);return;}
  OS<<(compact?"44 ":"Ellipsoid ")<<e->SemiAxisA()<<" "<<e->SemiAxisB()<<" "<<e->SemiAxisC()<<"\n";}
 Standard_IStream& ReadSurface(const int ctype,Standard_IStream& IS,occ::handle<Geom_Surface>& S) const override{
  if(ctype!=44)return GeomTools_UndefinedTypeHandler::ReadSurface(ctype,IS,S);
  double a=0.,b=0.,c=0.;GeomTools::GetReal(IS,a);GeomTools::GetReal(IS,b);GeomTools::GetReal(IS,c);
  S=new GeomEval_EllipsoidSurface(gp_Ax3(),a,b,c);return IS;}
};
// `kind spec bytes`: with GtHandler installed (a null handler is ignored), the texts of a
// geometry and the geometry read from a given text; then the original handler returns.
void gb_gt_handler_case(std::istream& in){
 GbReader r{in};occ::handle<GeomTools_UndefinedTypeHandler> original=GeomTools::GetUndefinedTypeHandler();
 occ::handle<GeomTools_UndefinedTypeHandler> mine=new GtHandler;
 GeomTools::SetUndefinedTypeHandler(mine);GeomTools::SetUndefinedTypeHandler(nullptr);
 gb_flag(GeomTools::GetUndefinedTypeHandler()==mine);int kind=r.integer();
 if(kind==0){occ::handle<Geom_Curve> g=gt_curve(r);gt_texts(g);std::istringstream t(gt_bytes(r));gt_summary(GeomTools_CurveSet::ReadCurve(t));gt_state(t);}
 if(kind==1){occ::handle<Geom2d_Curve> g=gt_curve2d(r);gt_texts(g);std::istringstream t(gt_bytes(r));gt_summary(GeomTools_Curve2dSet::ReadCurve2d(t));gt_state(t);}
 if(kind==2){occ::handle<Geom_Surface> g=gt_surface(r);gt_texts(g);std::istringstream t(gt_bytes(r));gt_summary(GeomTools_SurfaceSet::ReadSurface(t));gt_state(t);}
 GeomTools::SetUndefinedTypeHandler(original);gb_flag(GeomTools::GetUndefinedTypeHandler()==original);gb_end();}

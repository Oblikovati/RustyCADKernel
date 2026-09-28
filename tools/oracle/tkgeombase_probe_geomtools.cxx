// GeomTools oracle, part 2: the text written by GeomTools and its sets, and the geometry
// read back from native or malformed text. Every text is printed as one `t<hex bytes>`
// token, so the comparison is byte exact; read-back geometry is printed as its type
// name, its compact text at precision 17 (which identifies every stored double) and the
// bits of a point on it. The `std::cout` output of GeomTools (the undefined-type lines
// and the "Not a ... table" messages) is captured and printed the same way. The Rust
// twin is crates/tkgeombase/examples/gb_probe/geom_tools/cases.rs.
#include <GeomTools.hxx>
#include <GeomTools_Curve2dSet.hxx>
#include <GeomTools_CurveSet.hxx>
#include <GeomTools_SurfaceSet.hxx>
#include <GeomTools_UndefinedTypeHandler.hxx>
#include <Geom_CartesianPoint.hxx>
#include <Message_ProgressIndicator.hxx>
#include <Message_ProgressScope.hxx>
#include <OSD.hxx>
#include "tkgeombase_probe_geomtools_build.cxx"
// The readers run under OCC_CATCH_SIGNALS: with the OSD handlers installed, the access
// violation of a trimmed or offset curve built on a null basis becomes the
// Standard_Failure the reader catches, as in any OCCT application (DRAW installs them).
static const bool gt_signals_installed=(OSD::SetSignal(false),true);
// GeomTools_Debug.cxx defines it without a header.
const char* GeomTools_Dump(const occ::handle<Standard_Transient>& theGeom);
void gt_text(const std::string& text){
 static const char digits[]="0123456789abcdef";std::cout<<'t';
 for(unsigned char c:text)std::cout<<digits[c>>4]<<digits[c&15];std::cout<<' ';}
// Runs `body` with std::cout redirected, and returns what it printed there.
template<class F> std::string gt_captured(F body){
 std::ostringstream console;std::streambuf* old=std::cout.rdbuf(console.rdbuf());
 try{body();}catch(...){std::cout.rdbuf(old);throw;}
 std::cout.rdbuf(old);return console.str();}
// `count byte...`: a text given as byte codes.
std::string gt_bytes(GbReader& r){int n=r.integer();std::string s;for(int i=0;i<n;++i)s+=char(r.integer());return s;}
// The stream state after a read, then what is left unread.
void gt_state(std::istream& in){gb_int(int(in.rdstate()));in.clear();std::string rest;char c;while(in.get(c))rest+=c;gt_text(rest);}
// A read-back curve, curve2d or surface: type name, compact text at precision 17, a point.
void gt_summary(const occ::handle<Geom_Curve>& c){
 if(c.IsNull()){std::cout<<"null ";return;}
 std::cout<<c->DynamicType()->Name()<<' ';std::ostringstream s;s.precision(17);
 std::string console=gt_captured([&]{GB_TRY(GeomTools_CurveSet::PrintCurve(c,s,true));});
 gt_text(s.str());gt_text(console);GB_TRY(gb_pnt(c->Value(0.3)));}
void gt_summary(const occ::handle<Geom2d_Curve>& c){
 if(c.IsNull()){std::cout<<"null ";return;}
 std::cout<<c->DynamicType()->Name()<<' ';std::ostringstream s;s.precision(17);
 std::string console=gt_captured([&]{GB_TRY(GeomTools_Curve2dSet::PrintCurve2d(c,s,true));});
 gt_text(s.str());gt_text(console);GB_TRY(gb_pnt2d(c->Value(0.3)));}
void gt_summary(const occ::handle<Geom_Surface>& c){
 if(c.IsNull()){std::cout<<"null ";return;}
 std::cout<<c->DynamicType()->Name()<<' ';std::ostringstream s;s.precision(17);
 std::string console=gt_captured([&]{GB_TRY(GeomTools_SurfaceSet::PrintSurface(c,s,true));});
 gt_text(s.str());gt_text(console);GB_TRY(gb_pnt(c->Value(0.3,0.2)));}
// One text written by `print` into a stream of the given precision, with its console output.
template<class F> std::string gt_written(int precision,F print){
 std::ostringstream s;s.precision(precision);
 std::string console=gt_captured([&]{GB_TRY(print(s));});gt_text(s.str());gt_text(console);return s.str();}
// The four texts of one geometry (Dump and Write at the default precision, then the
// dump and the compact form at 17), then the compact text read back when there is one.
template<class H> void gt_texts(const H& g){
 gt_written(6,[&](std::ostream& s){GeomTools::Dump(g,s);});
 gt_written(6,[&](std::ostream& s){GeomTools::Write(g,s);});
 gt_written(17,[&](std::ostream& s){GeomTools::Dump(g,s);});
 std::string compact=gt_written(17,[&](std::ostream& s){GeomTools::Write(g,s);});
 if(compact.empty())return;
 std::istringstream in(compact);H back;
 gt_text(gt_captured([&]{GeomTools::Read(back,in);}));gt_summary(back);gt_state(in);}
void gb_gt_curve_case(std::istream& in){
 GbReader r{in};occ::handle<Geom_Curve> c;GB_TRY(c=gt_curve(r));
 if(!c.IsNull())gt_texts(c);gb_end();}
void gb_gt_curve2d_case(std::istream& in){
 GbReader r{in};occ::handle<Geom2d_Curve> c;GB_TRY(c=gt_curve2d(r));
 if(!c.IsNull())gt_texts(c);gb_end();}
void gb_gt_surface_case(std::istream& in){
 GbReader r{in};occ::handle<Geom_Surface> c;GB_TRY(c=gt_surface(r));
 if(!c.IsNull())gt_texts(c);gb_end();}
// `kind bytes`: ReadCurve (0), ReadCurve2d (1) or ReadSurface (2) on a given text.
void gb_gt_read_case(std::istream& in){
 GbReader r{in};int kind=r.integer();std::istringstream text(gt_bytes(r));std::string console;
 if(kind==0){occ::handle<Geom_Curve> c;console=gt_captured([&]{c=GeomTools_CurveSet::ReadCurve(text);});gt_summary(c);}
 if(kind==1){occ::handle<Geom2d_Curve> c;console=gt_captured([&]{c=GeomTools_Curve2dSet::ReadCurve2d(text);});gt_summary(c);}
 if(kind==2){occ::handle<Geom_Surface> c;console=gt_captured([&]{c=GeomTools_SurfaceSet::ReadSurface(text);});gt_summary(c);}
 gt_text(console);gt_state(text);gb_end();}
// `count bytes`: `count` GetReal calls, each value's bits and the stream state after it.
void gb_gt_get_real_case(std::istream& in){
 GbReader r{in};int n=r.integer();std::istringstream text(gt_bytes(r));
 for(int i=0;i<n;++i){double v=-1.;GeomTools::GetReal(text,v);gb_real(v);gb_int(int(text.rdstate()));}
 gt_state(text);gb_end();}
#include "tkgeombase_probe_geomtools_sets.cxx"

// Shared reader and printers of the TKGeomAlgo oracle (tkgeomalgo_probe.cxx). Every double
// is printed as raw IEEE bits, every raised exception as its type name, as in the other
// toolkit oracles; the Rust twin is crates/tkgeomalgo/examples/ga_probe/common.rs.
#define GA_TRY(...) try{__VA_ARGS__;}catch(const Standard_Failure& ga_error){std::cout<<ga_error.ExceptionType()<<' ';}
uint64_t ga_bits(double value){uint64_t result;std::memcpy(&result,&value,8);return result;}
void ga_real(double v){std::cout<<ga_bits(v)<<' ';}
void ga_int(int v){std::cout<<v<<' ';}
void ga_flag(bool v){std::cout<<(v?1:0)<<' ';}
void ga_end(){std::cout<<'\n';}
void ga_xyz(const gp_XYZ& v){ga_real(v.X());ga_real(v.Y());ga_real(v.Z());}
void ga_pnt(const gp_Pnt& p){ga_xyz(p.XYZ());}
void ga_dir(const gp_Dir& d){ga_xyz(d.XYZ());}
void ga_xy(const gp_XY& v){ga_real(v.X());ga_real(v.Y());}
void ga_pnt2d(const gp_Pnt2d& p){ga_xy(p.XY());}
// A text as one `t<hex bytes>` token, as the TKBRep oracle prints texts.
void ga_text(const std::string& text){
 static const char digits[]="0123456789abcdef";std::cout<<'t';
 for(unsigned char c:text)std::cout<<digits[c>>4]<<digits[c&15];std::cout<<' ';}
// Runs `body` with std::cout captured, then prints the captured text and the raised
// exception type, if any (a native member may write part of its text, then raise).
template<class F> void ga_captured(F body){
 std::ostringstream console;std::streambuf* old=std::cout.rdbuf(console.rdbuf());std::string error;
 try{body();}catch(const Standard_Failure& failure){error=failure.ExceptionType();}
 std::cout.rdbuf(old);ga_text(console.str());if(!error.empty())std::cout<<error<<' ';}
// Sequential reader over one request; the corpus only feeds it valid directions.
struct GaReader{
 std::istream& in;
 double real(){double v;in>>v;return v;}
 int integer(){return int(real());}
 bool flag(){return integer()!=0;}
 gp_XYZ xyz(){double x=real(),y=real();return gp_XYZ(x,y,real());}
 gp_Pnt pnt(){double x=real(),y=real();return gp_Pnt(x,y,real());}
 gp_Dir dir(){double x=real(),y=real();return gp_Dir(x,y,real());}
 gp_Pnt2d pnt2d(){double x=real();return gp_Pnt2d(x,real());}
 gp_Dir2d dir2d(){double x=real();return gp_Dir2d(x,real());}
 std::vector<double> reals(){int n=integer();std::vector<double> v(n);for(double& x:v)x=real();return v;}
};

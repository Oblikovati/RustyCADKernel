// Shared printers and readers of the TKBRep oracle (tkbrep_probe.cxx). Every double is
// printed as raw IEEE bits, every raised exception as its type name, every text as one
// `t<hex bytes>` token, as in the other toolkit oracles; the Rust twin is
// crates/tkbrep/examples/br_probe/common.rs.
#define BR_TRY(...) try{__VA_ARGS__;}catch(const Standard_Failure& br_error){std::cout<<br_error.ExceptionType()<<' ';}
uint64_t br_bits(double value){uint64_t result;std::memcpy(&result,&value,8);return result;}
void br_real(double v){std::cout<<br_bits(v)<<' ';}
void br_int(long long v){std::cout<<v<<' ';}
void br_flag(bool v){std::cout<<(v?1:0)<<' ';}
void br_word(const char* w){std::cout<<w<<' ';}
void br_end(){std::cout<<'\n';}
void br_text(const std::string& text){
 static const char digits[]="0123456789abcdef";std::cout<<'t';
 for(unsigned char c:text)std::cout<<digits[c>>4]<<digits[c&15];std::cout<<' ';}
// Object addresses differ between runs and implementations: every `0x` pointer text
// (the libstdc++ `0x55...` and the Standard_Dump short form `0xx55...`) becomes `0xP`.
std::string br_masked(const std::string& text){
 std::string out;size_t i=0;
 while(i<text.size()){
  if(text.compare(i,2,"0x")!=0){out+=text[i++];continue;}
  out+="0xP";i+=2;
  while(i<text.size()&&(std::isxdigit((unsigned char)text[i])||text[i]=='x'))++i;}
 return out;}
// Runs `body` with std::cout redirected, and returns what it printed there.
template<class F> std::string br_captured(F body){
 std::ostringstream console;std::streambuf* old=std::cout.rdbuf(console.rdbuf());
 try{body();}catch(...){std::cout.rdbuf(old);throw;}
 std::cout.rdbuf(old);return console.str();}
// Sequential reader over one request.
struct BrReader{
 std::istream& in;
 double real(){double v;in>>v;return v;}
 int integer(){return int(real());}
 bool flag(){return integer()!=0;}
 // `count byte...`: a text given as byte codes.
 std::string bytes(){int n=integer();std::string s;for(int i=0;i<n;++i)s+=char(integer());return s;}
};

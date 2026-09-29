// BinTools oracle, part 1: the shared printers, the scalar encodings of the static
// BinTools members and the BinTools_OStream / BinTools_IStream primitives. Byte strings
// are printed as `t<hex>` tokens (a string longer than 4096 bytes as its length and
// FNV-1a hash), failures as their type and, for a plain Standard_Failure, their message
// with the text of every inner failure of another class masked (other kernels word
// those messages differently). The Rust twin is crates/tkbrep/examples/br_probe/bintools.rs.
#include <BinTools.hxx>
#include <BinTools_IStream.hxx>
#include <BinTools_OStream.hxx>
#include <gp_Ax1.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt2d.hxx>
// A byte string: its hex text, or `h <length> <fnv1a64>` when it is long.
void bt_bytes(const std::string& bytes){
 if(bytes.size()<=4096){br_text(bytes);return;}
 uint64_t hash=1469598103934665603ULL;
 for(unsigned char c:bytes){hash^=c;hash*=1099511628211ULL;}
 br_word("h");br_int((long long)bytes.size());std::cout<<hash<<' ';}
// One message line `Type: text` keeps only `Type` when Type names another exception
// class than Standard_Failure (letters, digits and underscores with one underscore).
std::string bt_mask_line(const std::string& line){
 size_t colon=line.find(": ");
 if(colon==std::string::npos)return line;
 std::string prefix=line.substr(0,colon);
 bool name=!prefix.empty()&&prefix.find('_')!=std::string::npos;
 for(char c:prefix)if(!std::isalnum((unsigned char)c)&&c!='_')name=false;
 if(!name||prefix=="Standard_Failure")return line;
 return prefix;}
std::string bt_masked_message(const std::string& text){
 std::string out;size_t start=0;
 while(true){size_t end=text.find('\n',start);
  if(end==std::string::npos){out+=bt_mask_line(text.substr(start));break;}
  out+=bt_mask_line(text.substr(start,end-start))+"\n";start=end+1;}
 return out;}
// A byte from 0x80 on becomes `?`: the Rust messages are UTF-8 text and cannot hold the
// raw byte some messages insert (`aMsg << (uint8_t)value`).
std::string bt_ascii(std::string text){for(char& c:text)if((unsigned char)c>=0x80)c='?';return text;}
void bt_failure_parts(const std::string& type,const std::string& message){
 br_word(type.c_str());
 if(type=="Standard_Failure")br_text(bt_ascii(bt_masked_message(message)));}
void bt_failure(const Standard_Failure& e){bt_failure_parts(e.ExceptionType(),e.GetMessageString());}
#define BT_TRY(...) try{__VA_ARGS__;}catch(const Standard_Failure& bt_error){bt_failure(bt_error);}
std::string bt_read_bytes(BrReader& r){
 int zeros=r.integer();std::string s(size_t(std::max(zeros,0)),'\0');s+=r.bytes();return s;}
// `br_bin_scalars n value...`: every value as PutReal, PutShortReal, PutInteger, PutBool
// and PutExtChar; the bytes; the values read back; then GetReal and GetBool past the end.
void br_bin_scalars_case(std::istream& in){
 BrReader r{in};int n=r.integer();std::vector<double> values;for(int i=0;i<n;++i)values.push_back(r.real());
 std::ostringstream o;
 for(double v:values){BinTools::PutReal(o,v);BinTools::PutShortReal(o,float(v));BinTools::PutInteger(o,int(v));
  BinTools::PutBool(o,v!=0.0);BinTools::PutExtChar(o,char16_t(int(v)));}
 bt_bytes(o.str());
 std::istringstream is(o.str());
 for(size_t i=0;i<values.size();++i){
  double d;float f;int k;bool b;char16_t c;
  BinTools::GetReal(is,d);BinTools::GetShortReal(is,f);BinTools::GetInteger(is,k);BinTools::GetBool(is,b);BinTools::GetExtChar(is,c);
  br_real(d);br_real(double(f));br_int(k);br_flag(b);br_int(int(c));}
 double d=0;BT_TRY(BinTools::GetReal(is,d));
 bool b=false;BinTools::GetBool(is,b);br_flag(b);br_int(int(is.rdstate()));
 float f;int k;char16_t c;
 std::istringstream e1("abc");BT_TRY(BinTools::GetShortReal(e1,f));
 std::istringstream e2("ab");BT_TRY(BinTools::GetInteger(e2,k));
 std::istringstream e3("a");BT_TRY(BinTools::GetExtChar(e3,c));
 br_end();}
// `br_bin_ostream prefix n (op args)...`: BinTools_OStream after `prefix` zero bytes; ops:
// 0 reference p, 1 shape type orientation, 2 object type t, 3 byte b, 4 real x, 5 bool b,
// 6 int i, 7 ext char c, 8 point xyz, 9 direction xyz, 10 point2d xy, 11 direction2d xy,
// 12 trsf (translation xyz, rotation axis xyz, angle, scale), 13 triangle abc,
// 14 float vector xyz, 15 three bools, 16 seven bools, 17 n zero bytes. The position
// after each op, then the bytes.
void bt_ostream_op(BinTools_OStream& s,BrReader& r,int op){
 switch(op){
  case 0:{uint64_t p=uint64_t(r.real());s.WriteReference(p);break;}
  case 1:{int t=r.integer();s.WriteShape(TopAbs_ShapeEnum(t),TopAbs_Orientation(r.integer()));break;}
  case 2:s<<BinTools_ObjectType(r.integer());break;
  case 3:s<<uint8_t(r.integer());break;
  case 4:s<<r.real();break;
  case 5:s<<bool(r.flag());break;
  case 6:s<<int(r.integer());break;
  case 7:s<<char16_t(r.integer());break;
  case 8:s<<br_point(r);break;
  case 9:s<<br_direction(r);break;
  case 10:s<<br_point2d(r);break;
  case 11:{double x=r.real();s<<gp_Dir2d(x,r.real());break;}
  case 12:{gp_Trsf t,rot,sc;double x=r.real(),y=r.real(),z=r.real();t.SetTranslation(gp_Vec(x,y,z));
   gp_Dir axis=br_direction(r);rot.SetRotation(gp_Ax1(gp_Pnt(0,0,0),axis),r.real());sc.SetScale(gp_Pnt(0,0,0),r.real());
   s<<t*rot*sc;break;}
  case 13:{int a=r.integer(),b=r.integer();s<<Poly_Triangle(a,b,r.integer());break;}
  case 14:{float x=float(r.real()),y=float(r.real());s<<NCollection_Vec3<float>(x,y,float(r.real()));break;}
  case 15:{bool a=r.flag(),b=r.flag();s.PutBools(a,b,r.flag());break;}
  case 16:{bool v[7];for(bool& x:v)x=r.flag();s.PutBools(v[0],v[1],v[2],v[3],v[4],v[5],v[6]);break;}
  default:{int n=r.integer();for(int i=0;i<n;++i)s<<uint8_t(0);break;}}}
void br_bin_ostream_case(std::istream& in){
 BrReader r{in};std::ostringstream o;int prefix=r.integer();for(int i=0;i<prefix;++i)o.put(0);
 BinTools_OStream s(o);std::cout<<s.Position()<<' ';
 int n=r.integer();
 for(int i=0;i<n;++i){BT_TRY(bt_ostream_op(s,r,r.integer()));std::cout<<s.Position()<<' ';}
 bt_bytes(o.str());br_end();}
// `br_bin_istream zeros bytes n op...`: BinTools_IStream over `zeros` zero bytes and the
// given bytes; ops: 0 ReadType, 1 ShapeType and ShapeOrientation, 2 IsReference,
// 3 ReadReference, 4 GoTo p, 5 ReadReal, 6 ReadInteger, 7 ReadPnt, 8 ReadByte,
// 9 ReadBool, 10 ReadShortReal, 11 gp_Trsf, 12 ReadBools(3), 13 ReadBools(7),
// 14 UpdatePosition, 15 operator bool, 16 LastType. Each prints its result, the
// position and the stream state.
void bt_istream_op(BinTools_IStream& s,BrReader& r,int op){
 switch(op){
  case 0:br_int(int(s.ReadType()));break;
  case 1:br_int(int(s.ShapeType()));br_int(int(s.ShapeOrientation()));break;
  case 2:br_flag(s.IsReference());break;
  case 3:std::cout<<s.ReadReference()<<' ';break;
  case 4:s.GoTo(uint64_t(r.real()));break;
  case 5:br_real(s.ReadReal());break;
  case 6:br_int(s.ReadInteger());break;
  case 7:{gp_Pnt p=s.ReadPnt();br_real(p.X());br_real(p.Y());br_real(p.Z());break;}
  case 8:br_int(int(s.ReadByte()));break;
  case 9:br_flag(s.ReadBool());break;
  case 10:br_real(double(s.ReadShortReal()));break;
  case 11:{gp_Trsf t;s>>t;for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)br_real(t.Value(i,j));break;}
  case 12:{bool a,b,c;s.ReadBools(a,b,c);br_flag(a);br_flag(b);br_flag(c);break;}
  case 13:{bool v[7];s.ReadBools(v[0],v[1],v[2],v[3],v[4],v[5],v[6]);for(bool x:v)br_flag(x);break;}
  case 14:s.UpdatePosition();break;
  case 15:br_flag(bool(s));break;
  default:br_int(int(s.LastType()));break;}}
void br_bin_istream_case(std::istream& in){
 BrReader r{in};std::string bytes=bt_read_bytes(r);std::istringstream is(bytes);
 BinTools_IStream s(is);std::cout<<s.Position()<<' ';
 int n=r.integer();
 for(int i=0;i<n;++i){BT_TRY(bt_istream_op(s,r,r.integer()));std::cout<<s.Position()<<' ';br_int(int(is.rdstate()));}
 br_end();}

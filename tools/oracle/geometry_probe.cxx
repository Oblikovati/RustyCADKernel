// Differential harness for OCCT's dump parser and OCC25545 dependency path.
#include <Standard_Dump.hxx>
#include <Standard_Failure.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <Bnd_Box.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopLoc_Location.hxx>
#include <sstream>
#include <iostream>
#include <cstring>
#include <cstdint>
std::string unhex(const std::string& s) {
  if (s == "-") return "";
  std::string result;
  for (size_t i=0;i<s.size();i+=2) result.push_back(char(std::stoul(s.substr(i,2),nullptr,16)));
  return result;
}
std::string hex(const std::string& s) {
  if(s.empty()) return "-";
  const char* digits="0123456789abcdef";std::string out;
  for(unsigned char c:s){out+=digits[c>>4];out+=digits[c&15];} return out;
}
uint64_t bits(double v){ uint64_t n; std::memcpy(&n,&v,8);return n; }
void parse(const std::string& op, std::istringstream& input) {
  std::string encoded; int pos;input>>pos>>encoded;
  std::string text=unhex(encoded); TCollection_AsciiString source(text.c_str());
  double x=91,y=92,z=93; bool ok=false; std::string result;
  try {
    if(op=="vector") ok=Standard_Dump::InitRealValues(source,pos,3,&x,&y,&z);
    else if(op=="key") ok=Standard_Dump::ProcessStreamName(source,"gp_Pnt",pos);
    else if(op=="point") { Standard_SStream stream;stream<<text;gp_Pnt p(x,y,z);
      try {ok=p.InitFromJson(stream,pos);} catch(...) {x=p.X();y=p.Y();z=p.Z();throw;} x=p.X();y=p.Y();z=p.Z(); }
    else { TCollection_AsciiString value;ok=Standard_Dump::InitValue(source,pos,value);result=value.ToCString(); }
    std::cout<<"ok "<<ok<<" "<<hex(result);
  } catch(const Standard_Failure& error) {std::cout<<"err "<<error.ExceptionType()<<" "<<hex(error.what());}
  std::cout<<" "<<pos<<" "<<bits(x)<<" "<<bits(y)<<" "<<bits(z)<<"\n";
}
void geometry(const std::string& op, std::istringstream& input) {
  double x,y,z;input>>x>>y>>z;
  if(op=="dump") {Standard_SStream stream;gp_Pnt(x,y,z).DumpJson(stream);std::cout<<hex(stream.str())<<"\n";return;}
  int power; input>>power;gp_Trsf trsf;trsf.SetTranslation(gp_Vec(x,y,z)); trsf.Power(power);
  TopoDS_Shape shape=BRepBuilderAPI_MakeVertex(gp_Pnt(0.125,-0.25,0.5));
  TopoDS_Vertex vertex=TopoDS::Vertex(shape.Moved(TopLoc_Location(trsf)));
  gp_Pnt result=BRep_Tool::Pnt(vertex);
  std::cout<<bits(result.X())<<" "<<bits(result.Y())<<" "<<bits(result.Z())<<"\n";
}
int main() {std::string line;while(std::getline(std::cin,line)) {std::istringstream in(line);std::string op;in>>op;
 if(op=="dump"||op=="translation") geometry(op,in);else parse(op,in);}}

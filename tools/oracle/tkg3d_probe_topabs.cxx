// TKG3d oracle: the TopAbs package functions (tkg3d_probe.cxx).
#include <TopAbs.hxx>
// g3_topabs: Compose for every ordered pair, Reverse and Complement of every orientation,
// then every name through ToString and Print (TopAbs_State only has Print).
void g3_topabs_case(std::istream&){
 for(int a=0;a<4;++a)for(int b=0;b<4;++b)g3_int(TopAbs::Compose(TopAbs_Orientation(a),TopAbs_Orientation(b)));
 for(int a=0;a<4;++a){g3_int(TopAbs::Reverse(TopAbs_Orientation(a)));g3_int(TopAbs::Complement(TopAbs_Orientation(a)));}
 for(int t=0;t<=TopAbs_SHAPE;++t){std::ostringstream s;TopAbs::Print(TopAbs_ShapeEnum(t),s);
  std::cout<<TopAbs::ShapeTypeToString(TopAbs_ShapeEnum(t))<<' '<<s.str()<<' ';}
 for(int o=0;o<4;++o){std::ostringstream s;TopAbs::Print(TopAbs_Orientation(o),s);
  std::cout<<TopAbs::ShapeOrientationToString(TopAbs_Orientation(o))<<' '<<s.str()<<' ';}
 for(int st=0;st<4;++st){std::ostringstream s;TopAbs::Print(TopAbs_State(st),s);std::cout<<s.str()<<' ';}
 std::cout<<'\n';
}
// g3_topabs_parse n byte*n: the string of those bytes (valid UTF-8, NUL allowed) through
// both FromString overloads; the out-parameter starts at WIRE (REVERSED) to show that an
// unknown name leaves it unchanged.
void g3_topabs_parse_case(std::istream& in){
 G3Reader r{in};int n=r.integer();std::string text;for(int i=0;i<n;++i)text.push_back(char(r.integer()));
 g3_int(TopAbs::ShapeTypeFromString(text.c_str()));
 TopAbs_ShapeEnum type=TopAbs_WIRE;g3_flag(TopAbs::ShapeTypeFromString(text.c_str(),type));g3_int(type);
 g3_int(TopAbs::ShapeOrientationFromString(text.c_str()));
 TopAbs_Orientation orientation=TopAbs_REVERSED;
 g3_flag(TopAbs::ShapeOrientationFromString(text.c_str(),orientation));g3_int(orientation);
 std::cout<<'\n';
}

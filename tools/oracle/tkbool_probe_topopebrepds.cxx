// TopOpeBRepDS probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/top_ope_brep_ds.rs.
#include <TopOpeBRepDS.hxx>
#include <TopOpeBRepDS_Transition.hxx>
// The states, shapes and indices of a transition.
static void bl_transition(const TopOpeBRepDS_Transition& t){
 ga_int((int)t.Before());ga_int((int)t.After());ga_int((int)t.ShapeBefore());ga_int((int)t.ShapeAfter());
 ga_int(t.IndexBefore());ga_int(t.IndexAfter());}
// `sb sa onb ona ib ia o`: a TopOpeBRepDS_Transition of the states, shapes and indices;
// prints its Orientation seen from each state, Complement, IsUnknown, Index and the
// transition of the orientation `o` (Set(O) on a copy).
void bl_ds_transition_case(std::istream& input){
 GaReader r{input};
 TopAbs_State sb=(TopAbs_State)r.integer(),sa=(TopAbs_State)r.integer();
 TopAbs_ShapeEnum onb=(TopAbs_ShapeEnum)r.integer(),ona=(TopAbs_ShapeEnum)r.integer();
 int ib=r.integer(),ia=r.integer();TopAbs_Orientation o=(TopAbs_Orientation)r.integer();
 TopOpeBRepDS_Transition t(sb,sa,onb,ona);t.IndexBefore(ib);t.IndexAfter(ia);
 for(int s=0;s<4;s++)ga_int((int)t.Orientation((TopAbs_State)s));
 bl_transition(t.Complement());ga_flag(t.IsUnknown());
 GA_TRY(ga_int(t.Index()))
 TopOpeBRepDS_Transition c=t;c.Set(o);bl_transition(c);bl_transition(TopOpeBRepDS_Transition(o));
 ga_end();}
// `k i`: the package names of the kind and of the shape type of the same index, IsGeometry,
// IsTopology, KindToShape and ShapeToKind (when the index is a shape type).
void bl_ds_kind_case(std::istream& input){
 GaReader r{input};int k=r.integer(),i=r.integer();
 TopOpeBRepDS_Kind kind=(TopOpeBRepDS_Kind)k;
 ga_text(TopOpeBRepDS::SPrint(kind).ToCString());ga_text(TopOpeBRepDS::SPrint(kind,i,"<",">").ToCString());
 ga_flag(TopOpeBRepDS::IsGeometry(kind));ga_flag(TopOpeBRepDS::IsTopology(kind));
 GA_TRY(ga_int((int)TopOpeBRepDS::KindToShape(kind)))
 if(k<=8){GA_TRY(ga_int((int)TopOpeBRepDS::ShapeToKind((TopAbs_ShapeEnum)k)))
  GA_TRY(ga_text(TopOpeBRepDS::SPrint((TopAbs_ShapeEnum)k,i).ToCString()))}
 if(k<4){ga_text(TopOpeBRepDS::SPrint((TopAbs_State)k).ToCString());ga_text(TopOpeBRepDS::SPrint((TopAbs_Orientation)k).ToCString());}
 if(k<3)ga_text(TopOpeBRepDS::SPrint((TopOpeBRepDS_Config)k).ToCString());
 ga_end();}

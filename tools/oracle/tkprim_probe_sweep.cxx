// Sweep probe operations; the Rust twin is crates/tkprim/examples/pr_probe/sweep.rs.
// A directing shape is read as `index type closed begInf endInf` (type: TopAbs_ShapeEnum).
#include <Sweep_NumShape.hxx>
#include <Sweep_NumShapeIterator.hxx>
#include <Sweep_NumShapeTool.hxx>
Sweep_NumShape pr_num_shape(GaReader& r){
 int index=r.integer();int type=r.integer();bool closed=r.flag(),beg=r.flag();
 return Sweep_NumShape(index,TopAbs_ShapeEnum(type),closed,beg,r.flag());}
// Every accessor of a Sweep_NumShape.
void pr_print_num_shape(const Sweep_NumShape& s){
 ga_int(s.Index());ga_int(s.Type());ga_flag(s.Closed());ga_flag(s.BegInfinite());ga_flag(s.EndInfinite());
 ga_int(s.Orientation());}
// The iteration from the current state: up to 5 steps of More, Value and Orientation.
void pr_iterate(Sweep_NumShapeIterator& it){
 for(int step=0;step<5;++step){ga_flag(it.More());if(!it.More())return;
  pr_print_num_shape(it.Value());ga_int(it.Orientation());it.Next();}}
// `shape`: the accessors, the tool (NbShapes, Index, Shape over 0..NbShapes+1 with
// their indices and types, the first and last vertices) and the iteration.
void pr_sw_numshape_case(std::istream& input){
 GaReader r{input};Sweep_NumShape s=pr_num_shape(r);
 pr_print_num_shape(s);Sweep_NumShapeTool tool(s);
 ga_int(tool.NbShapes());ga_int(tool.Index(s));ga_int(tool.Type(s));ga_int(tool.Orientation(s));
 for(int i=0;i<=tool.NbShapes()+1;++i){Sweep_NumShape sub=tool.Shape(i);pr_print_num_shape(sub);ga_int(tool.Index(sub));}
 ga_flag(tool.HasFirstVertex());ga_flag(tool.HasLastVertex());
 GA_TRY(pr_print_num_shape(tool.FirstVertex()))
 GA_TRY(pr_print_num_shape(tool.LastVertex()))
 Sweep_NumShapeIterator it;it.Init(s);pr_iterate(it);
 ga_end();}
// `first second steps`: Init on `first`, `steps` Next calls, then Init on `second` (a
// non-edge keeps the state of the first iteration) and the iteration from there.
void pr_sw_reinit_case(std::istream& input){
 GaReader r{input};Sweep_NumShape first=pr_num_shape(r);Sweep_NumShape second=pr_num_shape(r);int steps=r.integer();
 Sweep_NumShapeIterator it;it.Init(first);for(int i=0;i<steps;++i)it.Next();
 it.Init(second);ga_flag(it.More());pr_print_num_shape(it.Value());pr_iterate(it);
 ga_end();}
// `shape new`: Init of an existing shape, then the accessors, and the default constructor.
void pr_sw_init_case(std::istream& input){
 GaReader r{input};Sweep_NumShape s=pr_num_shape(r);Sweep_NumShape n=pr_num_shape(r);
 s.Init(n.Index(),n.Type(),n.Closed(),n.BegInfinite(),n.EndInfinite());pr_print_num_shape(s);
 pr_print_num_shape(Sweep_NumShape());
 Sweep_NumShapeIterator it;ga_flag(it.More());pr_print_num_shape(it.Value());
 ga_end();}

// BRepAlgo_NormalProjection and BRepProj_Projection probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_proj.rs.
//
// The shapes are shapes of a shape program (tktopalgo_probe_shapes.cxx); shapes print
// through pr_shape (tkprim_probe_shape.cxx).
#include <BRepAlgo_NormalProjection.hxx>
#include <BRepProj_Projection.hxx>
// The index of a shape among the program shapes (IsSame), -1 for none.
static int bl_program_index(const TpShapes& t,const TopoDS_Shape& s){
 for(size_t i=0;i<t.shapes.size();i++)if(t.shapes[i].IsSame(s))return (int)i;return -1;}
// `program shape n (edge)... maxdist with3d limit`: a BRepAlgo_NormalProjection of the
// edges on the shape; prints IsDone, the projection, the ancestor and couple of each
// projected edge, the count of Generated of each edge and BuildWire.
void bl_ba_normal_projection_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 BRepAlgo_NormalProjection np(t.shapes.at(r.integer()));
 int n=r.integer();std::vector<TopoDS_Shape> edges;
 for(int i=0;i<n;i++){edges.push_back(t.shapes.at(r.integer()));np.Add(edges.back());}
 double maxdist=r.real();bool with3d=r.flag(),limit=r.flag();
 np.SetMaxDistance(maxdist);np.Compute3d(with3d);np.SetLimit(limit);
 try{np.Build();}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(np.IsDone());
 GA_TRY(pr_shape(np.Projection()))
 for(TopExp_Explorer ex(np.Projection(),TopAbs_EDGE);ex.More();ex.Next()){
  const TopoDS_Edge& e=TopoDS::Edge(ex.Current());
  GA_TRY(ga_int(bl_program_index(t,np.Ancestor(e))))
  GA_TRY(ga_int(bl_program_index(t,np.Couple(e))))}
 for(const TopoDS_Shape& e:edges)GA_TRY(ga_int(np.Generated(e).Extent()))
 GA_TRY(NCollection_List<TopoDS_Shape> wires;ga_flag(np.BuildWire(wires));ga_int(wires.Extent()))
 ga_end();}
// `program wire shape mode x y z`: a BRepProj_Projection of the wire on the shape along
// the direction (mode 0) or from the point (mode 1); prints IsDone, the compound and
// each wire of the iteration.
void bl_bp_projection_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape w=t.shapes.at(r.integer()),s=t.shapes.at(r.integer());int mode=r.integer();
 double x=r.real(),y=r.real(),z=r.real();
 std::unique_ptr<BRepProj_Projection> p;
 try{if(mode==0)p.reset(new BRepProj_Projection(w,s,gp_Dir(x,y,z)));else p.reset(new BRepProj_Projection(w,s,gp_Pnt(x,y,z)));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(p->IsDone());
 GA_TRY(pr_shape(p->Shape()))
 int n=0;for(p->Init();p->More();p->Next())n++;ga_int(n);
 for(p->Init();p->More();p->Next())GA_TRY(pr_shape(p->Current()))
 ga_end();}

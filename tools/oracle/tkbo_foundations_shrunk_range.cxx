#include <IntTools_ShrunkRange.hxx>
#include <TopExp.hxx>
void bo_shrunk_report(const IntTools_ShrunkRange& r){
 bo_int(r.IsDone());bo_int(r.IsSplittable());bo_real(r.Length());double a,b;r.ShrunkRange(a,b);bo_real(a);bo_real(b);bo_int(r.Edge().IsNull());
 const auto& box=r.BndBox();bo_int(box.IsVoid());bo_real(box.GetGap());try{double x0,y0,z0,x1,y1,z1;box.Get(x0,y0,z0,x1,y1,z1);for(double v:{x0,y0,z0,x1,y1,z1})bo_real(v);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
void bo_shrunk(const std::vector<double>& v){
 try{IntTools_ShrunkRange r;bo_shrunk_report(r);r.Perform();bo_shrunk_report(r);occ::handle<IntTools_Context> ctx=new IntTools_Context();r.SetContext(ctx);bo_int(r.Context()==ctx);r.SetContext(nullptr);bo_int(r.Context().IsNull());
 auto geometry=bo_tools_geometry(int(v[0]),v[1],v[2]);TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(geometry,v[1],v[2]);TopoDS_Vertex first,last;TopExp::Vertices(edge,first,last);BRep_Builder builder;
 builder.UpdateEdge(edge,v[3]);builder.UpdateVertex(first,v[4]);builder.UpdateVertex(last,v[5]);r.SetData(edge,v[1],v[2],first,last);bo_int(r.Edge().IsSame(edge));
 for(int i=0;i<2;++i){try{r.Perform();}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}bo_shrunk_report(r);}
 try{r.SetShrunkRange(v[1],v[2]);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}bo_shrunk_report(r);
 r.SetData(edge,v[6],v[7],last,first);bo_shrunk_report(r);try{r.Perform();}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}bo_shrunk_report(r);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}

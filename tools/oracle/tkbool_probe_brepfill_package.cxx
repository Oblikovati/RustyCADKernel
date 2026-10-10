// BRepFill package probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_package.rs.
//
// The shapes are those of a shape program (tktopalgo_probe_shapes.cxx), named by their
// table index. Results print through pr_shape (tkprim_probe_shape.cxx). SearchOrigin is a
// private member (a friend of BRepFill_PipeShell), reached here by opening the class.
#define private public
#include <BRepFill.hxx>
#undef private
#include <BRepTools_WireExplorer.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Ax3.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Wire.hxx>
// `program e1 e2`: BRepFill::Face of two edges of the program.
void bl_bf_face_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int e1=r.integer(),e2=r.integer();
 GA_TRY(pr_shape(BRepFill::Face(TopoDS::Edge(t.shapes.at(e1)),TopoDS::Edge(t.shapes.at(e2)))))
 ga_end();}
// `program w1 w2`: BRepFill::Shell of two wires of the program.
void bl_bf_shell_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int w1=r.integer(),w2=r.integer();
 GA_TRY(pr_shape(BRepFill::Shell(TopoDS::Wire(t.shapes.at(w1)),TopoDS::Wire(t.shapes.at(w2)))))
 ga_end();}
// `program spine profile tol`: BRepFill::Axe; prints the frame and ProfOnSpine.
void bl_bf_axe_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int spine=r.integer(),profile=r.integer();double tol=r.real();
 gp_Ax3 axe;bool on=false;
 GA_TRY(BRepFill::Axe(t.shapes.at(spine),TopoDS::Wire(t.shapes.at(profile)),axe,on,tol);
        ga_pnt(axe.Location());ga_dir(axe.Direction());ga_dir(axe.XDirection());ga_dir(axe.YDirection());ga_flag(on))
 ga_end();}
// `program wire P Dir tol`: BRepFill::SearchOrigin on a copy of the wire handle; prints
// the rebuilt wire.
void bl_bf_search_origin_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int w=r.integer();gp_Pnt p=r.pnt();gp_Vec d(r.xyz());double tol=r.real();
 TopoDS_Wire wire=TopoDS::Wire(t.shapes.at(w));
 GA_TRY(BRepFill::SearchOrigin(wire,p,d,tol);pr_shape(wire))
 ga_end();}
// `program wire`: BRepFill::ComputeACR into an array 0..n (n the number of edges of the
// wire explorer); prints n and the array.
void bl_bf_compute_acr_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Wire wire=TopoDS::Wire(t.shapes.at(r.integer()));
 int n=0;for(BRepTools_WireExplorer ex(wire);ex.More();ex.Next())++n;
 ga_int(n);
 NCollection_Array1<double> acr(0,n);
 GA_TRY(BRepFill::ComputeACR(wire,acr);for(int i=0;i<=n;++i)ga_real(acr(i)))
 ga_end();}
// `program wire n (cut)... prec`: BRepFill::InsertACR with the cuts in an array 1..n.
void bl_bf_insert_acr_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Wire wire=TopoDS::Wire(t.shapes.at(r.integer()));
 int n=r.integer();NCollection_Array1<double> cuts(1,n);for(int i=1;i<=n;++i)cuts(i)=r.real();
 double prec=r.real();
 GA_TRY(pr_shape(BRepFill::InsertACR(wire,cuts,prec)))
 ga_end();}

// Exact oracle for the TKExpress packages (Expr, ExprIntrp) at pinned OCCT 3d097a0.
// Each input line is `operation arguments...`; tools/verify_tkexpress_numeric.py compares
// the output with crates/tkexpress/examples/numeric_probe.rs token for token. Texts travel
// as `x` + hexadecimal bytes in both directions, so the String() of every expression and
// the characters the scanner echoes are compared byte for byte; doubles print as their
// 64-bit patterns; a raised exception prints as `!` and its type name.
// Every request prints exactly one line and never depends on an earlier request: the
// verifier deals the corpus to parallel shards (tools/probe_shards.py).
// The prebuilt TKernel is compiled with OCC_CONVERT_SIGNALS (adm/cmake/occt_defs_flags.cmake
// on Linux): OCC_CATCH_SIGNALS must expand the same way here, so that a null dereference
// (a crash natively) becomes the OSD_SIGSEGV failure the Rust twin reports for its
// Standard_NullObject.
#ifndef OCC_CONVERT_SIGNALS
  #define OCC_CONVERT_SIGNALS
#endif
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <functional>
#include <iostream>
#include <map>
#include <sstream>
#include <string>
#include <vector>
#include <OSD.hxx>
#include <Standard_ErrorHandler.hxx>
#include <Standard_Failure.hxx>
#include <TCollection_AsciiString.hxx>
#include <Expr.hxx>
#include <Expr_Absolute.hxx>
#include <Expr_ArcCosine.hxx>
#include <Expr_ArcSine.hxx>
#include <Expr_ArcTangent.hxx>
#include <Expr_ArgCosh.hxx>
#include <Expr_ArgSinh.hxx>
#include <Expr_ArgTanh.hxx>
#include <Expr_BinaryFunction.hxx>
#include <Expr_Cosh.hxx>
#include <Expr_Cosine.hxx>
#include <Expr_Difference.hxx>
#include <Expr_Division.hxx>
#include <Expr_Equal.hxx>
#include <Expr_Exponential.hxx>
#include <Expr_Exponentiate.hxx>
#include <Expr_FunctionDerivative.hxx>
#include <Expr_GeneralExpression.hxx>
#include <Expr_LogOf10.hxx>
#include <Expr_LogOfe.hxx>
#include <Expr_NamedConstant.hxx>
#include <Expr_NamedFunction.hxx>
#include <Expr_NamedUnknown.hxx>
#include <Expr_NumericValue.hxx>
#include <Expr_Operators.hxx>
#include <Expr_PolyFunction.hxx>
#include <Expr_Product.hxx>
#include <Expr_RelationIterator.hxx>
#include <Expr_RUIterator.hxx>
#include <Expr_Sign.hxx>
#include <Expr_Sine.hxx>
#include <Expr_Sinh.hxx>
#include <Expr_Square.hxx>
#include <Expr_SquareRoot.hxx>
#include <Expr_Sum.hxx>
#include <Expr_SystemRelation.hxx>
#include <Expr_Tangent.hxx>
#include <Expr_Tanh.hxx>
#include <Expr_UnaryFunction.hxx>
#include <Expr_UnaryMinus.hxx>
#include <Expr_UnknownIterator.hxx>
#include <ExprIntrp_GenExp.hxx>

// The flex output stream of the scanner (`yyout`): unmatched characters are echoed there.
extern "C" void ExprIntrpset_out(FILE* theOut);

typedef occ::handle<Expr_GeneralExpression> ExprH;
typedef occ::handle<Expr_NamedUnknown>      UnknownH;

static std::ostringstream ex_out;
static void ex_token(const std::string& t){ex_out<<' '<<t;}
static std::string ex_hex(const char* bytes,size_t length){
 static const char digits[]="0123456789abcdef";std::string text="x";
 for(size_t i=0;i<length;++i){unsigned char c=(unsigned char)bytes[i];text+=digits[c>>4];text+=digits[c&15];}
 return text;}
static void ex_text(const TCollection_AsciiString& s){ex_token(ex_hex(s.ToCString(),(size_t)s.Length()));}
static void ex_real(double v){uint64_t bits;std::memcpy(&bits,&v,8);ex_token(std::to_string(bits));}
static void ex_int(int v){ex_token(std::to_string(v));}
static void ex_bool(bool v){ex_token(v?"1":"0");}
static std::string ex_unhex(const std::string& t){
 std::string bytes;for(size_t i=1;i+1<t.size();i+=2)bytes+=(char)std::stoi(t.substr(i,2),nullptr,16);return bytes;}
static double ex_bits(const std::string& t){uint64_t bits=std::stoull(t);double v;std::memcpy(&v,&bits,8);return v;}
// One guarded step: its tokens, or `!Type` when it raises.
static void ex_try(const std::function<void()>& step){
 try{OCC_CATCH_SIGNALS step();}catch(const Standard_Failure& e){ex_token(std::string("!")+e.ExceptionType());}}
static void ex_end(){std::cout<<ex_out.str()<<"\n";ex_out.str("");ex_out.clear();}

// The unknowns of an expression in iteration order.
static std::vector<UnknownH> ex_unknowns(const ExprH& e){
 std::vector<UnknownH> all;Expr_UnknownIterator it(e);while(it.More()){all.push_back(it.Value());it.Next();}return all;}
// The structural, simplification, derivation and evaluation battery of one expression.
static void ex_battery(const ExprH& e){
 ex_try([&]{ex_text(e->String());});
 ex_try([&]{ex_int(e->NbSubExpressions());ex_bool(e->IsShareable());});
 ex_try([&]{ex_bool(e->ContainsUnknowns());});
 ex_try([&]{ex_bool(e->IsLinear());});
 ex_try([&]{ex_text(e->Simplified()->String());});
 ex_try([&]{ex_text(e->ShallowSimplified()->String());});
 ex_try([&]{ExprH c=e->Copy();ex_text(c->String());ex_bool(e->IsIdentical(c));});
 ex_try([&]{ex_int(Expr::NbOfFreeVariables(e));});
 std::vector<UnknownH> vars;ex_try([&]{vars=ex_unknowns(e);ex_int((int)vars.size());});
 for(size_t k=0;k<vars.size()&&k<3;++k){const UnknownH& x=vars[k];
  ex_try([&]{ex_text(x->GetName());ex_bool(e->Contains(x));});
  ex_try([&]{ex_text(e->Derivative(x)->String());});
  ex_try([&]{ex_text(e->NDerivative(x,2)->String());});}
 ex_try([&]{ex_real(e->EvaluateNumeric());});
 ex_try([&]{int n=(int)vars.size();if(n==0)return;NCollection_Array1<UnknownH> vr(1,n);NCollection_Array1<double> vl(1,n);
  for(int i=1;i<=n;++i){vr(i)=vars[i-1];vl(i)=0.5+0.25*i;}ex_real(e->Evaluate(vr,vl));});}

// Process the hexadecimal `text` and return the characters the scanner echoed.
static std::string ex_process(const occ::handle<ExprIntrp_GenExp>& g,const std::string& text){
 char* buffer=nullptr;size_t size=0;FILE* echo=open_memstream(&buffer,&size);ExprIntrpset_out(echo);
 g->Process(TCollection_AsciiString(ex_unhex(text).c_str()));
 std::fflush(echo);ExprIntrpset_out(stdout);std::string echoed(buffer,size);std::fclose(echo);std::free(buffer);
 return echoed;}
// `ex_text <text>...`: one interpreter processes each text in order; per text the done
// flag, the echoed characters and, for an expression, its battery.
static void ex_text_case(std::istream& in){
 occ::handle<ExprIntrp_GenExp> g=ExprIntrp_GenExp::Create();std::string t;
 while(in>>t){std::string echoed=ex_process(g,t);
  ex_token(std::string("d")+(g->IsDone()?"1":"0"));ex_token(ex_hex(echoed.data(),echoed.size()));
  if(!g->IsDone())continue;ExprH e=g->Expression();if(e.IsNull()){ex_token("n");continue;}ex_battery(e);}
 ex_int(g->GetNamed().Length());ex_int(g->GetFunctions().Length());ex_end();}

// The classes of `ex_fold`, by code.
static ExprH ex_unary(int code,const ExprH& op){
 switch(code){case 0:return new Expr_Absolute(op);case 1:return new Expr_ArcCosine(op);case 2:return new Expr_ArcSine(op);
  case 3:return new Expr_ArcTangent(op);case 4:return new Expr_ArgCosh(op);case 5:return new Expr_ArgSinh(op);
  case 6:return new Expr_ArgTanh(op);case 7:return new Expr_Cosh(op);case 8:return new Expr_Cosine(op);
  case 9:return new Expr_Exponential(op);case 10:return new Expr_LogOf10(op);case 11:return new Expr_LogOfe(op);
  case 12:return new Expr_Sign(op);case 13:return new Expr_Sine(op);case 14:return new Expr_Sinh(op);
  case 15:return new Expr_Square(op);case 16:return new Expr_SquareRoot(op);case 17:return new Expr_Tangent(op);
  case 18:return new Expr_Tanh(op);default:return new Expr_UnaryMinus(op);}}
static ExprH ex_binary(int code,const ExprH& a,const ExprH& b){
 switch(code){case 0:return new Expr_Sum(a,b);case 1:return new Expr_Difference(a,b);case 2:return new Expr_Product(a,b);
  case 3:return new Expr_Division(a,b);default:return new Expr_Exponentiate(a,b);}}
// The simplified value (or its text when not a value) and the evaluated value.
static void ex_folded(const ExprH& e){
 ex_try([&]{ExprH s=e->ShallowSimplified();occ::handle<Expr_NumericValue> v=occ::down_cast<Expr_NumericValue>(s);
  if(v.IsNull())ex_text(s->String());else ex_real(v->GetValue());});
 ex_try([&]{ex_real(e->EvaluateNumeric());});
 ex_try([&]{ex_text(e->String());});}
// `ex_fold <code> <bits>`: a standard function (or the opposite, code 19) of a value.
static void ex_fold_case(std::istream& in){int code;std::string v;in>>code>>v;
 ex_folded(ex_unary(code,new Expr_NumericValue(ex_bits(v))));ex_end();}
// `ex_fold2 <code> <bits> <bits>`: a sum, difference, product, quotient or power of values.
static void ex_fold2_case(std::istream& in){int code;std::string a,b;in>>code>>a>>b;
 ex_folded(ex_binary(code,new Expr_NumericValue(ex_bits(a)),new Expr_NumericValue(ex_bits(b))));ex_end();}

// `ex_rel <left> <right> ...`: the equations of the text pairs (one interpreter), in a
// system when there are several, and the relation battery.
static void ex_rel_case(std::istream& in){
 occ::handle<ExprIntrp_GenExp> g=ExprIntrp_GenExp::Create();std::vector<occ::handle<Expr_GeneralRelation>> rels;std::string l,r;
 while(in>>l>>r){ex_process(g,l);ExprH a=g->Expression();ex_process(g,r);ExprH b=g->Expression();
  rels.push_back(new Expr_Equal(a,b));}
 occ::handle<Expr_GeneralRelation> rel=rels[0];
 if(rels.size()>1){occ::handle<Expr_SystemRelation> s=new Expr_SystemRelation(rels[0]);for(size_t i=1;i<rels.size();++i)s->Add(rels[i]);rel=s;}
 ex_try([&]{ex_text(rel->String());});
 ex_try([&]{ex_bool(rel->IsSatisfied());});
 ex_try([&]{ex_bool(rel->IsLinear());});
 ex_try([&]{ex_int(rel->NbOfSubRelations());ex_int(rel->NbOfSingleRelations());});
 ex_try([&]{Expr_RelationIterator it(rel);while(it.More()){ex_text(it.Value()->String());it.Next();}});
 ex_try([&]{Expr_RUIterator it(rel);while(it.More()){ex_text(it.Value()->GetName());it.Next();}});
 ex_try([&]{ex_int(Expr::NbOfFreeVariables(rel));});
 ex_try([&]{ex_text(rel->Simplified()->String());});
 ex_try([&]{ex_text(rel->Copy()->String());});
 ex_try([&]{ex_text(rel->SubRelation(1)->String());});
 ex_try([&]{occ::handle<Expr_SystemRelation> s=occ::down_cast<Expr_SystemRelation>(rel);if(s.IsNull())return;
  ex_try([&]{s->Remove(rels[0]);});ex_try([&]{s->Remove(new Expr_Equal(new Expr_NumericValue(1.),new Expr_NumericValue(1.)));});});
 ex_try([&]{Expr_RUIterator it(rel);if(it.More()){UnknownH x=it.Value();ex_bool(rel->Contains(x));
  rel->Replace(x,new Expr_NumericValue(2.));ex_text(rel->String());}});
 ex_try([&]{rel->Simplify();ex_text(rel->String());ex_bool(rel->IsSatisfied());});
 ex_end();}
#include "tkexpress_probe_cases.cxx"

const std::map<std::string,void(*)(std::istream&)> probe_cases={
 {"ex_text", ex_text_case},
 {"ex_fold", ex_fold_case},
 {"ex_fold2", ex_fold2_case},
 {"ex_rel", ex_rel_case},
 {"ex_case", ex_case_case},
};
int main(){
 // A null dereference inside ExprIntrp::Parse becomes a caught failure, as the native
 // OCC_CATCH_SIGNALS intends; the Rust twin reports those calls as failures too.
 OSD::SetSignal(false);
 std::string line;
 while(std::getline(std::cin,line)){
  std::istringstream input(line);std::string op;input>>op;
  if(op.empty())continue;
  auto found=probe_cases.find(op);
  if(found==probe_cases.end()){std::cerr<<"Unknown probe operation: "<<op<<std::endl;return 2;}
  found->second(input);
 }
}

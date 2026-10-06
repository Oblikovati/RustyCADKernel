#include <NCollection_Sequence.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct Trace {
  int value; bool failAssign=false, failCopy=false; static inline std::string events;
  Trace(int v):value(v){}
  Trace(const Trace& s):value(s.value){events+="C"+std::to_string(value)+";";if(s.failCopy)throw std::runtime_error("copy");}
  Trace& operator=(const Trace& s){events+="A"+std::to_string(value)+">"+std::to_string(s.value)+";";value=s.value;if(s.failAssign)throw std::runtime_error("assign");return *this;}
  ~Trace(){events+="D"+std::to_string(value)+";";}
};
using Sequence=NCollection_Sequence<Trace>;
void assignment(){
  Sequence s;s.EmplaceAppend(1);s.EmplaceAppend(2);Trace source(9);
  const Trace* address=&s.Value(1);Trace::events.clear();s.SetValue(1,source);
  std::cout<<"assign "<<Trace::events<<' '<<s.Value(1).value<<' '<<(&s.Value(1)==address)<<'\n';
  Trace::events.clear();s.SetValue(1,s.Value(1));std::cout<<"self "<<Trace::events<<'\n';
  Trace::events.clear();s.SetValue(2,s.Value(1));s.SetValue(1,s.Value(2));std::cout<<"indexed "<<Trace::events<<'\n';
  source.value=7;source.failAssign=true;Trace::events.clear();bool caught=false;
  try{s.SetValue(1,source);}catch(const std::runtime_error&){caught=true;}
  std::cout<<"partial_assign "<<Trace::events<<' '<<caught<<' '<<s.Value(1).value<<' '<<(&s.Value(1)==address)<<'\n';
  Trace::events.clear();int errors=0;for(int index: {-1,0,3}){try{s.SetValue(index,source);}catch(const Standard_Failure&){++errors;}}
  std::cout<<"invalid "<<errors<<' '<<Trace::events.size()<<'\n';
}
void cursorCopy(){
  Sequence s;s.EmplaceAppend(1);s.EmplaceAppend(2);Sequence::Iterator cursor(s);Trace source(8);
  Trace::events.clear();s.InsertAfter(cursor,std::move(source));
  std::cout<<"cursor_rvalue "<<Trace::events<<' '<<source.value<<' '<<s.Size()<<' '<<s.Value(2).value<<' '<<cursor.Value().value<<'\n';
  Sequence empty;Sequence::Iterator end(empty);Trace::events.clear();empty.InsertAfter(end,std::move(source));
  std::cout<<"empty_rvalue "<<Trace::events<<' '<<source.value<<' '<<empty.First().value<<' '<<end.More()<<'\n';
  source.failCopy=true;Trace::events.clear();bool caught=false;
  try{s.InsertAfter(cursor,std::move(source));}catch(const std::runtime_error&){caught=true;}
  std::cout<<"copy_failure "<<Trace::events<<' '<<caught<<' '<<source.value<<' '<<s.Size()<<'\n';
}
int main(){assignment();cursorCopy();}

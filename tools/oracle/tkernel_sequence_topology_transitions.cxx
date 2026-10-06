#include <NCollection_Sequence.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <sstream>
struct Sequence:NCollection_Sequence<int>{
 using NCollection_Sequence<int>::NCollection_Sequence;
 std::string state() const {
  std::ostringstream out;out<<Size()<<' '<<myCurrentIndex<<' ';
  out<<(myCurrentItem?static_cast<Node*>(myCurrentItem)->Value():0)<<" :";
  for(const_iterator it(*this);it.More();it.Next())out<<' '<<it.Value();
  return out.str();
 }
};
Sequence initial(int count,int cache){Sequence result;for(int i=1;i<=count;++i)result.Append(i);if(cache)result.Value(cache);return result;}
void print(int count,int cache,const char* op,int a,int b,const Sequence& seq,bool error){
 std::cout<<count<<' '<<cache<<' '<<op<<' '<<a<<' '<<b<<' '<<error<<" | "<<seq.state()<<'\n';
}
void simple(int count,int cache){
 for(int op=0;op<7;++op){auto s=initial(count,cache);bool error=false;
  try{switch(op){case 0:s.Append(9);break;case 1:s.Prepend(9);break;case 2:s.Reverse();break;case 3:s.Clear();break;case 4:{Sequence::Iterator it(s);s.EmplaceAfter(it,9);break;}case 5:{Sequence::Iterator it(s);s.Remove(it);break;}case 6:{const Sequence& readonly=s;if(count)readonly.At(size_t(count-1));break;}}}catch(const Standard_Failure&){error=true;}
  print(count,cache,"simple",op,0,s,error);
 }
}
void indexed(int count,int cache){
 for(int index=0;index<=count+1;++index)for(int op=0;op<4;++op){auto s=initial(count,cache);bool error=false;
  try{switch(op){case 0:s.EmplaceAfter(index,9);break;case 1:s.Remove(index);break;case 2:s.SetValue(index,9);break;case 3:s.Value(index);break;}}catch(const Standard_Failure&){error=true;}
  print(count,cache,"indexed",op,index,s,error);
 }
}
void paired(int count,int cache){
 for(int a=0;a<=count+1;++a)for(int b=0;b<=count+1;++b)for(int op=0;op<2;++op){auto s=initial(count,cache);bool error=false;
  try{if(op==0)s.Exchange(a,b);else s.Remove(a,b);}catch(const Standard_Failure&){error=true;}
  print(count,cache,op==0?"exchange":"range",a,b,s,error);
 }
}
void transfers(int count,int cache){
 for(int index=0;index<=count+1;++index)for(int kind=0;kind<3;++kind){if(kind==2&&index>count)continue;auto s=initial(count,cache);Sequence target;target.Append(7);target.Append(8);target.Value(2);bool error=false;
  if(kind==2){target.Clear(new NCollection_IncAllocator);target.Append(7);target.Append(8);target.Value(2);}
  try{if(kind==0)s.Split(index,target);else s.InsertAfter(index,target);}catch(const Standard_Failure&){error=true;}
  print(count,cache,"transfer",kind,index,s,error);print(count,cache,"target",kind,index,target,error);
 }
}
void ownership(int count,int cache){
 auto s=initial(count,cache);Sequence copy(s);print(count,cache,"copy",0,0,copy,false);
 Sequence moved(std::move(s));print(count,cache,"move",0,0,moved,false);print(count,cache,"moved_from",0,0,s,false);
}
int main(){for(int count=0;count<=4;++count)for(int cache=0;cache<=count;++cache){simple(count,cache);indexed(count,cache);paired(count,cache);transfers(count,cache);ownership(count,cache);}}

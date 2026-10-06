#include <NCollection_SparseArrayBase.hxx>
#include <iostream>
#include <new>
#include <limits>
struct Pair { int marker,value; };
struct Prefix { unsigned long marker=42; };
struct CustomSparse:Prefix,NCollection_SparseArrayBase {
  using Mutable=BasicIterator<CustomSparse,int,false>;
  using Const=BasicIterator<CustomSparse,int,true>;
  mutable int pointers=0, capacities=0;
  size_t override_end=std::numeric_limits<size_t>::max();
  CustomSparse():NCollection_SparseArrayBase(sizeof(Pair),4,destroy){}
  static void destroy(void* p){static_cast<Pair*>(p)->~Pair();}
  static void create(void* p,void* q){new(p)Pair(*static_cast<Pair*>(q));}
  static void copy(void* p,void* q){*static_cast<Pair*>(p)=*static_cast<Pair*>(q);}
  void put(size_t i,int v){Pair p{100+int(i),v};setValue(i,&p,create,copy);}
  size_t Capacity()const{++capacities;return override_end==std::numeric_limits<size_t>::max()?NCollection_SparseArrayBase::Capacity():override_end;}
  template<class Block> void* getItem(const Block& b,size_t i)const{
    ++pointers;return &static_cast<Pair*>(NCollection_SparseArrayBase::getItem(b,i))->value;
  }
};
void access(){
 CustomSparse a;a.put(1,10);a.put(5,50);
 CustomSparse::Mutable m(a);CustomSparse::Const c(m);
 int first=*c;int* p=m.operator->();*p=11;int second=*m;const int* q=c.operator->();
 std::cout<<"projection "<<first<<' '<<second<<' '<<*q<<' '<<a.pointers<<' '<<a.capacities<<' '<<(p==q)<<'\n';
 std::cout<<"adjusted_base "<<(static_cast<const void*>(&a)!=static_cast<const void*>(static_cast<const NCollection_SparseArrayBase*>(&a)))<<'\n';
}
void copies(){
 CustomSparse a;a.put(1,10);a.put(5,50);CustomSparse::Mutable m(a);auto old=m++;
 CustomSparse::Const c;bool identity=&(c=m)==&c;
 std::cout<<"copies "<<old.Index()<<' '<<m.Index()<<' '<<c.Index()<<' '<<identity<<' '<<(c==m)<<' '<<(c!=old)<<'\n';
 bool prefix=&(++m)==&m;std::cout<<"prefix "<<prefix<<' '<<m.More()<<' '<<m.Key()<<'\n';
 ++m;m.Restart();std::cout<<"restart "<<m.Key()<<' '<<a.capacities<<'\n';
}
void ends(){
 CustomSparse a,b;a.put(5,50);b.put(5,50);
 CustomSparse::Mutable m(a);CustomSparse::Const c(b);bool owners=m==c;
 CustomSparse::Mutable e(a,a.Capacity()+3);CustomSparse::Const detached;
 ++e;std::cout<<"ends "<<owners<<' '<<e.Key()<<' '<<(e==detached)<<' '<<detached.More()<<' '<<detached.Index()<<'\n';
 e.Restart();std::cout<<"past_restart "<<e.Key()<<' '<<e.More()<<'\n';
}
void capacity(){
 CustomSparse a;a.put(3,30);a.override_end=1;CustomSparse::Const c(a);
 std::cout<<"short_end "<<c.More()<<' '<<c.Key()<<' '<<*c<<'\n';
 ++c;std::cout<<"short_end_exhausted "<<c.More()<<' '<<c.Key()<<'\n';
 a.override_end=0;c.Restart();std::cout<<"captured_end "<<c.More()<<' '<<c.Key()<<' '<<a.capacities<<'\n';
}
int main(){access();copies();ends();capacity();}

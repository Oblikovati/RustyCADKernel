#include <NCollection_BaseSequence.hxx>
#include <iostream>
struct Raw: NCollection_BaseSequence {
 NCollection_SeqNode a,b,c;
 Raw():NCollection_BaseSequence(nullptr){PAppend(&a);PAppend(&b);PAppend(&c);}
 int id(NCollection_SeqNode* n){return n==&a?1:n==&b?2:n==&c?3:0;}
 void show(const char* name){std::cout<<name<<' '<<Size()<<' '<<id(a.Previous())<<' '<<id(a.Next())<<' '<<id(b.Previous())<<' '<<id(b.Next())<<' '<<id(c.Previous())<<' '<<id(c.Next())<<'\n';}
 void appendSelf(){PAppend(*this);show("append_self");}
 void prependSelf(){PPrepend(*this);show("prepend_self");}
 void insertSelf(size_t i){PInsertAfter(i,*this);show("insert_self");}
};
int main(){Raw a;a.appendSelf();Raw b;b.prependSelf();for(size_t i=0;i<=3;i++){Raw c;c.insertSelf(i);}}

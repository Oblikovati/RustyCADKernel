#include <NCollection_BaseSequence.hxx>
#include <iostream>
struct ReadOnlyNode: NCollection_SeqNode { int value=42; const int& Value()const{return value;} };
struct Base: NCollection_BaseSequence { ReadOnlyNode node; using Cursor=BasicIterator<ReadOnlyNode,int,true>; using Mutable=BasicIterator<ReadOnlyNode,int,false>; Base():NCollection_BaseSequence(nullptr){PAppend(&node);} };
int main(){ Base base; Base::Cursor cursor(base); Base::Mutable legacy(cursor); std::cout << "readonly_only " << *cursor << ' ' << *cursor.operator->() << ' ' << legacy.Value() << '\n'; }

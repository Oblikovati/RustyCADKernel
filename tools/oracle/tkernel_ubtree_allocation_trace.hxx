// Shared real allocation/payload observations for native UBTree operations.
#pragma once
#include <NCollection_UBTree.hxx>
#include <cstdlib>
#include <iostream>
#include <map>
#include <stdexcept>
#include <string>
#include <type_traits>
struct TrackingArena;
struct PairTrace {
  static TrackingArena* arena;
  static bool active;
  static char phase,kind;
  static int remaining;
  static std::string events;
  static void event(char p,char k,int value);
  static void reset(char p='X',char k='X',int count=0) {active=true;phase=p;kind=k;remaining=count;events.clear();}
};
struct TrackingArena: NCollection_BaseAllocator {
  std::map<void*,int> blocks;int next=0;std::string releases;
  void* Allocate(size_t size) override {
    void* pointer=nullptr;if(posix_memalign(&pointer,64,size)!=0)throw std::bad_alloc();
    blocks[pointer]=next++;return pointer;
  }
  void* AllocateOptimal(size_t size) override {return Allocate(size);}
  void Free(void* pointer) override {
    if(!pointer)return;releases+="F"+std::to_string(blocks.at(pointer))+",";
    blocks.erase(pointer);std::free(pointer);
  }
  ~TrackingArena() override {for(const auto& entry:blocks)std::free(entry.first);}
};
TrackingArena* PairTrace::arena=nullptr;bool PairTrace::active=false;
char PairTrace::phase='X',PairTrace::kind='X';int PairTrace::remaining=0;std::string PairTrace::events;
void PairTrace::event(char p,char k,int value) {
  if(!active)return;
  events+=std::string{p,k}+std::to_string(value)+":"+std::to_string(arena->blocks.size())+",";
  if(p==phase && k==kind && --remaining==0) {phase='X';throw std::runtime_error("rejected payload callback");}
}
template<char Kind> struct Payload {
  int value;
  Payload():value(0) {PairTrace::event('N',Kind,0);}
  explicit Payload(int item):value(item) {}
  Payload(const Payload& source):value(source.value) {PairTrace::event('C',Kind,value);}
  Payload& operator=(const Payload& source) {value=source.value;PairTrace::event('A',Kind,value);return *this;}
  ~Payload() noexcept(false) {PairTrace::event('D',Kind,value);}
  void Add(const Payload& other) {value+=other.value;PairTrace::event('J',Kind,value);}
  bool IsOut(const Payload&)const {PairTrace::event('R',Kind,value);return false;}
  double SquareExtent()const {PairTrace::event('E',Kind,value);return value;}
};
using Object=Payload<'O'>;using Bounds=Payload<'B'>;using Tree=NCollection_UBTree<Object,Bounds>;using Node=Tree::TreeNode;

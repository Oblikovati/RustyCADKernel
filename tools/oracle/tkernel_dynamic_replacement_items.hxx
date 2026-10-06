// Shared native payloads keep raw and public replacement probes on identical operations.
#pragma once
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <new>
static std::string trace;
static int liveItems=0;
static void event(char kind,int value) {
  if(!trace.empty()) trace+=',';
  trace+=kind; trace+=std::to_string(value);
}
static void construct(int value) {
  event('C',value);
  if(value==99) throw std::runtime_error("construct99");
  ++liveItems;
}
struct Assignable {
  int value;
  explicit Assignable(int n=0):value(n) { construct(n); }
  ~Assignable() { event('D',value); --liveItems; }
  Assignable& operator=(Assignable&& other) {
    event('A',other.value); value=other.value; other.value=-7;
    if(value==77) throw std::runtime_error("assign77");
    return *this;
  }
};
struct MoveOnly {
  int value;
  explicit MoveOnly(int n=0):value(n) { construct(n); }
  MoveOnly(MoveOnly&& other) noexcept:value(other.value) { event('M',value);other.value=-7;++liveItems; }
  MoveOnly(const MoveOnly&)=delete;
  MoveOnly& operator=(MoveOnly&&)=delete;
  ~MoveOnly() { event('D',value); --liveItems; }
};
struct CopyPreferred {
  int value;
  explicit CopyPreferred(int n=0):value(n) { construct(n); }
  CopyPreferred(CopyPreferred&& other) noexcept(false):value(other.value) { event('M',value);other.value=-7;++liveItems; }
  CopyPreferred(const CopyPreferred& other):value(other.value) {
    event('K',value);if(value==88) throw std::runtime_error("copy88");
    ++liveItems;
  }
  CopyPreferred& operator=(CopyPreferred&&)=delete;
  ~CopyPreferred() { event('D',value); --liveItems; }
};
struct DirectOnly {
  int value;
  explicit DirectOnly(int n=0):value(n) { construct(n); }
  DirectOnly(DirectOnly&&)=delete;
  DirectOnly(const DirectOnly&)=delete;
  DirectOnly& operator=(DirectOnly&&)=delete;
  ~DirectOnly() { event('D',value); --liveItems; }
};

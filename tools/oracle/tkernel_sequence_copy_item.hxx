#pragma once
#include <NCollection_BaseAllocator.hxx>
#include <algorithm>
#include <stdexcept>
#include <string>
#include <vector>

static std::string events;
struct CopyArena : NCollection_BaseAllocator {
  std::vector<void*> live;
  void* Allocate(size_t size) override {
    void* result = NCollection_BaseAllocator::Allocate(size);
    live.push_back(result); return result;
  }
  void Free(void* address) override {
    live.erase(std::find(live.begin(), live.end(), address));
    NCollection_BaseAllocator::Free(address);
  }
};
static CopyArena* observed;
static int failing;
struct CopyItem {
  int number;
  mutable int reads = 0;
  explicit CopyItem(int value) : number(value) {}
  CopyItem(const CopyItem& source) : number(source.number) {
    ++source.reads;
    events += "c" + std::to_string(number) + "/" + std::to_string(observed->live.size()) + ";";
    if (number == failing) throw std::runtime_error("copy failure");
  }
  ~CopyItem() { events += "d" + std::to_string(number) + ";"; }
};

// OCCT 3d097a03: actual IBlock placement, cursor reset and allocator list membership.
#include <NCollection_IncAllocator.hxx>
#include <cstddef>
#include <cstdint>
#include <iostream>
#include <new>

using Block = NCollection_IncAllocator::IBlock;

void layout()
{
  std::cout << "layout " << sizeof(Block) << ' ' << alignof(Block) << ' '
            << offsetof(Block, CurPointer) << ' ' << offsetof(Block, AvailableSize) << ' '
            << offsetof(Block, NextBlock) << ' ' << offsetof(Block, NextOrderedBlock) << '\n';
}

void placement(size_t size)
{
  alignas(Block) char buffer[sizeof(Block)+128];
  Block* block = new(buffer) Block(buffer, size);
  const auto start = buffer+sizeof(Block);
  std::cout << "construct-" << size << ' ' << (block->CurPointer.load()==start) << ' '
            << block->AvailableSize.load() << ' ' << (block->NextBlock==nullptr)
            << ' ' << (block->NextOrderedBlock==nullptr) << '\n';
  block->~Block();
}

void reset_placement()
{
  alignas(Block) char buffer[sizeof(Block)+128];
  Block* block = new(buffer) Block(buffer, 128);
  char* start = buffer+sizeof(Block);
  block->CurPointer.store(start+31); block->AvailableSize.store(97);
  block->NextBlock=block; block->NextOrderedBlock=block;
  NCollection_IncAllocator arena;
  arena.resetBlock(block);
  std::cout << "reset-placement " << (block->CurPointer.load()==start) << ' '
            << block->AvailableSize.load() << ' ' << (block->NextBlock==block)
            << ' ' << (block->NextOrderedBlock==block) << '\n';
  block->~Block();
}

void reset_used_block()
{
  NCollection_IncAllocator arena(2048);
  char* first=static_cast<char*>(arena.Allocate(2048));
  Block* used=arena.myUsedHeap;
  arena.resetBlock(used);
  char* next=static_cast<char*>(arena.Allocate(8));
  std::cout << "reset-used " << (used->CurPointer.load()==first) << ' '
            << used->AvailableSize.load() << ' ' << (arena.myUsedHeap==used) << ' '
            << (next==first) << ' ' << arena.myBlockCount << '\n';
}

void reset_available_block()
{
  NCollection_IncAllocator arena(2048);
  char* first=static_cast<char*>(arena.Allocate(80));
  Block* head=arena.myAllocationHeap;
  arena.resetBlock(head);
  char* next=static_cast<char*>(arena.Allocate(16));
  std::cout << "reset-available " << (next==first) << ' '
            << head->AvailableSize.load() << ' ' << (head==arena.myAllocationHeap) << '\n';
}

void clean(bool enabled)
{
  NCollection_IncAllocator arena(4096);
  arena.SetThreadSafe(enabled); arena.Allocate(4096); arena.Allocate(8);
  arena.clean();
  std::cout << "clean-" << enabled << ' ' << arena.myBlockSize << ' ' << arena.myBlockCount
            << ' ' << (arena.myAllocationHeap==nullptr) << ' ' << (arena.myUsedHeap==nullptr)
            << ' ' << (arena.myOrderedBlocks==nullptr) << ' ' << (arena.myMutex!=nullptr);
  char* first=static_cast<char*>(arena.Allocate(4096));
  char* next=static_cast<char*>(arena.Allocate(8192));
  std::cout << ' ' << (next==first+4096) << '\n';
}

int main()
{
  layout();
  for(size_t size:{0,1,128}) placement(size);
  reset_placement(); reset_used_block(); reset_available_block();
  clean(false); clean(true);
}

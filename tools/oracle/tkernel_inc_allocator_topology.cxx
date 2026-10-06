// OCCT 3d097a03: actual IBlock link and cursor transitions, normalized by ordered-chain position.
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <vector>
using Arena = NCollection_IncAllocator;
using Block = Arena::IBlock;

size_t identity(const std::vector<Block*>& blocks, Block* address)
{
  for (size_t index = 0; index < blocks.size(); ++index)
    if (blocks[index] == address) return index + 1;
  return 0;
}

void block_row(const std::vector<Block*>& blocks, Block* block)
{
  char* start = reinterpret_cast<char*>(block) + sizeof(Block);
  std::cout << ' ' << identity(blocks, block) << ':' << block->CurPointer.load() - start
            << ':' << block->AvailableSize.load() << ':' << identity(blocks, block->NextBlock)
            << ':' << identity(blocks, block->NextOrderedBlock);
}

void snapshot(Arena& arena, bool mode, size_t step)
{
  std::vector<Block*> blocks;
  for (Block* block = arena.myOrderedBlocks; block; block = block->NextOrderedBlock)
    blocks.push_back(block);
  std::cout << "graph-" << mode << '-' << step << ' ' << arena.myBlockSize << ' '
            << arena.myBlockCount << ' ' << identity(blocks, arena.myAllocationHeap) << ' '
            << identity(blocks, arena.myUsedHeap) << ' ' << identity(blocks, arena.myOrderedBlocks);
  for (Block* block : blocks) block_row(blocks, block);
  std::cout << '\n';
}

void allocation_steps(Arena& arena, bool mode, size_t& step)
{
  for (size_t size : {1600, 1700, 64, 64, 64, 0, 2048, 8, 8, 16000, 8, 8}) {
    arena.Allocate(size);
    snapshot(arena, mode, ++step);
  }
}

void reset_steps(Arena& arena, bool mode, size_t& step)
{
  arena.resetBlock(arena.myUsedHeap); snapshot(arena, mode, ++step);
  arena.resetBlock(arena.myOrderedBlocks); snapshot(arena, mode, ++step);
  arena.allocateSlow(32); snapshot(arena, mode, ++step);
  arena.Reset(false); snapshot(arena, mode, ++step);
  for (size_t size : {16, 32, 64, 128, 2048, 10000}) {
    arena.Allocate(size);
    snapshot(arena, mode, ++step);
  }
}

void run(bool mode)
{
  Arena arena(2048);
  arena.SetThreadSafe(mode);
  size_t step = 0;
  snapshot(arena, mode, step);
  allocation_steps(arena, mode, step);
  reset_steps(arena, mode, step);
  arena.clean(); snapshot(arena, mode, ++step);
}

int main()
{
  run(false);
  run(true);
}

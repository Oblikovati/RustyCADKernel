// Shared assignment-only item for native handle-array constructor oracles.
#pragma once
#include <stdexcept>
#include <string>
#include <utility>

static std::string fill_calls;
static int default_count = 0, fail_assignment = 0;
struct FillProbe {
  int id, number;
  mutable int reads = 0;
  FillProbe() : id(++default_count), number(-id)
  {
    fill_calls += "d" + std::to_string(id) + ";";
  }
  explicit FillProbe(int value) : id(0), number(value) {}
  FillProbe(const FillProbe&) = delete;
  FillProbe(FillProbe&&) = delete;
  FillProbe& operator=(const FillProbe& donor)
  {
    ++donor.reads;
    number = donor.number;
    fill_calls += "a" + std::to_string(id) + "=" + std::to_string(number) + ";";
    if (id == fail_assignment) throw std::runtime_error("assignment failure");
    return *this;
  }
  ~FillProbe()
  {
    fill_calls += "x" + std::to_string(id) + "=" + std::to_string(number) + ";";
  }
};
static std::string take_fill_calls()
{
  std::string result = std::exchange(fill_calls, {});
  return result.empty() ? "-" : result;
}
static void reset_fill(int fail)
{
  fill_calls.clear();
  default_count = 0;
  fail_assignment = fail;
}

#pragma once
// Shared copy/fill payload phases distinguish object destruction from failed-member cleanup.
#include <memory>
#include <stdexcept>
#include <string>

static bool active = false;
static size_t attempts = 0, completed = 0, failing = 0;
static int resources = 0;
static std::string events;
static void* firstAttempt = nullptr;

struct CopyPayload
{
  int value;
  explicit CopyPayload(int initial) : value(initial) { ++resources; }
  ~CopyPayload() { --resources; }
};
struct CopyItem
{
  int value;
  std::unique_ptr<CopyPayload> payload;
  explicit CopyItem(int initial) : value(initial), payload(new CopyPayload(initial)) {}
  CopyItem(const CopyItem& source) : value(source.value), payload(new CopyPayload(source.payload->value))
  {
    if (!active) return;
    if (firstAttempt == nullptr) firstAttempt = this;
    events += "C" + std::to_string(value) + ',';
    if (++attempts == failing) throw std::runtime_error("injected copy-constructor failure");
    ++completed;
  }
  CopyItem& operator=(const CopyItem& source)
  { value = source.value; payload.reset(new CopyPayload(source.payload->value)); return *this; }
  ~CopyItem() { if (active) events += "D" + std::to_string(value) + ','; }
};

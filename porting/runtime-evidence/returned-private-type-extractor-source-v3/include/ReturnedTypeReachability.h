#pragma once

#include "Schema.h"

namespace api_inventory {
// Preserve inferred-type exposure without granting primary API coverage credit.
// Example: returnedTypeReachabilityEvents(getBlock, context, config).
std::vector<llvm::json::Object>
returnedTypeReachabilityEvents(const clang::Decl &declaration,
                               clang::ASTContext &context,
                               const Config &config);
} // namespace api_inventory

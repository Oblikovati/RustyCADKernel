#pragma once
#include "Schema.h"

namespace api_inventory {
// Preserve explicit upstream status independently of visibility and port
// claims. Example: upstreamRetirementMarkers(method, context).
llvm::json::Array upstreamRetirementMarkers(const clang::Decl &declaration,
                                            clang::ASTContext &context);
} // namespace api_inventory

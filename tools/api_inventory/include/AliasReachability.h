#pragma once

#include "Schema.h"
#include "clang/AST/DeclCXX.h"
#include <vector>

namespace api_inventory {
// Records public aliases to hidden template specializations as unresolved
// evidence.
std::vector<llvm::json::Object>
aliasReachabilityEvents(const clang::TypedefNameDecl &alias,
                        clang::ASTContext &context, const Config &config);
} // namespace api_inventory

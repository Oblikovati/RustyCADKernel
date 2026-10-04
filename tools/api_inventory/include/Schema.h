#pragma once

#include "clang/AST/Decl.h"
#include "clang/Basic/SourceManager.h"
#include "llvm/Support/JSON.h"
#include <filesystem>
#include <set>
#include <string>
#include <vector>

namespace api_inventory {
struct Config {
  std::filesystem::path sourceRoot;
  std::filesystem::path mainHeader;
  std::set<std::filesystem::path> extraEmitHeaders;
};

std::filesystem::path canonicalPath(const std::filesystem::path &path);
bool isUnder(const std::filesystem::path &path, const std::filesystem::path &root);
bool shouldEmit(const std::filesystem::path &path, const Config &config);
std::string declarationAccess(const clang::Decl &declaration);
llvm::json::Object locationObject(clang::SourceLocation location,
                                  const clang::SourceManager &sources);
llvm::json::Object declarationEvent(const clang::Decl &declaration,
                                    clang::ASTContext &context,
                                    const Config &config);
llvm::json::Object headerSeenEvent(const std::filesystem::path &path,
                                   const Config &config);
void writeEvent(llvm::json::Object event);
} // namespace api_inventory

#pragma once

#include "Schema.h"
#include "clang/AST/RecursiveASTVisitor.h"
#include <map>
#include <set>
#include <string>

namespace api_inventory {
class InventoryVisitor : public clang::RecursiveASTVisitor<InventoryVisitor> {
public:
  InventoryVisitor(clang::ASTContext &context, const Config &config);
  bool VisitDecl(clang::Decl *declaration);
  void emitSummary();
  bool hasUnhandledKinds() const;

private:
  void emitAliasReachability(const clang::Decl &declaration,
                             const llvm::json::Object &event);

  clang::ASTContext &context_;
  const Config &config_;
  std::map<std::string, unsigned> dispositions_;
  std::map<std::string, unsigned> kinds_;
  std::set<std::string> unhandledKinds_;
  unsigned emitted_ = 0;
  unsigned aliasReachabilityEvents_ = 0;
  unsigned returnedTypeReachabilityEvents_ = 0;
  unsigned externalDecls_ = 0;
  unsigned occtUnemittedDecls_ = 0;
};

} // namespace api_inventory

require "./visitor"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors

            # Visit a Infix node, evaluating both sides and performing an operation on them, producing one result.
            @[CompilerVisitor(node: AST::Infix)]
            class Infix < Visitor

                def run(compiler : Compiler, node : AST::Node)
                    node = node.as?(AST::Infix)
                    if node.nil?
                        return
                    end
                    
                    left = compiler.compile node.left
                    if left.nil?
                        return
                    end

                    right = compiler.compile node.right
                    if right.nil?
                        return
                    end

                    case node.operator
                    when "+"
                        return compiler.builder.add(left, right)
                    when "-"
                        return compiler.builder.sub(left, right)
                    when "=="
                        return compiler.builder.icmp(LLVM::IntPredicate::EQ, left, right)
                    when "<"
                        return compiler.builder.icmp(LLVM::IntPredicate::SLT, left, right)
                    when "<="
                        return compiler.builder.icmp(LLVM::IntPredicate::SLE, left, right)
                    when ">"
                        return compiler.builder.icmp(LLVM::IntPredicate::SGT, left, right)
                    when ">="
                        return compiler.builder.icmp(LLVM::IntPredicate::SGE, left, right)
                    when "%"
                        return compiler.builder.srem(left, right)
                    when "/"
                        return compiler.builder.sdiv(left, right)
                    when "or"
                        return compiler.builder.or(left, right)
                    when "and"
                        return compiler.builder.and(left, right)
                    end
                end

            end
        end
    end
end
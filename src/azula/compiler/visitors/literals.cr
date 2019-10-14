require "./visitor"
require "../../ast/*"
require "../compiler"
require "../../types"

module Azula
    module Compiler
        module Visitors

            # Visit a Integer and return the Value.
            @[CompilerVisitor(node: AST::IntegerLiteral)]
            class IntegerLiteral < Visitor
                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::IntegerLiteral)
                    if node.nil?
                        return
                    end
                    case node.size
                    when 8
                        return compiler.types[Types::Type::INT8].const_int node.value
                    else
                        return compiler.types[Types::Type::INT].const_int node.value
                    end
                end
            end

            # Visit a Float and return the Value.
            @[CompilerVisitor(node: AST::FloatLiteral)]
            class FloatLiteral < Visitor
                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::FloatLiteral)
                    if node.nil?
                        return
                    end
                    return compiler.types[Types::Type::FLOAT].const_double node.value.to_f64
                end
            end

            # Visit a String and return the Value.
            @[CompilerVisitor(node: AST::StringLiteral)]
            class StringLiteral < Visitor
                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::StringLiteral)
                    if node.nil?
                        return
                    end
                    return compiler.create_string node.value
                end
            end

            # Visit a Boolean and return the Value.
            @[CompilerVisitor(node: AST::BooleanLiteral)]
            class BooleanLiteral < Visitor
                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::BooleanLiteral)
                    if node.nil?
                        return
                    end
                    return compiler.types[Types::Type::BOOL].const_int (node.value ? 1 : 0)
                end
            end

        end
    end
end
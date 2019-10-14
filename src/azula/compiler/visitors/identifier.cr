require "./visitor"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors

            # Visit an Identifier, get the pointer that the name references and load the value, returning it.
            @[CompilerVisitor(node: AST::Identifier)]
            class Identifier < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::Identifier)
                    if node.nil?
                        return
                    end
                    ptr = compiler.vars.fetch node.ident, nil
                    if ptr.nil?
                        return
                    end
                    return compiler.builder.load ptr
                end

            end
        end
    end
end
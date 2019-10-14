require "./visitor"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors

            # Visit a Program node and then compile each individual statement inside the program.
            @[CompilerVisitor(node: AST::Continue)]
            class Continue < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::Continue)
                    if node.nil?
                        return
                    end

                    compiler.builder.br compiler.current_loop_cond.not_nil!
                end

            end
        end
    end
end
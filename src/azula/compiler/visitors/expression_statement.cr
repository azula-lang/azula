require "./visitor"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors

            # Visit a Program node and then compile each individual statement inside the program.
            @[CompilerVisitor(node: AST::ExpressionStatement)]
            class ExpressionStatement < Visitor

                def run(compiler : Compiler, node : AST::Node)
                    node = node.as?(AST::ExpressionStatement)
                    if node.nil?
                        return
                    end
                    compiler.compile node.expression
                end

            end
        end
    end
end
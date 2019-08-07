require "./visitor"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors

            # Visit a Program node and then compile each individual statement inside the program.
            @[CompilerVisitor(node: AST::Block)]
            class Block < Visitor

                def run(compiler : Compiler, node : AST::Node)
                    node = node.as?(AST::Block)
                    if node.nil?
                        return
                    end
                    node.statements.each do |stmt|
                        compiler.compile stmt
                    end
                end

            end
        end
    end
end
require "./visitor"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors

            # Visit a Array and return the Value.
            @[CompilerVisitor(node: AST::Package)]
            class Package < Visitor
                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::Package)
                    if node.nil?
                        return
                    end
                    
                    compiler.package_name = node.name
                    return
                end
            end
        end
    end
end
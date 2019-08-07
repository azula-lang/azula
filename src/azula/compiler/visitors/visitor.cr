require "llvm"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors
            annotation CompilerVisitor
            end

            abstract class Visitor
                abstract def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
            end
        end
    end
end
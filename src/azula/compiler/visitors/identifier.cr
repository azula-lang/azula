require "./visitor"
require "../../ast/*"
require "../compiler"
require "../../errors/*"

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
                        ErrorManager.add_error Error.new "variable not defined '#{node.ident}'", node.token.file, node.token.linenumber, node.token.charnumber
                        return
                    end
                    return compiler.builder.load ptr
                end

            end
        end
    end
end
require "./visitor"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors

            # Visit a Program node and then compile each individual statement inside the program.
            @[CompilerVisitor(node: AST::Return)]
            class Return < Visitor

                def run(compiler : Compiler, node : AST::Node)
                    node = node.as?(AST::Return)
                    if node.nil?
                        return
                    end

                    # No return value, just return
                    if node.values.size == 0
                        compiler.has_return = true
                        compiler.builder.ret
                        return
                    end

                    # Has return value
                    val = compiler.compile node.values[0]
                    if val.nil?
                        return
                    end

                    compiler.has_return = true
                    compiler.builder.ret val.not_nil!.to_unsafe
                    return
                end

            end
        end
    end
end
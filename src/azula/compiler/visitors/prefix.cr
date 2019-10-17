require "./visitor"
require "../../ast/*"
require "../compiler"
require "../../errors/*"

module Azula
    module Compiler
        module Visitors

            # Visit a Prefix node, evaluating both sides and performing an operation on them, producing one result.
            @[CompilerVisitor(node: AST::Prefix)]
            class Prefix < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::Prefix)
                    if node.nil?
                        return
                    end

                    if node.operator == "*"
                        right = node.right.as?(AST::Identifier)
                        if right.nil?
                            return
                        end
                        ptr = compiler.vars.fetch right.ident, nil
                        if ptr.nil?
                            ErrorManager.add_error Error.new "variable not defined '#{right.ident}'", node.token.file, node.token.linenumber, node.token.charnumber
                            return
                        end
                        return ptr
                    end

                    right = compiler.compile(node.right)
                    if right.nil?
                        return
                    end
                    
                    case node.operator
                    when "-"
                        return compiler.builder.mul(right.not_nil!, compiler.context.int32.const_int(-1))
                    when "!"
                        return compiler.builder.not(right.not_nil!)
                    when "&"
                        return compiler.builder.load right.not_nil!
                    end
                    
                end

            end
        end
    end
end
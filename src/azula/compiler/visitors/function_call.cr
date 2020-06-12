require "./visitor"
require "../../ast/*"
require "../compiler"
require "llvm"
require "../../errors"

module Azula
    module Compiler
        module Visitors

            # Visit a FunctionCall node and return the value(s) it returns.
            @[CompilerVisitor(node: AST::FunctionCall)]
            class FunctionCall < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::FunctionCall)
                    if node.nil?
                        return
                    end

                    # Compile each of the arguments
                    args = [] of LLVM::Value
                    node.arguments.each do |arg|
                        val = compiler.compile arg
                        if val.nil?
                            ErrorManager.add_error Error.new "could not compile value", node.token.file, node.token.linenumber, node.token.charnumber
                            return
                        end
                        # if val.type.kind == LLVM::Type::Kind::Pointer
                        #     val = compiler.builder.load val
                        # end
                        args << val
                    end

                    if node.function_name.ident == "println"
                        args.each do |arg|
                            compiler.builder.call compiler.print_funcs[arg.type.to_s], arg
                            compiler.builder.call compiler.builtin_printfunc, [compiler.builder.global_string_pointer(" ")]
                        end
                        compiler.builder.call compiler.builtin_printfunc, [compiler.builder.global_string_pointer("%c"), compiler.context.int32.const_int(10)]
                        return
                    end

                    if compiler.access == "external"
                        return compiler.builder.call compiler.main_module.functions[node.function_name.ident], args
                    end
                    return compiler.builder.call compiler.main_module.functions[(compiler.access == nil ? compiler.package_name.not_nil! : compiler.access.not_nil!) + "." + node.function_name.ident], args
                end

            end
        end
    end
end
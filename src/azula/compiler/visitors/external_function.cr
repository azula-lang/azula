require "./visitor"
require "../../ast/*"
require "../compiler"
require "llvm"

module Azula
    module Compiler
        module Visitors

            # Visit a External Function node and then compile each individual statement inside the program.
            @[CompilerVisitor(node: AST::ExternFunction)]
            class ExternFunction < Visitor

                def run(compiler : Compiler, node : AST::Node)
                    node = node.as?(AST::ExternFunction)
                    if node.nil?
                        return
                    end
                    
                    # Keep the builder to set it back once the function body is compiled
                    old_builder = compiler.builder

                    # Get the arguments for the function
                    args = [] of LLVM::Type
                    node.parameters.each do |param|
                        arg_type = compiler.types.fetch param.type, nil
                        if arg_type.nil?
                            arg_type = compiler.structs.fetch param.type, nil
                            if arg_type.nil?
                                next
                            end
                        end
                        args << arg_type
                    end

                    # Get the return type of the function
                    return_type = compiler.types.fetch node.return_types[0], nil
                    if return_type.nil?
                        return_type = compiler.structs.fetch node.return_types[0], nil
                        if return_type.nil?
                            return
                        end
                    end

                    compiler.main_module.functions.add(node.function_name.ident, args, return_type)
                    compiler.builder = old_builder
                    return
                end

            end
        end
    end
end
require "./visitor"
require "../../ast/*"
require "../compiler"
require "llvm"

module Azula
    module Compiler
        module Visitors

            # Visit a Program node and then compile each individual statement inside the program.
            @[CompilerVisitor(node: AST::Function)]
            class Function < Visitor

                def run(compiler : Compiler, node : AST::Node)
                    node = node.as?(AST::Function)
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
                            # Check if struct
                            next
                        end
                        args << arg_type
                    end

                    # Get the return type of the function
                    return_type = compiler.types.fetch node.return_types[0], nil
                    if return_type.nil?
                        # Check structs
                        return
                    end

                    compiler.main_module.functions.add(node.function_name.ident, args, return_type) do |func|
                        compiler.current_func = func
                        entry = func.basic_blocks.append "entry" do |builder|
                            compiler.builder = builder

                            index = 0
                            node.parameters.each do |param|
                                ptr = builder.alloca compiler.types[param.type], param.ident
                                compiler.vars[param.ident] = ptr
                                builder.store func.params[index], ptr
                                index += 1
                            end

                            compiler.compile node.body

                            if !compiler.has_return
                                compiler.builder.ret
                            end

                            compiler.has_return = false
                        end
                        compiler.current_func = nil
                    end
                    compiler.builder = old_builder
                    return
                end

            end
        end
    end
end
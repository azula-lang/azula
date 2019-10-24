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

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::Function)
                    if node.nil?
                        return
                    end

                    
                    # Keep the builder to set it back once the function body is compiled
                    old_builder = compiler.builder

                    # Get the arguments for the function
                    args = [] of LLVM::Type
                    node.parameters.each do |param|
                        if param.type.main_type == Types::TypeEnum::ARRAY
                            assign_type = compiler.array_type(param.type.secondary_type.not_nil!, 10).pointer
                            args << assign_type
                            next
                        end
                        arg_type = compiler.types.fetch param.type.main_type, nil
                        if arg_type.nil?
                            arg_type = compiler.structs.fetch param.type.main_type, nil
                            if arg_type.nil?
                                ErrorManager.add_error Error.new "could not find type #{param.type.main_type}", node.token.file, node.token.linenumber, node.token.charnumber
                                return
                            end
                        end
                        args << arg_type
                    end

                    # Get the return type of the function
                    return_type = compiler.types.fetch node.return_type.main_type, nil
                    if return_type.nil?
                        return_type = compiler.structs.fetch node.return_type.main_type, nil
                        if return_type.nil?
                            ErrorManager.add_error Error.new "could not find type #{node.return_type.main_type}", node.token.file, node.token.linenumber, node.token.charnumber
                            return
                        end
                    end

                    compiler.main_module.functions.add(node.function_name.ident, args, return_type) do |func|
                        compiler.current_func = func
                        entry = func.basic_blocks.append "entry" do |builder|
                            compiler.builder = builder

                            index = 0
                            node.parameters.each do |param|
                                param_type = compiler.types.fetch param.type.main_type, nil
                                if param.type.main_type == Types::TypeEnum::ARRAY
                                    param_type = compiler.array_type(param.type.secondary_type.not_nil!, 10).array(0).pointer
                                else
                                    if param_type.nil?
                                        param_type = compiler.structs.fetch param.type.main_type, nil
                                        if param_type.nil?
                                            ErrorManager.add_error Error.new "could not find type #{param.type.main_type}", node.token.file, node.token.linenumber, node.token.charnumber
                                            return
                                        end
                                    end
                                end
                                ptr = builder.alloca param_type, param.ident
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
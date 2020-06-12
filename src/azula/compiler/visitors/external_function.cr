require "./visitor"
require "../../ast/*"
require "../compiler"
require "llvm"
require "../../errors"

module Azula
    module Compiler
        module Visitors

            # Visit a External Function node and then compile each individual statement inside the program.
            @[CompilerVisitor(node: AST::ExternFunction)]
            class ExternFunction < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::ExternFunction)
                    if node.nil?
                        return
                    end
                    
                    # Keep the builder to set it back once the function body is compiled
                    old_builder = compiler.builder

                    # Get the arguments for the function
                    args = [] of LLVM::Type
                    node.parameters.each do |param|
                        if param.type.main_type == Types::TypeEnum::POINTER
                            arg_type = compiler.types.fetch param.type.secondary_type.not_nil!.main_type, nil
                            if arg_type.nil?
                                arg_type = compiler.structs.fetch param.type.secondary_type.not_nil!.main_type, nil
                                if arg_type.nil?
                                    ErrorManager.add_error Error.new "could not find type #{param.type.secondary_type.not_nil!.main_type}", node.token.file, node.token.linenumber, node.token.charnumber
                                    return
                                end
                            end
                            args << arg_type.pointer
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
                        if node.return_type.main_type == Types::TypeEnum::POINTER
                            return_type = compiler.types.fetch node.return_type.secondary_type.not_nil!.main_type, nil
                            if return_type.nil?
                                if compiler.access == "external"
                                    name = node.return_type.secondary_type.not_nil!.main_type.as(String)
                                else
                                    name = (compiler.access == nil ? compiler.package_name.not_nil! : compiler.access.not_nil!) + "." + node.return_type.secondary_type.not_nil!.main_type.as(String)
                                end
                                return_type = compiler.structs.fetch name, nil
                                if return_type.nil?
                                    ErrorManager.add_error Error.new "could not find type '#{node.return_type.secondary_type.not_nil!.main_type}'", node.token.file, node.token.linenumber, node.token.charnumber
                                    return
                                end
                            end
                            return_type = return_type.pointer
                        else
                            return_type = compiler.structs.fetch node.return_type.main_type, nil
                            if return_type.nil?
                                ErrorManager.add_error Error.new "could not find type #{node.return_type.main_type}", node.token.file, node.token.linenumber, node.token.charnumber
                                return
                            end
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
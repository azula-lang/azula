require "./visitor"
require "../../ast/*"
require "../compiler"
require "../../errors/*"

module Azula
    module Compiler
        module Visitors

            # Visit a Assign node and then assign the compiled value to the identifier.
            @[CompilerVisitor(node: AST::Assign)]
            class Assign < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::Assign)
                    if node.nil?
                        return
                    end
                    ident = node.idents[0].as?(AST::TypedIdentifier)
                    if ident.nil?
                        ident = node.idents[0].as(AST::Identifier)
                        alloca = compiler.vars.fetch ident.ident, nil
                        if alloca.nil?
                            ErrorManager.add_error Error.new "attempted to read a variable that doesn't exist", node.token.file, node.token.linenumber, node.token.charnumber
                            return
                        end
                        val = compiler.compile node.values[0]
                        if val.nil?
                            ErrorManager.add_error Error.new "couldn't compile value of assign", node.token.file, node.token.linenumber, node.token.charnumber
                            return
                        end
                        compiler.builder.store val.not_nil!.to_unsafe, alloca
                        return
                    end

                    var = false
                    if ident.type.main_type == "var"
                        var = true
                    else
                        assign_type : LLVM::Type? = nil
                        if ident.type.main_type == Types::TypeEnum::POINTER
                            assign_type = compiler.types.fetch ident.type.secondary_type.not_nil!.main_type, nil
                            if assign_type.nil?
                                name = (compiler.access == nil ? compiler.package_name.not_nil! : compiler.access.not_nil!) + "." + ident.type.secondary_type.not_nil!.main_type.as(String)
                                assign_type = compiler.structs.fetch name, nil
                                if assign_type.nil?
                                    ErrorManager.add_error Error.new "could not find type '#{ident.type.secondary_type.not_nil!.main_type}'", node.token.file, node.token.linenumber, node.token.charnumber
                                    return
                                end
                            end
                            assign_type = assign_type.pointer
                        elsif ident.type.main_type == Types::TypeEnum::ARRAY
                            # assign_type = ident.type.secondary_type.not_nil!
                        else
                            # Get type of vars to be assigned
                            assign_type = compiler.types.fetch ident.type.main_type, nil
                            if assign_type.nil?
                                name = (compiler.access == nil ? compiler.package_name.not_nil! : compiler.access.not_nil!) + "." + ident.type.main_type.as(String)
                                assign_type = compiler.structs.fetch name, nil
                                if assign_type.nil?
                                    ErrorManager.add_error Error.new "could not find type '#{ident.type.main_type}'", node.token.file, node.token.linenumber, node.token.charnumber
                                    return
                                end
                            end
                        end
                        if assign_type.nil?
                            ErrorManager.add_error Error.new "error assigning type. Could not find '#{ident.type.main_type}'.", node.token.file, node.token.linenumber, node.token.charnumber
                            return
                        end
                    end

                    # Compile value of assign statement
                    if node.values.size <= 0
                        ErrorManager.add_error Error.new "incorrect number of values.", node.token.file, node.token.linenumber, node.token.charnumber
                        return
                    end
                    val = compiler.compile node.values[0]
                    if val.nil?
                        return
                    end

                    if var
                        assign_type = val.type
                        if assign_type.kind.struct?
                            assign_type = compiler.last_struct
                        end
                    end

                    # Create allocation for variable
                    ptr = compiler.builder.alloca assign_type.not_nil!, ident.ident
                    compiler.vars[ident.ident] = ptr
                    compiler.builder.store val.not_nil!.to_unsafe, ptr

                    # Assign each other variable
                    (node.values.size - 1).times do |index|
                        val = compiler.compile node.values[index + 1]
                        ident = node.idents[index + 1].as?(AST::Identifier)
                        ptr = compiler.builder.alloca assign_type.not_nil!, ident.not_nil!.ident
                        compiler.vars[ident.not_nil!.ident] = ptr
                        compiler.builder.store val.not_nil!.to_unsafe, ptr
                    end
                end

            end
        end
    end
end
require "./visitor"
require "../../ast/*"
require "../compiler"
require "llvm"
require "../../errors"

module Azula
    module Compiler
        module Visitors

            # Visit a Struct statement
            @[CompilerVisitor(node: AST::Struct)]
            class Struct < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::Struct)
                    if node.nil?
                        return
                    end

                    vars = [] of LLVM::Type
                    indexes = {} of String=>Int32
                    i = 0
                    node.fields.each do |field|
                        type = compiler.types.fetch field.type.main_type, nil
                        if !type.nil?
                            vars << type
                        else
                            struc = compiler.structs.fetch field.type.main_type, nil
                            if struc.nil?
                                ErrorManager.add_error Error.new "could not find type #{field.type.main_type}", node.token.file, node.token.linenumber, node.token.charnumber
                                return
                            end
                            vars << struc
                        end
                        indexes[field.ident] = i
                        i += 1
                    end
                    compiler.structs[compiler.package_name.not_nil! + "." + node.struct_name.ident] = compiler.context.struct(vars, compiler.package_name.not_nil! + "." + node.struct_name.ident)
                    compiler.struct_fields[compiler.package_name.not_nil! + "." + node.struct_name.ident] = indexes

                    return
                end

            end

            # Visit a Struct initialise expression and return the result
            @[CompilerVisitor(node: AST::StructInitialise)]
            class StructInitialise < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::StructInitialise)
                    if node.nil?
                        return
                    end

                    name = (compiler.access == nil ? compiler.package_name.not_nil! + "." : (compiler.access == "external" ? "" : compiler.access.not_nil! + ".")) + node.struct_name.ident
                    struc = compiler.structs.fetch name, nil
                    if struc.nil?
                        ErrorManager.add_error Error.new "unknown struct: '#{name}'.", node.token.file, node.token.linenumber, node.token.charnumber
                        return
                    end
                    vals = [] of LLVM::Value
                    node.values.each do |val|
                        value = compiler.compile val
                        if !value.nil?
                            vals << value
                        end
                    end
                    val = struc.context.const_struct(vals)
                    compiler.last_struct = struc
                    return val
                end

            end

            # # Visit a Struct field access expression and return the result
            # @[CompilerVisitor(node: AST::StructAccess)]
            # class StructAccess < Visitor

            #     def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
            #         node = node.as?(AST::StructAccess)
            #         if node.nil?
            #             return
            #         end

            #         struc = compiler.compile node.struct_exp
            #         if struc.nil?
            #             return
            #         end

            #         alloca = compiler.builder.alloca struc.not_nil!.type
            #         compiler.builder.store struc.not_nil!, alloca

            #         fields = compiler.struct_fields[struc.type.struct_name]
            #         field = fields.fetch node.field.as(Azula::AST::Identifier).ident, nil
            #         if field.nil?
            #             puts "invalid field"
            #             return
            #         end

            #         gep = compiler.builder.gep alloca, compiler.context.int32.const_int(0), compiler.context.int32.const_int(field)
            #         load = compiler.builder.load gep
            #         return load
            #     end

            # end
        end
    end
end
require "./visitor"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors

            # Visit a Program node and then compile each individual statement inside the program.
            @[CompilerVisitor(node: AST::Access)]
            class Access < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::Access)
                    if node.nil?
                        return
                    end
                    
                    struc = nil
                    ident = node.left_exp.as?(AST::Identifier)
                    if ident.nil?
                        struc = compiler.compile node.left_exp
                        if struc.nil?
                            return
                        end
                    else
                        is_var = compiler.vars.fetch ident.not_nil!.ident, nil
                        if is_var != nil
                            struc = compiler.compile node.left_exp
                            if struc.nil?
                                return
                            end
                        end
                    end

                    if !struc.nil?
                        alloca = compiler.builder.alloca struc.not_nil!.type
                        compiler.builder.store struc.not_nil!, alloca

                        fields = compiler.struct_fields[struc.type.struct_name.not_nil!]
                        field = fields.fetch node.access_field.as(Azula::AST::Identifier).ident, nil
                        if field.nil?
                            puts "invalid field"
                            return
                        end

                        gep = compiler.builder.gep alloca, compiler.context.int32.const_int(0), compiler.context.int32.const_int(field)
                        load = compiler.builder.load gep
                        return load
                    end

                    old_access = compiler.access
                    compiler.access = ident.not_nil!.ident
                    val = compiler.compile node.access_field

                    compiler.access = old_access
                    return val
                end

            end
        end
    end
end
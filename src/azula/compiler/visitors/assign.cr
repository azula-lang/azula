require "./visitor"
require "../../ast/*"
require "../compiler"

module Azula
    module Compiler
        module Visitors

            # Visit a Assign node and then assign the compiled value to the identifier.
            @[CompilerVisitor(node: AST::Assign)]
            class Assign < Visitor

                def run(compiler : Compiler, node : AST::Node)
                    node = node.as?(AST::Assign)
                    if node.nil?
                        return
                    end
                    ident = node.idents[0].as?(AST::TypedIdentifier)
                    if ident.nil?
                        ident = node.idents[0].as(AST::Identifier)
                        alloca = compiler.vars.fetch ident.ident, nil
                        if alloca.nil?
                            return
                        end
                        val = compiler.compile node.values[0]
                        if val.nil?
                            return
                        end
                        compiler.builder.store val.not_nil!.to_unsafe, alloca
                        return
                    end
                    # Get type of vars to be assigned
                    assign_type = compiler.types.fetch ident.type, nil
                    if assign_type.nil?
                        # Check structs
                        return
                    end
                    # Compile value of assign statement
                    val = compiler.compile node.values[0]
                    # Create allocation for variable
                    ptr = compiler.builder.alloca assign_type, ident.ident
                    compiler.vars[ident.ident] = ptr
                    compiler.builder.store val.not_nil!.to_unsafe, ptr

                    # Assign each other variable
                    (node.values.size - 1).times do |index|
                        val = compiler.compile node.values[index + 1]
                        ident = node.idents[index + 1].as?(AST::Identifier)
                        ptr = compiler.builder.alloca assign_type, ident.not_nil!.ident
                        compiler.vars[ident.not_nil!.ident] = ptr
                        compiler.builder.store val.not_nil!.to_unsafe, ptr
                    end
                end

            end
        end
    end
end
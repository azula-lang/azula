require "./visitor"
require "../../ast/*"
require "../compiler"
require "llvm"
require "../../errors/*"

module Azula
    module Compiler
        module Visitors

            # Visit a External Function node and then compile each individual statement inside the program.
            @[CompilerVisitor(node: AST::ExternStruct)]
            class ExternalStruct < Visitor

                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::ExternStruct)
                    if node.nil?
                        return
                    end
                    
                    llvm_struct = LibLLVM.struct_create_named(compiler.context.to_unsafe, node.struct_name.ident)
                    the_struct = LLVM::Type.new llvm_struct
                    compiler.structs[node.struct_name.ident] = the_struct
                    return
                end

            end
        end
    end
end
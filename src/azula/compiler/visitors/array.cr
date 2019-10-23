require "./visitor"
require "../../ast/*"
require "../compiler"
require "../../types"

module Azula
    module Compiler
        module Visitors

            # Visit a Array and return the Value.
            @[CompilerVisitor(node: AST::ArrayExp)]
            class ArrayLiteral < Visitor
                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::ArrayExp)
                    if node.nil?
                        return
                    end
                    
                    array_type = compiler.array_type node.type, node.values.size
                    values = [] of LLVM::Value
                    node.values.each do |v|
                        v = compiler.compile v
                        if v.nil?
                            return
                        end
                        values << v
                    end
                    alloca = compiler.builder.alloca array_type
                    array = array_type.const_array(values)
                    puts array.type
                    compiler.builder.store array, alloca
                    return alloca
                end
            end

            # Visit a Integer and return the Value.
            @[CompilerVisitor(node: AST::ArrayAccess)]
            class ArrayIndex < Visitor
                def run(compiler : Compiler, node : AST::Node) : LLVM::Value?
                    node = node.as?(AST::ArrayAccess)
                    if node.nil?
                        return
                    end
                    
                    array = compiler.compile node.array
                    if array.nil?
                        return
                    end

                    index = compiler.compile node.index
                    if index.nil?
                        return
                    end

                    gep = compiler.builder.inbounds_gep array, compiler.context.int32.const_int(0), index
                    load = compiler.builder.load gep
                    return load
                end
            end
        end
    end
end
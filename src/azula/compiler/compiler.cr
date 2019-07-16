require "llvm"
require "../parser/*"

macro convert_and_check_nil(val)
    node = node.as?(AST::{{val}})
    return_if_nil node
end

module Azula
    class Compiler

        @module : LLVM::Module
        @builder : LLVM::Builder
        
        def initialize(@node : Azula::AST::Node)
            @context = LLVM::Context.new
            @module = @context.new_module("main_module")
            glob = @module.globals.add @context.int8.array(5), "yes"
            glob.initializer = @context.const_string("heyo")
            @builder = @context.new_builder
            printfunc = @module.functions.add("printf", [@context.void_pointer], @context.int32, true)
            mainfunc = @module.functions.add("main", [@context.void_pointer], @context.void, true)

            entry = mainfunc.basic_blocks.append "entry"

            #@builder.call printfunc, [glob]
            #@builder.ret

            @builder.ret @context.const_string "hi"

            #@builder.global_string_pointer "hi", "yes"
        end

        # def compile(node : Azula::AST::Node = @node)
        #     case node
        #     when .is_a?(Azula::AST::Program)
        #         convert_and_check_nil Program
        #         node.statements.each do |stmt|
        #             self.compile stmt
        #         end
        #     when .is_a?(Azula::AST::Block)
        #         convert_and_check_nil Block
        #         node.statements.each do |stmt|
        #             self.compile stmt
        #         end
        #     when .is_a?(Azula::AST::Function)
        #         convert_and_check_nil Function
        #         compile_function node
        #     when .is_a?(Azula::AST::FunctionCall)
        #         @module.invoke
        #     end
        #     end
        # end

        # def compile_function(function : Azula::AST::Function)
        #     @module.functions.add(function.function_name.ident, [@context.int32], @context.int32)
        #     builder = @context.new_builder
        #     self.compile function.body
        # end

        def write_to_file(file : String)
            @module.print_to_file file
        end

    end
end
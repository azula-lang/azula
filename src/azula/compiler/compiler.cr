require "./visitors/*"
require "../ast/*"
require "llvm"
require "../types"
require "./builtins"

module Azula
    module Compiler

        class Compiler

            @context : LLVM::Context
            @main_module : LLVM::Module
            @builder : LLVM::Builder
            @current_func : LLVM::Function? = nil

            @vars = {} of String=>LLVM::Value
            @types = {} of Types::Type=>LLVM::Type
            @builtin_funcs = {} of String=>LLVM::Function

            @structs = {} of String=>LLVM::Type
            @struct_fields = {} of String=>Hash(String, Int32)

            @visitors = {} of AST::Node.class=>Visitors::Visitor

            @has_return : Bool = false

            @string_type : LLVM::Type

            @print_funcs = {} of LLVM::Type=>LLVM::Function
            @builtin_printfunc : LLVM::Function

            @current_loop_cond : LLVM::BasicBlock? = nil

            getter context
            getter main_module
            getter builder
            getter current_func
            getter vars
            getter has_return
            getter types
            getter string_type
            getter print_funcs
            getter builtin_printfunc

            getter structs
            getter struct_fields

            setter builder
            setter current_func
            setter has_return
            getter current_loop_cond
            setter current_loop_cond

            def initialize
                @context = LLVM::Context.new
                @main_module = @context.new_module "main_module"
                @builder = @context.new_builder

                @string_type = @context.struct([@context.int8.pointer, @context.int32, @context.int32], "String")
                @types = {
                    Types::Type::VOID => @context.void,
                    Types::Type::INT => @context.int32,
                    Types::Type::INT8 => @context.int8,
                    Types::Type::BOOL => @context.int1,
                    Types::Type::FLOAT => @context.double,
                    Types::Type::STRING => @string_type,
                    Types::Type::CSTRING => @context.int8.pointer,
                }

                @builtin_printfunc = @main_module.functions.add("printf", [@context.void_pointer], @context.int32, true)

                add_builtins
            end

            # Macro that registers each subclass of Visitor against the Node it is meant to visit
            def register_visitors
                {% for visitor in Visitors::Visitor.all_subclasses %}
                    @visitors[{{visitor.annotation(Visitors::CompilerVisitor)[:node]}}] = {{visitor}}.new
                {% end %}
            end

            # Take each node, use its visitor to generate the LLVM code for it.
            def compile(node : AST::Node) : LLVM::Value?
                visitor = @visitors.fetch node.class, nil
                if visitor.nil?
                    puts node
                    puts "unknown node"
                    return
                end
                return visitor.run self, node
            end

            def create_string(value : String) : LLVM::Value?
                ptr = @builder.global_string_pointer value
                str = @context.const_struct [
                    ptr,
                    @context.int32.const_int(value.bytesize),
                    @context.int32.const_int(value.size),
                ]
                alloca = @builder.alloca @string_type
                @builder.store str, alloca
                return alloca
            end

            # Write the LLIR to a file.
            def write_to_file(file : String)
                @main_module.print_to_file file
            end

            # Create an executable file using clang.
            def create_executable(file : String)
                LLVM.init_x86
                target = LLVM::Target.from_triple(LLVM.default_target_triple)
                machine = target.create_target_machine LLVM.default_target_triple
                machine.emit_obj_to_file @main_module, "#{file}.o"

                system "clang -o #{file} -lstdc++ -static #{file}.o"

                File.delete "#{file}.o"
            end

            def get_pointer(type : Types::Type)
                return @types[type].pointer
            end

        end

    end
end
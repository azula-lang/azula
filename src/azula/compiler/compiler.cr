require "./visitors/*"
require "../ast/*"
require "llvm"
require "../types"

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

            @has_return : Bool = false

            @string_type : LLVM::Type

            getter context
            getter main_module
            getter builder
            getter current_func
            getter vars
            getter has_return
            getter types
            getter string_type

            setter builder
            setter current_func
            setter has_return

            def initialize
                @context = LLVM::Context.new
                @main_module = @context.new_module "main_module"
                @builder = @context.new_builder

                @string_type = @context.struct([@context.int8.pointer, @context.int32, @context.int32], "String")
                @types = {
                    Types::Type::VOID => @context.void,
                    Types::Type::INT => @context.int32,
                    Types::Type::BOOL => @context.int1,
                    Types::Type::FLOAT => @context.double,
                    Types::Type::STRING => @string_type,
                }

                add_builtins
            end

            # Add the builtin functions
            def add_builtins
                builtin_printfunc = @main_module.functions.add("printf", [@context.void_pointer], @context.int32, true)
                print_func = @main_module.functions.add("print", [@string_type.pointer], @context.void, true) do |func|
                    entry = func.basic_blocks.append "entry" do | builder |
                        v = builder.gep func.params[0], @context.int32.const_int(0), @context.int32.const_int(0)
                        val = builder.load v
                        builder.call builtin_printfunc, val#
                        builder.ret
                    end
                end
                add_builtin_func("__printf", builtin_printfunc)
                add_builtin_func("print", print_func)
            end

            # Register builtin function
            def add_builtin_func(name : String, func : LLVM::Function)
                @builtin_funcs[name] = func
            end

            @visitors = {} of AST::Node.class=>Visitors::Visitor

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

                system "clang -o #{file} -lstdc++ #{file}.o"
            end

        end

    end
end
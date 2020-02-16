require "./visitors/*"
require "../ast/*"
require "llvm"
require "../types"
require "./builtins"
require "../errors/*"

module Azula
    module Compiler

        class AzlFunction
            def initialize(@name : String, @args : Array(LLVM::Type), @return_type : LLVM::Type)
            end

            getter name
            getter args
            getter return_type
        end

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

            @print_funcs = {} of String=>LLVM::Function
            @builtin_printfunc : LLVM::Function

            @current_loop_cond : LLVM::BasicBlock? = nil

            @functions: Array(AzlFunction) = [] of AzlFunction

            @package_name : String? = nil

            @access : String? = nil

            @imports : Array(String) = [] of String

            @project_top : String
            @last_struct : LLVM::Type? = nil

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

            getter functions
            getter imports

            setter builder
            setter current_func
            setter has_return
            getter current_loop_cond
            setter current_loop_cond

            getter package_name
            setter package_name

            getter access
            setter access

            getter project_top
            setter project_top
            getter last_struct
            setter last_struct

            def initialize(std : Bool = false)
                @context = LLVM::Context.new
                @main_module = @context.new_module "main_module"
                @builder = @context.new_builder
                @project_top = Dir.current

                @string_type = @context.struct([@context.int8.pointer, @context.int32, @context.int32], "String")
                @types = {
                    Types::TypeEnum::VOID => @context.void,
                    Types::TypeEnum::INT => @context.int32,
                    Types::TypeEnum::INT8 => @context.int8,
                    Types::TypeEnum::INT64 => @context.int64,
                    Types::TypeEnum::BOOL => @context.int1,
                    Types::TypeEnum::FLOAT => @context.double,
                    Types::TypeEnum::STRING => @string_type,
                    Types::TypeEnum::CSTRING => @context.int8.pointer,
                }

                @builtin_printfunc = @main_module.functions.add("printf", [@context.void_pointer], @context.int32, true)

                if !std
                    load_builtins
                else
                    add_builtins
                end
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
                    file = "unknown"
                    linenumber = 0
                    colnumber = 0
                    ErrorManager.add_error Error.new "unknown node found #{node}", file, linenumber, colnumber
                    return
                end
                if node.class != AST::Program && node.class != AST::Package && @package_name == nil
                    ErrorManager.add_error Error.new "file missing package declaration", "unknown", 0, 0
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
                if !Dir.exists?(".build")
                    Dir.mkdir ".build"
                end
                create_std_lib
                
                create_object_file ".build/#{file}"

                obj_files = [] of String
                @imports.each do |i|
                    obj_files << ".build/" + i + ".o"
                end

                system "clang -o #{file} -lstdc++ -static .build/#{file}.o #{obj_files.join(" ")} -lm"

                File.delete ".build/#{file}.o"
                obj_files.each do |o|
                    File.delete "#{o}"
                end
                Dir.rmdir ".build"
            end

            # Create an object file using clang.
            def create_object_file(file : String)
                LLVM.init_x86
                target = LLVM::Target.from_triple(LLVM.default_target_triple)
                machine = target.create_target_machine LLVM.default_target_triple
                machine.emit_obj_to_file @main_module, "#{file}.o"
            end

            def create_std_lib
                c = Azula::Compiler::Compiler.new true
                c.register_visitors
                c.compile AST::Program.new [] of AST::Statement

                c.functions.each do |f|
                    @main_module.functions.add(f.name, f.args, f.return_type)
                end

                c.create_object_file ".build/std"
                @imports << "std"
            end

            def get_pointer(type : Types::Type)
                return @types[type.main_type].pointer
            end

            def array_type(type : Types::Type, size : Int32) : LLVM::Type
                if type.main_type == Types::TypeEnum::ARRAY
                    return self.array_type(type.secondary_type.not_nil!, size)
                else
                    if type.main_type == Types::TypeEnum::STRING
                        return @types[type.main_type].pointer
                    end
                    return @types[type.main_type]
                end
            end

        end

    end
end
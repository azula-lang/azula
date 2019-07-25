require "llvm"
require "../parser/*"
require "../types"

macro convert_and_check_nil(val)
    node = node.as?(AST::{{val}})
    return_if_nil node
end

module Azula
    class Compiler

        @main_module : LLVM::Module
        @builder : LLVM::Builder
        @printfunc : LLVM::Function
        @putsfunc : LLVM::Function

        @types : Hash(Types::Type, LLVM::Type)

        @builtin_funcs : Hash(String, LLVM::Function)
        @vars : Hash(String, LLVM::Value)

        @current_func : LLVM::Function?
        @has_return = false

        @random = Random.new

        getter main_module
        getter compiler
        getter context
        
        def initialize(@node : Azula::AST::Node)
            @context = LLVM::Context.new
            @main_module = @context.new_module("main_module")
            @builder = @context.new_builder
            @printfunc = @main_module.functions.add("printf", [@context.void_pointer], @context.int32, true)
            @putsfunc = @main_module.functions.add("puts", [@context.void_pointer], @context.int32, true)
            # mainfunc = @module.functions.add("main", [] of LLVM::Type, @context.void, true) do |func|
            #     entry = func.basic_blocks.append "entry" do | builder |
            #         @builder = builder
            #         self.printf "Error: this works"
            #         builder.ret

            #     end
            # end
            # tostring = @main_module.functions.add("string", [@context.int32], @context.void_pointer, true) do |func|
            #     entry = func.basic_blocks.append "entry" do |builder|
            #         ptr = builder.trunc(func.params[0], @context.int8)
            #         array = @context.int8.const_array([ptr])
            #         alloca = builder.alloca @context.int8.array(1), "tmp"
            #         ptr1 = builder.store alloca, array
            #         builder.ret array
            #     end
            # end
            @types = {
                Types::Type::VOID => @context.void,
                Types::Type::INT => @context.int32,
                Types::Type::BOOL => @context.int1,
                Types::Type::FLOAT => @context.float
                #Types::Type::STRING => @context.int8.array(),
            }
            @builtin_funcs = {
                "print" => @printfunc,
                "puts" => @putsfunc,
                #"to_string" => tostring,
            }
            @vars = {} of String=>LLVM::Value
            LLVM.init_x86
            @compiler = LLVM::JITCompiler.new @main_module
        end

        def compile(node : Azula::AST::Node = @node) : LLVM::Value?
            case node
            when .is_a?(Azula::AST::Program)
                convert_and_check_nil Program
                node.statements.each do |stmt|
                    self.compile stmt
                end
                return
            when .is_a?(Azula::AST::Block)
                convert_and_check_nil Block
                node.statements.each do |stmt|
                    self.compile stmt
                end
                return
            when .is_a?(Azula::AST::ExpressionStatement)
                convert_and_check_nil ExpressionStatement
                self.compile node.expression
                return
            when .is_a?(Azula::AST::Function)
                convert_and_check_nil Function
                compile_function node
                return
            when .is_a?(Azula::AST::FunctionCall)
                convert_and_check_nil FunctionCall
                func = @builtin_funcs.fetch node.function_name.ident, nil
                args = [] of LLVM::Value
                node.arguments.each do | arg |
                    v = self.compile arg
                    if v.nil?
                        next
                    end
                    args << v
                end
                if func.nil?
                    val = @builder.call @main_module.functions[node.function_name.ident], args
                else
                    val = @builder.call func, args
                end
                return val
            when .is_a?(Azula::AST::Return)
                convert_and_check_nil Return
                if node.values.size == 0
                    @has_return = true
                    @builder.ret
                    return
                else
                    val = compile(node.values[0])
                    if val.nil?
                        puts "return value invalid"
                        return
                    end
                    @has_return = true
                    @builder.ret val.not_nil!.to_unsafe
                    return
                end
            when .is_a?(Azula::AST::Assign)
                convert_and_check_nil Assign
                val = self.compile node.values[0]
                ident = node.idents[0].as?(Azula::AST::TypedIdentifier)
                if !ident.nil?
                    ptr = @builder.alloca @types[ident.type], ident.ident
                    @vars[ident.ident] = ptr
                    @builder.store val.not_nil!.to_unsafe, ptr
                else
                    ident = node.idents[0].as(Azula::AST::Identifier)
                    alloca = @vars.fetch ident.ident, nil
                    if alloca.nil?
                        return
                    end
                    @builder.store val.not_nil!.to_unsafe, alloca
                end
                return
            when .is_a?(Azula::AST::IntegerLiteral)
                convert_and_check_nil IntegerLiteral
                return @context.int32.const_int node.value
            when .is_a?(Azula::AST::FloatLiteral)
                convert_and_check_nil FloatLiteral
                return @context.float.const_float node.value
            when .is_a?(Azula::AST::StringLiteral)
                convert_and_check_nil StringLiteral
                return @builder.global_string_pointer(node.value)
            when .is_a?(Azula::AST::BooleanLiteral)
                convert_and_check_nil BooleanLiteral
                if node.value
                    return @context.int1.const_int 1
                else
                    return @context.int1.const_int 0
                end
            when .is_a?(Azula::AST::Identifier)
                convert_and_check_nil Identifier
                ptr = @vars.fetch node.ident, nil
                if ptr.nil?
                    return
                end
                return @builder.load ptr
            when .is_a?(Azula::AST::Infix)
                convert_and_check_nil Infix
                left = self.compile node.left
                if left.nil?
                    return
                end
                right = self.compile node.right
                if right.nil?
                    return
                end
                case node.operator
                when "+"
                    return @builder.add(left, right)
                when "-"
                    return @builder.sub(left, right)
                when "=="
                    return @builder.icmp(LLVM::IntPredicate::EQ, left, right)
                when "<"
                    return @builder.icmp(LLVM::IntPredicate::SLT, left, right)
                when "<="
                    return @builder.icmp(LLVM::IntPredicate::SLE, left, right)
                when ">"
                    return @builder.icmp(LLVM::IntPredicate::SGT, left, right)
                when ">="
                    return @builder.icmp(LLVM::IntPredicate::SGE, left, right)
                when "%"
                    return @builder.srem(left, right)
                when "/"
                    return @builder.sdiv(left, right)
                when "or"
                    return @builder.or(left, right)
                when "and"
                    return @builder.and(left, right)
                else
                    puts "unknown operand"
                end
            when .is_a?(Azula::AST::If)
                convert_and_check_nil If
                if @current_func.nil?
                    return
                end
                old_builder = @builder
                condition = self.compile node.condition

                after_block = @current_func.not_nil!.basic_blocks.append "after" do | builder |
                    @builder = builder
                end

                if_block = @current_func.not_nil!.basic_blocks.append "if" do | builder |
                    old = @builder
                    @builder = builder
                    self.compile node.consequence
                    if !@has_return
                        @builder.br after_block
                    end
                    @builder = old
                end

                if !node.alternative.nil?
                    old = @builder
                    else_func = @current_func.not_nil!.basic_blocks.append "else" do | builder |
                        @builder = builder
                        self.compile node.alternative.not_nil!
                        if !@has_return
                            builder.br after_block
                        end
                    end
                    old_builder.cond condition.not_nil!.to_unsafe, if_block, else_func
                    @builder = old
                    return
                end

                old_builder.cond condition.not_nil!.to_unsafe, if_block, after_block
            when .is_a?(Azula::AST::While)
                convert_and_check_nil While
                if @current_func.nil?
                    return
                end

                old_builder = @builder
                loop_cond_builder = @builder
                cond : LLVM::Value? = nil

                after_block = @current_func.not_nil!.basic_blocks.append "after" do |builder|
                    @builder = builder
                end

                loop_cond = @current_func.not_nil!.basic_blocks.append "loop-cond" do |builder|
                    old = @builder
                    @builder = builder
                    cond = self.compile node.iterator
                    loop_cond_builder = builder
                    @builder = old
                end

                loop_block = @current_func.not_nil!.basic_blocks.append "loop" do |builder|
                    old = @builder
                    @builder = builder
                    self.compile node.body
                    @builder.br loop_cond
                    @builder = old
                end

                if !cond.nil?
                    loop_cond_builder.cond cond, loop_block, after_block
                end

                old_builder.br loop_cond

                return
            end
        end

        def compile_function(function : Azula::AST::Function)
            old_builder = @builder
            args = [] of LLVM::Type
            function.parameters.each do | param |
                a = @types.fetch param.type, nil
                if a.nil?
                    next
                end
                args << a
            end
            @main_module.functions.add(function.function_name.ident, args, @types[function.return_types[0]]) do |func|
                @current_func = func
                entry = func.basic_blocks.append "entry" do | builder |
                    @builder = builder
                    i = 0
                    function.parameters.each do | param |
                        ptr = @builder.alloca @types[param.type], param.ident
                        @vars[param.ident] = ptr
                        @builder.store func.params[i], ptr
                    end
                    self.compile function.body
                    if !@has_return
                        @builder.ret
                    end
                    @has_return = false
                end
                @current_func = nil
            end
            @builder = old_builder
        end

        def write_to_file(file : String)
            @main_module.print_to_file file
        end

    end
end
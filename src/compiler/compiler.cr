require "llvm"

class Azula::Compiler

    getter main_module, errors

    property has_return

    @main_module : LLVM::Module
    @builder : LLVM::Builder?
    @has_return : Bool = false
    @size : Int32 = 0

    def initialize
        @errors = [] of Azula::Error
        @symbols = {} of String=>Azula::Value
        @llvm_context = LLVM::Context.new
        @main_module = @llvm_context.new_module "main_module"
        printff = @main_module.functions.add("printf", [@llvm_context.void_pointer], @llvm_context.int32, true)
        @symbols["printf"] = Azula::Value.new(Azula::FunctionType.new(VoidType.new()), printff)
    end

    def compile(node : AST::Node, builder : LLVM::Builder? = nil, function : LLVM::Function? = nil) : Azula::Value?
        case node
        when AST::Program
            compile node.block
            return
        when AST::Block
            last_val : Azula::Value? = nil
            node.nodes.each do |node|
                last_val = compile node, builder, function
            end
            return last_val
        when AST::Function
            return compile_function node
        when AST::Assign
            compile_assign node, builder.not_nil!, function
        when AST::IntegerLiteral
            value : LLVM::Value? = nil
            case @size
            when 8
                value = @llvm_context.int8.const_int(node.value)
            when 16
                value = @llvm_context.int16.const_int(node.value)
            when 32
                value = @llvm_context.int32.const_int(node.value)
            when 64
                value = @llvm_context.int64.const_int(node.value)
            end
            return Azula::Value.new(Azula::IntegerType.new, value.not_nil!)
        when AST::FloatLiteral
            return Azula::Value.new(Azula::FloatType.new, @llvm_context.float.const_double(node.value))
        when AST::BooleanLiteral
            return Azula::Value.new(Azula::BooleanType.new, @llvm_context.int1.const_int(node.value ? 1 : 0))
        when AST::StringLiteral
            return Azula::Value.new(Azula::StringType.new, builder.not_nil!.global_string_pointer(node.value))
        when AST::Identifier
            return @symbols[node.value]
        when AST::Return
            @has_return = true
            if node.value.nil?
                builder.not_nil!.ret
                return
            end
            val = compile node.value.not_nil!, builder
            builder.not_nil!.ret val.not_nil!.raw_value.not_nil!.to_unsafe
            return 
        when AST::Infix
            return compile_infix node, builder.not_nil!, function
        when AST::FunctionCall
            if node.function.is_a?(Azula::AST::Identifier)
                func = @symbols[node.function.as(Azula::AST::Identifier).value]
            else
                func = compile node.function, builder
                if func.nil?
                    @errors << Azula::Error.new "could not compile function", Azula::ErrorType::Compiling, node.token
                    return
                end
            end

            args = [] of LLVM::Value

            node.arguments.each do |arg|
                val = compile arg, builder
                if val.nil?
                    @errors << Azula::Error.new "could not compile argument value", Azula::ErrorType::Compiling, node.token
                    return
                end
                while val.type.is_pointer?
                    val = dereference_pointer val, builder.not_nil!
                end
                args << val.raw_value.not_nil!
            end

            return Azula::Value.new func.type.as(Azula::FunctionType).returns, builder.not_nil!.call(func.function.not_nil!, args)
        when AST::If
            condition = compile node.condition, builder
            if condition.nil?
                return
            end

            while condition.type.is_pointer?
                condition = dereference_pointer condition, builder.not_nil!
            end

            if function.nil?
                @errors << Azula::Error.new "cannot have an if block outside of function", Azula::ErrorType::Compiling, node.token
                return
            end

            end_block = function.not_nil!.basic_blocks.append("after") do |builder|
                @builder = builder
            end

            true_block = function.not_nil!.basic_blocks.append("if") do |builder|
                compile node.true_block, builder, function
                if !@has_return
                    builder.br end_block
                end
                @has_return = false
            end

            if node.false_block.nil?
                builder.cond condition.raw_value.not_nil!, true_block, end_block
                return
            end

            false_block = function.not_nil!.basic_blocks.append("else") do |builder|
                compile node.false_block.not_nil!, builder, function
                if !@has_return
                    builder.br end_block
                end
                @has_return = false
            end

            builder.cond condition.raw_value.not_nil!, true_block, false_block
            return
        else
            return
        end
    end

    def compile_function(function : AST::Function) : Azula::Value?
        arguments = [] of LLVM::Type
        function.arguments.each do |arg|
            llvm_type = azl_type_to_llvm(arg.type.not_nil!)
            if llvm_type.nil?
                @errors << Azula::Error.new("could not convert to LLVM type", Azula::ErrorType::Compiling, function.token)
                return
            end
            arguments << llvm_type
        end

        return_type = azl_type_to_llvm(function.returns.type.not_nil!)
        if return_type.nil?
            @errors << Azula::Error.new "could not convert to LLVM type #{function.returns.type.not_nil!}", Azula::ErrorType::Compiling, function.token
            return
        end

        if function.identifier.value == "main"
            return_type = @llvm_context.int32
        end

        val : Azula::Value? = nil

        @main_module.functions.add(function.identifier.value, arguments, return_type) do |func|
            val = Azula::Value.new Azula::FunctionType.new(function.arguments.map { |arg| arg.type.not_nil! }, function.returns.type.not_nil!), function: func
            @symbols[function.identifier.value] = val
            old_symbols = @symbols
            func.basic_blocks.append "entry" do |builder|
                function.arguments.each_with_index do |arg, index|
                    @symbols[arg.value] = Azula::Value.new(arg.type.not_nil!, func.params[index])
                end

                last_val = nil
                old_builder = @builder
                @builder = builder
                function.body.nodes.each do |node|
                    last_val = compile node, @builder.not_nil!, func
                end
                if !@has_return
                    if last_val.nil? || last_val.is_a?(Azula::VoidType)
                        if function.identifier.value == "main"
                            @builder.not_nil!.ret @llvm_context.int32.const_int(0)
                        else
                            @builder.not_nil!.ret
                        end
                    else
                        @builder.not_nil!.ret last_val.not_nil!.raw_value.not_nil!.to_unsafe
                    end
                end
                @builder = old_builder
            end
            @symbols = old_symbols
        end

        @has_return = false

        return val.not_nil!
    end

    def compile_assign(assign : AST::Assign, builder : LLVM::Builder, function : LLVM::Function?)
        assign_type : LLVM::Type? = nil
        if !assign.identifier.type.not_nil!.is(InferType.new)
            assign_type = azl_type_to_llvm assign.identifier.type.not_nil!
            if assign_type.nil?
                @errors << Azula::Error.new("could not convert to LLVM type", Azula::ErrorType::Compiling, assign.token)
                return
            end
        end

        if assign.identifier.type.not_nil!.is_a?(IntegerType)
            @size = assign.identifier.type.not_nil!.as(IntegerType).size
        end

        value = compile assign.value, builder, function
        if value.nil?
            return
        end

        while value.type.is_pointer?
            value = dereference_pointer value, builder
        end

        if assign_type.nil?
            if !value.function.nil?
                assign_type = value.function.not_nil!.type
            else
                assign_type = value.raw_value.not_nil!.type
            end
        end

        if value.type.is_a?(FunctionType)
            @symbols[assign.identifier.value] = value
            return
        end

        var_pointer = builder.alloca assign_type.not_nil!, assign.identifier.value
        @symbols[assign.identifier.value] = Azula::Value.new(Azula::PointerType.new(assign.identifier.type.not_nil!), var_pointer)
        builder.store value.raw_value.not_nil!, var_pointer

        return
    end

    def compile_infix(infix : AST::Infix, builder : LLVM::Builder, function : LLVM::Function?) : Azula::Value?
        left = compile infix.left, builder, function
        if left.nil?
            return
        end
        while left.type.is_pointer?
            left = dereference_pointer left, builder
        end
        right = compile infix.right, builder, function
        if right.nil?
            return
        end
        while right.type.is_pointer?
            right = dereference_pointer right, builder
        end
        case infix.operator.type
        when TokenType::Plus
            case
            when left.not_nil!.type.is(IntegerType.new)
                return Azula::Value.new(left.not_nil!.type, builder.add(left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
            when left.not_nil!.type.is(FloatType.new)
                return Azula::Value.new(left.not_nil!.type, builder.fadd(left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
            end
        when TokenType::Minus
            case
            when left.not_nil!.type.is(IntegerType.new)
                return Azula::Value.new(left.not_nil!.type, builder.sub(left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
            when left.not_nil!.type.is(FloatType.new)
                return Azula::Value.new(left.not_nil!.type, builder.fsub(left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
            end        when TokenType::Asterisk
            return Azula::Value.new(left.not_nil!.type, builder.mul(left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
        when TokenType::Slash
            return Azula::Value.new(left.not_nil!.type, builder.sdiv(left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
        when TokenType::Modulo
            return Azula::Value.new(left.not_nil!.type, builder.srem(left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
        when TokenType::ShiftLeft
            return Azula::Value.new(left.not_nil!.type, builder.shl(left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
        when TokenType::ShiftRight
            return Azula::Value.new(left.not_nil!.type, builder.lshr(left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
        when TokenType::Eq
            return Azula::Value.new(left.not_nil!.type, builder.icmp(LLVM::IntPredicate::EQ, left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
        when TokenType::NotEq
            return Azula::Value.new(left.not_nil!.type, builder.not(builder.icmp(LLVM::IntPredicate::EQ, left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!)))
        when TokenType::Lt
            return Azula::Value.new(left.not_nil!.type, builder.icmp(LLVM::IntPredicate::SLT, left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
        when TokenType::LtEq
            return Azula::Value.new(left.not_nil!.type, builder.icmp(LLVM::IntPredicate::SLE, left.not_nil!.raw_value.not_nil!, right.not_nil!.raw_value.not_nil!))
        when TokenType::Gt
            return Azula::Value.new(left.not_nil!.type, builder.icmp(LLVM::IntPredicate::SLT, right.not_nil!.raw_value.not_nil!, left.not_nil!.raw_value.not_nil!))
        when TokenType::GtEq
            return Azula::Value.new(left.not_nil!.type, builder.icmp(LLVM::IntPredicate::SLE, right.not_nil!.raw_value.not_nil!, left.not_nil!.raw_value.not_nil!))
        when TokenType::Or
            return Azula::Value.new(left.not_nil!.type, builder.or(right.not_nil!.raw_value.not_nil!, left.not_nil!.raw_value.not_nil!))
        when TokenType::And
            return Azula::Value.new(left.not_nil!.type, builder.and(right.not_nil!.raw_value.not_nil!, left.not_nil!.raw_value.not_nil!))
        else
            return
        end
    end

    def azl_type_to_llvm(type : Azula::Type) : LLVM::Type?
        case type
        when Azula::IntegerType
            case type.as(Azula::IntegerType).size
            when 8
                return @llvm_context.int8
            when 16
                return @llvm_context.int16
            when 32
                return @llvm_context.int32
            when 64
                return @llvm_context.int64
            else
                return
            end
        when Azula::VoidType
            return @llvm_context.void
        when Azula::BooleanType
            return @llvm_context.int1
        when Azula::StringType
            return @llvm_context.int8.pointer
        else
            return
        end
    end

    def dereference_pointer(val : Azula::Value, builder : LLVM::Builder) : Azula::Value
        return Azula::Value.new(val.type.as(Azula::PointerType).nested_type, builder.load(val.raw_value.not_nil!))
    end

end

class Azula::Value

    getter raw_value, function, type

    @function : LLVM::Function?

    def initialize(@type : Azula::Type, @raw_value : LLVM::Value?)
        @function = nil
    end

    def initialize(@type : Azula::Type, @function : LLVM::Function?)
        @raw_value = nil
    end

    def initialize(@type : Azula::Type, @raw_value : LLVM::Value?, @function : LLVM::Function?)
    end

end
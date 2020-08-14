require "../symbols"

class Azula::Typechecker

    getter errors

    @errors : Array(Azula::Error)
    @symbols : Azula::SymbolTable

    def initialize
        @errors = [] of Azula::Error
        @symbols = Azula::SymbolTable.new
    end

    def check(node : AST::Node) : Azula::Type
        if @errors.size > 0
            return VoidType.new
        end
        case node
        when AST::Program
            check node.block
        when AST::Block
            last : Azula::Type = Azula::VoidType.new
            node.nodes.each do |node|
                last = check(node)
                if @errors.size > 0
                    break
                end
            end
            return last
        when AST::IntegerLiteral
            return Azula::IntegerType.new 32
        when AST::FloatLiteral
            return Azula::FloatType.new 32
        when AST::BooleanLiteral
            return Azula::BooleanType.new
        when AST::StringLiteral
            return Azula::StringType.new
        when AST::Function
            func_type = function_type node
            if node.identifier.value != ""
                @symbols.add node.identifier.value, func_type
            end

            # Copy the arguments into the symbol table for use in the function
            node.arguments.each do |arg|
                @symbols.add arg.value, arg.type.not_nil!
            end

            last_type = VoidType.new
            node.body.nodes.each do |n|
                last_type = check(n)
                if n.is_a?(Azula::AST::Return)
                    if !last_type.is(node.returns.type.not_nil!)
                        @errors << Azula::Error.new "type mismatch in function, expecting #{node.returns.type.not_nil!.to_s}, returning #{last_type.to_s}", Azula::ErrorType::Typechecking, node.token
                        return Azula::VoidType.new
                    end
                end
                if @errors.size > 0
                    break
                end
            end

            # Set the symbols table back to what it was
            node.arguments.each do |arg|
                @symbols.remove arg.value
            end

            if node.returns.type.not_nil!.is_a?(Azula::VoidType)
                node.returns.type = last_type
            end

            if !last_type.is(node.returns.type.not_nil!)
                @errors << Azula::Error.new "type mismatch in function, expecting #{node.returns.type.not_nil!.to_s}, returning #{last_type.to_s}", Azula::ErrorType::Typechecking, node.token
                return Azula::VoidType.new
            end

            return func_type
        when AST::FunctionCall
            function = check(node.function)
            if !function.is_a?(Azula::FunctionType)
                @errors << Azula::Error.new "type mismatch, expecting function, got #{function.to_s}", Azula::ErrorType::Typechecking, node.token
                return Azula::VoidType.new
            end

            if function.any_args
                return function.returns
            end

            function.arguments.each_with_index do |arg, index|
                if index > node.arguments.size-1
                    @errors << Azula::Error.new "missing argument #{index}, expecting #{arg.to_s}", Azula::ErrorType::Typechecking, node.token
                    return Azula::VoidType.new
                end
                type = check(node.arguments[index])
                if !type.is(arg)
                    @errors << Azula::Error.new "type mismatch in argument #{index}, expecting #{arg.to_s}, got #{type.to_s}", Azula::ErrorType::Typechecking, node.token
                    return Azula::VoidType.new
                end
            end

            return function.returns
        when AST::If
            condition = check node.condition
            if !condition.is_a?(Azula::BooleanType)
                @errors << Azula::Error.new "if condition must be a boolean, got #{condition.to_s}", Azula::ErrorType::Typechecking, node.token
                return Azula::VoidType.new
            end

            return check(node.true_block)
        when AST::Infix
            left = check(node.left)
            right = check(node.right)
            case node.operator.type
            when Azula::TokenType::Plus
                if !left.is right
                    @errors << Azula::Error.new "type mismatch for infix operator #{node.operator.literal}, #{left.to_s} != #{right.to_s}", Azula::ErrorType::Typechecking, node.token
                    return Azula::VoidType.new
                end
                return left
            when Azula::TokenType::Eq, Azula::TokenType::NotEq, Azula::TokenType::Lt, Azula::TokenType::LtEq, Azula::TokenType::Gt, Azula::TokenType::GtEq
                if !left.is right
                    @errors << Azula::Error.new "type mismatch for infix operator #{node.operator.literal}, #{left.to_s} != #{right.to_s}", Azula::ErrorType::Typechecking, node.token
                    return Azula::VoidType.new
                end
                return Azula::BooleanType.new
            when Azula::TokenType::Or, Azula::TokenType::And
                if !left.is_a?(Azula::BooleanType) || !right.is_a?(Azula::BooleanType)
                    @errors << Azula::Error.new "cannot perform operation on non-boolean", Azula::ErrorType::Typechecking, node.token
                    return Azula::VoidType.new
                end
                return Azula::BooleanType.new
            else
                return left
            end
        when AST::Identifier
            if node.value == "printf"
                return Azula::FunctionType.new(VoidType.new)
            end
            if !@symbols.exists?(node.value)
                @errors << Azula::Error.new "undefined variable '#{node.value}'", Azula::ErrorType::Typechecking, node.token
                return Azula::VoidType.new
            end
            return @symbols.get(node.value).not_nil!
        when AST::Return
            if node.value.nil?
                return Azula::VoidType.new
            end
            return check(node.value.not_nil!)
        when AST::Assign
            check_assign(node)
        else
        end
        return Azula::VoidType.new
    end

    def check_assign(node : AST::Assign)
        val_type = check node.value

        assign_type = node.identifier.type
        if assign_type.nil?
            if !@symbols.exists?(node.identifier.value)
                @errors << Azula::Error.new "undefined variable '#{node.identifier.value}'", Azula::ErrorType::Typechecking, node.token
                return
            end
            assign_type = @symbols.get node.identifier.value
        end

        if assign_type.not_nil!.is(InferType.new)
            assign_type = val_type
        end

        if !val_type.is(assign_type.not_nil!)
            @errors << Azula::Error.new "type mismatch in assign, expecting #{assign_type.to_s}, got #{val_type.to_s}", Azula::ErrorType::Typechecking, node.token
            return
        end

        @symbols.add node.identifier.value, assign_type.not_nil!
    end

    def function_type(node : AST::Function) : Azula::FunctionType
        return Azula::FunctionType.new node.arguments.map { |arg| arg.type.not_nil! }, node.returns.type.not_nil!
    end

end
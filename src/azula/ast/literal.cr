require "./ast"
require "../token"

module Azula
    module AST

        # A literal integer value, eg. 4, 3, 2
        class IntegerLiteral < Expression

            @token : Token
            @size : Int32 = 32
            @value : Int32

            def initialize(@token, @value)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "#{@value}"
            end

            # Get the `Token` in this node.
            getter token
            # Get the size to be created in the compiler, eg. 8, 16.
            getter size
            setter size
            # Get the native Crystal `Int32` value of this node.
            getter value

        end

        # A literal string value, eg. "hello world"
        class StringLiteral < Expression

            @token : Token
            @value : String

            def initialize(@token, @value)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "\"#{@value}\""
            end

            # Get the `Token` in this node.
            getter token
            # Get the native Crystal `String` value of this node.
            getter value

        end

        # A literal boolean value, either `true` or `false`.
        class BooleanLiteral < Expression

            @token : Token
            @value : Bool

            def initialize(@token, @value)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "#{@value}"
            end

            # Get the `Token` in this node.
            getter token
            # Get the native Crystal `Boolean` value of this node.
            getter value

        end

        # A literal float value, eg. 10.23
        class FloatLiteral < Expression

            @token : Token
            @value : Float32

            def initialize(@token, @value)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "#{@value}"
            end

            # Get the `Token` in this node.
            getter token
            # Get the native Crystal `Float32` value of this node.
            getter value

        end

        # A literal null
        class NullLiteral < Expression

            @token : Token

            def initialize(@token)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                return "#{@token.literal}"
            end

            # Get the `Token` in this node.
            getter token

        end
    end
end
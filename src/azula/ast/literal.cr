require "./ast"
require "../token"

module Azula
    module AST

        class IntegerLiteral < Expression

            @token : Token
            @value : Int32

            def initialize(@token, @value)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "#{@value}"
            end

            getter token
            getter value

        end

        class StringLiteral < Expression

            @token : Token
            @value : String

            def initialize(@token, @value)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "\"#{@value}\""
            end

            getter token
            getter value

        end

        class BooleanLiteral < Expression

            @token : Token
            @value : Bool

            def initialize(@token, @value)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "#{@value}"
            end

            getter token
            getter value

        end

        class FloatLiteral < Expression

            @token : Token
            @value : Float32

            def initialize(@token, @value)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                return "#{@value}"
            end

            getter token
            getter value

        end
    end
end
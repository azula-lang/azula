require "./ast"
require "../token"
require "./identifier"

module Azula
    module AST

        class FunctionCall < Expression

            @token : Token
            @function_name : Identifier
            @arguments : Array(Expression)

            def initialize(@token, @function_name, @arguments)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                s = "#{function_name.to_string}("
                @arguments.each do |arg|
                    s += "#{arg.to_string},"
                end
                s += ")"
                return s
            end

            getter token
            getter function_name
            getter arguments

        end
    end
end
require "./ast"
require "../token"
require "./identifier"

module Azula
    module AST

        # Function calls are when a defined function is referenced, passing arguments, possibly expecting a return.
        class FunctionCall < Expression

            @token : Token
            @function_name : Identifier
            @arguments : Array(Expression)

            def initialize(@token, @function_name, @arguments)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "#{function_name.to_string}("
                @arguments.each do |arg|
                    s += "#{arg.to_string},"
                end
                s += ")"
                return s
            end

            # Get the `Token` in this node.
            getter token
            # The name of the `Function` to be called.
            getter function_name
            # A list of `Expression` to be passed to the `Function`.
            getter arguments

        end
    end
end
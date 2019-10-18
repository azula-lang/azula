require "./ast"
require "./identifier"
require "../token"

module Azula
    module AST

        # An Array is an expression which contains a number of items.
        class ArrayExp < Expression

            @token : Token
            @type : Types::Type
            @values : Array(Expression)

            def initialize(@token, @type, @values)   
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "["

                @values.each do |value|
                    s += "#{value.to_string}, "
                end
                s = s + "]"
            end

            # Get the `Token` in this node.
            getter token
            # Get the node that this array contains.
            getter type
            # Get the list of values `Expression` in this statement.
            getter values

        end
    end
end
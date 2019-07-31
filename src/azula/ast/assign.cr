require "./ast"
require "./identifier"
require "../token"

module Azula
    module AST

        # An Assign statement is used to give identifier(s) value(s). 
        # It has a list of identifiers, either typed or not. Non-typed identifiers are used for reassigning a variable.
        class Assign < Statement

            @token : Token
            @idents : Array(TypedIdentifier | Identifier)
            @values : Array(Expression)

            def initialize(@token, @idents, @values)   
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = ""
                @idents.each do |ident|
                    s += "#{ident.to_string}, "
                end
                s = s[0, s.size - 2] + " = "

                @values.each do |value|
                    s += "#{value.to_string}, "
                end
                s = s[0, s.size - 2]
            end

            # Get the `Token` in this node.
            getter token
            # Get the list of `Identifier` or `TypedIdentifier` in this statement.
            getter idents
            # Get the list of values `Expression` in this statement.
            getter values

        end
    end
end
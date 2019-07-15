require "./ast"
require "./identifier"
require "../token"

module Azula
    module AST

        class Assign < Statement

            @token : Token
            @idents : Array(TypedIdentifier | Identifier)
            @values : Array(Expression)

            def initialize(@token, @idents, @values)   
            end

            def token_literal : String
                return @token.literal
            end

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

            getter token
            getter idents
            getter values

        end
    end
end
require "./ast"
require "../token"
require "./identifier"

module Azula
    module AST

        class Struct < Statement

            @token : Token
            @struct_name : Identifier
            @fields : Array(TypedIdentifier)

            def initialize(@token, @struct_name, @fields)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                s = "#{self.token_literal} #{@struct_name.to_string} {\n"
                @fields.each do |field|
                    s += field.to_string + ", "
                end
                s += "\n}"
            end

            getter token
            getter struct_name
            getter fields

        end
    end
end
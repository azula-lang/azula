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

        class StructInitialise < Expression

            @token : Token
            @struct_name : Identifier
            @values : Array(Expression)

            def initialize(@token, @struct_name, @values)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                s = "#{@struct_name.to_string} {\n"
                @values.each do |val|
                    s += "#{val.to_string}, "
                end
                s = s[0, s.size - 2]
                s += "\n}"
                return s
            end

            getter token
            getter struct_name
            getter values

        end
    end
end
require "./ast"
require "../token"
require "./identifier"

module Azula
    module AST

        # Struct allows for custom datatypes that contain other datatypes.
        class Struct < Statement

            @token : Token
            @struct_name : Identifier
            @fields : Array(TypedIdentifier)

            def initialize(@token, @struct_name, @fields)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "#{self.token_literal} #{@struct_name.to_string} {\n"
                @fields.each do |field|
                    s += field.to_string + ", "
                end
                s += "\n}"
            end

            # Get the `Token` in this node.
            getter token
            # The `Identifier` used to reference this struct.
            getter struct_name
            # The fields that this struct contains.
            getter fields

        end

        # Struct initialisation, passing a an array of values for the struct to be initialised with.
        class StructInitialise < Expression

            @token : Token
            @struct_name : Identifier
            @values : Array(Expression)

            def initialize(@token, @struct_name, @values)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "#{@struct_name.to_string} {\n"
                @values.each do |val|
                    s += "#{val.to_string}, "
                end
                s = s[0, s.size - 2]
                s += "\n}"
                return s
            end

            # Get the `Token` in this node.
            getter token
            # The `Identifier` used to reference the struct.
            getter struct_name
            # The array of values for the struct to be initialised with.
            getter values

        end
    end
end
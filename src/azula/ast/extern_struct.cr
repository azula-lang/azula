require "./ast"
require "./block"
require "./identifier"
require "../token"
require "../types"

module Azula
    module AST
        # ExternFunction represents a callable function built into the compiler.
        class ExternStruct < Statement

            @token : Token
            @struct_name : Identifier

            def initialize(@token, @struct_name)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "#{self.token_literal} #{@struct_name.to_string}("
                return s
            end

            # Get the `Token` in this node.
            getter token
            # The name assigned to this function, as an `Identifier`.
            getter struct_name

        end
    end
end
require "./ast"
require "./block"
require "./identifier"
require "../token"
require "../types"

module Azula
    module AST
        # ExternFunction represents a callable function built into the compiler.
        class ExternFunction < Statement

            @token : Token
            @function_name : Identifier
            @parameters : Array(TypedIdentifier)
            @return_type : Types::Type

            def initialize(@token, @function_name, @parameters, @return_type)
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "#{self.token_literal} #{@function_name.to_string}("
                @parameters.each do |param|
                    s += param.to_string + ","
                end
                s += "): #{@return_type.main_type} "
                return s
            end

            # Get the `Token` in this node.
            getter token
            # The name assigned to this function, as an `Identifier`.
            getter function_name
            # The parameters to be passed to this function, an array of `Expression`.
            getter parameters
            # The return type of this function, as a Type.
            getter return_type

        end
    end
end
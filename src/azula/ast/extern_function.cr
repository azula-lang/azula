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
            @return_types : Array(Types::Type | String)

            def initialize(@token, @function_name, @parameters, @return_types)
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
                s += ") : ("
                @return_types.each do |rt|
                    s += "#{rt.to_s.downcase}, "
                end
                s = s[0, s.size-2]
                s += ")"
                return s
            end

            # Get the `Token` in this node.
            getter token
            # The name assigned to this function, as an `Identifier`.
            getter function_name
            # The parameters to be passed to this function, an array of `Expression`.
            getter parameters
            # The return types of this function, as an array of `TypedIdentifier`.
            getter return_types

        end
    end
end
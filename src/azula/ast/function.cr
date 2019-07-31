require "./ast"
require "./block"
require "./identifier"
require "../token"
require "../types"

module Azula
    module AST
        # Function represents a callable function.
        class Function < Statement

            @token : Token
            @function_name : Identifier
            @parameters : Array(TypedIdentifier)
            @return_types : Array(Types::Type | String)
            @body : Block

            def initialize(@token, @function_name, @parameters, @return_types, @body)
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
                s += ") {\n#{@body.to_string}}"
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
            # The body of the function, to be executed when the function is called.
            getter body

        end
    end
end
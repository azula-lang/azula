require "./ast"
require "./block"
require "./identifier"
require "../token"
require "../types"

module Azula
    module AST
        # Function represents a callable function
        class Function < Statement

            @token : Token
            @function_name : Identifier
            @parameters : Array(TypedIdentifier)
            @return_types : Array(Types::Type | String)
            @body : Block

            def initialize(@token, @function_name, @parameters, @return_types, @body)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                s = "#{self.token_literal} #{@function_name.to_string}("
                @parameters.each do |param|
                    s += param.to_string + ","
                end
                s += ") : ("
                @return_types.each do |rt|
                    s += "#{rt}, "
                end
                s += ") {\n#{@body.to_string}\n}"
                return s
            end

            getter token
            getter function_name
            getter parameters
            getter return_types
            getter body

        end
    end
end
require "./ast"
require "./block"
require "./identifier"
require "../token"
require "../types"

module Azula
    module AST
        # Function represents a callable function
        class Macro < Statement

            @token : Token
            @macro_name : Identifier
            @parameters : Array(Identifier)
            @body : Block

            def initialize(@token, @macro_name, @parameters, @body)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                s = "#{self.token_literal} #{@macro_name.to_string}("
                @parameters.each do |param|
                    s += param.to_string + ","
                end
                s += ") {\n#{@body.to_string}}"
                return s
            end

            getter token
            getter macro_name
            getter parameters
            getter body

        end
    end
end
require "./ast"
require "../token"

module Azula
    module AST

        # Return statement is used to pass value(s) back from a function.
        class Return < Statement

            @token : Token
            @values : Array(Expression)

            def initialize(@token, @values)   
            end

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "#{self.token_literal} "
                @values.each do |val|
                    s += "#{val.to_string}, "
                end
                if @values.size > 0
                    s = s[0, s.size-2]
                end
                return s
            end

            # Get the `Token` in this node.
            getter token
            # The array of values to be returned.
            getter values

        end
    end
end
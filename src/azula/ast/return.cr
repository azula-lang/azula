require "./ast"
require "../token"

module Azula
    module AST

        class Return < Statement

            @token : Token
            @values : Array(Expression)

            def initialize(@token, @values)   
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                s = "#{self.token_literal} "
                @values.each do |val|
                    s += "#{val.to_string}, "
                end
                return s
            end

            getter token
            getter value

        end
    end
end
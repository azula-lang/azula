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
                if @values.size > 0
                    s = s[0, s.size-2]
                end
                return s
            end

            getter token
            getter values

        end
    end
end
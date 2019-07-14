require "./ast"
require "./block"
require "../token"

module Azula
    module AST

        class If < Statement

            @token : Token
            @condition : Expression
            @consequence : Block
            @alts : Array(If)
            @alternative : Block?

            def initialize(@token, @condition, @consequence, @alts, @alternative)
            end

            def token_literal : String
                return @token.literal
            end

            def to_string : String
                s = "#{self.token_literal}(#{@condition.to_string}) {\n#{@consequence.to_string}\n }"
                @alts.each do |alt|
                    s += alt.to_string
                end
                if @alternative.nil?
                    s += "else {\n#{@alternative.not_nil!.to_string}\n}"
                end
                return s
            end

            getter token
            getter condition
            getter consequence
            getter alts
            getter alternative
        end
    end
end
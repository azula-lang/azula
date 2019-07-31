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

            # The literal token representation of this node.
            def token_literal : String
                return @token.literal
            end

            # The literal string representation of this node.
            def to_string : String
                s = "#{self.token_literal}(#{@condition.to_string}) {\n#{@consequence.to_string}}"
                @alts.each do |alt|
                    s += alt.to_string
                end
                if !@alternative.nil?
                    s += " else {\n#{@alternative.not_nil!.to_string}}"
                end
                return s
            end

            # Get the `Token` in this node.
            getter token
            # The if condition that will execute the consequence if evaluated truthy.
            getter condition
            # The block of statements that will be executed if the condition is truthy.
            getter consequence
            # The alternative elseifs that will be evaluated if the condition is false.
            getter alts
            # The else clause, that will be evaluated if no other condition is truthy.
            getter alternative
        end
    end
end
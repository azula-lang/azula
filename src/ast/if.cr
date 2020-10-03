module Azula::AST

    # If allows for if conditions
    class If < Node
        getter token, condition, true_block, false_block, alternatives
        def initialize(@token : Azula::Token, @condition : AST::Node, @true_block : AST::Block, @false_block : AST::Block?, @alternatives : Array(If))
        end

        def to_s : String
            output = "if #{@condition.to_s} {"
            output += @true_block.to_s
            output += "}"
            @alternatives.each do |alt|
                output += "else " + alt.to_s
            end
            if !@false_block.nil?
                output += "else {"
                output += @false_block.to_s
                output += "}"
            end
            return output
        end
    end

end
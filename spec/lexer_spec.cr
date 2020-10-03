require "./spec_helper"

describe Azula::Lexer do

    it "works" do
        input = "
        func x(int8 i, string y): int32 {
            var asd2 = 12;
            if(x == y) {
                x
            } else {
                y
            }
            return z
        }
        "
        types = [
            Azula::TokenType::Function,
            Azula::TokenType::Identifier,
            Azula::TokenType::LBracket,
            Azula::TokenType::Type,
            Azula::TokenType::Identifier,
            Azula::TokenType::Comma,
            Azula::TokenType::Type,
            Azula::TokenType::Identifier,
            Azula::TokenType::RBracket,
            Azula::TokenType::Colon,
            Azula::TokenType::Type,
            Azula::TokenType::LBrace,
            Azula::TokenType::Var,
            Azula::TokenType::Identifier,
            Azula::TokenType::Assign,
            Azula::TokenType::NumberLiteral,
            Azula::TokenType::Semicolon,
            Azula::TokenType::If,
            Azula::TokenType::LBracket,
            Azula::TokenType::Identifier,
            Azula::TokenType::Eq,
            Azula::TokenType::Identifier,
            Azula::TokenType::RBracket,
            Azula::TokenType::LBrace,
            Azula::TokenType::Identifier,
            Azula::TokenType::RBrace,
            Azula::TokenType::Else,
            Azula::TokenType::LBrace,
            Azula::TokenType::Identifier,
            Azula::TokenType::RBrace,
            Azula::TokenType::Return,
            Azula::TokenType::Identifier,
            Azula::TokenType::RBrace,
        ]

        literals = [
            "func",
            "x",
            "(",
            "int8",
            "i",
            ",",
            "string",
            "y",
            ")",
            ":",
            "int32",
            "{",
            "var",
            "asd2",
            "=",
            "12",
            ";",
            "if",
            "(",
            "x",
            "==",
            "y",
            ")",
            "{",
            "x",
            "}",
            "else",
            "{",
            "y",
            "}",
            "return",
            "z",
            "}",
        ]

        lexer = Azula::Lexer.new input
        types.each do |expec|
            token = lexer.next_token
            token.type.should eq expec
        end

        lexer = Azula::Lexer.new input
        literals.each do |expec|
            token = lexer.next_token
            token.literal.should eq expec
        end
    end
end
require "./spec_helper"

describe Azula::Lexer do

    input = "
int i = 5;
string s = \"hi\";
float f = 5.5;
bool b = true;
struct Person {
    string name,
    int age,
}
p.name, p.age; // example comment
string s = i as string;
if(s != null) {
    s;
} elseif(x) {
    x;
} else {
    y;
}
func x(int i) : (int, int) {
    return i, i;
}
==
!=
<
<=
>
>=
or
and
!
+
-
*
/
**
%
macro add(x, y) {
    return {{x}}, {{y}};
}
"
    file = "repl"

    expected = [
        Azula::Token.new(Azula::TokenType::TYPE, "int", file, 1, 1),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "i", file, 1, 5),
        Azula::Token.new(Azula::TokenType::ASSIGN, "=", file, 1, 7),
        Azula::Token.new(Azula::TokenType::NUMBER, "5", file, 1, 9),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 1, 10),

        Azula::Token.new(Azula::TokenType::TYPE, "string", file, 2, 1),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "s", file, 2, 8),
        Azula::Token.new(Azula::TokenType::ASSIGN, "=", file, 2, 10),
        Azula::Token.new(Azula::TokenType::STRING, "hi", file, 2, 12),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 2, 16),

        Azula::Token.new(Azula::TokenType::TYPE, "float", file, 3, 1),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "f", file, 3, 7),
        Azula::Token.new(Azula::TokenType::ASSIGN, "=", file, 3, 9),
        Azula::Token.new(Azula::TokenType::NUMBER, "5", file, 3, 11),
        Azula::Token.new(Azula::TokenType::DOT, ".", file, 3, 12),
        Azula::Token.new(Azula::TokenType::NUMBER, "5", file, 3, 13),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 3, 14),

        Azula::Token.new(Azula::TokenType::TYPE, "bool", file, 4, 1),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "b", file, 4, 6),
        Azula::Token.new(Azula::TokenType::ASSIGN, "=", file, 4, 8),
        Azula::Token.new(Azula::TokenType::TRUE, "true", file, 4, 10),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 4, 14),

        Azula::Token.new(Azula::TokenType::STRUCT, "struct", file, 5, 1),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "Person", file, 5, 8),
        Azula::Token.new(Azula::TokenType::LBRACE, "{", file, 5, 15),
        Azula::Token.new(Azula::TokenType::TYPE, "string", file, 6, 5),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "name", file, 6, 12),
        Azula::Token.new(Azula::TokenType::COMMA, ",", file, 6, 16),
        Azula::Token.new(Azula::TokenType::TYPE, "int", file, 7, 5),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "age", file, 7, 9),
        Azula::Token.new(Azula::TokenType::COMMA, ",", file, 7, 12),
        Azula::Token.new(Azula::TokenType::RBRACE, "}", file, 8, 1),

        Azula::Token.new(Azula::TokenType::IDENTIFIER, "p", file, 9, 1),
        Azula::Token.new(Azula::TokenType::DOT, ".", file, 9, 2),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "name", file, 9, 3),
        Azula::Token.new(Azula::TokenType::COMMA, ",", file, 9, 7),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "p", file, 9, 9),
        Azula::Token.new(Azula::TokenType::DOT, ".", file, 9, 10),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "age", file, 9, 11),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 9, 14),

        Azula::Token.new(Azula::TokenType::TYPE, "string", file, 10, 1),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "s", file, 10, 8),
        Azula::Token.new(Azula::TokenType::ASSIGN, "=", file, 10, 10),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "i", file, 10, 12),
        Azula::Token.new(Azula::TokenType::AS, "as", file, 10, 14),
        Azula::Token.new(Azula::TokenType::TYPE, "string", file, 10, 17),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 10, 23),

        Azula::Token.new(Azula::TokenType::IF, "if", file, 11, 1),
        Azula::Token.new(Azula::TokenType::LBRACKET, "(", file, 11, 3),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "s", file, 11, 4),
        Azula::Token.new(Azula::TokenType::NOT_EQ, "!=", file, 11, 6),
        Azula::Token.new(Azula::TokenType::NULL, "null", file, 11, 9),
        Azula::Token.new(Azula::TokenType::RBRACKET, ")", file, 11, 13),
        Azula::Token.new(Azula::TokenType::LBRACE, "{", file, 11, 15),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "s", file, 12, 5),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 12, 6),
        Azula::Token.new(Azula::TokenType::RBRACE, "}", file, 13, 1),
        Azula::Token.new(Azula::TokenType::ELSEIF, "elseif", file, 13, 3),
        Azula::Token.new(Azula::TokenType::LBRACKET, "(", file, 13, 9),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "x", file, 13, 10),
        Azula::Token.new(Azula::TokenType::RBRACKET, ")", file, 13, 11),
        Azula::Token.new(Azula::TokenType::LBRACE, "{", file, 13, 13),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "x", file, 14, 5),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 14, 6),
        Azula::Token.new(Azula::TokenType::RBRACE, "}", file, 15, 1),
        Azula::Token.new(Azula::TokenType::ELSE, "else", file, 15, 3),
        Azula::Token.new(Azula::TokenType::LBRACE, "{", file, 15, 8),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "y", file, 16, 5),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 16, 6),
        Azula::Token.new(Azula::TokenType::RBRACE, "}", file, 17, 1),

        Azula::Token.new(Azula::TokenType::FUNCTION, "func", file, 18, 1),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "x", file, 18, 6),
        Azula::Token.new(Azula::TokenType::LBRACKET, "(", file, 18, 7),
        Azula::Token.new(Azula::TokenType::TYPE, "int", file, 18, 8),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "i", file, 18, 12),
        Azula::Token.new(Azula::TokenType::RBRACKET, ")", file, 18, 13),
        Azula::Token.new(Azula::TokenType::COLON, ":", file, 18, 15),
        Azula::Token.new(Azula::TokenType::LBRACKET, "(", file, 18, 17),
        Azula::Token.new(Azula::TokenType::TYPE, "int", file, 18, 18),
        Azula::Token.new(Azula::TokenType::COMMA, ",", file, 18, 21),
        Azula::Token.new(Azula::TokenType::TYPE, "int", file, 18, 23),
        Azula::Token.new(Azula::TokenType::RBRACKET, ")", file, 18, 26),
        Azula::Token.new(Azula::TokenType::LBRACE, "{", file, 18, 28),
        Azula::Token.new(Azula::TokenType::RETURN, "return", file, 19, 5),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "i", file, 19, 12),
        Azula::Token.new(Azula::TokenType::COMMA, ",", file, 19, 13),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "i", file, 19, 15),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 19, 16),
        Azula::Token.new(Azula::TokenType::RBRACE, "}", file, 20, 1),

        Azula::Token.new(Azula::TokenType::EQ, "==", file, 21, 1),
        Azula::Token.new(Azula::TokenType::NOT_EQ, "!=", file, 22, 1),
        Azula::Token.new(Azula::TokenType::LT, "<", file, 23, 1),
        Azula::Token.new(Azula::TokenType::LT_EQ, "<=", file, 24, 1),
        Azula::Token.new(Azula::TokenType::GT, ">", file, 25, 1),
        Azula::Token.new(Azula::TokenType::GT_EQ, ">=", file, 26, 1),
        Azula::Token.new(Azula::TokenType::OR, "or", file, 27, 1),
        Azula::Token.new(Azula::TokenType::AND, "and", file, 28, 1),
        Azula::Token.new(Azula::TokenType::NOT, "!", file, 29, 1),
        Azula::Token.new(Azula::TokenType::PLUS, "+", file, 30, 1),
        Azula::Token.new(Azula::TokenType::MINUS, "-", file, 31, 1),
        Azula::Token.new(Azula::TokenType::ASTERISK, "*", file, 32, 1),
        Azula::Token.new(Azula::TokenType::SLASH, "/", file, 33, 1),
        Azula::Token.new(Azula::TokenType::EXPONENT, "**", file, 34, 1),
        Azula::Token.new(Azula::TokenType::MODULO, "%", file, 35, 1),

        Azula::Token.new(Azula::TokenType::MACRO, "macro", file, 36, 1),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "add", file, 36, 7),
        Azula::Token.new(Azula::TokenType::LBRACKET, "(", file, 36, 10),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "x", file, 36, 11),
        Azula::Token.new(Azula::TokenType::COMMA, ",", file, 36, 12),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "y", file, 36, 14),
        Azula::Token.new(Azula::TokenType::RBRACKET, ")", file, 36, 15),
        Azula::Token.new(Azula::TokenType::LBRACE, "{", file, 36, 17),
        Azula::Token.new(Azula::TokenType::RETURN, "return", file, 37, 5),
        Azula::Token.new(Azula::TokenType::LBRACE, "{", file, 37, 12),
        Azula::Token.new(Azula::TokenType::LBRACE, "{", file, 37, 13),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "x", file, 37, 14),
        Azula::Token.new(Azula::TokenType::RBRACE, "}", file, 37, 15),
        Azula::Token.new(Azula::TokenType::RBRACE, "}", file, 37, 16),
        Azula::Token.new(Azula::TokenType::COMMA, ",", file, 37, 17),
        Azula::Token.new(Azula::TokenType::LBRACE, "{", file, 37, 19),
        Azula::Token.new(Azula::TokenType::LBRACE, "{", file, 37, 20),
        Azula::Token.new(Azula::TokenType::IDENTIFIER, "y", file, 37, 21),
        Azula::Token.new(Azula::TokenType::RBRACE, "}", file, 37, 22),
        Azula::Token.new(Azula::TokenType::RBRACE, "}", file, 37, 23),
        Azula::Token.new(Azula::TokenType::SEMICOLON, ";", file, 37, 24),
    ]

    it "correctly lexes input and returns tokens" do
        l = Azula::Lexer.new input
        expected.each do |token|
            t = l.next_token
            t.type.should eq token.type
            t.literal.should eq token.literal
            t.file.should eq token.file
            t.linenumber.should eq token.linenumber
            t.charnumber.should eq token.charnumber
        end
    end
    
end
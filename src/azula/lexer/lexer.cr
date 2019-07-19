require "../token/*"

module Azula
    
    class Lexer

        @input : String
        @position : Int32 = 0
        @read_position : Int32 = 0
        @current_char : Char = Char::ZERO

        @file : String = "repl"
        @current_line : Int32 = 0
        @current_char_num : Int32 = 0

        setter file

        def initialize(@input : String)
            self.read_char
        end

        # Read the next character of the input
        def read_char
            if @read_position >= @input.size
                @current_char = Char::ZERO
            else
                @current_char = @input[@read_position]
            end
            if @current_char == '\n'
                @current_line += 1
                @current_char_num = 0
            else
                @current_char_num += 1
            end
            @position = @read_position
            @read_position += 1
        end

        def next_token : Token
            self.skip_whitespace

            token_type : TokenType
            literal : (String | Char)
            line_num = @current_line
            char_num = @current_char_num

            read = true

            case @current_char
            when '='
                if self.peek_char == '='
                    ch = @current_char
                    self.read_char
                    token_type, literal = TokenType::EQ, "#{ch}#{@current_char}"
                else
                    token_type, literal = TokenType::ASSIGN, @current_char
                end
            when ':'
                token_type, literal = TokenType::COLON, @current_char
            when ';'
                token_type, literal = TokenType::SEMICOLON, @current_char
            when '"'
                str = self.read_string
                token_type, literal = TokenType::STRING, str
            when '!'
                if self.peek_char == '='
                    ch = @current_char
                    self.read_char
                    token_type, literal = TokenType::NOT_EQ, "#{ch}#{@current_char}"
                else
                    token_type, literal = TokenType::NOT, @current_char
                end
            when '+'
                token_type, literal = TokenType::PLUS, @current_char
            when '-'
                token_type, literal = TokenType::MINUS, @current_char
            when '*'
                if self.peek_char == '*'
                    ch = @current_char
                    self.read_char
                    token_type, literal = TokenType::EXPONENT, "#{ch}#{@current_char}"
                else
                    token_type, literal = TokenType::ASTERISK, @current_char
                end
            when '%'
                token_type, literal = TokenType::MODULO, @current_char
            when '/'
                if self.peek_char == '/'
                    self.next_line
                    return self.next_token
                else
                    token_type, literal = TokenType::SLASH, @current_char
                end
            when '<'
                if self.peek_char == '='
                    ch = @current_char
                    self.read_char
                    token_type, literal = TokenType::LT_EQ, "#{ch}#{@current_char}"
                else
                    token_type, literal = TokenType::LT, @current_char
                end
            when '>'
                if self.peek_char == '='
                    ch = @current_char
                    self.read_char
                    token_type, literal = TokenType::GT_EQ, "#{ch}#{@current_char}"
                else
                    token_type, literal = TokenType::GT, @current_char
                end
            when '('
                token_type, literal = TokenType::LBRACKET, @current_char
            when ')'
                token_type, literal = TokenType::RBRACKET, @current_char
            when '{'
                token_type, literal = TokenType::LBRACE, @current_char
            when '}'
                token_type, literal = TokenType::RBRACE, @current_char
            when '['
                token_type, literal = TokenType::LSQUARE, @current_char
            when ']'
                token_type, literal = TokenType::RSQUARE, @current_char
            when ','
                token_type, literal = TokenType::COMMA, @current_char
            when '.'
                token_type, literal = TokenType::DOT, @current_char
            when Char::ZERO
                token_type, literal = TokenType::EOF, @current_char
            else
                if @current_char.letter? || @current_char == '_'
                    ident = self.read_identifier
                    type = Azula::Keywords.fetch ident, TokenType::IDENTIFIER
                    token_type, literal = type, ident
                    read = false
                elsif @current_char.number?
                    num = read_number
                    token_type, literal = TokenType::NUMBER, num
                    read = false
                else
                    token_type, literal = TokenType::ILLEGAL, @current_char
                end
            end
            self.read_char unless !read
            return Token.new token_type, "#{literal}", @file, line_num, char_num
        end

        def read_identifier : String
            position = @position
            while @current_char.alphanumeric? || @current_char == '_'
                self.read_char
            end
            return @input[position...@position]
        end

        def read_number : String
            position = @position
            while @current_char.number?
                self.read_char
            end
            return @input[position...@position]
        end

        def skip_whitespace
            while @current_char == '\n' || @current_char == ' ' || @current_char == '\t' || @current_char == '\r'
                self.read_char
            end
        end

        def peek_char
            return @read_position >= @input.size ? Char::ZERO : @input[@read_position]
        end

        def next_line
            self.read_char
            i = 0
            while @current_char != '\n' && @current_char != Char::ZERO && @current_char != '\r'
                self.read_char
            end
        end

        def read_string : String
            self.read_char
            position = @read_position - 1
            while @current_char != '"' && @current_char != Char::ZERO
                self.read_char
            end
            return @input[position...@read_position-1]
        end

    end

end
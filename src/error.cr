module Azula
    class Error
        getter message, type, file, line, character
        def initialize(@message : String, @type : ErrorType, @file : String, @line : Int32, @character : Int32)
        end

        def initialize(@message : String, @type : ErrorType, token : Token)
            @file = token.file
            @line = token.line_number
            @character = token.char_number
        end
    end

    enum ErrorType
        Parsing
        Typechecking
        Compiling
    end
end
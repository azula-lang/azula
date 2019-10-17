require "colorize"

module Azula
    class ErrorManager

        @@Errors = [] of Error
        @@Code = [] of String

        def self.add_error(error : Error)
            @@Errors << error
        end

        def self.can_compile : Bool
            return @@Errors.size == 0
        end

        def self.print_errors
            puts "#{"COMPILATION ERRORS".colorize(:red)}"
            @@Errors.each do |e|
                puts "#{"ERROR".colorize(:red)} #{e.content} \nFile #{e.filename.colorize(:blue)} (line #{e.line}, character #{e.col})\n"
                puts "#{@@Code[e.line]}"
                puts "#{(" " * e.col)}#{"^".colorize(:red)}"
            end
        end

        def self.set_file(code)
            @@Code = code
        end

    end

    class Error

        @content : String
        @filename : String
        @line : Int32
        @col : Int32

        getter content
        getter filename
        getter line
        getter col

        def initialize(@content : String, @filename : String, @line : Int32, @col : Int32)
            
        end

    end
end
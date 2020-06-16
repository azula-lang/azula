require "./program"
require "colorize"

module Azula
    VERSION = "0.4.0"
end

def main
    code = File.read "test.azl"
    program = Azula::Program.new("test.azl", code)
    program.parse
    if program.errors.size > 0
        print_errors program.errors, code
        return
    end

    program.typecheck
    if program.errors.size > 0
        print_errors program.errors, code
        return
    end

    program.compile
    if program.errors.size > 0
        print_errors program.errors, code
        return
    end

    program.optimise
    program.write_llvm("test.ll")
    program.create_executable("test", false)
end

def print_errors(errors : Array(Azula::Error), code : String)
    errors.each do |error|
        puts "#{error.type.to_s.upcase} ERROR".colorize(:red)
        puts error.message.colorize(:red)
        puts code.lines[error.line]
        puts "#{" " * (error.character-1)}^".colorize(:red)
    end
end

main

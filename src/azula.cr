require "./program"
require "colorize"

module Azula
    VERSION = "0.4.0"
end

def main
    if ARGV.size != 2
        puts "Incorrect usage.".colorize(:red)
        puts "Usage: azula [build/run/llir] file.azl".colorize(:red)
        exit(1)
    end

    cmd = ARGV[0]
    
    if !["run", "build", "llir"].includes?(cmd.downcase)
        puts "Unknown command.".colorize(:red)
        exit(1)
    end

    if !File.file?(ARGV[1])
        puts "Couldn't find the file at: #{ARGV[1]}".colorize(:red)
        exit(1)
    end

    code = File.read ARGV[1]
    program = Azula::Program.new(ARGV[1], code)
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
    
    name = File.basename(ARGV[1]).gsub(File.extname(ARGV[1]), "")

    if ARGV[0] == "llir"
        program.write_llvm("#{name}.ll")
        exit(0)
    end

    program.create_executable(name, false)

    if ARGV[0] == "run"
        system "./#{name}"
        File.delete name
    end
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

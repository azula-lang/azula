require "./azula/lexer/"
require "./azula/parser/"
require "./azula/types/"
require "./azula/compiler/"
require "./azula/errors/"

require "option_parser"
require "colorize"

# Azula is a strongly-typed compiled language, using an LLVM backend
module Azula
end

VERSION = "0.3.1"
PROMPT = ">> "

type_check = true

OptionParser.parse do |parser|
    parser.banner = "Usage: azula run [arguments]"
    parser.on("-nt", "--no-typecheck", "Disable type checking") { type_check = false }
end

puts "#{"Azula".colorize(Colorize::ColorRGB.new(253, 117, 155))}" + " Version #{VERSION}\n"

if ARGV.size == 0
    puts "Incorrect number of arguments."
    exit
end
todo = ARGV[0]

if todo == "version"
    exit
end

file = ARGV[1]
content = File.read file

# Setup error manager
Azula::ErrorManager.set_file content.split("\n")

puts "Lexing".colorize(:green)
l = Azula::Lexer.new content
l.file = file
puts "Parsing".colorize(:green)
p = Azula::Parser.new l
smt = p.parse_program

if !Azula::ErrorManager.can_compile
    Azula::ErrorManager.print_errors
    exit
end

if type_check
    puts "Typechecking".colorize(:green)
    t = Azula::Types::Typechecker.new
    t.check smt
end

if !Azula::ErrorManager.can_compile
    Azula::ErrorManager.print_errors
    exit
end

puts "Compiling".colorize(:green)
c = Azula::Compiler::Compiler.new
c.register_visitors
c.compile smt

if !Azula::ErrorManager.can_compile
    Azula::ErrorManager.print_errors
    exit
end

outfile = file.split("/")[file.split("/").size-1].sub(".azl", "")

if todo == "build"
    puts "Building file".colorize(:green)
    c.create_executable "#{outfile}"
    puts "Compiled to ./#{outfile}"
elsif todo == "run"
    puts "Running\n".colorize(:green)
    puts "Output"
    puts "-" * 30
    c.create_executable "#{outfile}"
    system "./#{outfile}"
    File.delete "#{outfile}"
elsif todo == "llir"
    c.write_to_file "#{outfile}.ll"
    puts "Writing LLIR to #{outfile}.ll".colorize(:green)
end

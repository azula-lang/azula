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

VERSION = "0.3.0"
PROMPT = ">> "

type_check = true

OptionParser.parse do |parser|
    parser.banner = "Usage: azula run [arguments]"
    parser.on("-nt", "--no-typecheck", "Disable type checking") { type_check = false }
  end

puts "#{"Azula".colorize(Colorize::ColorRGB.new(253, 117, 155))}" + " Version #{VERSION}\n\n"

if ARGV.size != 2
    puts "Incorrect number of arguments."
    exit
end
todo = ARGV[0]
file = ARGV[1]
content = File.read file

# Setup error manager
Azula::ErrorManager.set_file content.split("\n")

l = Azula::Lexer.new content
l.file = file
p = Azula::Parser.new l
smt = p.parse_program

if !Azula::ErrorManager.can_compile
    Azula::ErrorManager.print_errors
    exit
end

if type_check
    t = Azula::Types::Typechecker.new
    t.check smt
end

if !Azula::ErrorManager.can_compile
    Azula::ErrorManager.print_errors
    exit
end

c = Azula::Compiler::Compiler.new
c.register_visitors
c.compile smt

if !Azula::ErrorManager.can_compile
    Azula::ErrorManager.print_errors
    exit
end

outfile = file.split("/")[file.split("/").size-1].sub(".azl", "")

if todo == "build"
    c.create_executable "#{outfile}"
    puts "Compiled to ./#{outfile}"
elsif todo == "run"
    c.create_executable "#{outfile}"
    system "./#{outfile}"
    File.delete "#{outfile}"
elsif todo == "llir"
    c.write_to_file "#{outfile}.ll"
    puts "Wrote LLIR to #{outfile}.ll"
end

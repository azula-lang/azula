require "./azula/lexer/"
require "./azula/parser/"
require "./azula/types/"
require "./azula/compiler/"

# Azula is a strongly-typed compiled language, using an LLVM backend
module Azula
end

VERSION = "0.3.0"
PROMPT = ">> "

puts "Azula " + VERSION

#print PROMPT
if ARGV.size != 1
  puts "Incorrect number of arguments."
  exit
end
file = ARGV[0]
content = File.read file
l = Azula::Lexer.new content
l.file = file
p = Azula::Parser.new l
smt = p.parse_program

if !p.errors.empty?
  p.errors.each do |error|
    puts error
  end
end

# t = Azula::Types::Typechecker.new
# puts t.check smt

# if !t.errors.empty?
#   t.errors.each do |error|
#     puts error
#   end
# end

c = Azula::Compiler.new smt
c.compile

outfile = file.split("/")[file.split("/").size-1].sub(".azl", "") + ".ll"
c.write_to_file outfile

puts "Compiled >> #{outfile}"

# c.compiler.run_function c.main_module.functions["main"], c.context

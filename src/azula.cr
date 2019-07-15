require "./azula/lexer/"
require "./azula/parser/"
require "./azula/types/"

# Azula is a strongly-typed compiled language, using an LLVM backend
module Azula

end
VERSION = "0.3.0"
PROMPT = ">> "

puts "Azula " + VERSION
print PROMPT
input = gets
while input && input != ""
  l = Azula::Lexer.new input
  p = Azula::Parser.new l
  smt = p.parse_program

  if !p.errors.empty?
    p.errors.each do |error|
      puts error
    end
  end

  t = Azula::Types::Typechecker.new
  puts t.check smt

  if !t.errors.empty?
    t.errors.each do |error|
      puts error
    end
  end

  print PROMPT
  input = gets
end

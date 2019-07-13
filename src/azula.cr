require "./azula/token/"
require "./azula/lexer/"

# Azula is a strongly-typed compiled language, using an LLVM backend
module Azula
  VERSION = "0.1.0"
  PROMPT = ">> "

  puts "Azula V0.3"
  print PROMPT
  input = gets
  while input && input != ""
    l = Lexer.new input
    token = l.next_token
    while token.type != TokenType::EOF
      puts token.to_string
      token = l.next_token
    end
    print PROMPT
    input = gets
  end
end

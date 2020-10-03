require "./spec_helper"

describe Azula::Typechecker do
    
    it "works" do
        l = Azula::Lexer.new "func my_func: int32 { 5 }"
        parser = Azula::Parser.new l
        program = parser.parse_program
        typechecker = Azula::Typechecker.new
        typechecker.check program
        typechecker.errors.size.should eq 0
    end

    it "assign mismatch" do
        l = Azula::Lexer.new "int x = 1.5"
        parser = Azula::Parser.new l
        program = parser.parse_program
        typechecker = Azula::Typechecker.new
        typechecker.check program
        typechecker.errors.size.should eq 1
        typechecker.errors[0].message.should eq "type mismatch in assign, expecting int32, got float32"
    end

    it "function args" do
        l = Azula::Lexer.new "
        func x(float z) {
            return;
        }

        x(5)
        "
        parser = Azula::Parser.new l
        program = parser.parse_program
        typechecker = Azula::Typechecker.new
        typechecker.check program
        typechecker.errors.size.should eq 1
        typechecker.errors[0].message.should eq "type mismatch in argument 0, expecting float32, got int32"
    end

    it "if" do
        l = Azula::Lexer.new "
        if(5) {
            print(5)
        }
        "
        parser = Azula::Parser.new l
        program = parser.parse_program
        typechecker = Azula::Typechecker.new
        typechecker.check program
        typechecker.errors.size.should eq 1
        typechecker.errors[0].message.should eq "if condition must be a boolean, got int32"

        l = Azula::Lexer.new "
        func test(): int {
            if(true) {
                return 5.0
            }
        }
        "
        parser = Azula::Parser.new l
        program = parser.parse_program
        typechecker = Azula::Typechecker.new
        typechecker.check program
        typechecker.errors.size.should eq 1
        typechecker.errors[0].message.should eq "type mismatch in function, expecting int32, returning float32"

        l = Azula::Lexer.new "
        func test(): int {
            if(true) {
                return 5
            }
        }
        "
        parser = Azula::Parser.new l
        program = parser.parse_program
        typechecker = Azula::Typechecker.new
        typechecker.check program
        typechecker.errors.size.should eq 0
    end

end
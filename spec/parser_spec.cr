require "./spec_helper"

describe Azula::Parser do

    it "assign" do
        l = Azula::Lexer.new "int i = 5"
        parser = Azula::Parser.new l
        program = parser.parse_program
        program.block.nodes[0].is_a?(Azula::AST::Assign).should be_true

        assign = program.block.nodes[0].as(Azula::AST::Assign)
        assign.to_s.should eq "int32 i = 5"

        l = Azula::Lexer.new "bool i = true"
        parser = Azula::Parser.new l
        program = parser.parse_program
        program.block.nodes[0].is_a?(Azula::AST::Assign).should be_true

        assign = program.block.nodes[0].as(Azula::AST::Assign)
        assign.to_s.should eq "bool i = true"

        l = Azula::Lexer.new "int64 i = 123"
        parser = Azula::Parser.new l
        program = parser.parse_program
        program.block.nodes[0].is_a?(Azula::AST::Assign).should be_true

        assign = program.block.nodes[0].as(Azula::AST::Assign)
        assign.to_s.should eq "int64 i = 123"
    end

    it "function" do
        l = Azula::Lexer.new "func my_func(int y): int { y }"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::Function).should be_true
        func = program.block.nodes[0].as(Azula::AST::Function)
        func.identifier.value.should eq "my_func"
        func.arguments.size.should eq 1
        func.arguments[0].value.should eq "y"
        func.arguments[0].type.is_a?(Azula::IntegerType).should be_true
        func.returns.value.should eq ""
        func.returns.type.is_a?(Azula::IntegerType).should be_true
        func.body.nodes[0].is_a?(Azula::AST::Identifier).should be_true

        l = Azula::Lexer.new "func my_func(int y, int z) { y }"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::Function).should be_true
        func = program.block.nodes[0].as(Azula::AST::Function)
        func.identifier.value.should eq "my_func"
        func.arguments.size.should eq 2
        func.arguments[0].value.should eq "y"
        func.arguments[0].type.is_a?(Azula::IntegerType).should be_true
        func.arguments[1].value.should eq "z"
        func.arguments[1].type.is_a?(Azula::IntegerType).should be_true
        func.returns.value.should eq ""
        func.returns.type.is_a?(Azula::VoidType).should be_true
        func.body.nodes[0].is_a?(Azula::AST::Identifier).should be_true

        l = Azula::Lexer.new "func my_func { 5 }"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::Function).should be_true
        func = program.block.nodes[0].as(Azula::AST::Function)
        func.identifier.value.should eq "my_func"
        func.arguments.size.should eq 0
        func.returns.type.is_a?(Azula::VoidType).should be_true
        func.body.nodes[0].is_a?(Azula::AST::IntegerLiteral).should be_true

        l = Azula::Lexer.new "func test(func(int, int): int math, int x, int y): int {
            math(x, y)
        }"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::Function).should be_true
        func = program.block.nodes[0].as(Azula::AST::Function)
        func.identifier.value.should eq "test"
        func.arguments.size.should eq 3
    end

    it "function call" do
        l = Azula::Lexer.new "my_func(5, 2)"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::FunctionCall).should be_true
        call = program.block.nodes[0].as(Azula::AST::FunctionCall)
        call.function.is_a?(Azula::AST::Identifier).should be_true
        function = call.function.as(Azula::AST::Identifier)
        function.value.should eq "my_func"
        call.arguments.size.should eq 2
        call.to_s.should eq "my_func(5, 2)"

        l = Azula::Lexer.new "my_func(x, y, z)"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::FunctionCall).should be_true
        call = program.block.nodes[0].as(Azula::AST::FunctionCall)
        call.function.is_a?(Azula::AST::Identifier).should be_true
        function = call.function.as(Azula::AST::Identifier)
        function.value.should eq "my_func"
        call.arguments.size.should eq 3
        call.to_s.should eq "my_func(x, y, z)"

        l = Azula::Lexer.new "my_func(other_func(1), y, z)"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::FunctionCall).should be_true
        call = program.block.nodes[0].as(Azula::AST::FunctionCall)
        call.function.is_a?(Azula::AST::Identifier).should be_true
        function = call.function.as(Azula::AST::Identifier)
        function.value.should eq "my_func"
        call.arguments.size.should eq 3
        call.to_s.should eq "my_func(other_func(1), y, z)"

        l = Azula::Lexer.new "int i = my_func(other_func(1), y, z)"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::Assign).should be_true
        assign = program.block.nodes[0].as(Azula::AST::Assign)
        assign.identifier.value.should eq "i"
        assign.value.is_a?(Azula::AST::FunctionCall).should be_true

        l = Azula::Lexer.new "int i = my_func()"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::Assign).should be_true
        assign = program.block.nodes[0].as(Azula::AST::Assign)
        assign.identifier.value.should eq "i"
        assign.value.is_a?(Azula::AST::FunctionCall).should be_true

        l = Azula::Lexer.new "func test: int { 5 }()"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::FunctionCall).should be_true
        call = program.block.nodes[0].as(Azula::AST::FunctionCall)
        call.function.is_a?(Azula::AST::Function).should be_true
        function = call.function.as(Azula::AST::Function)
        function.identifier.value.should eq "test"
        call.arguments.size.should eq 0
        call.to_s.should eq "func test(): int32 {\n5\n}()"

        l = Azula::Lexer.new "func { 5 }()"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::FunctionCall).should be_true
        call = program.block.nodes[0].as(Azula::AST::FunctionCall)
        call.function.is_a?(Azula::AST::Function).should be_true
        function = call.function.as(Azula::AST::Function)
        function.identifier.value.should eq ""
        call.arguments.size.should eq 0
        call.to_s.should eq "func (): void {\n5\n}()"
    end

    it "operator precedence" do
        l = Azula::Lexer.new "5 + 3 * 2 / 5 >> 3"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.to_s.should eq "(5 + (((3 * 2) / 5) >> 3))"
    end

    it "return" do
        l = Azula::Lexer.new "return 5"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::Return).should be_true
        ret = program.block.nodes[0].as(Azula::AST::Return)
        ret.value.is_a?(Azula::AST::IntegerLiteral).should be_true
        program.to_s.should eq "return 5"

        l = Azula::Lexer.new "return func { 5 }()"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::Return).should be_true
        ret = program.block.nodes[0].as(Azula::AST::Return)
        ret.value.is_a?(Azula::AST::FunctionCall).should be_true
        program.to_s.should eq "return func (): void {\n5\n}()"

        l = Azula::Lexer.new "func { return; }"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::Function).should be_true
    end

    it "if" do
        l = Azula::Lexer.new "if(x) { x }"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::If).should be_true
        if_node = program.block.nodes[0].as(Azula::AST::If)
        if_node.true_block.nodes[0].is_a?(Azula::AST::Identifier).should be_true

        l = Azula::Lexer.new "if(x) { x } else { int i = 5; print(i); }"
        parser = Azula::Parser.new l
        program = parser.parse_program
        parser.errors.size.should eq 0
        program.block.nodes.size.should eq 1
        program.block.nodes[0].is_a?(Azula::AST::If).should be_true
        if_node = program.block.nodes[0].as(Azula::AST::If)
        if_node.true_block.nodes[0].is_a?(Azula::AST::Identifier).should be_true
        if_node.false_block.not_nil!.nodes.size.should eq 2
    end
end
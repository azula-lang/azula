require "./spec_helper"
require "../src/azula/parser"

macro check_parser_errors(p)
    if !{{p}}.errors.empty?
        {{p}}.errors.each do |error|
            puts error
        end
        ({{p}}.errors.size > 0).should be_false
    end
end

def test_assign(node : Azula::AST::Statement, expected : String)
    node.should be_a(Azula::AST::Assign)
    assign = node.as(Azula::AST::Assign)

    assign.idents[0].as(Azula::AST::TypedIdentifier).ident.should eq expected
    assign.idents[0].as(Azula::AST::TypedIdentifier).token_literal.should eq expected
end

def test_literal_expression(expression : Azula::AST::Expression, expected : (Int32 | Bool | String | Float32)) : Bool
    case expected
    when .is_a?(Int32)
        expression = expression.as?(Azula::AST::IntegerLiteral)
        expression.nil?.should be_false
        return expression.not_nil!.value == expected
    when .is_a?(Bool)
        expression = expression.as?(Azula::AST::BooleanLiteral)
        expression.nil?.should be_false
        return expression.not_nil!.value == expected
    when .is_a?(String)
        expression = expression.as?(Azula::AST::StringLiteral)
        expression.nil?.should be_false
        return expression.not_nil!.value == expected
    when .is_a?(Float32)
        expression = expression.as?(Azula::AST::FloatLiteral)
        expression.nil?.should be_false
        return expression.not_nil!.value == expected
    end

    return false
end

describe Azula::Parser do

    describe "parse expressions" do
        describe "literals" do
            it "integer" do

                tests = {
                    "5;" => [5],
                    "10;" => [10],
                    "15; 20;" => [15, 20],
                    "2147483647;" => [2147483647],
                }

                tests.each do | test, result |
                    l = Azula::Lexer.new test
                    p = Azula::Parser.new l

                    program = p.parse_program
                    check_parser_errors p

                    program.statements.size.should eq result.size

                    program.statements.size.times do |index|
                        stmt = program.statements[index].as?(Azula::AST::ExpressionStatement)
                        stmt.nil?.should be_false

                        integer = stmt.not_nil!.expression.as?(Azula::AST::IntegerLiteral)
                        integer.nil?.should be_false

                        integer.not_nil!.value.should eq result[index]
                        integer.not_nil!.token_literal.should eq result[index].to_s
                    end
                end

            end

            it "float" do

                tests = {
                    "5.123;" => [5.123.to_f32],
                    "10.182;" => [10.182.to_f32],
                    "15.653; 20.254;" => [15.653.to_f32, 20.254.to_f32],
                    "2147483647.19;" => [2147483647.19.to_f32],
                }

                tests.each do | test, result |
                    l = Azula::Lexer.new test
                    p = Azula::Parser.new l

                    program = p.parse_program
                    check_parser_errors p

                    program.statements.size.should eq result.size

                    program.statements.size.times do |index|
                        stmt = program.statements[index].as?(Azula::AST::ExpressionStatement)
                        stmt.nil?.should be_false

                        integer = stmt.not_nil!.expression.as?(Azula::AST::FloatLiteral)
                        integer.nil?.should be_false

                        integer.not_nil!.value.should eq result[index]
                    end
                end

            end

            it "string" do

                tests = {
                    "\"hello world\";" => ["hello world"],
                    "\"hello\"; \"world\";" => ["hello", "world"],
                }

                tests.each do | test, result |
                    l = Azula::Lexer.new test
                    p = Azula::Parser.new l

                    program = p.parse_program
                    check_parser_errors p

                    program.statements.size.should eq result.size

                    program.statements.size.times do |index|
                        stmt = program.statements[index].as?(Azula::AST::ExpressionStatement)
                        stmt.nil?.should be_false

                        integer = stmt.not_nil!.expression.as?(Azula::AST::StringLiteral)
                        integer.nil?.should be_false

                        integer.not_nil!.value.should eq result[index]
                        integer.not_nil!.token_literal.should eq result[index].to_s
                    end
                end

            end

            it "boolean" do

                tests = {
                    "true;" => [true],
                    "false;" => [false],
                }

                tests.each do | test, result |
                    l = Azula::Lexer.new test
                    p = Azula::Parser.new l

                    program = p.parse_program
                    check_parser_errors p

                    program.statements.size.should eq result.size

                    program.statements.size.times do |index|
                        stmt = program.statements[index].as?(Azula::AST::ExpressionStatement)
                        stmt.nil?.should be_false

                        integer = stmt.not_nil!.expression.as?(Azula::AST::BooleanLiteral)
                        integer.nil?.should be_false

                        integer.not_nil!.value.should eq result[index]
                        integer.not_nil!.token_literal.should eq result[index].to_s
                    end
                end

            end
        end

        it "identifier" do

            input = "my_identifier;"
    
            l = Azula::Lexer.new input
            p = Azula::Parser.new l
    
            program = p.parse_program
            check_parser_errors p
    
            program.statements.size.should eq 1
    
            stmt = program.statements[0].as?(Azula::AST::ExpressionStatement)
            stmt.nil?.should be_false
    
            identifier = stmt.not_nil!.expression.as?(Azula::AST::Identifier)
            identifier.nil?.should be_false
    
            identifier.not_nil!.ident.should eq "my_identifier"
            identifier.not_nil!.token_literal.should eq "my_identifier"
    
        end

        it "prefix" do

            tests = {
                "!true;" => ["!", true],
                "!false;" => ["!", false],
            }

            tests.each do | test, result |
                l = Azula::Lexer.new test
                p = Azula::Parser.new l

                program = p.parse_program
                check_parser_errors p

                program.statements.size.should eq 1

                stmt = program.statements[0].as?(Azula::AST::ExpressionStatement)
                stmt.nil?.should be_false

                prefix = stmt.not_nil!.expression.as?(Azula::AST::Prefix)
                prefix.nil?.should be_false

                right = prefix.not_nil!.right.as?(Azula::AST::BooleanLiteral)
                right.nil?.should be_false

                prefix.not_nil!.operator.should eq result[0]
                right.not_nil!.value.should eq result[1]
            end

        end

        it "infix" do

            tests = {
                "5 + 5;" => [5, "+", 5],
                "10.5 + 12.2;" => [10.5.to_f32, "+", 12.2.to_f32],
                "true == true;" => [true, "==", true],
                "\"hello\" + \"world\";" => ["hello", "+", "world"],
                "5 == 2;" => [5, "==", 2],
                "5 > 2;" => [5, ">", 2],
                "5 >= 2;" => [5, ">=", 2],
                "5 < 2;" => [5, "<", 2],
                "5 <= 2;" => [5, "<=", 2],
                "5 * 2;" => [5, "*", 2],
                "5 / 2;" => [5, "/", 2],
                "5 % 2;" => [5, "%", 2],
                "true and true;" => [true, "and", true],
                "true or true;" => [true, "or", true],
            }

            tests.each do | test, result |
                l = Azula::Lexer.new test
                p = Azula::Parser.new l

                program = p.parse_program
                check_parser_errors p

                program.statements.size.should eq 1

                stmt = program.statements[0].as?(Azula::AST::ExpressionStatement)
                stmt.nil?.should be_false

                infix = stmt.not_nil!.expression.as?(Azula::AST::Infix)
                infix.nil?.should be_false

                (test_literal_expression infix.not_nil!.left, result[0]).should be_true
                infix.not_nil!.operator.should eq result[1]
                (test_literal_expression infix.not_nil!.right, result[2]).should be_true
            end

        end
    end

    describe "parse statements" do
    
        it "assign" do

            tests = {
                "int x = 5;" => ["x"],
                "int y = 10;" => ["y"],
                "float x = 10.2; bool y = 11;" => ["x", "y"],
                "string my_string = \"hello world\";" => ["my_string"],
                "int x, y = 5, 2;" => ["x"],
            }

            tests.each do | test, result |
                l = Azula::Lexer.new test
                p = Azula::Parser.new l

                program = p.parse_program
                check_parser_errors p

                program.statements.size.should eq result.size

                program.statements.size.times do |index|
                    test_assign(program.statements[index], result[index])
                end
            end

        end

        it "return" do

            input = "
            return 5;
            return 5.2;
            return true;
            return \"yes\";
            "

            l = Azula::Lexer.new input
            p = Azula::Parser.new l

            program = p.parse_program
            check_parser_errors p

            program.statements.size.should eq 4

            program.statements.each do |stmt|
                stmt = stmt.as?(Azula::AST::Return)
                stmt.nil?.should be_false
                stmt.not_nil!.token_literal.should eq "return"
            end

        end

        it "if" do

            input = "
            if(5 < 2) {
                5;
            } elseif(2 > 5) {
                10;
            } else {
                20;
            }
            "

            l = Azula::Lexer.new input
            p = Azula::Parser.new l

            program = p.parse_program
            check_parser_errors p

            program.statements.size.should eq 1

            ifstmt = program.statements[0].as?(Azula::AST::If)
            ifstmt.nil?.should be_false

            # Condition
            condition = ifstmt.not_nil!.condition.as?(Azula::AST::Infix)
            condition.nil?.should be_false
            (test_literal_expression condition.not_nil!.left, 5).should be_true
            condition.not_nil!.operator.should eq "<"
            (test_literal_expression condition.not_nil!.right, 2).should be_true

            # Consequence
            consequence = ifstmt.not_nil!.consequence.statements[0].as?(Azula::AST::ExpressionStatement)
            consequence.nil?.should be_false
            (test_literal_expression consequence.not_nil!.expression, 5).should be_true

            # Alts
            alt = ifstmt.not_nil!.alts[0].condition.as?(Azula::AST::Infix)
            alt.nil?.should be_false
            (test_literal_expression alt.not_nil!.left, 2).should be_true
            alt.not_nil!.operator.should eq ">"
            (test_literal_expression alt.not_nil!.right, 5).should be_true

            # Alternative
            alternative = ifstmt.not_nil!.alternative.not_nil!.statements[0].as?(Azula::AST::ExpressionStatement)
            alternative.nil?.should be_false
            (test_literal_expression alternative.not_nil!.expression, 20).should be_true

        end

        it "function definition" do

            input = "
            func function(int x, string y) : bool {
                5;
            }
            "

            l = Azula::Lexer.new input
            p = Azula::Parser.new l

            program = p.parse_program
            check_parser_errors p

            program.statements.size.should eq 1

            function = program.statements[0].as?(Azula::AST::Function)
            function.nil?.should be_false

            function.not_nil!.parameters.size.should eq 2

            function.not_nil!.parameters[0].type.main_type.should eq Azula::Types::TypeEnum::INT
            function.not_nil!.parameters[0].ident.should eq "x"

            function.not_nil!.parameters[1].type.main_type.should eq Azula::Types::TypeEnum::STRING
            function.not_nil!.parameters[1].ident.should eq "y"

            function.not_nil!.return_type.main_type.should eq Azula::Types::TypeEnum::BOOL

            body = function.not_nil!.body.statements[0].as?(Azula::AST::ExpressionStatement)
            body.nil?.should be_false

            (test_literal_expression body.not_nil!.expression, 5).should be_true

        end

        it "external function definition" do

            input = "
            extern func function(int x, string y) : bool;
            "

            l = Azula::Lexer.new input
            p = Azula::Parser.new l

            program = p.parse_program
            check_parser_errors p

            program.statements.size.should eq 1

            function = program.statements[0].as?(Azula::AST::ExternFunction)
            function.nil?.should be_false

            function.not_nil!.parameters.size.should eq 2

            function.not_nil!.parameters[0].type.main_type.should eq Azula::Types::TypeEnum::INT
            function.not_nil!.parameters[0].ident.should eq "x"

            function.not_nil!.parameters[1].type.main_type.should eq Azula::Types::TypeEnum::STRING
            function.not_nil!.parameters[1].ident.should eq "y"

            function.not_nil!.return_type.main_type.should eq Azula::Types::TypeEnum::BOOL
        end

        it "function call" do 
            input = "
            function(5, \"yes\");
            "

            l = Azula::Lexer.new input
            p = Azula::Parser.new l

            program = p.parse_program
            check_parser_errors p

            program.statements.size.should eq 1

            stmt = program.statements[0].as?(Azula::AST::ExpressionStatement)
            stmt.nil?.should be_false

            function_call = stmt.not_nil!.expression.as?(Azula::AST::FunctionCall)
            function_call.nil?.should be_false

            function_call.not_nil!.function_name.ident.should eq "function"

            function_call.not_nil!.arguments.size.should eq 2

            (test_literal_expression function_call.not_nil!.arguments[0], 5).should be_true
            (test_literal_expression function_call.not_nil!.arguments[1], "yes").should be_true
        end

        it "while" do

            input = "
            while(true) {
                5;
            }
            "

            l = Azula::Lexer.new input
            p = Azula::Parser.new l

            program = p.parse_program
            check_parser_errors p

            program.statements.size.should eq 1

            while_stmt = program.statements[0].as?(Azula::AST::While)
            while_stmt.nil?.should be_false

            (test_literal_expression while_stmt.not_nil!.iterator, true).should be_true

            body = while_stmt.not_nil!.body.statements[0].as?(Azula::AST::ExpressionStatement)
            body.nil?.should be_false

            (test_literal_expression body.not_nil!.expression, 5).should be_true

        end

    end

    it "operator precedence" do

        tests = {
            "-a * b;" => "((-a) * b)",
            "3 + 4 * 5 == 3 * 1 + 4 * 5;" => "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            "(5 + 5) * 2;" => "((5 + 5) * 2)",
            "!(true == true);" => "(!(true == true))",
            "-(5 + 5);" => "(-(5 + 5))",
            "1 + (2 + 3) + 4;" => "((1 + (2 + 3)) + 4)",
        }

        tests.each do | test, result |
            l = Azula::Lexer.new test
            p = Azula::Parser.new l

            program = p.parse_program
            check_parser_errors p

            program.statements.size.should eq 1

            program.to_string.should eq result
        end

    end

end
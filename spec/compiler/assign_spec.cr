require "./spec_helper"

describe Azula::Compiler do

    describe Azula::Compiler::Visitors::Assign do

        it "int" do 
            input = wrap_main("int i = 5; println(i);", "void")
            run(input).not_nil!.to_i.should eq 5
        end

        it "float" do
            input = wrap_main("float y = 5.234; println(y);", "void")
            # Floats are being awkward
            run(input).not_nil!.should eq "5.234000"
        end

        it "string" do
            input = wrap_main("string s = \"hi\"; println(s);", "void")
            compile_and_run(input).should eq "hi"
        end

        it "boolean" do
            input = wrap_main("bool x = true; println(x);", "void")
            run(input).not_nil!.should eq "true"

            input = wrap_main("bool x = false; println(x);", "void")
            run(input).not_nil!.should eq "false"
        end

        it "array" do
            input = wrap_main("array(int) y = [1, 2, 3, 4, 5]; println(y[3]);", "void")
            run(input).not_nil!.should eq "4"
        end

        it "multiple assign" do
            input = wrap_main("int i, y = 5, 10; println(y);", "void")
            run(input).not_nil!.should eq "10"
        end

        it "multiple assign function" do
            input = "
            func test(int x): int {
                return x;
            }

            func main(): void {
                int z = 5;
                int x, y = test(z), test(z + 5);

                println(x, y);
            }
            "
            compile_and_run(input).should eq "5 10"
        end

        it "var" do
            input = wrap_main("var x, y = 5, 10; println(x + y);", "void")
            run(input).not_nil!.should eq "15"
        end

    end

end
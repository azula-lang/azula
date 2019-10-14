require "./spec_helper"

describe Azula::Compiler do

    describe Azula::Compiler::Visitors::Assign do

        it "int" do 
            input = wrap_main("int i = 5; return i;", "int")
            run(input).not_nil!.to_i.should eq 5
        end

        it "float" do
            input = wrap_main("float y = 5.234; return y;", "float")
            # Floats are being awkward
            run(input).not_nil!.to_f64.to_f32.should eq 5.234.to_f32
        end

        it "string" do
            input = wrap_main("string s = \"hi\"; println(s);", "string")
            compile_and_run(input).should eq "hi"
        end

        it "boolean" do
            input = wrap_main("bool x = true; return x;", "bool")
            run(input).not_nil!.to_b.should be_true

            input = wrap_main("bool x = false; return x;", "bool")
            run(input).not_nil!.to_b.should be_false
        end

        it "multiple assign" do
            input = wrap_main("int i, y = 5, 10; return y;", "int")
            run(input).not_nil!.to_i.should eq 10
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

    end

end
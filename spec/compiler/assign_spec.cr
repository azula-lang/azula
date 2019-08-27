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

        # it "string" do
        #     input = wrap_main("string s = \"hi\"; return s;", "string")
        #     run(input).not_nil!.to_string.should eq "hi"
        # end

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

    end

end
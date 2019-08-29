require "../spec_helper"

describe Azula::Compiler do

    describe "std" do

        describe "print" do

            it "string" do
                input = wrap_main("string s = \"hi\"; print(s);", "void")
                compile_and_run(input).should eq "hi"
            end

            it "int" do
                input = wrap_main("print(5);", "void")
                compile_and_run(input).should eq "5"
            end

            it "string concat" do
                input = wrap_main("print(\"hi\", \"there\");", "void")
                compile_and_run(input).should eq "hi there"
            end

        end

    end

end
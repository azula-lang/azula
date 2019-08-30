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

                input = wrap_main("int x = 10; print(x);", "void")
                compile_and_run(input).should eq "10"
            end

            it "boolean" do
                input = wrap_main("print(true);", "void")
                compile_and_run(input).should eq "true"

                input = wrap_main("bool x = false; print(x);", "void")
                compile_and_run(input).should eq "false"
            end

            it "string concat" do
                input = wrap_main("string y = \"friend\"; print(\"hi\", \"there\", y);", "void")
                compile_and_run(input).should eq "hi there friend"
            end

        end

    end

end
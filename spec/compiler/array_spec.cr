require "./spec_helper"

describe Azula::Compiler do

    describe "array" do

        it "basic" do
            input = "
            func main(): int {
                array(int) y = [1, 2, 3, 4, 5, 11];
                return y[5];
            }
            "
            run(input).not_nil!.to_i.should eq 11
        end

        it "string" do
            input = "
            func main(): void {
                array(string) y = [\"hi\", \"strings\"];
                println(y[1]);
            }
            "
            compile_and_run(input).not_nil!.should eq "strings"
        end

    end

end
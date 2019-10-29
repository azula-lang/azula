require "./spec_helper"

describe Azula::Compiler do

    describe "functions" do

        it "basic function" do
            input = "
            func x(): int {
                return 5;
            }

            func main(): void {
                println(x());
            }
            "

            run(input).not_nil!.to_i.should eq 5
        end

        it "function args" do
            input = "
            func x(int y, int z): int {
                return y + z;
            }

            func main(): void {
                println(x(10, 20));
            }
            "

            run(input).not_nil!.to_i.should eq 30
        end

    end

end
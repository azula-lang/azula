require "./spec_helper"

describe Azula::Compiler do

    it "compile code" do
        run("func main(): void { println(5); }").not_nil!.to_i.should eq 5
    end

    it "fibonacci" do
        input = "
        func fib(int x): int {
            if(x == 0 or x == 1) {
                return x;
            }
            return fib(x-1) + fib(x-2);
        }

        func main(): void {
            println(fib(15));
        }
        "
        run(input).not_nil!.to_i.should eq 610
    end

end
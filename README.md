## Azula

Azula is a strongly-typed compiled language, using an LLVM backend, with the following goals:
- Static typing
- Easy-to-read syntax
- Efficient execution

[Documentation](https://azula-lang.github.io/azula/#/)

[Discord](https://discord.gg/Hkx8XnB)

### Current State of Azula

Azula is currently working at a basic level. Azula code can be compiled to a binary with support for packages and imports. All of the examples in the [examples](https://github.com/azula-lang/azula/tree/master/examples) folder are runnable. Azula is still missing some base features and a standard library - something I hope to fix soon.

### Installing

Clone the repository, run the `install.sh` script.

### Origin

Azula started as a learning exercise through the [Writing a Interpreter in Go](https://interpreterbook.com) and [Writing a Compiler in Go](https://compilerbook.com) books by Thorsten Ball. As I reached the end of the compiler, rather than expanding the VM for Azula, I decided compiling to LLVM would make the language far more usable. Rather than attempting to change the version written in Go, I decided to rewrite Azula from scratch in Crystal.

### Goals

- ~~Tokenizer~~
- ~~Lexer~~
- ~~Parser~~
- Compiler
- Write documentation in code
- Compiler Optimization
- Standard Library
- Self-hosting compiler

## Compiling Your Code
```
azula build FILENAME
```

or to run directly:
```
azula run FILENAME
```

or to view the LLIR output:
```
azula llir FILENAME
```

### Example Code

```
func fib(int x): int {
    if(x == 0 or x == 1) {
        return x;
    }
    return fib(x - 1) + fib(x - 2);
}

func main(): void {
    println(fib(15));
}
```

## Contributors
- [OisinA](https://github.com/OisinA)

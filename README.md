## Azula

Azula is a strongly-typed compiled language, using an LLVM backend, with the following goals:
- Static typing
- Easy-to-read syntax
- Efficient execution

[Documentation](https://azula-lang.github.io/azula/#/)

[Discord](https://discord.gg/Hkx8XnB)

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
    if(x == 0 || x == 1) {
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

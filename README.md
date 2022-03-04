## Azula

[![chat](https://img.shields.io/discord/606118150655705088)](https://discord.gg/Hkx8XnB) [![issues](https://img.shields.io/github/issues/azula-lang/azula)](https://github.com/azula-lang/azula/issues)

Azula is a strongly-typed compiled language, using an LLVM backend, with the following goals:
- Static typing
- Easy-to-read syntax
- Efficient execution

[Discord](https://discord.gg/Hkx8XnB)

## Compiling Your Code
```
azula build FILENAME
```

or to run directly:
```
azula run FILENAME
```

## Progress

Short term goals:

- [x] Lexing
- [x] Parsing
- [x] Typechecking
- [x] Azula IR codegen
- [x] LLVM backend
- [x] Hooking into C standard library functions
- [ ] Arrays
- [ ] Loops
- [ ] Structures
- [ ] Methods
- [ ] Multi-file projects

### Example Code

```
func fib(x: int): int {
    if x == 0 || x == 1 {
        return x;
    }
    return fib(x - 1) + fib(x - 2);
}

func main {
    printf("%d\n", fib(15));
}
```

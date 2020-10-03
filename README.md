## Azula

![build](https://img.shields.io/github/workflow/status/azula-lang/azula/Azula%20tests) [![chat](https://img.shields.io/discord/606118150655705088)](https://discord.gg/Hkx8XnB) [![issues](https://img.shields.io/github/issues/azula-lang/azula)](https://github.com/azula-lang/azula/issues)

Azula is a strongly-typed compiled language, using an LLVM backend, with the following goals:
- Static typing
- Easy-to-read syntax
- Efficient execution

[Documentation](https://azula-lang.github.io/azula/#/)

[Discord](https://discord.gg/Hkx8XnB)

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
        return x
    }
    return fib(x - 1) + fib(x - 2)
}

func main {
    printf("%d", fib(15))
}
```

## Contributors
- [OisinA](https://github.com/OisinA)
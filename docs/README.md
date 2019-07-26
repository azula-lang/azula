## Azula

Azula is a strongly-typed compiled language, using an LLVM backend, with the following goals:
- Static typing
- Easy-to-read syntax
- Efficient execution

### Origin

Azula started as a learning exercise through the [Writing a Interpreter in Go](https://interpreterbook.com) and [Writing a Compiler in Go](https://compilerbook.com) books by Thorsten Ball. As I reached the end of the compiler, rather than expanding the VM for Azula, I decided compiling to LLVM would make the language far more usable. Rather than attempting to change the version written in Go, I decided to rewrite Azula from scratch in Crystal.

### Goals

- Tokenizer
- Lexer
- Parser
- Compiler
- Compiler Optimization
- Standard Library
- Self-hosting compiler

### Example Code

```
func fib(int x): int {
    if(x == 0 or x == 1) {
        return x;
    }
    return fib(x - 1) + fib(x - 2);
}

func main(): void {
    print(fib(15));
}
```

## Syntax

### Types

Azula supports a number of built-in types, including:
- int
- float
- string
- bool
- array
- map
- error

Type must be declared when creating a variable, for examples:
```
int i = 5;
string s = 10;
float f = 5.5;
bool b = true;
array(int) a = [1, 2, 3, 4];
map(string, int) m = {"Azula"=>1};
```

#### Structs

Custom types, known as structs, can be declared using the `struct` keyword.

```
struct Person {
    string name,
    int age,
}

Person p = Person{"Oisin", 19};
print(p.name, p.age); // Oisin 19
```

#### Type casting

Types can be cast to one another using the `as` keyword.

```
int i = 5;
string s = i as string;
```

#### Null

Null is a potential value of any variable. However if that variable is accessed, a runtime exception will be caused. Null cannot be explicitly assigned to a variable, but can be returned from a function. It can implictly assigned by not giving the variable a value.

```
string s = null; // invalid
string s; // valid

if(s != null) {
    print(s); // won't error, as s is checked
}

print(s); // causes an error if s is null.
```

### Functions

Functions can be declared using the `func` keyword. Parameter and return types must be explicitly declared. Azula supports multiple return types. If a function does not return anything, `void` is used as the return type.

```
func my_func(int x, int y): (int, int) {
    return x, y;
}
int x, y = my_func(5, 2);

func print_message(string message): void {
    print(message);
    return; // not necessary - no return returns null
}

string s = print_message("hello!"); // s is now null
```

### Operators

#### Syntax

Azula uses the following in its syntax:
- `=>` Map assignment and switch statements
    - `{5=>true};`
- `=` Assignment
    - `int x = 5;`
- `:` Return Type
    - `func x(): int {}`

#### Arithmetic

Azula supports the following arithmetic operators:
- `+` addition
    - `1 + 2; // 3`
- `-` subtraction
    - `2 - 1; // 1`
- `*` multiplication
    - `2 * 3; // 6`
- `/` division
    - `15 / 2; // 7.5`
- `**` exponentation
    - `2 ** 3; // 8`
- `%` modulus
    - `15 % 2; // 1`

#### Logical

Azula supports the following logical operators:
- `==` equal
    - `5 == 5; // true`
- `!=` not equal
    - `5 != 3; // true`
- `<` less than
    - `3 < 5; // true`
- `<=` less than or equal
    - `5 <= 5; // true`
- `>` greater than
    - `5 > 3; // true`
- `>=` greater than or equal
    - `5 >= 5; // true`
- `or` Logical OR
    - `x or y; // true if x or y are true`
- `and` Logical AND
    - `x and y; // true if x and y are true`
- `!` Logical NOT
    - `!false; // true`

### Conditionals

#### If

Azula has `if`, `elseif`, `else` and `unless` statements.

```
string y = "azula";

if(y == "azula") {
    print("azula!");
} elseif(y == "abc") {
    print("else if!");
} else {
    print("else!");
}
```

#### Switch

Azula has switch statements using the `switch` keyword.

```
int x = 5;
switch(x) {
    x => 0 {
        print("it is 0!");
    },
    x => 1 {
        print("it is 1!");
    },
    default {
        print("it was something else");
    },
}
```

### Loops

Azula has loops through the `for` keyword.

```
bool x = true;

for(x) {
    print(x);
}

array(int) list = [1, 2, 3, 4, 5];

for(int x in list) {
    print(x); // prints 1, 2, 3, 4, 5
}
```

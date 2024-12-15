# SERV

Serv is a swiss army knife for writing web servers and APIs, designed
to make the developer experience as simple and enjoyable as possible.
It is an entirely new language and runtime, which virtually eliminates
the need for boilerplate code and makes many of the most common web
tasks, such as reading a file, rendering a template, and querying a
database, trivial one liners.

Serv was heavily inspired by this lecture by Brian Kernighan:
In it, he references languages like Awk and talks about the
practice of crafting languages around a particular domain,
such that problems inside that domain are easy to express.
This is becoming increasingly rare, particularly in the web domain
where most tools are implemented as libraries on top of existing
languages like javascript or python. Doing so lowers the barrier to
entry, but loses some of the power that a truly custom language can
provide. Serv aims to continue this tradition by becoming a
sort of "Awk for web programming", not necessarily the most
efficient, or easiest to learn, but once understood, the most
natural way to solve most problems in its domain.

[https://www.youtube.com/watch?v=Sg4U4r_AgJU&pp=ygUfYnJpYW4ga2VybmlnaGFuIGxhbmd1YWdlIGRlc2lnbg%3D%3D](https://www.youtube.com/watch?v=Sg4U4r_AgJU&pp=ygUfYnJpYW4ga2VybmlnaGFuIGxhbmd1YWdlIGRlc2lnbg%3D%3D)


Here are some examples of Serv in action:

```python
# Hello World:
# curly brackets denote strings
/ => {Hello World!}

# Hello World with a more personal touch:
# (all strings are templates, reference variables with a $)
/hello/{name} => {Hello $name!}

# Statically serve all files in the directory
# the 'file' function reads the contents of a file
/{*f} => file f

# Serve a markdown file dynamically rendered to HTML
# functions can be chained together to create complex expressions
/index => markdown file {www/index.md}

# Define your own functions:
square = %{ $: * $: }
/api/square/{x} => square x

# Write API endpoints directly in SQL
# serv automatically sanitizes your input, and converts the output to JSON
sql.database  = {my_database.sqlite}
/api/{user}/posts => sql { SELECT (title, content) FROM posts WHERE user = $user; }

# Compute the Fibonnaci sequence
fib = switch {
	(eq 0) => 1
	(eq 1) => 1
	(else) => sum | (fib-) (fib--)
}

/fib/{length} => map fib count length
```

## Installation and Usage

Serv can be installed using the rust toolchain and cargo:

```bash
cargo install --git https://github.com/connorwiniarczyk/serv.git
```

Run serv in any directory with a main.serv file, or specify
a file manually. Serv will run on port 4000 by default,
or you can specify a port by defining the `@serv.port` function.

Serv will use TLS encryption if the `@serv.tlskey` and `@serv.tlscert`
functions are defined.

```python
# main.serv
serv.port    = 443
serv.tlscert = file {certfile.pem}
serv.tlskey  = file {keyfile.pem}

/ => {Hello World}
```

```bash
serv main.serv
starting encrypted server
listening on port 443
```

## Syntax

Serv is a *concatinative* functional language with a prefix call notation.
Every expression is composed of a sequence of functions, and every function
operates on everything that comes after it in the expression. Typically, an
expression is evaluated by taking the leftmost function out of the expression,
then recursively evaluating the remainder of the expression until no functions remain,
then calling the function on the result.

For example, the expression `+ + + 0` is evaluated by taking the first function (`+`)
which adds 1 to it's input, then evaluating all of `+ + 0`, and so on until reaching `0`,
which is a function returning 0. Each `+` is then called in reverse order, right to left,
until the expression eventually returns 3.

### Metaprogramming

Metaprogramming in serv is implemented with a pair of functions called quote (`]`)
and dequote (`[`). The quote function skips the recursive evaluation step described above,
and instead returns the remaining program as a list of functions to be manipulated.
The dequote function does the reverse of this, evaluating a list of functions into
a value. Typically a meta expression will exist between a dequote and a quote function.

For example, it is common in serv for functions to take more than one argument, such
as `map` which maps a function onto a list. This can be done with the meta expression
`[ pop arg ]`, which will first call quote on the remainder of the program, then
call `pop` on the list, removing the first element and placing it into the scope as
the word `arg`, then calling dequote on the rest of the list. Other common meta functions
are drop (`!`) which removes the next word, list (`|`) which calls dequote on each
element individually to create a list of values.

```
sum map (++) | 1 2 3 4
```

This example is evaluated in the following way: First sum is taken out of
the expression and the remainder is evaluated as its input. When map is called,
it also pops the function `(++)` out of the expression as its argument. Then
`| 1 2 3 4` is evaluated. The list function `|` does not evaluate, `1 2 3 4`,
(which would return 1), but instead evaluates each element individually and
returns the list `[1, 2, 3, 4]`. Then map calls `(++)` on each element,
returning `[3, 4, 5, 6]`, and sum adds them together, returning 18.

### Strings

Strings in serv are functions as well, and are denoted with curly brackets `{}`. Strings
are allowed to span multiple lines and can contain most special characters, even additional
balanced pairs of curly brackets, without needing to be escaped. If a string contains
a `$` followed by an expression, that expression will be evaluated and its result inserted into
the string at that location. 

## Functions

This is a non-exhaustive list, I'm adding more
constantly

| Word      | Effect                            |
|:----------|:----------------------------------|
| hello     | the string "Hello World"          |
| uppercase | convert the input to uppercase    |
| %         | compute a mathematical expression |
| !         | drop the next word                |
| +         | increase the value by 1           |
| -         | decrease the value by 1           |
| file      | read the contents of the file to a string     |
| file.raw  | read the contents of the file to a byte array    |
| exec      | execute a program on the host machine and return the result    |
| exec.pipe | same as exec, but the second argument is piped into stdin of the child program |
| markdown  | render a markdown string as HTML |
| sql       | execute the sql query           |
| sql.exec  | execute the sql query, but ignore the result          |
| ls        | generate a list of files at a given path           |
| count     | generate a list of counting numbers of size n |
| map       | map a function onto a list |
| fold      | reduce a list by calling a function on each element |
| using     | define additional words to be used in the rest of the expression |
| switch    | take an index and a list of functions, apply the function at that index to the input |

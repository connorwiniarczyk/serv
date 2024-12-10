# SERV

Serv is a swiss army knife for writing http servers quickly and with as
little friction as possible. It is similar in function to web frameworks
like Express and Flask, but instead of existing as a library on top of a
general purpose language like Javascript or Python, Serv uses its own
custom language, implemented from the ground up specifically
for writing web backends.

By doing so, Serv frees itself from almost
all of the boilerplate required to build a web server in the traditional
way. It is a single executable with a fully featured web server built
directly into the runtime, a standard library full of useful tools,
and a dedicated syntax for assigning routes.

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
plus3 = %{ $: * $: }
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

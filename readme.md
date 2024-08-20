# SERV!

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
# in refers to the input of the function, % calculates a mathematical expression
@square => %{ $in * $in }
/api/square/{x} => square x

# Write API endpoints directly in SQL
# serv automatically sanitizes your input, and converts the output to JSON
@sql.database  => {my_database.sqlite}
/api/{user}/posts => sql { SELECT (title, content) FROM posts WHERE user = $user; }

# Compute the Fibonnaci sequence
@fib => switch (in) [1, 1, %{ $(fib decr) + $(fib decr decr) }]
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
@serv.port    => 443
@serv.tlscert => file {certfile.pem}
@serv.tlskey  => file {keyfile.pem}

/ => {Hello World}
```

```bash
serv main.serv
starting encrypted server
listening on port 443
```

## Functions

This is a non-exhaustive list, I'm adding more
constantly

| Word      | Effect                            |
|:----------|:----------------------------------|
| hello     | the string "Hello World"          |
| uppercase | convert the input to uppercase    |
| %         | compute a mathematical expression |
| !         | drop the next word                |
| incr      | increase the value by 1           |
| decr      | decrease the value by 1           |
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
| join      | join a list into a single string with the given separator |
| split     | split a string into a list by the given separator |

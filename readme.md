# SERV

Serv is an experimental backend web framework built using its own bespoke scripting language.
By using a custom language that is tailored to the needs of writing a web server, it is able to virtually eliminate the need for boilerplate and dependencies,
while also providing quality of life improvements over more traditional web frameworks.

Hello world in Serv is one line and looks like this, it creates a server on port 4000 that will reply with the string hello world.

```
/ => {hello world}
```

A static file server is also one line and looks like this:

```
/{*f} => file f
```

## Getting Started

Serv can be installed using the rust toolchain and cargo:

```bash
git clone https://github.com/connorwiniarczyk/serv.git
cargo install --path ./serv

# or

cargo install --git https://github.com/connorwiniarczyk/serv.git

# run hello world
serv -e "/ => {hello world!}"
```

Once your hello world server is running, navigate to
[http://localhost:4000](http://localhost:4000) to see it working.
If no argument is given, serv will look for a main.serv file in
the current directory and run that. Serv will run on port 4000 by default,
or you can specify a port by defining `server.port`.

### Server Config

The server created by serv can be configured by modifying fields in the
server module. It supports changing the port by assigning server.port,
and using Https by assigning server.certificate and server.private_key.

```
server.port = 443
server.certificate = file {certfile.pem}
server.private_key = file {keyfile.pem}

/ => {Encrypted Hello World!}

print {listening on port: $server.port}
```

## Background (Ramble)

Most of the web servers I write end up looking very similar to each other.
They have a handful of routes, each serving a file or the result of a database
query, sometimes with a transformation or two applied. For something that simple,
web frameworks like Express, Flask, or Rocket always felt like overkill, they
are far more powerful than I actually need, and the cost of that power is
boilerplate and dependency management. I wanted to try creating a less powerful
tool that was more tailored to the tasks my web servers actually do.

There is a fantastic
[lecture by Brian Kernighan](https://www.youtube.com/watch?v=Sg4U4r_AgJU)
about language design which became a huge inspiration for many of the ideas in serv.
He talks specifically about the process of crafting a language for a domain
such that, if a problem seemed simple in your head, it would be simple to express
when you wrote the code. The example he gave was Awk for the domain of
data processing, but it struck me that web servers ought to have such a language too.

### Concatenative Languages and Om

Serv falls into a family of languages called [concatenative](https://concatenative.org/wiki/view/Concatenative%20language),
which are named for the property that expressions are built by concatenating smaller units together into a list.
Of these, almost all use a *postfix* call notation, where each function operates on what comes before
it in the program. These languages typically have a single global stack that every function operates on.

Serv works slightly differently, it uses a *prefix* call notation, meaning that every function operates on what comes
after it in the expression, and rather than operating on a global stack, functions operate on the program itself.

From what I can tell, only one other language shares this property, an experimental language called [Om](https://www.om-language.com/index.html)
and it was the inspiration for so many of the ideas that ended up in Serv.

## Tutorial

Every expression in serv is a list of functions concatenated together, where each
function is an operation on the rest of the list. For example, the below expression
will simply print the text "hello.md"

```
print {hello.md}
```

If we add the `file` function, it will instead print the contents of hello.md

```
print file {hello.md}
```

Finally, if we add the `markdown` function, we will get the contents of hello.md parsed
as markdown into HTML.

```
print markdown file {hello.md}
```

Functions can be chained together in this way to create arbitrarily complex behavior.
Expressions can be mapped onto http routes to define the endpoints of the server,
or onto identifiers in order to create new functions. If any expressions are not mapped
onto a route or identifier, the interpreter will run them in order.

### Strings

A string is any piece of text enclosed by curly brackets `{}`. Strings are allowed to
span multiple lines, and can contain balanced pairs of nested curly brackets without
escaping. This makes it very easy to embed other languages inside of serv strings.

```
/main.js => {
    window.onload = function() {
    	console.log("hello from javascript");
    }
}
```

Serv strings are also templates, and can embed other serv expressions inside
of them. This can be done with the `$` character followed by a single identifier of
by an expression enclosed by parentheses.

```
name = {connor}

/hello => {hello $name!}
/HELLO => {hello $(uppercase name)}
```

Expressions inside of string templates can use the special identifier `*` to reference the remainder
of their expression, which allows you to use them as functions.

```
/=> {<html>$*</html>} {<body>$*</body>} {
    <h1> this is the body </h1>
    <p>$(lorem 100)</p>
    <p>$(lorem 100)</p>
}
```

### Route Patterns

Route definitions are allowed to contain patterns in order to match multiple requests. Patterns
take the form `/literal/{var1}/{var2}_literal/{*rest}`, where text inside of `{}` brackets is treated
as a variable that can match arbitrary text. Variables that are matched in this will be added to the
scope of the matched expression.

```
/hello_{name} => {hello $name}
```

Variables that begin with a `*` will match across path delimiters, allowing you to request files
nested in other directories.

```
/{*f} => file f
```

### Structured Data

In addition to serving content, serv also provides tools for working with and serving
structured data from databases and other sources. Serv currently includes a an `sqlite`
module with functions for working with sqlite databases.

```
@include sqlite

# connect to the database file `example.sqlite`
connect {example.sqlite}

# run some queries to initialize the database
query {create table if not exists users (id INTEGER PRIMARY KEY, name TEXT UNIQUE);}
query {insert into users (name) values ('connor');}

/users => query {select name from users;}
```

If a function returns structured data of any kind, serv will automatically serialize it
into JSON before sending the response.

```bash
$ curl localhost:4000/users
[
  {
    "name": "connor",
    "id": 1
  }
]

```

### Modules

The output of the serv parser is a data structure called a module, which is a table mapping
each route or identifier in the file to its corresponding expression. At the top level, this
table is used to define the behavior of the web server, but the language also gives you tools
to generate and use modules inside of expressions. The `serv` function can be used to generate
a new serv module out of arbitrary text.

```
@include serv {
	name = {connor}
}

/ => name
```

This allows you to split larger programs into several different files and import them like so:

```
@include serv file {utils.serv}
@include serv file {components.serv}
```

Modules can also be created inline by with parentheses `()`. Expressions can be delimited by
line breaks, semicolons, or commas. By default, modules created this way are functions that when evaluated,
create a new scope with each of their assignments, then run each of their unassigned
expressions in order, returning the result of the last expression. This can be a useful tool for creating
functions with side effects, or for functions that might need a more complex local scope.

```
/{*f} => (
	directory = {/www}
    print {serving file: $directory/$f};
    file {$directory/$f};
)
```

### Advanced Evaluation

As stated above, each function in a serv expression performs an operation on
the rest of the expression, not necessarily just the result of that expression.
Most typically, functions in serv work by first evaluating the remaining expression
into a single value, and then doing an operation on that value, but this is not
always the case, and many functions perform operations on the list of values
that follow them before evaluating them.

The simplest example of such a function is the drop function `!` which simply removes the next
word in the list. It can be used to comment out pieces of an expression.

```
print !{not evaluated} ++3
```

The most common type of function in this family is one that uses the next word in the list as an
additional argument to modify its behavior. For example, the `map` function takes the next function
out of the expression, evaluates the remainder, and if it evaluates to a list, calls the function
on each member of that list.

```
# this will add 2 to every member in the list of numbers from 0 to 4
# prints [2, 3, 4, 5, 6]
print map (++) count 5
```

The `list` function takes a module as an argument and uses it to build a list with a member
for each expression.

```
# list builds a list with a member for each expression
#prints [1, 2, 5, 4]
print list (1, 2, ++, +) 3
```

The `try` function takes a module as an argument and runs each expression in order,
returning the first one to not fail.

```
/main.css => try (
	file {custom.css}
	file {fallback.css}
)
```

The choose function (`?`) takes a module as an argument and picks an expression to run
based on its input. If the input is a number, it will run the nth expression, if the input
is a boolean, it will run the first expression if true, and the second one if false

```
suffix = ? ({th}, {st}, {nd}, {rd}, {th}) modulo 10
print map {$*$suffix } count 10
```

More complex function patterns are possible as well, but their potential remains mostly
unexplored.

### Recursion

Functions in serv are allowed to reference themselves in their definition. Many "serious"
functional languages will brag about their ability to elegantly express recursive concepts
like the Fibonacci sequence in just a few lines of code, and despite it not being super
relevant for writing web servers, it became important to me to make sure serv could do
the same. Below are programs for calculating the Fibonacci sequence and factorials.

```
fib = ? (1, 1, sum list (fib-, fib--))
print map fib count 10

factorial = ? (1, product list ((), factorial-))
print map factorial count 10
```

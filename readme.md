# SERV!

Serv is a swiss army knife for writing http servers quickly and with as
little friction as possible. It is similar in function to web frameworks like Express
and Flask, but instead of existing as a library on top of a general purpose
language like Javascript or Python, Serv uses its own,
entirely custom, language implemented from the
ground up specifically for writing web backends.
By doing so, Serv
frees itself from almost all of the boilerplate required to build a web server
in the traditional way. It is a single executable with a fully featured web
server built directly into the runtime, a standard library full of useful
tools, and a dedicated syntax for assigning routes.

Here are some examples of Serv in action:

```python
# Hello World: (curly brackets denote strings)
/ => {Hello World!}

# Hello World with a more personal touch:
# (all strings are templates, reference variables with a $)
/hello/{name} => {Hello $name!}

# Statically serve all files in the directory
# (the 'file' function reads the contents of a file)
/{*f} => file f

# Serve a markdown file dynamically rendered to HTML
# (functions are composed by concatinating them)
/index => markdown file {www/index.md}

# Define your own functions:
# (
@square => %{ $in * $in }
/api/square/{x} => square x

# Write APIs endpoints directly in SQL
# Serv automatically sanitizes your input, and converts
# the output to JSON
@sql.database  => {my_database.sqlite}
/api/user/{id} => sql { SELECT * FROM users WHERE id = $id; }
```

Serv is a
[concatinative](https://en.wikipedia.org/wiki/Concatenative_programming_language)
functional language for writing web servers quickly and concisely.  It focuses
on minimizing boilerplate and optimizing the most commonly used operations
to be as syntactically concise as possible.

Serv was born out of a frustration with traditional web frameworks like
[express]() and [flask](), which aim to map the functionality of a web
server onto existing languages like javascript, python, and more recently
rust. These are very good at what they do, but are limited in a lot of ways
by the languages they are built on. I wanted to see what a web framework
could look like if it's language was built from the ground up specifically
for that purpose.

Serv offers the following advantages over traditional web frameworks:

- No boilerplate: The serv runtime already knows it is an HTTP server, so it will behave as one without needing to be told
- Built in templates: All strings in serv are a robust templating language. They use curly braces *{}* rather than quotes,
  can span multiple lines, and can call subexpressions with the *$* syntax.
- All values are a valid HTTP response: Every single value can be automatically converted into a valid and sensible HTTP response,
  lists and maps will be serialized to valid json automatically.

### Comparison to Express

Express offers the following example of a hello world program
on their website.

```javascript
const express = require('express')
const app = express()
const port = 3000

app.get('/', (req, res) => {
  res.send('Hello World!')
})

app.listen(port, () => {
  console.log(`Example app listening on port ${port}`)
})
```

In my opinion, 9 lines of code is too many for such a simple
task, and it is mostly boilerplate and noise. Serv can express
the same program in 3 lines:

```python
@port => 3000
@out  => {Example app listening on port $port}
/     => {Hello World!}
```

## Installation and Usage

Install serv using cargo:

```bash
cargo install --git https://github.com/connorwiniarczyk/serv.git
```

Test by starting a simple hello world program. Then
navigate to localhost:4000

```
serv -c "/ => hello"
```

Or print the result of a single expression

```bash
serv -ec "uppercase {hello}"
HELLO
```
## Syntax

```
/ => {hello world!}
```

A serv program is list of *definitions*. Every definition contains a *route*
followed by an *expression*. When the script receives
an http request, it looks for an expression at the requested route and runs
it to produce an output. In the above script, the root node is mapped to an
expression that produces the string "hello world!"

```
/hello/{name} => {hello $name!}
```

Serv uses the [matchit](https://github.com/ibraheemdev/matchit) crate
under the hood for its routing, and shares the matchit syntax for matching
patterns. If a pattern is matched, serv makes that available as a variable
in the corresponding expression. So here the request `/hello/connor` will
return the string `hello connor!`

```
/       => markdown file {home.md}
/{page} => markdown file page
```

Serv expressions are whitespace seperated lists of functions and
string templates.  Each function performs an operation on the rest of the
expression. In this example the `file` function reads the contents of a file
into a string, and the `markdown` function renders a markdown string into
html. Multiple serv functions can be composed by simply concatinating them
in this way.

Strings in serv are written as text in between pairs of curly brackets. Strings
are free to span multiple lines, and to include balanced pairs of nested
curly brackets.  This is done to make it as easy as possible to embed
snippets of other languages such as javascript and html into serv strings
without creating a visual mess.

Serv strings are allowed to contain any number of `$` follwed by
expressions. When the string is evaluated, the expression will be called and
its output placed into the text. Strings are allowed to access any function
or variable in this way.  In addition, strings can be treated like other
function by using the $in variable to refer to the computed value of the
expression to it's right.


### Recursion

Like any good functional language, serv is capable of using recursive
expressions to elegantly express certain computations. Although it is
generally outside of the language's scope, it was important to me that serv
be able to compute the fibonacci sequence in an aesthetically pleasing way.

```
# compute an element in the fibonacci sequence
@fib => switch (in) [1, 1, %{ $(fib decr) + $(fib decr decr) }]

# map the fibonacci function onto a list from 0 to 14
@out => map fib count 15

# prints to stdout:
# [1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610]
```

I think the result is an incredibly satisfying two lines of code, and is
an instructive use of some of serv's more advanced features:

switch is a function that takes an index, a list of functions, and an input,
and applies the function at the given index to the input.  In this case, the
index given is `in`, meaning the same value as the input, and the functions
are 1, 1, and the calculation `% { $(fib decr) + $(fib decr decr) }`. Switch
implicitly clamps the index to the length of the list.

% is a function that performs a mathematical calculation on the
string given. In this case, we are simply adding the result of two sub
expressions. decr is the decrement operator, so `$(fib decr)` calls fib with
the input subtracted by one, and `$(fib decr decr)` calls fib with the input
subtracted by two. Summing these produces the next element in the series.

map is a function that takes another function and a list, and calls that
function on each element in the list. Here, we are mapping the fib function
we just defined onto the result of count 15, which produces list of numbers
from 0 until the number given, and allows us to generate a full sequence.

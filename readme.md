# SERV!

Serv is a
[concatinative](https://en.wikipedia.org/wiki/Concatenative_programming_language)
functional language for writing web servers quickly and concisely.  It focuses
on minimizing boilerplate and making the most common operations for a web
server as easy to write as possible.

## Syntax

```
/ => {hello world!}
```

A serv program is list of *declarations*. Every declaration contains a *route*
followed by an *expression* that that route points to. When the script receives
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
perhaps an instructive use of some of serv's more advanced features:

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

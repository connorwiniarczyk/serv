# SERV!

Serv is a [concatinative](https://en.wikipedia.org/wiki/Concatenative_programming_language)
functional language for writing web servers quickly and concisely.
It focuses on minimizing boilerplate and making the most common operations for a web server as
easy to write as possible.

## Syntax

```
/ => {hello world!}
```

A serv program is list of *declarations*. Every declaration contains a *route* followed by an
*expression* that that route points to. When the script receives an http request, it looks
for an expression at the requested route and runs it to produce an output. In the above
script, the root node is mapped to an expression that produces the string "hello world!"

```
/hello/{name} => {hello $name!}
```

Serv uses the [matchit](https://github.com/ibraheemdev/matchit)
crate under the hood for its routing, and shares the matchit syntax
for matching patterns. If a pattern is matched, serv makes that
available as a variable in the corresponding expression. So
here the request `/hello/connor` will return the string `hello connor!`

```
/       => markdown file {home.md}
/{page} => markdown file page
```

Serv expressions are whitespace seperated lists of functions and string templates.
Each function performs an operation on the rest of the expression. In this example
the `file` function reads the contents of a file into a string, and the `markdown`
function renders a markdown string into html. Multiple serv functions can be composed
by simply concatinating them in this way.

Strings in serv are written as text in between pairs of curly brackets. Strings are
free to span multiple lines, and to include balanced pairs of nested curly brackets.
This is done to make it as easy as possible to embed snippets of other
languages such as javascript and html into serv strings without creating a visual
mess.

Serv strings are allowed to contain any number of `$` follwed by expressions. When
the string is evaluated, the expression will be called and its output placed into
the text. Strings are allowed to access any function or variable in this way.
In addition, strings can be treated like other

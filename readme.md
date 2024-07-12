# SERV!

Serv is a prefix [concatinative](https://en.wikipedia.org/wiki/Concatenative_programming_language)
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

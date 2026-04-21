# SERV

Serv is an experimental language for writing web servers.

It provides a standard library of "words" that represent common web server tasks,
such as reading a file, querying a database, rendering a template, and so on, and lets
you freely combine them into "phrases" to produce more complex behavior. Phrases
can be mapped directly onto routes to define the server's behavior, or onto new
words to build increasingly abstract operations. Everything else is handled by
the interpreter implicitly.

Here is a small example, it shows a web server with 3 routes, one for the root
which renders a template from a file, one which serves a file, and one which
serves a string literal. The custom function `visitor_counter`, is more complex,
combining the parse math (%), increment (+) and store operators to increase the
value in visitors.txt by one each time it is called.

```
/ => template file {index.template}
/styles.css => file {styles.css}

/script.js => {
    window.onload = function() {
        console.log("this website has been visited $visitor_counter times!");
    }
}

#increment and return the value of visitors.txt
visitor_counter => store {visitors.txt} + %file {visitors.txt}
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
or you can specify a port by defining the `server.port`.

### TLS encryption

Enabling TLS is as easy as assigning `server.certificate` and `server.private_key`
in your serv file.

```
server.port    = 443
server.certificate = file {certfile.pem}
server.private_key = file {keyfile.pem}

/ => {Encrypted Hello World!}
```

## Background

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
He talks specifically about the process of crafting a language for a specific domain
such that, if a problem seemed simple in your head, it would be simple to express
when you wrote the code. The example he gave was Awk for the domain of
data processing, but it struck me that web servers might deserve such a language too.

## Concepts

A serv expression is made up of a list of words concatenated together, where every
word is a function that operates on the rest of the expression. This is called
a *prefix call* notation, and it makes serv somewhat unusual among concatenative languages.
Expressions generally get evaluated right to left, for example, the following expression
applies 4 increment operators (+) to 0, and then prints the result.

```
print ++++ 0
```

A slightly more complex example might look like this, which creates a list of numbers from
1 to 10, maps the increment operator onto each one, sums them, and prints the result.

```
print sum map+ count 10
```

### Assigning Routes

Expressions can be assigned to routes with the `=>` or `=` operators (they are interchangeable).
Routes can be exact, such as `/index.html` or patterns of the form `/literal/{var1}/{var2}/{*rest}`.

See [https://github.com/ibraheemdev/matchit](https://github.com/ibraheemdev/matchit) for more details.

When the server receives a new request, it checks each of the routes that has been assigned,
and if one matches, it resolves the corresponding expression and responds with the result.
If the pattern contains a named parameter, it can be used as a word in the expression.
So for example, a static file server can be written with the following line:

```
/{*f} => file f
```

Expressions can also be assigned to identifiers to create new words. For example:

```
plus_two   = ++
plus_three = +++
minus_four = ----

# prints "2"
print minus_four plus_two plus_three 1
```

### Strings

A string is any piece of text enclosed by curly brackets `{}`. Strings are allowed to
span multiple lines, and can contain balanced pairs of nested curly brackets without
escaping. This makes it very easy to embed other languages inside of serv strings.

Every string is also a template, which can embed other words and expressions with
the `$` character.

```
print {hello world}

name = {connor}
print {hello $name}

# prints "hello CONNOR"
print {hello $(uppercase name)}
```

### Metaprogramming

Because words in serv are functions that operate on the remainder of the expression,
every word can be a metaprogram. There is an implicit function, called resolve,
which turns each sub expression into a value that can be operated on. Most functions work
by resolving the remaining expression immediately, and then operating on the result,
but they are free to do other things with the expression as well.

A good example of a metaprogram is the list function (`|`), rather than resolve the
remainder of the expression, it constructs a list out of each remaining word.

```
# prints "18"
print sum | 5 6 7
```

Many functions take 2 or more arguments instead of one. They can do this by popping the
next word out of the expression and using that as their argument. A good example of this
is the `map` function, which applies its first argument as a function onto every
element in a list.

```
# prints [3, 4, 5]
print map (++) | 1 2 3
```

---

```python

# Write API endpoints directly in SQL
# serv automatically sanitizes your input, and converts the output to JSON
sqlite.connect = {:memory:}
/api/{user}/posts => sqlite.query { SELECT (title, content) FROM posts WHERE user = $user; }

# Compute the Fibonnaci sequence
fib = ? (1, 1, sum list (fib-, fib--))
print map fib count 10
```

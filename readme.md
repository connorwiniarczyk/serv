# Serv

Serv is a web server written in Rust. There are a lot of web servers written
in Rust, but this one is mine. My goal for it is to have an HTTP server that
expresses routes as concisely as possible, while still being flexible enough
to produce complex APIs with arbitrary behavior.

Each route is defined with a line in a file called `routes`. `routes` is a
whitespace separated values file where the first column represents potential
HTTP requests, the second column represents the corresponding resource on the
host system, and the remaining columns are a list of zero or more options.

Serv derives its flexibility from these options, which give the user much more
control over the behavior of individual routes than would be possible in a more
traditional static file server. Of particular importance is the `exec` option,
which tells serv to treat the specified file as a program and execute it,
returning its output, instead of just its contents. The `exec` function also
allows you to specify different parts of the http request that can be passed
to the program as arguments, allowing for APIs with very complex behavior to
be written in fewer lines of code than I've seen in any other http framework.

Exec also makes serv API's language agnostic. Endpoints can be written in bash,
javascript, python, c, rust, lisp, fortran, or any other language that you can
find a compiler or interpreter for.

## Installation / Usage

You'll need rust installed on your system in order to install serv. It can be 
installed from [here](https://rustup.rs/) and by then running `rustup default nightly`
and `rustup update`.

Most of the examples will have dependencies on external programs. In the case
of the CMS example, you will need `pandoc` installed in order to render 
markdown files.

```bash
# install rust and cargo, tell cargo to use the nightly version of the compiler
rustup default nightly
rustup update

# install serv using cargo
cargo install --git https://github.com/connorwiniarczyk/serv.git

# run an example
git clone https://github.com/connorwiniarczyk/serv.git
cd serv/examples/cms

# serv takes a port argument (the default is 4000) and path to a directory.
# The directory must contain a valid routes file
serv -p 4000 .

# test the server
curl localhost:4000
```

## Options

The first option in the list is special and is referred to as the access type.
There are two access type:

- `read` : read the file directly and serve the contents as is 
- `exec` : attempt to execute the file and return the output generated

If the option in the list is neither of these, than serv will automatically
insert the `read` access type, so it is only necessary to specify when you
intend on executing the file.

Options that are not the access type are called post processors. I have two
implemented as a proof of concept:

- `header` : add a list of http headers to the response
- `cors`   : add specific CORS related headers to the response

### Options with Arguments

Options can take arguments, which can sometimes have values. The syntax for
this is as follows `<option>(<arg1>:<value1> <arg2>:<value2>)`. So for example
all of the following are valid options:
```
exec exec() exec(query) exec(query:key) exec(query:key query:key2)
```

The valid arguments for each option are listed below below:

`exec` : arguments to exec map parts of the http query to arguments that will
         get passed to the program being executed. The first argument will
         become $1, the second $2, etc.

- `query:<key>`  : Get the part of the http query string with the given key
- `wild:<index>` : Get the part matched wildcard at the given index

`header` : arguments to header become the key and value pairs of http headers
           in the response


## Path Expressions

Both the request and resource paths can contain wildcards. ie. `/one/*/two`.
In the request path, this indicates that it will match any request with the
same structure, so `/one/one/two`, `/one/any/two`, etc. but not `/two/any/one`.

In the resource path, wildcards will be filled in by the corresponding
wildcards in the request path. For example, the route
`/styles/*     css/*    read`
will route the request `/styles/main.css` to `css/main.css`.



## Routes Example 

```
# Routes example
# <request path> <resource path> <options>

# Normal Stuff
/               index.html              header(content-type:text/html)
/css/*          public/styles/*         header(content-type:text/css)
/js/*           public/scripts/*        header(content-type:text/javascript)

# Images/Media
/splash         media/background.jpg    read # the read option can be stated explicitly or ignored
/images/*       media/images/*/large    

# API
/api/date       api/get_date.sh         exec
/api/register   api/register_user.py    exec(query:username, query:password)
/user/*/info    api/get_user_info.py    exec(wild:0) # using part of the path as an argument

# Rendered Content
/content/*      render_markdown.sh      exec(wild:0)

```

## To Do / Known Issues:

- I would like some kind of `pipe` option that would let me pipe the body of a response into another program. You can get around this by using `exec` on a shell script that does the piping for you, but I think there are instances where it would be cleaner to include all of the logic in the routes file.

- `exec` needs an argument that just accepts a constant string and passes that as an arg. It also needs access to the rest of the http request for stuff like post bodies, the path, etc.

- A way for executables to dynamically change the response header and status code. I'm thinking an option that strips the first line from the body and parses that into information about the headers and/or status code

- Better debug information for stuff like explaining whether/why a route is valid or not or when requests fail.

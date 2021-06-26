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

Exec also makes serv API's language agnostic. Endpoints can be written in bash
for maximum convenience, javascript or python if they require complex logic,
C if it is important that they run fast, etc.

## Usage

You'll need rust installed on your system in order to install serv. It can be 
installed from [here](https://rustup.rs/) and by then running

```bash
rustup default nightly
rustup update
```

```bash
# install using cargo
cargo install --git https://github.com/connorwiniarczyk/serv.git

# run an example
git clone https://github.com/connorwiniarczyk/serv.git
cd serv/examples/content-management-system
serv -p 3000 .

# test the server
curl localhost:3000
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
this is as follows `<option>(<arg1>:<value1> <arg2>:<value2>)`. So for example:
```
exec exec() exec(query) exec(query:key) exec(query:key query:key2)
```
are all valid options. 

The list of valid arguments for each option is outlined below:

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
/               index.html              read
/css/*          public/styles/*         read
/js/*           public/scripts/*        read

# Images/Media
/splash         media/background.jpg    read
/images/*       media/images/*/large    read

# API
/api/date       api/get_date.sh         exec
/api/register   api/register_user.py    exec(query:username, query:password)
/user/*/info    api/get_user_info.py    exec(wild:0) # using part of the path as an argument

# Rendered Content
/content/*      render_markdown.sh      exec(wild:0)

```


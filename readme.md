# Serv

Serv is a web server written in Rust. There are a lot of web servers written
in Rust, but this one is mine. It is configured at runtime using a domain
specific language that tries to express HTTP routes concisely and with as little
boilerplate as possible. Right now it's meant mainly for rapid prototyping and
small projects that don't really matter, but I'd like to keep improving it until
it can be used in more and more important applications.

## Configuration

Serv reads its configuration from a file called `routes.conf` in whatever directory
it is run in. Each entry in this file maps one or more potential HTTP requests
onto a series of commands that are executed to generage the response. Commands
do things like read the contents of a file, execute an external program, set an
http header, etc.

```sh
# Example routes.conf file
# This is a relatively common web server that serves a homepage, some additional
# content, stylesheets and media, and has a small API that the user can interact
# with 

/: 	                read index.html
/content/*page:		read pages/$(page).html
/styles/*stylesheet:    read css/$(stylesheet)
/images/*image: 	read media/$(image)

/api/now:		exec date
/api/fun/now:           shell date | cowsay
/api/login:             exec ./login.sh $(query:username) $(query:password)
```

### Variables

Information about the HTTP request are available to the commands as variables.
These can be accessed using a `$(<key>)` syntax (similar to in a Makefile),
which will be substituted for whatever is stored in that variable at the time
the command is executed. Wildcards in the request pattern will appear as
variables, as will entries in the URL query and the contents of the request
body. It is also possible for commands to set variables to share information
with the commands that might follow them.

### Commands

- `echo <value>` : Appends the given value to the body
- `read/file <filename>` :  Reads a file and append its contents to the body
- `exec <program> <args>` : Execute a program with the given arguments and append its output to the body
- `shell/sh <input>` : Pipes the input into /bin/sh and appends the output to the body

- `header <key> <value>` : Set the value of an HTTP header
- `filetype/type <type>` : Set the MIME type of the response (shorthand for `header content-type <type>`)

- `set <key> <var>` : Set the value of a variable
- `debug` : Print the current state of the route (headers, body, status, variables) to stdout

## Installation

You'll need rust installed on your system in order to install serv. It can be 
installed from [here](https://rustup.rs/) and by then running `rustup default nightly`
and `rustup update`.

Most of the examples will have dependencies on external programs. In the case
of the Blog example, you will need `pandoc` installed in order to render 
markdown files, as well as `cowsay` for the date api.

```bash
# install rust and cargo, tell cargo to use the nightly version of the compiler
rustup default nightly
rustup update

# install serv using cargo
cargo install --git https://github.com/connorwiniarczyk/serv.git

# run an example
git clone https://github.com/connorwiniarczyk/serv.git
cd serv/examples/blog

# serv takes a port argument (the default is 4000) and path to a directory.
# If the directory does not contain a valid routes.conf file, a default one
# will be automatically generated
serv -p 4000

# test the server
curl localhost:4000/api/date

# or open it in a browser
```

## To Do / Known Issues:

- I would like some kind of `pipe` option that would let me pipe the body of a response into another program. You can get around this by using `exec` on a shell    script that does the piping for you, but I think there are instances where it would be cleaner to include all of the logic in the routes file.

- A way for executables to dynamically change the response header and status code. I'm thinking an option that strips the first line from the body and parses that   into information about the headers and/or status code

- Better debug information for stuff like explaining whether/why a route is valid or not or when requests fail.
  
- I would love to get streams working. This would make it very easy to set up your own streaming server by integrating with something like ffmpeg


### Ideas for More Commands

Just brainstorming

- `log(file)` : write information about the request to a file
- `hook(program, args)` : execute a given 'hook' script. Would be a superset of log, but the distinction might be semantically useful
- `status(code)` : change the status code
- `striplines(range)` : remove a set of lines from the body

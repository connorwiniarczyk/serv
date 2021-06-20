## Serv

Serv is a web server written in Rust. There are a lot of web servers written
in Rust, but this one is mine. It's goal is to be extremely easy to use and
flexible for rapid prototyping of websites and apis.

It assigns routes by reading from a file called `routes` in the target
directory. `routes` is a whitespace separated values file with each line
containing an http request, followed by a path to a resource on the host
system, followed by a list of options.

Both the request and resource paths can contain wildcards. ie. `/one/*/two`.
In the request path, this indicates that it will match any request with the
same structure, so `/one/one/two`, `/one/any/two`, etc. but not `/two/any/one`.

In the resource path, wildcards will be filled in by the corresponding
wildcards in the request path. For example, the route
`/styles/*     css/*    read`
will route the request `/styles/main.css` to `css/main.css`.

Options can be either `read`, or `exec` 

- `read` : read the file directly and serve the contents as is 
- `exec` : attempt to execute the file and return the text generated

The `exec` option can take additional arguments, with the syntax
`exec(param1:value1, param2:value2, etc.)` which can be used to send
information about the http request to the program being executed. I currently
have two parameters implemented:

- `query:key` : pass the part of the url query with key `key` as an argument
- `wild:index` : send the wildcard at index `index` as an argument


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


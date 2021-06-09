## Serv

Serv is a web server written in Rust. There are a lot of web servers written
in Rust, but this one is mine. It's goal is to be extremely easy to use and
flexible for rapid prototyping of websites and apis.

It assigns routes by reading from a file called `routes` in the target
directory. `routes` is a whitespace separated values file with each line
containing an http request, followed by a path to a resource on the host
system, followed by a preprocessor.

Preprocessors can be one of:

- f (file): serve this as is
- x (exec): attempt to execute this file and return whatever the result is

With the idea that this list could be added to as needed. I eventually want to
add preprocessors for Markdown and Handlebars rendering, serving full
directories, image compression, etc.

```
# Routes example
# <http path> <local path> <preprocessor>

/               index.html              f
/style.css      stylesheets/style.css   f
/main.js        scripts/main.js         f

/splash         media/background.jpg    f

/api/date       api/get_date.sh         x
/api/register   api/register_user.py    x
```


# This is Markdown

A shell script is piping this through Pandoc before serving it as HTML and I
think that's pretty neat. You could probably write a pretty cool blog or
something and spend very little effort to maintain it.

This is the code that's being used to render this markdown right now. It's
only three lines of bash script. Serv knows from the routes file to pass the 
part of the path after `/content/` as the first argument to this script, which
passes as an argument into pandoc before surrounding it with the values of
`head.html` and `tail.html`.

```bash
#!/bin/sh
cat renderer/head.html
pandoc content/$1.md
cat renderer/tail.html
```

This is the line in the `routes` file that defines this route.
The `exec` option tells serv to execute render.sh instead of just reading it,
and the `wild:0` argument tells it to use the first matched wildcard as an
argument.

```bash
/content/*  renderer/render.sh  exec(wild:0) header(content-type:text/html)
```

I think this could make a really nice content management system for a blog or
personal website. Below are some images. The rest of the paragraph will be
dummy text.
Lorem ipsum dolor sit amet, consectetur adipiscing elit. In tincidunt sodales vestibulum. Nulla facilisi. Duis porta risus eu arcu cursus rutrum. Integer ut orci sed nibh faucibus tincidunt vitae et augue. Maecenas maximus vehicula quam, sollicitudin semper felis aliquam sit amet. Aliquam nisi orci, fringilla vitae tortor sed, sollicitudin sagittis neque. Sed tempor mattis nisl ac malesuada. Morbi id justo quis libero sagittis viverra vitae quis dolor. Vivamus eget tincidunt risus.

Mauris at pellentesque quam. Integer non tellus non ligula vestibulum tincidunt eget tempor nunc. In vel magna vitae quam iaculis hendrerit at a est. Vestibulum nec sodales ante, id consectetur ante. Ut ac arcu ac dui lobortis dapibus et ut nunc. Phasellus ac facilisis sem, eget vestibulum eros. Suspendisse egestas eleifend nunc et bibendum. Etiam ullamcorper consequat convallis. Quisque ultrices sollicitudin augue ut porta. 

Here are some images. If this was a real blog I'd bet you'd be really engaged

![](https://i.huffpost.com/gen/1271717/thumbs/o-LONDON-MAP-570.jpg?3)
![](/images/paris.jpg)
![](https://i.huffpost.com/gen/1271776/thumbs/o-LA-MAP-570.jpg?1)

Lorem ipsum dolor sit amet, consectetur adipiscing elit. In tincidunt sodales vestibulum. Nulla facilisi. Duis porta risus eu arcu cursus rutrum. Integer ut orci sed nibh faucibus tincidunt vitae et augue. Maecenas maximus vehicula quam, sollicitudin semper felis aliquam sit amet. Aliquam nisi orci, fringilla vitae tortor sed, sollicitudin sagittis neque. Sed tempor mattis nisl ac malesuada. Morbi id justo quis libero sagittis viverra vitae quis dolor. Vivamus eget tincidunt risus.

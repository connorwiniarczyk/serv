# This is Markdown


### Pages

- [index](/)
- [page two](/content/two)
- [what's the date?](/api/date)

This page is being rendered on the fly from content stored in a markdown file.
The server pipes the contents of the markdown file through pandoc to render it
as html before appending a header and footer that are common to every page.
You could probably write a pretty cool blog this way.

This is the routes.conf file used to generate this example. Each page of
content uses a combination of the read command for the header and footer, and
the shell (sh) command to render the content page with pandoc. Note that the
order of these commands matters as it will determine the order that their
output appears in the final response. The type command simply adds the
'content-type:text/html' http header to the response.

```bash
/: read head.html; sh pandoc content/index.md; read tail.html; type text/html
/content/*page:	read head.html; sh pandoc content/$(path:page).md; read tail.html; type text/html

/styles/*stylesheet: file css/$(path:stylesheet); type text/css; 
/images/*image: file media/$(path:image); type text/css; 

/api/date: sh date | cowsay; type text/plain
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

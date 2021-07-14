# Example: Blog

```bash
# routes.conf
/content/*  ./render.sh	  exec(wild:0) ft(html)
/           pandoc 	   read("head.html") exec("content/index.md") read("tail.html") ft(html)

/styles/*   css/*          	   ft(css)
/images/*   media/*  		   ft(image/jpg)


# Tells you what the date is
/api/date   ./cowsay-date.sh	exec
```

# Example: Reverse Proxy

```bash
# By using curl as a resource, serv can very easily be configured to run as a
# reverse proxy or load balancer

/google        curl     exec("www.google.com") cors
/wallpaper     curl     exec("https://wall.alphacoders.com/api2.0/get.php?method=wallpaper_info&id=865098") ft(application/json) cors
/apotd         curl     exec("https://api.nasa.gov/planetary/apod?api_key=DEMO_KEY&date=2019-03-17") cors ft(application/json)

# A dirty hack that will remove CORS restrictions from any URL
/cors-hack     curl     exec(query:url) cors
```

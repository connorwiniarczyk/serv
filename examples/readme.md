## Example: Blog


```bash
/:
	read head.html
	shell pandoc content/index.md
	read tail.html

/content/*page:
	read head.html
	shell pandoc content/$(page).md
	read tail.html
/self: read routes.conf
/api/date:
	exec date
	exec cowsay
/**path : read $(path)

```

# Example: Reverse Proxy

```bash
# By using curl as a resource, serv can very easily be configured to run as a
# reverse proxy or load balancer

/**path: exec curl http://server.com/$(path)

```

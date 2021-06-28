#!/bin/sh
# render index.md for the / route. This is a workaround for the fact that the
# exec() option does not yet accept a raw argument

./renderer/render.sh index

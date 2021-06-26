#!/bin/sh
cat renderer/head.html
pandoc content/$1.md
cat renderer/tail.html

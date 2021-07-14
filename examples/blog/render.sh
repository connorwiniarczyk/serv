#!/bin/sh
cat head.html
pandoc content/$1.md
cat tail.html

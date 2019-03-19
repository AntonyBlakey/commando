#!/usr/bin/env python

from subprocess import check_output
from libqtile.command import Client
import sys

here = sys.path[0]
output = check_output([here + "/../target/debug/commando", "select", here + "/select.css", here + "/root.json5"])
if len(output) > 0:
    Client().window[int(output)].focus()
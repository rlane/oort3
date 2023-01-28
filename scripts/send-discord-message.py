#!/usr/bin/env python3
import json
import requests
import sys

def main():
    url = sys.argv[1]
    message = sys.stdin.read()
    data = { "content": message }
    requests.post(url, json=data)

if __name__ == "__main__":
    main()

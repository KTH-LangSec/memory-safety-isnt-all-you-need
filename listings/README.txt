Tested on:

- .html: Firefox 154.0 (64-bit)
- .java: Eclipse Temurin 17.0.19
- .js: Node.js 22.22.1
- .py: Python 3.14.4

To run the Java listing:

    java listing-1.java

To run the JavaScript listings:

    node listing-<N>.js

To run the Python listing:

    python3 listing-3.py

To run the HTML listings:

    open listing-<N>.html

You can use the Dockerfile to create a container to run the (non-web) listings:

    docker build -t listings .
    docker run -it --rm -v .:/listings -w /listings listings

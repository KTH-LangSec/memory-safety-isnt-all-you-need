To run the case study:

    go test ./example

You can use a container to run the case study:

    docker run --rm -it -v .:/case-study -w /case-study golang:1.24.0-alpine sh

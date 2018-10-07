# pid1-rust

Build it:

    $ rust-musl-builder cargo build --release

Dockerize it:

    $ docker image build . -t fpco/pid1-rust

## Example

Interrupting `sleep`:

    chris@precision:~$ docker run --rm --name sleeper fpco/pid1-rust sleep 10
    Sleeping for 10...
    Running as pid1 ...
    ^CProcess was signalled to end with signal: INT
    chris@precision:~$

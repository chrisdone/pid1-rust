FROM alpine@sha256:621c2f39f8133acb8e64023a94dbdf0d5ca81896102b9e57c0dc184cadaf5528

COPY target/x86_64-unknown-linux-musl/release/pid1-rust /sbin/pid1-rust

ENTRYPOINT ["/sbin/pid1-rust", "--RTS"]

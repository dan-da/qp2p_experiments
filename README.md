# qp2p_experiments

## About

These are some simple experiments and test cases using the [qp2p](https://github.com/maidsafe/qp2p) library.

## Usage

### node_bi

node A
```
$ cargo run --bin node_bi create
Process running at: 127.0.0.1:51408
Waiting for connections
```

node B

```
$ cargo run --bin node_bi connect 127.0.0.1:51408
```

### node_uni

note: These nodes will exchange packets for a while but eventually halt without any error, due to unknown issue/bug.


node A
```
$ cargo run --bin node_bi create
Listening for messages on 127.0.0.1:10000
```

node B

```
$ cargo run --bin node_bi
```


### node_uni_alt

note: This code will error out immediately due to [issue 205](https://github.com/maidsafe/qp2p/issues/205).

node A

```
$ cargo run --bin node_bi create
Listening for messages on 127.0.0.1:10000
```

node B

```
$ cargo run --bin node_bi
```


# hashlife

Rust implementation of [HashLife](https://en.wikipedia.org/wiki/Hashlife). This
project includes an implementation of HashLife and a front-end powered by WebAssembly.

# Dependencies

The front-end requires Nodejs and NPM.

To build this project with ease, you'll need [cargo-make](https://github.com/sagiegurari/cargo-make).

# Build

First run

```shell
cargo make setup
```

to install dependencies for the front-end part. Then run

```shell
cargo make build
```

to build the project.

# References

- [An Algorithm for Compressing Space and Time](https://github.com/mafm/HashLife)
- <https://tomas.rokicki.com/hlife/>
- <https://github.com/rokicki/lifealg>
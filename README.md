# cldr-musl-extractor

Generates a musl locale extracting the code from CLDR data.

## Running 

**Note: Right now it will panic since it still doesn't have the information to get some required fields.**

You will need Rust, follow the instructions [here](https://rust-lang.org/es/learn/get-started/) to install it.

```shell
git clone https://github.com/sheosi/cldr-musl-extractor
cd cldr-musl-extractor
cargo run -- <locale-id>

#Example:
cargo run -- es-ES
```

## Current state

Most of the basic plumbing is there: parsing the target locale as language and region, formatting the output and the code infrastructure to get the files, and parse them. What's missing is actually extracting from those files all the data needed.

## License

Licensed under either of

- [Apache License v2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

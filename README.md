# readability-rs

readability-rs is a library for extracting the primary readable content of a webpage.
This is a rust port of arc90's readability project. Inspired by
[kingwkb/readability](https://github.com/kingwkb/readability), forked from [kumabook/readability](https://github.com/kumabook/readability).

## Hot to use

- Add `readability-rs` to dependencies in Cargo.toml

```toml
[dependencies]
readability-rs = "^0"
```

- Then, use it as below

```rust

extern crate readability_rs;
use readability_rs::extractor;

fn main() {
  match extractor::scrape("https://spincoaster.com/chromeo-juice") {
      Ok(product) => {
          println!("------- html ------");
          println!("{}", product.content);
          println!("---- plain text ---");
          println!("{}", product.text);
      },
      Err(_) => println!("error occured"),
  }
}

```

## Related Projects

- [ar90-readability ports](https://github.com/masukomi/ar90-readability#ports)

## License

[MIT](LICENSE)

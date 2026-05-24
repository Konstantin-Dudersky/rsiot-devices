```sh
cargo build --example DS3231 --target="armv7-unknown-linux-gnueabihf" --release --features "time"; scp target/armv7-unknown-linux-gnueabihf/release/examples/DS3231 root@target:/root
```


Прочитать все регистры

```sh
i2ctransfer -y 1 w1@0x68 0x00 r19@0x68
```

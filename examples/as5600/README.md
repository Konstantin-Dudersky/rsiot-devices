```sh
cargo build --example as5600 --target="armv7-unknown-linux-gnueabihf" --release; scp target/armv7-unknown-linux-gnueabihf/release/examples/as5600 root@target:/root
```


Прочитать STATUS - регистр состояния

```sh
i2ctransfer -y 0 w1@0x36 0x0B r1@0x36
```

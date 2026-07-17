```sh
cargo build --example MPU6050 --target="armv7-unknown-linux-gnueabihf" --release; scp target/armv7-unknown-linux-gnueabihf/release/examples/MPU6050 root@target:/root
```

# Полезные запросы

- Чтение всех регистров

```sh
i2cdump -y 0 0x68
```

- Перевести в рабочий режим

```sh
i2ctransfer -y 0 w2@0x68 0x6b 0x00
```

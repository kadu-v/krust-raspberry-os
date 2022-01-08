# krust-raspberry-os




```
$ cargo build
```

```
$ cargo run
```

```
$ cargo objdump --bin kernel -- --disassemble --demangle --section .text --section .rodata --section .got  | rustfilt
```

```
$ cargo readobj --bin kernel -- --headers
```
# Виртуальная машина

_Выполнил Рябков Дмитрий_

## Сборка и запуск

Для сборки проекта необходимо иметь компилятор языка Rust.

Команда для сборки:
```
cargo build --release
```

Команда для запуска виртуальной машины:
```
target/release/vm <FILE>
```

Команда для запуска инспектора исполняемых файлов:
```
target/release/inspect <FILE>
```

Команда для запуска ассемблера:
```
target/release/asm <SOURCE> <OUTPUT>
```

## Ссылки

* [Описание инструкций](docs/instructions.md)
* [Описание ассемблера](docs/assembler.md)
* [Описание формата исполняемых файлов](docs/binfile.md)
* [Hello, world!](examples/hello-world)
* [Фибоначчи (цикл)](examples/fib-loop)
* [Фибоначчи (рекурсия)](examples/fib-rec)
* [Printf](examples/printf)

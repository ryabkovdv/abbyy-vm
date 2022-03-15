# Описание ассемблера

## Соглашение вызовов

| Имя регистра | ABI имя | Описание                        |
|--------------|---------|---------------------------------|
| `x0`         | `zero`  | Всегда имеет значение 0.        |
| `x1`         | `lr`    | Адрес возврата.                 |
| `x2`         | `sp`    | Указатель на стек.              |
| `x3-x8`      | `a0-a5` | Аргументы и результаты функций. |
| `x9-x15`     | `s0-s6` | Callee-saved.                   |

Стек растет вниз. Первые 6 аргументов функции передаются в регистрах `a0-a5`,
а остальные – на стеке в обратном порядке. Результаты функции также передаются
через регистры `a0-a5`.

## Формат исходного кода

Каждая строка исходного кода на ассемблере имеет следующий вид:

```
[ label: ] [ instruction arguments ] [ ; comment ]
```

Имена меток и инструкций являются идентификаторами. В составе идентификаторов
могут быть латинские буквы, цифры, `_` и `.`, причем первый символ не может
быть цифрой. Между именем метки и двоеточием не может быть пробелов.

Аргументами инструкций могут быть регистры (`%zero`, `%sp`, `%x15`),
строки, заключенные в кавычки (`"Some string"`), и целочисленные выражения.
Аргументы разделяются запятыми.

Инструкция вида `symbol =` присваивает символу `symbol` значение аргумента.
Между именем символа и знаком `=` может быть произвольное число пробелов.

## Грамматика выражений

```
Expression = PrimaryExpression
           | Expression "+" Expression
           | Expression "-" Expression
           | Expression "*" Expression
           | Expression "&" Expression
           | Expression "|" Expression
           | Expression "^" Expression
           | Expression "<<" Expression
           | Expression ">>" Expression
           | Expression ">>>" Expression

PrimaryExpression = integer literal
                  | character literal
                  | identifier
                  | "(" Expression ")"
                  | "+" PrimaryExpression
                  | "-" PrimaryExpression
                  | "^" PrimaryExpression
```

## Таблица псевдоинструкций

| Инструкция |                                Примечание                                |
|------------|--------------------------------------------------------------------------|
| `mem expr` | Задает количество памяти, доступной программе.                           |
| `seg expr` | Начинает новый сегмент по указанному адресу.                             |
| `d8  arg+` | Объявляет 8-битные данные. Может принимать строки в качестве аргументов. |
| `d16 arg+` | Объявляет 16-битные данные.                                              |
| `d32 arg+` | Объявляет 32-битные данные.                                              |

## Таблица инструкций

|           Инструкция            |              Примечание               |
|---------------------------------|---------------------------------------|
| `lui    %rd, expr`              |                                       |
| `sysfn  %r, expr`               |                                       |
| `st.u8  %rs, %rb, expr`         |                                       |
| `st.u16 %rs, %rb, expr`         |                                       |
| `st.s8  %rs, %rb, expr`         | Эквивалентно `st.u8  %rs, %rb, expr`. |
| `st.s16 %rs, %rb, expr`         | Эквивалентно `st.u16 %rs, %rb, expr`. |
| `st     %rs, %rb, expr`         |                                       |
| `ld.s8  %rd, %rb, expr`         |                                       |
| `ld.u8  %rd, %rb, expr`         |                                       |
| `ld.s16 %rd, %rb, expr`         |                                       |
| `ld.u16 %rd, %rb, expr`         |                                       |
| `ld     %rd, %rb, expr`         |                                       |
| `jal    %rd, expr`              |                                       |
| `jalr   %rd, %rs, expr`         |                                       |
| `jmp    expr`                   | Эквивалентно `jal  %zero, expr`.      |
| `call   expr`                   | Эквивалентно `jal  %lr, expr`.        |
| `ret`                           | Эквивалентно `jalr %zero, %lr, 0`.    |
| `beq    %rs1, %rs2, expr`       |                                       |
| `bne    %rs1, %rs2, expr`       |                                       |
| `blt    %rs1, %rs2, expr`       |                                       |
| `bge    %rs1, %rs2, expr`       |                                       |
| `bltu   %rs1, %rs2, expr`       |                                       |
| `bgeu   %rs1, %rs2, expr`       |                                       |
| `bgt    %rs1, %rs2, expr`       | Эквивалентно `blt  %rs2, %rs1, expr`. |
| `ble    %rs1, %rs2, expr`       | Эквивалентно `bge  %rs2, %rs1, expr`. |
| `bgtu   %rs1, %rs2, expr`       | Эквивалентно `bltu %rs2, %rs1, expr`. |
| `bleu   %rs1, %rs2, expr`       | Эквивалентно `bgeu %rs2, %rs1, expr`. |
| `mov    %rd, %rs`               | Эквивалентно `addi %rd, %rs, 0`.      |
| `addi   %rd, %rs, expr`         |                                       |
| `rsubi  %rd, %rs, expr`         |                                       |
| `muli   %rd, %rs, expr`         |                                       |
| `andi   %rd, %rs, expr`         |                                       |
| `ori    %rd, %rs, expr`         |                                       |
| `xori   %rd, %rs, expr`         |                                       |
| `shli   %rd, %rs, expr`         |                                       |
| `lshri  %rd, %rs, expr`         |                                       |
| `ashri  %rd, %rs, expr`         |                                       |
| `add    %rd, %rs1, %rs2`        |                                       |
| `sub    %rd, %rs1, %rs2`        |                                       |
| `mul    %rd, %rs1, %rs2`        |                                       |
| `and    %rd, %rs1, %rs2`        |                                       |
| `or     %rd, %rs1, %rs2`        |                                       |
| `xor    %rd, %rs1, %rs2`        |                                       |
| `shl    %rd, %rs1, %rs2`        |                                       |
| `lshr   %rd, %rs1, %rs2`        |                                       |
| `ashr   %rd, %rs1, %rs2`        |                                       |
| `mulw   %rd1, %rd2, %rs1, %rs2` |                                       |
| `mulwu  %rd1, %rd2, %rs1, %rs2` |                                       |
| `div    %rd1, %rd2, %rs1, %rs2` |                                       |
| `divu   %rd1, %rd2, %rs1, %rs2` |                                       |

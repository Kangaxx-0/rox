
# Chapter of Chunks of Bytecodes

## What is `uint8_t`?
It is shorthand for: a type of unsigned integer of length 8 bits. With `<stdint.h>`, this new header defines a set of cross-platform types that can be used when you need an exact amount of bits,
with or without the sign.

## Why I use `prt::write`
Since we need a contiguous block of memory for a vec, so be careful for raw pointer [`write`](https://doc.rust-lang.org/std/primitive.pointer.html#method.write) function because it overwrites a memory location, [`ptr::write`](https://doc.rust-lang.org/std/ptr/fn.write.html) expects a dst.

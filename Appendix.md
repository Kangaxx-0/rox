
# Chapter of Chunks of Bytecodes

## What is `uint8_t`?
It is shorthand for: a type of unsigned integer of length 8 bits. With `<stdint.h>`, this new header defines a set of cross-platform types that can be used when you need an exact amount of bits,
with or without the sign.

## Why I use `prt::write`
Since we need a contiguous block of memory for a vec, so be careful for raw pointer [`write`](https://doc.rust-lang.org/std/primitive.pointer.html#method.write) function because it overwrites a memory location, [`ptr::write`](https://doc.rust-lang.org/std/ptr/fn.write.html) expects a dst.

## The Constant Pool

Java Virtual Machine instructions do not rely on the run-time layout of classes, interfaces, class instances, or arrays. Instead, instructions refer to symbolic information in the `constant_pool` table.
All `constant_pool` table entries have the following general format:

```
cp_info {
    u1 tag;
    u1 info[];
}
```

| Constant Kind   | Tag  |
------------------|------|
|CONSTANT_Utf8    |   1  |
|CONSTANT_Integer |   3  |
|CONSTANT_Float   |   4  |
|CONSTANT_Long    |   5  |
|CONSTANT_Double  |   6  |
|CONSTANT_Class   |   7  |
|CONSTANT_String  |   8  |
|CONSTANT_Fieldref|   9  |
|CONSTANT_Methodref|   10  |
|CONSTANT_InterfaceMethodRef|   11  |
|CONSTANT_NameAndType|   12  |
|CONSTANT_MethodHandle|   15  |
|CONSTANT_MethodType|   16  |
|CONSTANT_Dynamic|   17  |
|CONSTANT_InvokeDynamic|   18  |
|CONSTANT_Module    |   19  |
|CONSTANT_Package    |   20  |

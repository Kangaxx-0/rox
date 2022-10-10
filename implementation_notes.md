
# 1.字节码块章节
> # Chapter of Chunks of Bytecodes

## Vec
> ## Vec
我们向量的实现`Lec`是基于`Unsafe`，rustonomicon有更多的细节
> Our vector implementation `Lec` is leveraging the power of `Unsafe`, rustonomicon has more details

## `uint8_t`是什么?
> ## What is `uint8_t`?
`uint8_t`是无符号1个字节的整型，它一般被用来处理跨平台上需要同样比特的无符号或有符号类型
> It is shorthand for: a type of unsigned integer of length 8 bits. With `<stdint.h>`, this new header defines a set of cross-platform types that can be used when you need an exact amount of bits, with or without the sign.

## 为什么是`prt::write`
> ## Why I use `prt::write`
我们的向量需要连续的内存块，所以要必须小心原始指针的[`write`](https://doc.rust-lang.org/std/primitive.pointer.html#method.write) 方法覆写内存地址, [`ptr::write`](https://doc.rust-lang.org/std/ptr/fn.write.html) 则期望一个目标地址
> Since we need a contiguous block of memory for a vec, so be careful for raw pointer [`write`](https://doc.rust-lang.org/std/primitive.pointer.html#method.write) function because it overwrites a memory location, [`ptr::write`](https://doc.rust-lang.org/std/ptr/fn.write.html) expects a dst.

## 出栈都发生了
> ## What happens when stack pops value
我们实现解析器进行出栈操作的时候，我们会保留内存的值，这样做是因为在Rust中清理值会操作内存为初始化。
> In our Pasrer stack, we save the value in momory when moveing the point to different location.Rust won't just let us dereference the location of memory to move the value out, because that would leave the memory uninitialized


## 常量池
> ## The Constant Pool
Java虚拟机用`constant_pool`表来存储程序中的类/接口/类实例/数组等类型
> Java Virtual Machine instructions do not rely on the run-time layout of classes, interfaces, class instances, or arrays. Instead, instructions refer to symbolic information in the `constant_pool` table.

`constant_pool`表中的都会遵循下面的格式
> All `constant_pool` table entries have the following general format:

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

## 操作码和Rust枚举
> ## Operation code and Rust Enum
如原书中 [常量](https://github.com/munificent/craftinginterpreters/blob/master/book/chunks-of-bytecode.md#constants) 章节所解释, 我们不打算直接在操作码中存储对应的常量
> As the book [Constant](https://github.com/munificent/craftinginterpreters/blob/master/book/chunks-of-bytecode.md#constants) section explains, we do not plan to save constant diectly with operation code

<aside name="header">
<img src="https://github.com/Kangaxx-0/rox/blob/main/assets/ConstantPool.png" alt="code and constant index" />
</aside>

所以想达成和clox一样的结果，Rustacean可能倾向类似下面的方案
> So achive the same result as clox,  Rustacean might like an approach like

```
enum OpCode {
    Constant(u8), // Save the index
}
```
如果索引值很大，我们可以用`usize`
> We can change `u8` to `usize` if the index goes large

# 2.虚拟机章节
> # 2.Chapter of A Virtual Machine

TBD

# 3.按需扫描章节
> # 3.Chapter of Scanning on Demand

TBD

# 4.编译表达式章节
> # 4.Chapter of Compiling Expressions

对于解析这部分，理解Vaughan Pratt的“自顶向下算符优先解析”算法有着非常重要的作用
> For the parsing part, it is important to understand how Vaughan Pratt’s “top-down operator precedence parsing” algorithms works
```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
// Precedence symbols:
//  No -> no Precedence
//  Assignment -> =
//  Or -> or
//  And -> and
//  Equality -> == !=
//  Comparison -> < > <= >=
//  Term -> + -
//  Factor -> * /
//  Unary -> ! -
//  Call -> . ()
//  Primary -> literals and grouping
//
enum Precedence {
    No,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}
```
<aside name="header">
<img src="https://github.com/Kangaxx-0/rox/blob/main/assets/connections.png" alt="Precedence" />
</aside>

## `Parser`结构
> ## `Parser` structure
在Rox，'Parser'包含一个对于`Chunk`的引用，同时拥有`Scanner`的所有权，这样做的目的是将关于解析分布的逻辑封装在一起
> In rox, `parser` has a reference of `Chunk` and also owns `Scanner`, by doing this, a lot of funtional code can be grouped and co-existed together


##  `ParseFn`
在实现[ParseFn](https://github.com/Kangaxx-0/rox/blob/main/src/compiler.rs#L56])的时候，我个人比较倾向用`fn`指针而不是`FnMut`特征因为我们并不需要捕获上下文环境
> When implementing [ParseFn](https://github.com/Kangaxx-0/rox/blob/main/src/compiler.rs#L56]) I perfer to use funtion pointer `fn` instread of `FnMut` trait because we don't really need to capture context environment.

## ParseRule数组
> ## ParseRule Array

在Rox里，我并没有为它去实现一个ParseRule数组，方法[get_rule](https://github.com/Kangaxx-0/rox/blob/main/src/compiler.rs)基于输入的`TokenType`返回一个规则
> In rox, I did not implement an array of ParseRule, instead, the function [get_rule](https://github.com/Kangaxx-0/rox/blob/main/src/compiler.rs) gets the rule based on the input `TokenType`


# 5.值类型章节
> # 5.Chapter of Type of Values

Rox的值类型充分利用了Rust里的`Enum`,所以当你阅读原书发现里面大量的C代码被抛弃， 不要感到惊讶
> The Rox value types are fully leveraging the power of Rust `Enum`, do not be suprsied when you saw lots of C code from the book gets ditched in Rox

# 6.字符串
> # 6.Strings

在原书中，作者单独花了不少篇幅来讨论Obj以及实现,Obj就是代表存在堆上的动态类型，Rust是类型安全，我们并不需要为此单独实现
> In the book, author spend a few sections to talk about Obj and its implementation, basically, Obj represents all types which are stored on heap. Since Rust is type-safe, so we do not need extra implementation.

```
struct Obj {
  ObjType type;
};

typedef enum {
  OBJ_STRING,
} ObjType;
```

# 7.哈希表
> # 7.Hash Tables

当前的实现基于rust safe代码，当我完全剩下章节的工作，会基于`Unsafe`重新审视这块
> The implementation is based on safe code, I will re-visit this with `Unsafe` after finishing everything

在我们哈希表实现中，在已知不会超过阈值得情况下，我们不希望在插入新值时变化内部Vec的容量，所以我们使用`std::mem::swap`
> If we know new insertion won't exceed the threadhold value we setup, thus, we do not expect to update internal `Vec`'s capacity, this is why we choose `std::mem::swap`

# 8.全局变量
> # 8.Global Variables

在Rox的全部变量解析中，我们首先将代表常量的指令加入`Chunk`，然后才添加代表全局变量的`DefineGlobal`的指令，当我们运行时,常量会被压入栈，全局变量只进行出栈操作，最后把键值添加到我们的哈希表中
> Regards to global variables in rox, we first add the constant of insturction to `Chunk`, and then we add `DefineGlobal` instuction, when we call `vm.run`,the constant instuction will push the value to stack, the global definition will only pop the value from stack, and finally add key and value to our hash table.
<aside name="header">
<img src="https://github.com/Kangaxx-0/rox/blob/main/assets/Rox_global.gif" alt="global" />
</aside>

# 9.局部变量
> # 9.Local Variables
关于局部变量最重要的一点就是 - 我们并不会在我们的哈希表保存它们，当作用域结束时对他们进行出栈
> The most important note for local variables is - We do not add them to our hash table ever, and pop them out when scope ends

# 10. 跳转
> # 10.Jumping Back and Forth

关于跳转的填充物，我们这里用的是Rust Enum(i16)作为临时的指令码，需要注意的是因为我们代表常量的指令码是`usize`,所以每一个enum variants都是`usize`大小，而并不是原书中的两个字节
> Regards to the jump placeholder offset operand, we use Rust Enum(i16) as op_code, and please notice we have `Constant(usize)` represent constant, so every enum variant is the same size of `usize`, not two bytes the book noted

# 11. 调用和函数
> # 11. Calls and Functions

在这一章节之前，虚拟机拥有`Chunk`的所有权，解释器结构体只引用`Chunk`，在解析过程中将生成的指令，常量等加入`chunk`,最后虚拟机调用`run`方法来执行.在这一章，我们会重新整理结构
> Before this chapter, the VM owns `Chunk`, and our parser only reference `Chunk`, parser push instruction code and constants to `Chunk`, and finally, vm calls its `run` function. In this chapter, we are going to re-organize  the structure.

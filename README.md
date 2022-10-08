# rox

An RUST implementation of clox from Robert Nystrom's book "Crafting Interpreters", it does rely on some Rust features coming out of the box, but also contains features/structures like an implementation of `Vector` and `HashTable`. If you would like to catch the details faster, you might want to read the previous sections about [AST section](https://craftinginterpreters.com/a-tree-walk-interpreter.html), or taking the reading list below

> This repo has two main branches: `main` and `safe`, `safe` is with rapid ongoing changes, and it can be considered as main branch at this moment

How bytecode VM interpreter works:

<aside name="header">
<img src="https://github.com/Kangaxx-0/rox/blob/main/assets/rox_flow.png" alt="diagram" />
</aside>

Also, a lot of implementation details and questions can be found [here](https://github.com/Kangaxx-0/rox/blob/main/implementation_notes.md).  It contains both English and Chinese. Check it out!


阅读推荐:
> Recommendation reading list:
- [Crafting Interpreters](https://github.com/munificent/craftinginterpreters)
- 上面的中文版 - [手撸解释器教程](https://github.com/GuoYaxiang/craftinginterpreters_zh)(未完成)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)


引用:
> References:
- [Java Constant Pool](https://docs.oracle.com/javase/specs/jvms/se18/html/jvms-4.html)
- [BSD Exit Code](https://github.com/openbsd/src/blob/master/include/sysexits.h)


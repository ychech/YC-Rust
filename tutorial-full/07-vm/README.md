# 07. 虚拟机 - WASM执行环境

## 本章目标

理解WASM虚拟机，实现智能合约执行环境。

## 什么是WASM？

WebAssembly（WASM）是一种二进制指令格式，设计为可移植、高性能的编译目标。

```
高级语言（Rust/C++）
       ↓ 编译
    WASM字节码
       ↓ 在VM中执行
    机器码
```

## 为什么用WASM？

- **高性能**：接近原生代码速度
- **安全**：沙箱执行，内存隔离
- **多语言**：Rust、C++、AssemblyScript
- **标准化**：WebAssembly标准

## Gas机制

```rust
/// Gas计费
/// 
/// 防止无限循环和滥用
/// 每指令消耗一定Gas
pub struct GasMeter {
    pub limit: u64,      // Gas上限
    pub used: u64,       // 已使用
    pub price: u64,      // Gas价格
}
```

## 核心代码

见 [main.rs](main.rs)

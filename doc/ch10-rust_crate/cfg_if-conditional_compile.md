# cfg_if 库和条件编译

## cfg_if 库

### 简单说明

cfg_if 是一个 Rust 的实用工具库,主要用于简化条件编译相关的代码。它提供了一个宏  `cfg_if!`,可以让你更方便地编写基于编译时配置的条件代码。

cfg_if 库的核心是 `cfg_if!` 宏,它允许你根据不同的编译时条件选择性地编译代码块。这对于处理跨平台代码、特性标志或其他编译时变量非常有用。

### 使用介绍

1. Cargo.toml 中添加依赖

```toml
cfg-if = "1.0.0"
```

2. 代码中使用 `cfg_if!` 宏
  
```rust
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(unix)] {
        fn foo() { /* unix-specific functionality */ }
    } else if #[cfg(windows)] {
        fn foo() { /* windows-specific functionality */ }
    } else {
        fn foo() { /* fallback implementation */ }
    }
}
```

### 特性说明

根据上面的使用可以看到这个库的优势：

* 简化语法: 相比于嵌套的  `#[cfg()]`  属性,cfg_if 提供了更清晰、更易读的语法。
    
* 避免错误: 它可以帮助避免一些常见的条件编译错误,如忘记 else 分支。
    
* 可组合: 可以轻松组合多个条件。
    
还可以使用更高级的语法：

```rust
cfg_if! {
    if #[cfg(all(unix, target_pointer_width = "32"))] {
        // 32-bit unix systems
    } else if #[cfg(all(unix, target_pointer_width = "64"))] {
        // 64-bit unix systems
    } else if #[cfg(windows)] {
        // windows systems
    } else {
        // other systems
    }
}
```

`cfg_if!` 宏在**编译时**展开,生成等效的 `if-else 链` 或 `match 表达式`。这意味着它没有运行时开销,所有的条件检查都在编译时完成。

### 使用场景说明

1. 适用场景：

    * 跨平台代码
    
    * 特性标志处理
    
    * 编译时优化
    
    * API 兼容性层
    
2. 不适用场景：

    * cfg_if 主要用于编译时条件,不适用于运行时动态选择。
    
    * 过度使用可能会使代码难以理解,应适度使用。

3. 总结:

    * 虽然 cfg_if 很有用,但在某些简单情况下,直接使用 Rust 的 `#[cfg()]` 属性可能更直接
    * 选择使用 cfg_if 还是原生 cfg 属性取决于具体的使用场景和个人偏好。

## 条件编译

接下来简单介绍 Rust 的  `#[cfg()]`

### 简单介绍

`#[cfg()]` 是 Rust 的一个内置属性,用于条件编译。它允许您根据指定的配置选项来包含或排除代码。这是 Rust 中进行条件编译的主要方式

### 使用说明

```rust
#[cfg(unix)]
fn unix_only() {
    // 这个函数只在 Unix 系统上编译
}

#[cfg(windows)]
fn windows_only() {
    // 这个函数只在 Windows 系统上编译
}
```

### 常用的配置选项

* 操作系统: `unix`, `windows`, `macos`, `linux` 等等
    
* 架构: `x86`, `x86_64`, `arm` 等等
    
* 特性标志: 在  `Cargo.t它可以应用在多个场景：oml`  中定义的特性
    
* 编译器版本: `rustc_1_40`  等
    
也可以进行逻辑运算符组合多个条件：

```rust
#[cfg(all(unix, target_pointer_width = "64"))]
fn unix_64bit_function() {
    // 只在 64 位 Unix 系统上编译
}

#[cfg(any(windows, macos))]
fn windows_or_mac_function() {
    // 在 Windows 或 macOS 上编译
}

#[cfg(not(debug_assertions))]
fn release_only() {
    // 只在 release 模式下编译
}
```

### 使用场景说明

1、可以应用于模块级：

```rust
#[cfg(feature = "advanced")]
mod advanced_features {
    // 这个模块只在启用 "advanced" 特性时编译
}
```

2、条件导入

```rust
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
```

3、在代码中使用

```rust
if cfg!(debug_assertions) {
    println!("Debug mode");
} else {
    println!("Release mode");
}
```

4、甚至可以使用自定义标志 在  `Cargo.toml`  中定义自己的特性标志:

```toml
[features]
my_feature = []
```

然后在代码中使用:

```rust
#[cfg(feature = "my_feature")]
fn feature_specific_function() {
    // ...
}
```

### Rust 的 `cfg` 属性和 `cfg!` 宏支持的逻辑组合(简单列举)

综合来说, Rust 的 `cfg` 属性和 `cfg!` 宏支持多种逻辑组合。

以下是完整的列表：

1. `all`: 所有条件都必须为真

```rust
#[cfg(all(unix, target_pointer_width = "64"))]
```
    
2. `any`: 任意一个条件为真即可
   
```rust
#[cfg(any(windows, macos))]
``` 

3. `not`: 条件必须为假
    
```rust
#[cfg(not(windows))]
```

4. `target_os`: 目标操作系统
    
```rust
#[cfg(target_os = "linux")]
```

5. `target_arch`: 目标架构
    
```rust
#[cfg(target_arch = "x86_64")]
```

6. `target_feature`: 特定的 CPU 特性
    
```rust
#[cfg(target_feature = "avx2")]
```

7. `target_endian`: 字节序
    
```rust
#[cfg(target_endian = "little")]
```

8. `target_pointer_width`: 指针宽度
    
```rust
#[cfg(target_pointer_width = "64")]
```

9. `target_env`: 目标环境
    
```rust
#[cfg(target_env = "gnu")]
```

10. `target_vendor`: 目标供应商
    
```rust
#[cfg(target_vendor = "apple")]
```

11. `feature`: 编译时特性
    
```rust
#[cfg(feature = "my_feature")]
```

12. `debug_assertions`: 是否启用调试断言
    
```rust
#[cfg(debug_assertions)]
```

13. `test`: 是否在测试模式下编译
   
```rust
#[cfg(test)]
``` 

这些条件可以任意组合使用：

```rust
#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
```

此外，还可以使用 `cfg_attr` 来条件性地应用其他属性（第一个参数是条件，第二个参数是条件为真时的属性）：

```rust
#[cfg_attr(feature = "nightly", feature(some_unstable_feature))]
...

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct MyStruct {
    // fields...
}
```

记住，这些配置可以在编译时使用 `--cfg` 标志来设置，也可以在 `Cargo.toml` 中通过 features 来控制。使用这些逻辑组合，你可以非常精细地控制哪些代码在特定条件下被编译。

`#[cfg()]`  在**编译时**评估,不会增加运行时开销。

## cfg_if vs cfg

* `#[cfg()]` 是 Rust 的内置功能,不需要额外的依赖。
    
* 对于简单的条件,`#[cfg()]` 通常更直接。
    
* 对于复杂的嵌套条件,cfg_if 可能提供更好的可读性。
    
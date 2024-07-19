# thiserror 和 anyhow

## thiserror
---------

thiserror 是一个用于简化自定义错误类型定义的库, 它为标准库的 `std::error::error` trait 提供了一个方便的派生宏。。以下是 thiserror 的主要特点和用法。

要使用 thiserror,首先在 Cargo.toml 中添加依赖:

```toml
[dependencies] 
thiserror = "1.0"
```

然后,您可以这样定义一个错误类型:

```rust
fn main() {
    use std::fs::File;
    use std::io::Read;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum MyError {
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),

        #[error("Parse error: {0}")]
        Parse(#[from] std::num::ParseIntError),
    }

    fn read_file_and_parse_number(file_path: &str) -> Result<i32, MyError> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let number: i32 = contents.trim().parse()?;
        Ok(number)
    }
    
    match read_file_and_parse_number("test.txt") {
        Ok(number) => println!("Parsed number: {}", number),
        Err(e) => {
            println!("Error: {}", e);
            match e {
                MyError::Io(io_err) => println!("IO Error details: {}", io_err),
                MyError::Parse(parse_err) => println!("Parse Error details: {}", parse_err),
            }
        },
    }
}
```

* `#[derive(Error)]`: 这个派生宏自动实现了`std::error::Error` trait。
    
* `#[error("...")]`: 定义错误消息的格式。可以使用占位符引用字段。
    
* 如果你在结构体或枚举的每个变体上提供 `#[error("...")]` 消息，会自动生成 `Display` 实现。
    
* `#[error("{var}")]` 对应 `write!("{}", self.var)`
    
* `#[error("{0}")]` 对应 `write!("{}", self.0)`
    
* 还支持带格式的版本，如 `#[error("{var:?}")]`
    
* `#[from]`: 自动实现`From` trait,允许从其他错误类型转换。
    
thiserror 支持多种格式化选项:

```rust
fn main() {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum AnotherError {
        #[error("Another error occurred")]
        General,
    }

    #[derive(Error, Debug)]
    pub enum FormatExample {
        #[error("Error code {code}")]
        Simple { code: i32 },

        #[error("Complex error: {msg} (code: {code})")]
        Complex { msg: String, code: i32 },

        #[error("Error with source: {0}")]
        WithSource(#[source] AnotherError),
    }

    // Example usage of the FormatExample enum
    let simple_error = FormatExample::Simple { code: 404 };
    let complex_error = FormatExample::Complex {
        msg: String::from("Not Found"),
        code: 404,
    };
    let source_error = FormatExample::WithSource(AnotherError::General);

    println!("Simple error: {}", simple_error);
    println!("Complex error: {}", complex_error);
    println!("Source error: {}", source_error);
}
```

thiserror 可以与其他常见的 Rust 特性很好地集成:

```rust
fn main() {
    use anyhow::Error;
    use serde::Serialize;
    use thiserror::Error;

    #[derive(Debug, Serialize)]
    pub enum ServerErrorKind {
        NotFound,
        Internal,
    }

    #[derive(Error, Debug)]
    #[error("Server error: {kind:?}")]
    pub struct ServerError {
        kind: ServerErrorKind,
        #[source]
        source: Error,
    }

    // Example usage of the ServerError struct
    let source_error = anyhow::anyhow!("An internal error occurred");
    let source_not_found_error = anyhow::anyhow!("An not found error occurred");
    let server_error = ServerError {
        kind: ServerErrorKind::Internal,
        source: source_error,
    };
    let server_not_found_error = ServerError {
        kind: ServerErrorKind::NotFound,
        source: source_not_found_error,
    };

    println!("Server error: {}", server_error);
    println!("Server not found error: {}", server_not_found_error);
}
```

**与 anyhow 的比较:**

* **thiserror**适用于库作者,用于定义明确的错误类型。
    
* **anyhow**更适合应用程序开发,用于处理各种可能的错误情况。
    
**thiserror 的限制**：

* 只能用于枚举和结构体。 错误可以是枚举、带命名字段的结构体、元组结构体或单元结构体。 thiserror 主要设计用于枚举和结构体。这意味着你不能使用它来为基本类型或其他复杂类型实现 Error trait。 可以这样使用：
    
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("File not found: {0}")]
    FileNotFound(String),
}

#[derive(Error, Debug)]
pub struct DatabaseError {
    #[error("Database connection failed: {0}")]
    message: String,
}
```

但不能这样使用：

```rust
use thiserror::Error;

// 这将无法编译
#[derive(Error, Debug)]
type MyErrorType = String;
```

* 不支持泛型错误类型(虽然可以在枚举变体中使用泛型)。 thiserror 不直接支持在顶层使用泛型参数的错误类型。这意味着你不能创建一个泛型的错误枚举或结构体。 例如下面的例子不正确：
    
```rust
use thiserror::Error;

// 这将无法编译
#[derive(Error, Debug)]
pub enum MyError<T> {
    #[error("Invalid value: {0}")]
    InvalidValue(T),
}
```

然而，你可以在枚举的变体中使用泛型：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("Parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
    #[error("Other error: {0}")]
    Other(Box<dyn std::error::Error>),
}
```

thiserror 库极大地简化了在 Rust 中定义和管理自定义错误类型的过程,使得创建专业、易于使用的错误处理变得更加容易。

## anyhow
---------

这个库提供了`anyhow::Error`，一种基于 trait 对象的错误类型，用于在 Rust 应用程序中轻松处理惯用错误。

1. 使用 `Result<T, anyhow::Error>` 或 `anyhow::Result<T>` 作为任何可能失败的函数的返回类型。在函数内部，使用 `?` 运算符可以轻松传播任何实现了 `std::error::Error` trait 的错误。
    
```rust
fn main() {
    use anyhow::Result;
    use std::collections::HashMap;

    type ClusterMap = HashMap<String, String>;
    fn get_cluster_info() -> Result<ClusterMap> {
        let config = std::fs::read_to_string("cluster.json")?;
        let map = serde_json::from_str(&config)?;
        Ok(map)
    }

    match get_cluster_info() {
        Ok(cluster_map) => {
            // Handle the case where get_cluster_info() returns Ok
            println!("Cluster info: {:?}", cluster_map);
        }
        Err(e) => {
            // Handle the case where get_cluster_info() returns an Err
            println!("Failed to get cluster info: {}", e.to_string());
            println!("Error Stack");
            println!("{:?}", e);
        }
    }
}
```

2. 添加上下文信息以帮助排查错误。例如，使用 `context` 或 `with_context` 方法为低级错误添加更多信息。
    
```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    ...
    it.detach().context("Failed to detach the important thing")?;

    let content = std::fs::read(path)
        .with_context(|| format!("Failed to read instrs from {}", path))?;
    ...
}
```

3. 支持向下转型（`downcasting`），可以按值、共享引用或可变引用进行。
    
```rust
use anyhow::{anyhow, Result};
fn main() -> Result<()>{
    use std::fmt;
    use rand::Rng;

    #[derive(Debug)]
    enum DataStoreError {
        Censored(String),
        Other(String),
    }

    impl fmt::Display for DataStoreError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                DataStoreError::Censored(msg) => write!(f, "Censored: {}", msg),
                DataStoreError::Other(msg) => write!(f, "Other: {}", msg),
            }
        }
    }

    impl std::error::Error for DataStoreError {}
    fn get_data() -> Result<String> {
        let mut rng = rand::thread_rng();
        let n: u32 = rng.gen();
        if n % 2 == 0 {
            Err(anyhow!(DataStoreError::Censored("Sensitive data".into())))
        } else {
            Err(anyhow!(DataStoreError::Other("unknow data".into())))
        }
        
    }

    let error = get_data().unwrap_err();
    let root_cause = error.root_cause();

    // If the error was caused by redaction, then return a tombstone instead of the content.
    match root_cause.downcast_ref::<DataStoreError>() {
        Some(DataStoreError::Censored(_)) => {
            println!("REDACTED_CONTENT");
            Ok(())
        }
        Some(DataStoreError::Other(_)) => {
            println!("OTHER_ERROR");
            Ok(())
        }
        None => Err(error),
    }
}
```

4. 在 Rust 1.65 及以上版本中，如果底层错误类型没有提供自己的回溯，anyhow 会捕获并打印回溯。可以通过环境变量控制回溯的行为。
    
5. Anyhow 可以与任何实现了 `std::error::Error` 的错误类型一起使用，包括在你的 crate 中定义的错误类型。
    
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Invalid header (expected {expected:?}, got {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("Missing attribute: {0}")]
    MissingAttribute(String),
}
```

6. 提供 `anyhow!` 宏用于构造一次性错误消息，支持字符串插值。还提供了 `bail!` 宏作为提前返回错误的简写。
    
```rust
return Err(anyhow!("Missing attribute: {}", missing));

bail!("Missing attribute: {}", missing);
```

7. 支持 no_std 模式，几乎所有 API 都可用并以相同方式工作。在 no_std 模式下使用时，需要在 Cargo.toml 中禁用默认的 "std" 特性。
    
这些特性使 anyhow 成为一个强大而灵活的错误处理库，适用于各种 Rust 项目，特别是在需要统一错误处理和提供丰富错误上下文的场景中。

其它一些库，虽然曾经被广泛的使用，但是多年已经处于无人维护的状态，基本算是被废弃了。failure 库的最后一次实质性更新是在 2019 年。自那以后，它就没有收到任何重大更新或维护，它的 GitHub 仓库上有一个明确的声明，表示该库不再被积极维护。error-chain 库也处于类似的情况，可以被认为是实质上废弃了，error-chain 的最后一次实质性更新是在 2018 年。自那以后，它只收到了很少的维护更新。虽然 error-chain 的 GitHub 仓库没有正式声明废弃，但其长期缺乏更新实际上表明它已不再被积极维护。

目前 Rust 生态系统已经发展出了更好的错误处理方案。主要的替代品就是上面介绍的两个库：

* `anyhow`：用于应用程序级别的错误处理
    
* `thiserror`：用于库级别的错误定义
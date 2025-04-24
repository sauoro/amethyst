# Logging
## Example usage:
```rust
use log::{Log, Logger};

fn main() {
    let logger = Logger::new("Amethyst".to_string());
    let nameless_logger = Logger::nameless();
    logger.info("Hi");
    logger.info("Bye");
    logger.warn("Hi");
    logger.error("Bye");
    nameless_logger.info("Hi");
    nameless_logger.info("Bye");
    nameless_logger.warn("Hi");
    nameless_logger.error("Bye");
}
```

## Terminal:
![img.png](https://github.com/sauoro/amethyst/blob/main/assets/log_test.png)

## Extra Resources
> Terminal Colors: [ansi](https://github.com/fidian/ansi)
> 
> Logging Inspired by [NetrexMC](https://github.com/NetrexMC/Netrex/tree/master/netrex/src/log)
# About
Uyta is a farming game, where your goal is to expand your island, fill it with various plants, trees and animals and raise your farm's level. 

# Features
- 15+ plants, trees and animals
- 45+ purchasable upgrades
- Progress saves automatically
- Supported languages: English and Russian
- About 30-60 minutes of gameplay

# Building from source
```
cargo build --release
```

## Cross-compilation from Linux to Windows
Use cross:
```
cross build --release --target=x86_64-pc-windows-gnu
```

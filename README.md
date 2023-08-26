# rsblox
Wine prefix manager for Roblox, made in Rust
## NOTE:
As of May 3, 2023, Roblox does not support Wine anymore.

If you want to play Roblox on Linux, you need to install it through Waydroid. More info on that [here.](https://gitlab.com/TestingPlant/roblox-on-waydroid-guide)


This repository will remain open, as [Roblox plans to allow Wine users to play again in the future.](https://devforum.roblox.com/t/the-new-roblox-64-bit-byfron-client-forbids-wine-users-from-using-it-most-likely-unintentional/2305528/28)

## Installation
```sh
cargo install --git github.com/zurrty/rsblox
```

## Usage
```sh
rsblox
```
yeah thats pretty much it

you can also have multiple installs:
```sh
rsblox --install OtherInstall
```

## Building
To build rsblox, you need to have the rust nightly toolchain. This is because rsblox makes use of `std::fs::try_exists` which is currently in experimental stages. This will likely be implemented into the stable standard library in the future.
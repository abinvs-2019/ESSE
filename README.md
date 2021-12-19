<h1 align="center"><img src="https://cympletech.com/screenshots/esse_words_logo.png" alt="ESSE"></h1>

**ESSE** (Encrypted Symmetrical Session Engine) An open source encrypted peer-to-peer system for data security, would allow data to be sent securely from one terminal to another without going through third-party services.

**My place, my rules.**

![image](https://cympletech.com/statics/esse-show.gif)

ESSE, stands for Encrypted Symmetrical Session Engine, positioned as an engine. The engine is coded in [**Rust**](https://github.com/rust-lang/rust) language based on [**TDN**](https://github.com/cypherlink/TDN) framework, and the cross-platform user interface is built using [**Flutter**](https://github.com/flutter/flutter).

## Features
- Data Security Fortress
- Encryption Everywhere
- Distributed Identity
- Distributed Devices
- Distributed Notes & Files
- Distributed Storage & Synchronization
- Built-in IM Application
- Built-in Group Chat Application
- Built-in Domain Application
- Built-in Wallet Application (Support ETH/ERC20/ERC721)
- Built-in Robot assistant Application
- Multi-identity System
- Multi-platform Support: Android, iOS, iPadOS, MacOS, Windows, Linux, etc.

[About ESSE(English)](https://github.com/CympleTech/esse/wiki/About-ESSE) / [关于ESSE(简体中文)](https://github.com/CympleTech/esse/wiki/%E5%85%B3%E4%BA%8E-ESSE)

[Screenshots](https://github.com/CympleTech/esse/wiki/Screenshots)

## Usage
### 1. Use Binary executable
[Download](https://github.com/cympletech/esse/releases)

### 2. Compile
#### 2.1. Pre-installed
- Rustup [install](https://rustup.rs/)
- Rust (Lastest Stable version)
- Flutter (Lastest Stable channel)

#### 2.2. Compile Rust code to dynamic link library (FFI)
##### 2.2.1. Auto-compile script
It is recommended to use [rust.sh](./rust.sh) to auto-compile the Rust code

##### 2.2.2. Manually compile
##### Linux / MacOS / Windows
- `cargo build --release`

##### Linux
- `cp target/release/libesse.a core/linux/share/libesse.a`

##### MacOS
- `cp target/release/libesse.a core/macos/share/libesse.a`

##### Windows
- `cp target/release/esse.dll core/windows/share/esse.dll`
- `cp target/release/esse.dll.lib core/windows/share/esse.dll.lib`

##### Android
1. Add your android device target

- `rustup target add aarch64-linux-android`
- `rustup target add armv7-linux-androideabi`
- `rustup target add x86_64-linux-android`

2. Configure your NDK

3. Build the jniLibs
- `cargo build --release --target=aarch64-linux-android`
- `cp target/aarch64-linux-android/release/libesse.so core/android/src/main/jniLibs/arm64-v8a/`

##### iOS
1. Install [lipo](https://github.com/TimNN/cargo-lipo)
2. `cargo lipo --release`
3. `cp target/universal/release/libesse.a core/ios/share/libesse.a`

#### 2.3. Run flutter to build binary
- Run `flutter run` or `flutter run --release` in terminal, or
- for Android, run `flutter build apk`, or
- for Linux, run `flutter build linux`, or
- for MacOS, run `flutter build macos`, or
- for Windows, run `flutter build windows`

## License

This project is licensed under

 * GNU GENERAL PUBLIC LICENSE, Version 3.0, [LICENSE](LICENSE)
 * Any question, please contact: contact@cympletech.com

## Donation

**ESSE is still in its infancy, both technical and financial support are welcome. Thank you for your support.**

ETH：0xbB64D716FAbDEC3a106bb913Fb4f82c1EeC851b8

Paypal：[**https://www.paypal.me/cympletech**](https://www.paypal.me/cympletech)

gitcoin：[**https://gitcoin.co/grants/2516/esse-an-encrypted-peer-to-peer-session-system**](https://gitcoin.co/grants/2516/esse-an-encrypted-peer-to-peer-session-system)

## For more information, please visit:
- Website: https://cympletech.com/
- Github: https://github.com/CympleTech/esse
- Twitter: https://twitter.com/cympletech
- E-mail: contact@cympletech.com
- Discord: https://discord.gg/UfFjp6Kaj4

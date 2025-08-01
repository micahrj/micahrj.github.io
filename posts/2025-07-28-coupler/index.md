+++
title = "Coupler: an audio plugin framework for Rust"
date = "2025-07-28T00:00:00-05:00"
+++

[Coupler](https://coupler.rs) is a framework for writing audio plugins in Rust. Like the [Rust language](https://www.rust-lang.org/) itself, the goal of Coupler is to empower everyone to build reliable and efficient audio software.

Coupler strives to balance three qualities:

1. **Performance**: Coupler follows a "pay for what you use" philosophy and strives to impose as little overhead as possible between hosts and plugins.
2. **Reliability**: A plugin API is a contract, with rules for threading and memory management that must be followed by both host and plugin. Coupler uses Rust's type system and ownership model to ensure that plugins implement their end of this contract correctly.
3. **Productivity**: Coupler provides a [Serde](https://crates.io/crates/serde)-like `#[derive(Params)]` macro for declaring plugin parameters and a `cargo coupler bundle` command for generating plugin bundles.

Coupler is still in the early stages of development, and should not be considered production-ready. The overall architecture has been through a good amount of iteration and I'm fairly confident in it at this point, but there will definitely continue to be API churn. Basic support for CLAP and VST 3 is in place, and support for AUv2 and AAX is planned for the future but isn't there yet.

Coupler is dual-licensed under MIT and Apache 2.0.

<!--excerpt-->

# Motivation

Real-time audio software has a very particular set of constraints. There are strict performance requirements, as end-to-end latency needs to be kept under a fixed limit or else there will be audible glitches in the output. Audio software is also unavoidably concurrent in a way that is harder to ignore than in other domains, as logic must be split between the real-time audio thread and threads dealing with other concerns like the GUI, file I/O, etc. Audio plugins in particular face an additional set of constraints, as they need to be good citizens when running as just one of many shared libraries in the context of a host process.

The performance constraints necessitate using a language that provides sufficient low-level control over operations like allocation and synchronization. The constraints of plugins in particular rule out languages with heavy runtimes. Due to this intersection of constraints, the majority of audio plugins continue to be written in C++. However, using C++ means losing the safety and productivity benefits of higher-level programming languages, and the inherent concurrency of real-time audio programming only makes this more acute.

The landscape of programming languages generally poses a hard choice between performance and low-level control, on the one hand, and safety and productivity on the other; the hard constraints in the domain of audio software then necessitate choosing the former and giving up on the latter. However, the achievement of the Rust project is to reveal this to be a false dichotomy and show that, with careful design, it is possible to retain performance and low-level control without having to sacrifice safety and productivity. For this reason, I believe that Rust holds unique promise in the context of audio software, and my goal with the Coupler project is to achieve the same thing in the context of audio plugins.

# Bindings

The plugin APIs that Coupler interfaces with are defined in terms of C or C++ headers, so it has been necessary to write Rust bindings for them. Coupler uses [`clap-sys`](https://github.com/micahrj/clap-sys) for CLAP and [`vst3-rs`](https://github.com/coupler-rs/vst3-rs) for VST 3. AUv2 support will be implemented using [`auv2-sys`](https://github.com/coupler-rs/auv2-sys). CLAP was straightforward to write bindings for, since it uses simple C constructs. AUv2 was also fairly straightforward, although it took some digging in the macOS platform headers. However, developing bindings for VST 3 was significantly more involved, as the API makes extensive use of C++ virtual method tables.

When I started work on VST 3 support for Coupler, there was in fact already an existing set of Rust bindings, [`vst3-sys`](https://github.com/RustAudio/vst3-sys), but it was released under the GPLv3 license due to the way the original VST 3 SDK is licensed, and I wanted to have a permissively licensed solution. I also investigated using [`rust-bindgen`](https://github.com/rust-lang/rust-bindgen), but the necessary features for dealing with C++ vtables weren't there yet. Ultimately, I ended up writing my own binding generator using `libclang`. I hope to be able to reuse much of this work for the AAX API. I plan to write more about the `vst3-rs` project in a future post.

# GUI

Plugin GUIs also have a unique set of considerations, due to the fact that they run as embedded child windows and as guests of the host application's event loop. These considerations rule out most GUI libraries that are not expressly designed to be compatible with the plugin use case, and so I have also been working on a library for cross-platform window management called [Portlight](https://github.com/coupler-rs/portlight/). Portlight currently has backends for Windows, macOS, and X11, with basic support for input events, timers, DPI, and vsync. There's a lot of functionality left to implement, but the overall event loop architecture is solid.

# Links

Coupler isn't quite ready for a crates.io release yet, and there's not much in the way of documentation, but if you're interested in with experimenting with it, feel free to check out the you're interested in experimenting with Coupler, feel free to check out the [GitHub repository](github.com/coupler-rs/coupler).

If you're interested in collaborating, learning more about Coupler, or discussing plugin development in Rust, please feel free to join the [Zulip instance](https://coupler.zulipchat.com/) or the [Rust Audio Discord](https://discord.gg/yVCFhmQYPC)!

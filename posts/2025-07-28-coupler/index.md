+++
title = "Coupler: an audio plugin framework for Rust"
date = "2025-07-28T00:00:00-05:00"
+++

[Coupler](https://coupler.rs) is a framework for writing audio plugins in Rust. Like the [Rust language](https://www.rust-lang.org/) itself, the goal of Coupler is to empower everyone to build reliable and efficient audio software.

Coupler prioritizes:

1. **Performance**: Coupler follows a "pay for what you use" philosophy and strives to impose as little overhead as possible between hosts and plugins.
2. **Reliability**: A plugin API is a contract, with rules for threading and memory management that must be followed by both host and plugin. Coupler uses Rust's type system and ownership model to ensure that plugins implement their end of this contract correctly.
3. **Productivity**: Coupler provides a [Serde](https://crates.io/crates/serde)-like `#[derive(Params)]` macro for declaring plugin parameters and a `cargo coupler bundle` command for generating plugin bundles.

<!--excerpt-->

# Motivation

Real-time audio software has a very particular set of constraints. There are strict performance requirements, as end-to-end latency needs to be kept under a fixed limit or else there will be audible glitches in the output. You also have to deal directly with concurrency in an uncommonly direct way, since your code is inherently split between the real-time audio thread and other non-real-time threads. The performance constraints necessitate using a language like C++ that provides sufficient low-level control over things like allocation, but using C++ means losing the safety and productivity benefits of higher-level programming languages.

C++ does not protect you from dangling pointers, out-of-bounds accesses, use-after-frees, and data races. It's easy for any given piece of code to introduce one of these mistakes if the programmer is not constantly vigilant, and there is no safety net, so these mistakes directly result in undefined behavior. The inherent concurrency of real-time audio programming exacerbates this problem.

The landscape of existing programming languages poses a hard choice between performance and low-level control on the one hand, and safety/reliability and productivity/ease of use on the other; and then the hard performance constraints in the domain of real-time audio software necessitate choosing the former and giving up on the latter. However, the Rust project shows that this is actually a false dichotomy, and that with careful design it is possible to retain performance and low-level control without having to sacrifice safety/reliability and productivity/easy of use. My goal with the Coupler project is to achieve the same thing in the context of audio plugins.

# Status

Coupler is still in the early stages of development. It is definitely not production-ready yet. The overall architecture has been through a decent amount of iteration and I'm fairly confident in it at this point, but there will definitely continue to be API churn. Some basic functionality is still missing, but enough is there that it's possible to write basic effect plugins. Basic support for VST 3 and CLAP is in place. Support for AUv2 and AAX is planned at some point in the near future but isn't there yet.

# Bindings

The plugin APIs that Coupler interfaces with are defined in terms of the C ABI, so it has been necessary to write Rust bindings for them. CLAP was very straightforward to write bindings for, since it uses simple C constructs and there were no licensing issues (the C headers are permissively licensed). AUv2 was also fairly straightforward, although it took some digging in the macOS platform headers.

With VST 3, there was an existing set of Rust bindings, [vst3-sys](https://github.com/RustAudio/vst3-sys), but it was licensed as GPLv3 due to the way the original VST 3 SDK is licensed, and I wanted to have a permissively licensed solution. This necessitated writing my own binding generator using libclang. I hope to be able to reuse much of this work for the AAX API. I plan to write more about the vst3-rs project in a future post.

# GUI



# Plans



# Links

- https://github.com/micahrj/clap-sys/
- https://github.com/coupler-rs/vst3-rs
- https://github.com/coupler-rs/auv2-sys

- https://github.com/coupler-rs/portlight
- https://github.com/coupler-rs/flicker

- https://coupler.rs/



[Zulip instance](https://coupler.zulipchat.com/)

[Rust Audio Discord](https://discord.gg/yVCFhmQYPC)

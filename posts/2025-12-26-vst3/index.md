+++
title = "Simplifying the build process for vst3-rs"
date = "2025-12-26T12:13:00-07:00"
+++

[VST 3](https://github.com/steinbergmedia/vst3sdk) is an audio plugin interface which is developed by Steinberg and supported by a large number of host applications and plugins. The VST 3 API comprises a set of C++ header files which contain definitions for structs, constants, and abstract base classes (used in a similar way to [COM](https://en.wikipedia.org/wiki/Component_Object_Model)). I maintain a set of [Rust bindings](https://github.com/coupler-rs/vst3-rs) for VST 3 which are automatically generated from the original C++ headers using [libclang](https://clang.llvm.org/doxygen/group__CINDEX.html).

Earlier this month, I released version 0.3.0 of the `vst3` crate. In previous versions of the crate, bindings were generated at build time, and users were required to supply both libclang and the VST 3 SDK as third-party dependencies. As of this latest version, the generated bindings are now published directly as part of the crate's source, and the build-time generation step and third-party dependencies are no longer necessary. This change both simplifies the setup process for downstream users of the crate and significantly improves build times.

These improvements were made possible by a recent change to the licensing situation around the VST 3 SDK, but they also required solving some technical problems regarding the output of the binding generator. I'll talk about both of these aspects below.

<!--excerpt-->

# Licensing issues

The first version of the `vst3` crate was released in August 2023. At that time, Steinberg's SDK was dual-licensed, and developers had the option of using it under either the [GPLv3](https://www.gnu.org/licenses/gpl-3.0.html) or a proprietary license. Selecting the GPLv3 license meant that developers could make use of the SDK in open-source projects, but those projects in turn also had to be distributed under a GPLv3-compatible license. The proprietary license required signing an agreement with Steinberg and explicitly prohibited the distribution of derivative works of the SDK.

My goal with the `vst3` crate was to enable developers to use the VST 3 interfaces from Rust in all the same situations where they might use them from C++, including in closed-source hosts and plugins. Distributing bindings under GPLv3 would not have accomplished this, and the proprietary license did not allow for distributing bindings at all. Due to these constraints, the solution I ultimately settled on was for the `vst3` crate not to include any actual bindings in the first place; instead, the crate itself was purely a binding generator (released under MIT and Apache 2.0), and bindings would be generated at build time from a user-provided copy of the SDK (obtained under a license of their choice).

In October of this year, there was an unexpected development: Steinberg [released](https://forums.steinberg.net/t/vst-3-8-0-sdk-released/1011988) version 3.8.0 of the VST 3 SDK under the MIT license. This was exciting, because it meant that there were no longer any legal obstacles to directly publishing Rust bindings for VST 3; however, there remained some technical obstacles which had to be overcome.

# Platform inconsistencies

The `vst3` crate's binding generator uses libclang to extract information from the C++ headers in the VST 3 SDK. This saves an immense amount of work compared to writing a C++ parser from scratch. However, there is one significant limitation, which is that the information provided by libclang is specific to a particular target platform, and it may or may not be valid for other targets. Definitions can vary due to explicit preprocessor conditionals or arbitrary platform differences in toolchain behavior, and the only reliable way to obtain this information from libclang is to run it once for each target platform.

In the case of the VST 3 headers, there were handful of concrete differences in libclang's output on different platforms which were resulting in corresponding differences in the generated Rust bindings. This didn't pose a problem when generating bindings at build time, since the generator would always be run with the same target platform as the rest of the build, but now that I wanted to run the generation step ahead of time, those differences had to be addressed somehow. The brute-force option would have been to generate and commit a separate version of the bindings for every supported target, but I wanted to avoid that if possible. The approach I took instead was to add a CI check verifying that the generated bindings for each platform were character-identical and then address the failures one by one until it passed.

Comparing the generator's output on Windows, macOS, and Linux, I found the following differences:

1. Definitions were being written out in different orders.
2. The [fixed-width integer types](https://en.cppreference.com/w/cpp/types/integer.html) in `<cstdint>` (`int8_t`, `uint8_t`, etc.) were being mapped to different integer type aliases in Rust.
3. A small number of integer constants (specifically the [result codes](https://github.com/coupler-rs/vst3_pluginterfaces/blob/31d6eeba6daaa3e2a8bfbe3e7a90ca0b7fbfbc1c/base/funknown.h#L161-L202)) had different values.
4. Enumeration types without a [fixed underlying type](https://en.cppreference.com/w/cpp/language/enum.html) (i.e., the `int32_t` in `enum E : int32_t`) had different underlying types.

The first two on the list were straightforward to eliminate, as they were ultimately arbitrary and didn't correspond to any real functional difference between platforms. Difference #1 was caused by the fact that libclang itself was traversing the AST in different orders, and it was easily addressed by simply sorting the definitions before outputting them. Difference #2 was caused by the fact that the binding generator was mapping C++'s fixed-width integers to the FFI integer types in Rust's [`std::ffi`](https://doc.rust-lang.org/stable/std/ffi/index.html) module, which was a totally unnecessary indirection — while the fixed-width integers map to different Rust FFI integer types on different platforms, and the FFI integer types themselves map to different Rust primitive types on different platforms, these differences always cancel out — and was addressed by mapping them directly to Rust's primitive integer types instead.

Difference #3 corresponds to a real difference between platforms, as the result code constants are actually defined to have different values on Windows than on macOS or Linux. I dealt with this by just providing manual per-platform definitions for these constants using `#[cfg]` attributes.

Finally, difference #4 was occurring because enum declarations without fixed underlying type default to using `int` on Windows and `unsigned int` elsewhere ([`rust-bindgen`](https://github.com/rust-lang/rust-bindgen), which also uses libclang, suffers from the [same issue](https://github.com/rust-lang/rust-bindgen/issues/1966)). This was the most annoying inconsistency to deal with. In principle, it might have been possible to deal with it entirely automatically, but libclang provides no reliable way to detect whether an enum declaration specifies a fixed underlying type or not. In the end, I dealt with this in the same way as difference #3, by providing manual type definitions for each of these enums.

Despite some annoyances, I'm glad I took this approach. It will be easy to keep the generated bindings up to date with new releases of the SDK, and the CI check should reliably catch any new platform inconstencies as they are introduced.

# Conclusion

Version 0.3.0 of the `vst3` crate is [available on crates.io](https://crates.io/crates/vst3/0.3.0)! If you're currently making use of `vst3` in any projects, I would highly recommend updating for the simplified build process and improved build times. Updating should be as simple as bumping the dependency version and then removing any setup for including the VST 3 SDK.

Additionally, if you're currently using the older [`vst3-sys`](https://github.com/RustAudio/vst3-sys) crate, I would strongly encourage looking into replacing it with `vst3`. `vst3` is not only more permissively licensed (`vst3-sys` is still released under GPLv3), it also contains a more complete and up-to-date set of bindings, and this will likely continue to be the case as its bindings can be regenerated automatically whereas `vst3-sys` must be updated by hand.

Finally, if you run into any issues, questions, or difficulties while working with the `vst3` crate, please feel free to file an issue on the [GitHub repository](https://github.com/coupler-rs/vst3-rs), post in the #vst3 channel in the [Rust Audio Discord](https://discord.gg/8qW6q2k), or start a topic in the #vst3 channel on the [Coupler Zulip](https://coupler.zulipchat.com/).

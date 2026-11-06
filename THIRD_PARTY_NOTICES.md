# Third-party notices

Logpane is distributed under the MIT licence (see `LICENSE`).

This file lists the third-party Rust crates actually resolved for the current feature set, with
the licence expression declared by each crate. It was generated from `cargo metadata --locked`;
regenerate it when dependencies change:

```bash
cargo metadata --format-version 1 --locked > /tmp/meta.json
```

Notes:

- SQLite is compiled from the `rusqlite` `bundled` feature; SQLite itself is in the public
  domain. See <https://sqlite.org/copyright.html>.
- No font file is redistributed with Logpane. The app loads macOS system fonts at runtime (SF Mono or
  Menlo, then Arial Unicode for other scripts); egui's bundled default fonts are disabled. PDF
  exports embed a subset of those fonts in the generated document, which is normal document
  embedding.
- The application icon is not distributed yet; when one is added, its licence must be recorded
  here.

| Crate | Version | Licence | Repository |
|---|---|---|---|
| `accesskit` | 0.25.2 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| `adler2` | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | https://github.com/oyvindln/adler2 |
| `aes` | 0.9.4 | MIT OR Apache-2.0 | https://github.com/RustCrypto/block-ciphers |
| `ahash` | 0.8.12 | MIT OR Apache-2.0 | https://github.com/tkaitchuck/ahash |
| `aho-corasick` | 1.1.5 | Unlicense OR MIT | https://github.com/BurntSushi/aho-corasick |
| `aliasable` | 0.1.3 | MIT | https://github.com/avitex/rust-aliasable |
| `alloc-no-stdlib` | 2.0.4 | BSD-3-Clause | https://github.com/dropbox/rust-alloc-no-stdlib |
| `alloc-stdlib` | 0.2.4 | BSD-3-Clause | https://github.com/dropbox/rust-alloc-no-stdlib |
| `allsorts-azul` | 0.17.2 | Apache-2.0 | https://github.com/fschutt/allsorts-azul |
| `android-activity` | 0.6.1 | MIT OR Apache-2.0 | https://github.com/rust-mobile/android-activity |
| `android-properties` | 0.2.2 | MIT | https://github.com/miklelappo/android-properties |
| `android_system_properties` | 0.1.6 | MIT OR Apache-2.0 | https://github.com/nical/android_system_properties |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 | https://github.com/dtolnay/anyhow |
| `arrayvec` | 0.7.8 | MIT OR Apache-2.0 | https://github.com/bluss/arrayvec |
| `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT | https://github.com/smol-rs/atomic-waker |
| `autocfg` | 1.5.1 | Apache-2.0 OR MIT | https://github.com/cuviper/autocfg |
| `base64` | 0.22.1 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 |
| `bitflags` | 1.3.2 | MIT/Apache-2.0 | https://github.com/bitflags/bitflags |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 | https://github.com/bitflags/bitflags |
| `block-buffer` | 0.12.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| `block-padding` | 0.4.2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| `block2` | 0.5.1 | MIT | https://github.com/madsmtm/objc2 |
| `block2` | 0.6.2 | MIT | https://github.com/madsmtm/objc2 |
| `brotli-decompressor` | 5.0.3 | BSD-3-Clause/MIT | https://github.com/dropbox/rust-brotli-decompressor |
| `bumpalo` | 3.20.3 | MIT OR Apache-2.0 | https://github.com/fitzgen/bumpalo |
| `bytemuck` | 1.25.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| `bytemuck_derive` | 1.12.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| `byteorder-lite` | 0.1.0 | Unlicense OR MIT | https://github.com/image-rs/byteorder-lite |
| `bytes` | 1.12.1 | MIT | https://github.com/tokio-rs/bytes |
| `calloop` | 0.13.0 | MIT | https://github.com/Smithay/calloop |
| `cbc` | 0.2.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/block-modes |
| `cc` | 1.11.0 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/cfg-if |
| `cfg_aliases` | 0.2.2 | MIT | https://github.com/katharostech/cfg_aliases |
| `cgl` | 0.3.2 | MIT / Apache-2.0 | https://github.com/servo/cgl-rs |
| `chacha20` | 0.10.2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/stream-ciphers |
| `chrono` | 0.4.45 | MIT OR Apache-2.0 | https://github.com/chronotope/chrono |
| `cipher` | 0.5.2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| `color` | 0.3.3 | Apache-2.0 OR MIT | https://github.com/linebender/color |
| `combine` | 4.6.9 | MIT | https://github.com/Marwes/combine |
| `concurrent-queue` | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/concurrent-queue |
| `const-oid` | 0.10.2 | Apache-2.0 OR MIT | https://github.com/RustCrypto/formats |
| `core-foundation` | 0.9.4 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core-foundation-sys` | 0.8.7 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core-graphics` | 0.23.2 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core-graphics-types` | 0.1.3 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core_detect` | 1.0.0 | MIT/Apache-2.0 | https://github.com/thomcc/core_detect |
| `cpubits` | 0.1.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| `cpufeatures` | 0.3.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| `crc32fast` | 1.5.4 | MIT OR Apache-2.0 | https://github.com/srijs/rust-crc32fast |
| `crossbeam-channel` | 0.5.17 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| `crossbeam-utils` | 0.8.23 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| `crunchy` | 0.2.4 | MIT | https://github.com/eira-fransham/crunchy |
| `crypto-common` | 0.2.2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| `cursor-icon` | 1.2.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/cursor-icon |
| `deranged` | 0.5.9 | MIT OR Apache-2.0 | https://github.com/jhpratt/deranged |
| `digest` | 0.11.3 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| `directories` | 6.0.0 | MIT OR Apache-2.0 | https://github.com/soc/directories-rs |
| `dirs-sys` | 0.5.0 | MIT OR Apache-2.0 | https://github.com/dirs-dev/dirs-sys-rs |
| `dispatch` | 0.2.0 | MIT | http://github.com/SSheldon/rust-dispatch |
| `dispatch2` | 0.3.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `dlib` | 0.5.3 | MIT | https://github.com/elinorbgr/dlib |
| `document-features` | 0.2.12 | MIT OR Apache-2.0 | https://github.com/slint-ui/document-features |
| `dpi` | 0.1.2 | Apache-2.0 AND MIT | https://github.com/rust-windowing/winit |
| `ecb` | 0.2.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/block-modes |
| `ecolor` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| `eframe` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/eframe |
| `egui` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| `egui-winit` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/egui-winit |
| `egui_glow` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/egui_glow |
| `either` | 1.19.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/either |
| `emath` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/emath |
| `encoding_rs` | 0.8.44 | (Apache-2.0 OR MIT) AND BSD-3-Clause | https://github.com/hsivonen/encoding_rs |
| `enumflags2` | 0.7.12 | MIT OR Apache-2.0 | https://github.com/meithecatte/enumflags2 |
| `enumflags2_derive` | 0.7.12 | MIT OR Apache-2.0 | https://github.com/meithecatte/enumflags2 |
| `epaint` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/epaint |
| `equivalent` | 1.0.3 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/equivalent |
| `errno` | 0.3.14 | MIT OR Apache-2.0 | https://github.com/lambda-fairy/rust-errno |
| `euclid` | 0.22.14 | MIT OR Apache-2.0 | https://github.com/servo/euclid |
| `fallible-iterator` | 0.3.0 | MIT/Apache-2.0 | https://github.com/sfackler/rust-fallible-iterator |
| `fallible-streaming-iterator` | 0.1.9 | MIT/Apache-2.0 | https://github.com/sfackler/fallible-streaming-iterator |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/fastrand |
| `fax` | 0.2.7 | MIT | https://github.com/pdf-rs/fax |
| `fdeflate` | 0.3.7 | MIT OR Apache-2.0 | https://github.com/image-rs/fdeflate |
| `fearless_simd` | 0.4.1 | Apache-2.0 OR MIT | https://github.com/linebender/fearless_simd |
| `find-msvc-tools` | 0.1.15 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs |
| `flate2` | 1.1.10 | MIT OR Apache-2.0 | https://github.com/rust-lang/flate2-rs |
| `foldhash` | 0.2.0 | Zlib | https://github.com/orlp/foldhash |
| `font-types` | 0.12.7 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `foreign-types` | 0.5.0 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types |
| `foreign-types-macros` | 0.2.4 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types |
| `foreign-types-shared` | 0.3.1 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `gethostname` | 1.1.0 | Apache-2.0 | https://codeberg.org/swsnr/gethostname.rs.git |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| `gl_generator` | 0.14.0 | Apache-2.0 | https://github.com/brendanzab/gl-rs/ |
| `glifo` | 0.3.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| `global-hotkey` | 0.9.0 | Apache-2.0 OR MIT | https://github.com/tauri-apps/global-hotkey |
| `glow` | 0.17.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/grovesNL/glow |
| `glutin` | 0.32.3 | Apache-2.0 | https://github.com/rust-windowing/glutin |
| `glutin-winit` | 0.5.0 | MIT | https://github.com/rust-windowing/glutin |
| `glutin_egl_sys` | 0.7.1 | Apache-2.0 | https://github.com/rust-windowing/glutin |
| `glutin_wgl_sys` | 0.6.1 | Apache-2.0 | https://github.com/rust-windowing/glutin |
| `glyph-names` | 0.2.0 | BSD-3-Clause | https://github.com/yeslogic/glyph-names |
| `guillotiere` | 0.7.0 | MIT/Apache-2.0 | https://github.com/nical/guillotiere |
| `half` | 2.7.1 | MIT OR Apache-2.0 | https://github.com/VoidStarKat/half-rs |
| `harfrust` | 0.12.0 | MIT | https://github.com/harfbuzz/harfrust |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| `hashlink` | 0.12.2 | MIT OR Apache-2.0 | https://github.com/djc/hashlink |
| `heck` | 0.4.1 | MIT OR Apache-2.0 | https://github.com/withoutboats/heck |
| `hermit-abi` | 0.5.3 | MIT OR Apache-2.0 | https://github.com/hermit-os/hermit-rs |
| `hybrid-array` | 0.4.15 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hybrid-array |
| `iana-time-zone` | 0.1.65 | MIT OR Apache-2.0 | https://github.com/strawlab/iana-time-zone |
| `iana-time-zone-haiku` | 0.1.2 | MIT OR Apache-2.0 | https://github.com/strawlab/iana-time-zone |
| `image` | 0.25.10 | MIT OR Apache-2.0 | https://github.com/image-rs/image |
| `image-webp` | 0.2.4 | MIT OR Apache-2.0 | https://github.com/image-rs/image-webp |
| `indexmap` | 2.14.2 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/indexmap |
| `inout` | 0.2.5 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| `itertools` | 0.15.0 | MIT OR Apache-2.0 | https://github.com/rust-itertools/itertools |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 | https://github.com/dtolnay/itoa |
| `jni` | 0.22.4 | MIT OR Apache-2.0 | https://github.com/jni-rs/jni-rs |
| `jni-macros` | 0.22.4 | MIT OR Apache-2.0 | https://github.com/jni-rs/jni-rs |
| `jni-sys` | 0.3.1 | MIT OR Apache-2.0 | https://github.com/jni-rs/jni-sys |
| `jni-sys` | 0.4.1 | MIT OR Apache-2.0 | https://github.com/jni-rs/jni-sys |
| `jni-sys-macros` | 0.4.1 | MIT OR Apache-2.0 | https://github.com/jni-rs/jni-sys |
| `jobserver` | 0.1.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/jobserver-rs |
| `js-sys` | 0.3.107 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys |
| `keyboard-types` | 0.8.3 | MIT OR Apache-2.0 | https://github.com/rust-windowing/keyboard-types |
| `khronos_api` | 3.1.0 | Apache-2.0 | https://github.com/brendanzab/gl-rs/ |
| `kurbo` | 0.13.1 | Apache-2.0 OR MIT | https://github.com/linebender/kurbo |
| `lazy_static` | 1.5.1 | MIT OR Apache-2.0 | https://github.com/rust-lang-nursery/lazy-static.rs |
| `libc` | 0.2.191 | MIT OR Apache-2.0 | https://github.com/rust-lang/libc |
| `libloading` | 0.8.9 | ISC | https://github.com/nagisa/rust_libloading/ |
| `libredox` | 0.1.25 | MIT | https://gitlab.redox-os.org/redox-os/libredox.git |
| `libsqlite3-sys` | 0.39.1 | MIT | https://github.com/rusqlite/rusqlite |
| `linebender_resource_handle` | 0.1.1 | Apache-2.0 OR MIT | https://github.com/linebender/raw_resource_handle |
| `linux-raw-sys` | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/sunfishcode/linux-raw-sys |
| `linux-raw-sys` | 0.4.15 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/sunfishcode/linux-raw-sys |
| `litrs` | 1.0.1 | MIT OR Apache-2.0 | https://github.com/LukasKalbertodt/litrs |
| `lock_api` | 0.4.14 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| `log` | 0.4.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/log |
| `lopdf` | 0.44.0 | MIT | https://github.com/J-F-Liu/lopdf.git |
| `matchers` | 0.2.0 | MIT | https://github.com/hawkw/matchers |
| `md-5` | 0.11.0 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| `memchr` | 2.8.3 | Unlicense OR MIT | https://github.com/BurntSushi/memchr |
| `miniz_oxide` | 0.8.9 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| `miniz_oxide` | 0.9.1 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| `moxcms` | 0.8.1 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/moxcms.git |
| `multiversion_no_op` | 1.0.0 | Apache-2.0 OR MIT | https://github.com/hsivonen/multiversion_no_op |
| `ndk` | 0.9.0 | MIT OR Apache-2.0 | https://github.com/rust-mobile/ndk |
| `ndk-context` | 0.1.1 | MIT OR Apache-2.0 | https://github.com/rust-windowing/android-ndk-rs |
| `ndk-sys` | 0.6.0+11769913 | MIT OR Apache-2.0 | https://github.com/rust-mobile/ndk |
| `nohash-hasher` | 0.2.0 | Apache-2.0 OR MIT | https://github.com/paritytech/nohash-hasher |
| `nom` | 8.0.0 | MIT | https://github.com/rust-bakery/nom |
| `nu-ansi-term` | 0.50.3 | MIT | https://github.com/nushell/nu-ansi-term |
| `num-conv` | 0.2.2 | MIT OR Apache-2.0 | https://github.com/jhpratt/num-conv |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 | https://github.com/rust-num/num-traits |
| `num_enum` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | https://github.com/illicitonion/num_enum |
| `num_enum_derive` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | https://github.com/illicitonion/num_enum |
| `objc-sys` | 0.3.5 | MIT | https://github.com/madsmtm/objc2 |
| `objc2` | 0.5.3 | MIT | https://github.com/madsmtm/objc2 |
| `objc2` | 0.6.5 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-app-kit` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-app-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-cloud-kit` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-cloud-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-contacts` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-data` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-data` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-foundation` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-graphics` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-image` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-image` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-location` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-text` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-video` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-encode` | 4.1.0 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-foundation` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-foundation` | 0.3.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-io-surface` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-link-presentation` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-metal` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-quartz-core` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-quartz-core` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-symbols` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-ui-kit` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-ui-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-uniform-type-identifiers` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-user-notifications` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 | https://github.com/matklad/once_cell |
| `option-ext` | 0.2.0 | MPL-2.0 | https://github.com/soc/option-ext.git |
| `orbclient` | 0.3.55 | MIT | https://gitlab.redox-os.org/redox-os/orbclient |
| `ouroboros` | 0.18.5 | MIT OR Apache-2.0 | https://github.com/someguynamedjosh/ouroboros |
| `ouroboros_macro` | 0.18.5 | MIT OR Apache-2.0 | https://github.com/someguynamedjosh/ouroboros |
| `parking_lot` | 0.12.5 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| `parking_lot_core` | 0.9.12 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| `pathfinder_geometry` | 0.5.1 | MIT/Apache-2.0 | https://github.com/servo/pathfinder |
| `pathfinder_simd` | 0.5.6 | MIT OR Apache-2.0 | https://github.com/servo/pathfinder |
| `peniko` | 0.6.1 | Apache-2.0 OR MIT | https://github.com/linebender/peniko |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ |
| `pin-project` | 1.1.13 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project |
| `pin-project-internal` | 1.1.13 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project-lite |
| `pkg-config` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/pkg-config-rs |
| `plain` | 0.2.3 | MIT/Apache-2.0 | https://github.com/randomites/plain |
| `png` | 0.18.1 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png |
| `polling` | 3.11.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/polling |
| `polycool` | 0.4.0 | MIT OR Apache-2.0 | https://github.com/linebender/kurbo |
| `powerfmt` | 0.2.1 | MIT OR Apache-2.0 | https://github.com/jhpratt/powerfmt |
| `printpdf` | 0.12.8 | MIT | https://github.com/fschutt/printpdf |
| `proc-macro-crate` | 3.5.0 | MIT OR Apache-2.0 | https://github.com/bkchr/proc-macro-crate |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2 |
| `proc-macro2-diagnostics` | 0.10.1 | MIT/Apache-2.0 | https://github.com/SergioBenitez/proc-macro2-diagnostics |
| `profiling` | 1.0.18 | MIT OR Apache-2.0 | https://github.com/aclysma/profiling |
| `pxfm` | 0.1.30 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/pxfm |
| `quick-error` | 2.0.1 | MIT/Apache-2.0 | http://github.com/tailhook/quick-error |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote |
| `r-efi` | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | https://github.com/r-efi/r-efi |
| `rand` | 0.10.3 | MIT OR Apache-2.0 | https://github.com/rust-random/rand |
| `rand_core` | 0.10.1 | MIT OR Apache-2.0 | https://github.com/rust-random/rand_core |
| `rangemap` | 1.8.0 | MIT/Apache-2.0 | https://github.com/jeffparsons/rangemap |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/raw-window-handle |
| `read-fonts` | 0.41.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `redox_syscall` | 0.4.1 | MIT | https://gitlab.redox-os.org/redox-os/syscall |
| `redox_syscall` | 0.5.18 | MIT | https://gitlab.redox-os.org/redox-os/syscall |
| `redox_syscall` | 0.9.5 | MIT | https://gitlab.redox-os.org/redox-os/kernel |
| `redox_users` | 0.5.3 | MIT | https://gitlab.redox-os.org/redox-os/users |
| `regex-automata` | 0.4.18 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| `regex-syntax` | 0.8.11 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| `rsqlite-vfs` | 0.2.0 | MIT | https://github.com/Spxg/sqlite-wasm-rs |
| `rusqlite` | 0.41.1 | MIT | https://github.com/rusqlite/rusqlite |
| `rustc-hash` | 2.1.3 | Apache-2.0 OR MIT | https://github.com/rust-lang/rustc-hash |
| `rustc_version` | 0.4.1 | MIT OR Apache-2.0 | https://github.com/djc/rustc-version-rs |
| `rustix` | 0.38.44 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/rustix |
| `rustix` | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/rustix |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 | https://github.com/dtolnay/rustversion |
| `same-file` | 1.0.6 | Unlicense/MIT | https://github.com/BurntSushi/same-file |
| `scopeguard` | 1.2.0 | MIT OR Apache-2.0 | https://github.com/bluss/scopeguard |
| `self_cell` | 1.3.0 | Apache-2.0 OR GPL-2.0-only | https://github.com/Voultapher/self_cell |
| `semver` | 1.0.28 | MIT OR Apache-2.0 | https://github.com/dtolnay/semver |
| `send_wrapper` | 0.6.0 | MIT/Apache-2.0 | https://github.com/thk1/send_wrapper |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | https://github.com/serde-rs/json |
| `sha2` | 0.11.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| `sharded-slab` | 0.1.7 | MIT | https://github.com/hawkw/sharded-slab |
| `shlex` | 2.0.1 | MIT OR Apache-2.0 | https://github.com/comex/rust-shlex |
| `simd-adler32` | 0.3.10 | MIT | https://github.com/mcountryman/simd-adler32 |
| `simd_cesu8` | 1.2.0 | Apache-2.0 OR MIT | https://github.com/seancroach/simd_cesu8 |
| `simdutf8` | 0.1.5 | MIT OR Apache-2.0 | https://github.com/rusticstuff/simdutf8 |
| `skrifa` | 0.44.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `slab` | 0.4.12 | MIT | https://github.com/tokio-rs/slab |
| `slotmap` | 1.1.1 | Zlib | https://github.com/orlp/slotmap |
| `smallvec` | 1.16.4 | MIT OR Apache-2.0 | https://github.com/servo/rust-smallvec |
| `smol_str` | 0.2.2 | MIT OR Apache-2.0 | https://github.com/rust-analyzer/smol_str |
| `sqlite-wasm-rs` | 0.6.2 | MIT | https://github.com/Spxg/sqlite-wasm-rs |
| `static_assertions` | 1.1.0 | MIT OR Apache-2.0 | https://github.com/nvzqz/static-assertions-rs |
| `stringprep` | 0.1.5 | MIT/Apache-2.0 | https://github.com/sfackler/rust-stringprep |
| `syn` | 2.0.119 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| `syn` | 3.0.9 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | https://github.com/Stebalien/tempfile |
| `thiserror` | 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `thiserror-impl` | 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `thiserror-impl` | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `thread_local` | 1.1.10 | MIT OR Apache-2.0 | https://github.com/Amanieu/thread_local-rs |
| `tiff` | 0.11.3 | MIT | https://github.com/image-rs/image-tiff |
| `time` | 0.3.55 | MIT OR Apache-2.0 | https://github.com/time-rs/time |
| `time-core` | 0.1.9 | MIT OR Apache-2.0 | https://github.com/time-rs/time |
| `tinyvec` | 1.13.3 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/tinyvec |
| `tokio` | 1.54.1 | MIT | https://github.com/tokio-rs/tokio |
| `toml_datetime` | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `toml_edit` | 0.25.18+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `toml_parser` | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `tracing` | 0.1.44 | MIT | https://github.com/tokio-rs/tracing |
| `tracing-attributes` | 0.1.31 | MIT | https://github.com/tokio-rs/tracing |
| `tracing-core` | 0.1.36 | MIT | https://github.com/tokio-rs/tracing |
| `tracing-log` | 0.2.0 | MIT | https://github.com/tokio-rs/tracing |
| `tracing-subscriber` | 0.3.23 | MIT | https://github.com/tokio-rs/tracing |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 | https://github.com/paholg/typenum |
| `ucd-trie` | 0.1.7 | MIT OR Apache-2.0 | https://github.com/BurntSushi/ucd-generate |
| `unicode-bidi` | 0.3.18 | MIT OR Apache-2.0 | https://github.com/servo/unicode-bidi |
| `unicode-canonical-combining-class` | 1.0.0 | Apache-2.0 | https://github.com/yeslogic/unicode-canonical-combining-class |
| `unicode-general-category` | 1.1.0 | Apache-2.0 | https://github.com/yeslogic/unicode-general-category |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident |
| `unicode-joining-type` | 1.0.0 | Apache-2.0 | https://github.com/yeslogic/unicode-joining-type |
| `unicode-normalization` | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/unicode-rs/unicode-normalization |
| `unicode-properties` | 0.1.5 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/unicode-rs/unicode-properties |
| `unicode-segmentation` | 1.13.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-segmentation |
| `uuid` | 1.29.0 | Apache-2.0 OR MIT | https://github.com/uuid-rs/uuid |
| `valuable` | 0.1.1 | MIT | https://github.com/tokio-rs/valuable |
| `vcpkg` | 0.2.15 | MIT/Apache-2.0 | https://github.com/mcgoo/vcpkg-rs |
| `vello_common` | 0.2.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| `vello_cpu` | 0.2.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| `version_check` | 0.9.5 | MIT/Apache-2.0 | https://github.com/SergioBenitez/version_check |
| `walkdir` | 2.5.0 | Unlicense/MIT | https://github.com/BurntSushi/walkdir |
| `wasm-bindgen` | 0.2.130 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen |
| `wasm-bindgen-futures` | 0.4.80 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures |
| `wasm-bindgen-macro` | 0.2.130 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro |
| `wasm-bindgen-macro-support` | 0.2.130 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support |
| `wasm-bindgen-shared` | 0.2.130 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared |
| `web-sys` | 0.3.107 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys |
| `web-time` | 1.1.0 | MIT OR Apache-2.0 | https://github.com/daxpedda/web-time |
| `weezl` | 0.1.12 | MIT OR Apache-2.0 | https://github.com/image-rs/weezl |
| `weezl` | 0.2.1 | MIT OR Apache-2.0 | https://github.com/image-rs/weezl |
| `winapi-util` | 0.1.11 | Unlicense OR MIT | https://github.com/BurntSushi/winapi-util |
| `windows-core` | 0.62.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-implement` | 0.60.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-interface` | 0.59.3 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-result` | 0.4.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-strings` | 0.5.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-sys` | 0.52.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-sys` | 0.59.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-targets` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_aarch64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_aarch64_msvc` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_i686_gnu` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_i686_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_i686_msvc` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_x86_64_gnu` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_x86_64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_x86_64_msvc` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `winit` | 0.30.13 | Apache-2.0 | https://github.com/rust-windowing/winit |
| `winnow` | 1.0.4 | MIT | https://github.com/winnow-rs/winnow |
| `x11rb` | 0.13.2 | MIT OR Apache-2.0 | https://github.com/psychon/x11rb |
| `x11rb-protocol` | 0.13.2 | MIT OR Apache-2.0 | https://github.com/psychon/x11rb |
| `xkbcommon-dl` | 0.4.2 | MIT | https://github.com/rust-windowing/xkbcommon-dl |
| `xkeysym` | 0.2.1 | MIT OR Apache-2.0 OR Zlib | https://github.com/notgull/xkeysym |
| `xml-rs` | 0.8.29 | MIT | https://github.com/kornelski/xml-rs |
| `yansi` | 1.0.1 | MIT OR Apache-2.0 | https://github.com/SergioBenitez/yansi |
| `zerocopy` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy |
| `zerocopy-derive` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy |
| `zlib-rs` | 0.6.8 | Zlib | https://github.com/trifectatechfoundation/zlib-rs |
| `zmij` | 1.0.23 | MIT | https://github.com/dtolnay/zmij |
| `zune-core` | 0.5.3 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image |
| `zune-jpeg` | 0.5.15 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg |

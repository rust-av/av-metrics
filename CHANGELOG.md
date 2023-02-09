## decoder Version 0.2.1

- Fixes to ffmpeg decoder frame accuracy
- Migrate to forked ffmpeg crate that is maintained

## decoder Version 0.2.0

- Add Vapoursynth decoder, available with `vapoursynth` cargo feature

## decoder Version 0.1.10

- Fix a bug where certain codecs may not return the final frame

## decoder Version 0.1.9

- Include libdav1d with ffmpeg_build feature
- Add YUVJ support

## Version 0.9.0

- Bump dependencies, including `v_frame` to `0.3`
- Minor fixes

## Version 0.8.2

- Export `Frame` from the decoders crate as well
- Bump dependencies

## Version 0.8.1

- [Breaking] Remove the `FrameInfo` struct and use `v_frame::Frame` directly
- Migrate to edition 2021
- Upgrade `clap` to `3.x`
- Upgrade `ffmpeg-next` to `5.x`

## Version 0.7.2

- Add a new GUI version of the av-metrics-tool
- Bump some dependencies

## Version 0.7.1

- Add ffmpeg decoding support; this is optional and currently requires building from source
- Fix a math overflow on 32-bit for MSSSIM
- Publish a new av_metrics_decoder crate, to use the y4m and ffmpeg decoders independently.
  These are re-exported through av-metrics, so the av-metrics interface is unchanged.
- Add progress indicator
- Remove internal unwraps (enables cleaner exiting, especially when used as a crate)
- Improve error messages

## Version 0.7.0

- [CLI Feature] Support multiple file comparison
- [CLI Feature] Add Markdown output
- [CLI Feature] Add CSV output

## Version 0.6.0

- [Breaking] Simplify the `Decoder` trait
- [Breaking] Require `Send + Sync` on the `Decoder` trait
- Many performance and multi-threading improvements

## Version 0.5.1

- Remove unneeded library specifiers that were previously needed by cargo-c

## Version 0.5.0

- Bump y4m dependency to 0.6

## Version 0.4.0

- Breaking Change: Use Frame, Pixel, etc. types from the `v_frame` crate,
  instead of rolling our own. This should improve interoperability
  with other crates.
- Speed up y4m decoding.
- Minor internal changes and dependency updates.

## Version 0.3.0

- Breaking Change: Remove the `use_simd` flag from the public API.
  This was intended only for development purposes,
  and isn't generally useful to end users.
  If you really want to disable SIMD,
  you can set the environment variable `AV_METRICS_DISABLE_SIMD` to any value.
- Breaking Change: PSNR and APSNR have been split into separate result sets.
  This only impacts users of the API.
  The CLI output is identical.
- New Feature: `--metric` flag allows the CLI tool to output only one metric at a time,
  instead of all metrics.
- New Feature: `--json` flag allows the CLI tool to output the results as JSON.
  This is primarily useful if you want to use the output in some other script
  or as data on a web page.
- 25% speed improvement in CIEDE2000 with AVX2.

## Version 0.2.1

- Fix a bug where CIEDE2000 could report Infinity
- Performance improvements to PSNR-HSV metric

## Version 0.2.0

- Add a binary for running metrics on y4m files (and other future formats)
  - This binary can be installed from the `av-metrics-tool` crate.
- Breaking change: `Decoder<T>` is changed to `Decoder` and has a new method,
  `get_bit_depth` added. This allows us to dynamically dispatch to the correct
  version of a metric based on the bit depth, without the compiler getting
  in our way.
- Add a workspace for managing the library and binary independently.

## Version 0.1.0

- Initial release

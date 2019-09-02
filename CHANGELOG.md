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
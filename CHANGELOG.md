# RBuild - A Rust-based build system
**NOTE**: Dates are in dd-mm-yyyy format.

## vNext ()
- Adds the Target struct, which corresponds to a path in the rbuild file.
- Adds a basic command-line utility that accepts the rbuild file and targets as arguments.
- RBuild now ignores leading and trailing whitespace in values in the configuration file.
- Adds functionality for reading and writing to a hash cache, so that commands are rerun when the rbuild file is modified.
- Migrates command-line parsing to RgParse.
- Adds a `--threads` option that allows the user to specify the maximum number of parallel jobs.
- Adds a `--cache` option for specifying the path to the hash cache.
- Adds a `--verbose` option that prints out commands before running them.

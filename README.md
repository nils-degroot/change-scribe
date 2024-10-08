# change-scribe

`change-scribe` is a tool that validates that commit messages follow the
conventional commit format, and lints them according to a configuration file.

## Installation

Installation is done via `cargo`:

```sh
cargo install change-scribe
```

To build change-scribe from source, clone the repository and run:

```sh
cargo build --release
```

The binary will be located at `target/release/change-scribe`.

## Usage

To use `change-scribe`:

```sh
change-scribe lint "fix: resolved that bug"
```

This validates the message according to the default configuration. The commit
message could also be passed via stdin:

```sh
echo "fix: resolved that bug" | change-scribe lint -
```

To apply a custom config, use the `--config` flag:

```sh
change-scribe lint --config path/to/config.toml "fix: resolved that bug"
```

By default, `change-scribe` reads configuration from either
`change-scribe.toml` or `.change-scribe.toml` in the current directory.

## Linting rules

### Type

#### `type.enum`

Ensures that the commit type is on of the entered values. `*` is a wildcard
that matches any type.

**Default**:

```toml
type.enum = ["*"]
```

#### `type.min-length`

Ensures that the commit type is at least the entered length.

**Default**:

```toml
type.min-length = 0
```

#### `type.max-length`

Ensures that the commit type is at most the entered length.

**Default**:

```toml
type.max-length = 4294967295
```

#### `type.case`

Ensures that the commit type is in the entered case. Possible values are
`camel`, `kebab`, `pascal` and `snake`.

**Default**:

```toml
type.case = "kebabtype.case = "kebab""
```

### Scope

#### `scope.enum`

Ensures that the commit scope is on of the entered values. `*` is a wildcard
that matches any scope.

**Default**:

```toml
scope.enum = ["*"]
```

#### `scope.required`

Ensures that a commit has a scope.

**Default**:

```toml
scope.required = false
```

#### `scope.min-length`

Ensures that the commit scope is at least the entered length.

**Default**:

```toml
scope.min-length = 0
```

#### `scope.max-length`

Ensures that the commit scope is at most the entered length.

**Default**:

```toml
scope.max-length = 4294967295
```

#### `scope.case`

Ensures that the commit scope is in the entered case. Possible values are
`camel`, `kebab`, `pascal` and `snake`.

**Default**:

```toml
type.case = "kebab"
```


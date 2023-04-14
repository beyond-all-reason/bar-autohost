# bar-autohost

An autohost for [Beyond-All-Reason](https://github.com/beyond-all-reason/Beyond-All-Reason)
written in rust. It's currently being developed in linux but the code is cross platform
and could also potentially run on other platforms, though this hasn't been tested and
some third party dependencies may need to be installed.

## Development Quick Start

- Install [rust](https://www.rust-lang.org/tools/install) on your machine.
- Install and configure an IDE you're comfortable with.
- Clone this repository.
- Install the `Beyond-All-Reason` flatpak application. It can be installed by other means
but that hasn't been tested with any other packaged installations.
- Copy

## Configuration

Can be done by setting environment variables or via a `config.toml` in the same directory
as the executable when deployed or at the root of the working copy during development.
**NOTE** Please do **NOT** place the `config.toml` under source control as it could
contain sensitive data like credentials.

The configuration is set up in [config.rs](src/config.rs).

### Environment Variables

Environment variables need the `BAR_` prefix and the variable names should match their
counterparts in the [config.rs](src/config.rs).

Eg:

```rust
// In config.rs
//...
pub struct AutohostConfig {
    spring_relative_path: String,
}
//...
```

```bash
BAR_VARIABLE_IN_CAPS
```

### Config Toml

The field names in the config file should match their
counterparts in the [config.rs](src/config.rs).

Eg:

```rust
// In config.rs
//...
pub struct AutohostConfig {
    spring_relative_path: String,
}
//...
```

```toml
spring_relative_path = "/my/path/here"
```

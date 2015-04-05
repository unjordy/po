po
==

`po` is a third-party (*not affiliated with Superblock, LLC*) Rust API and command-line notification pusher for the **Pushover** push notification service.

libpo, the Rust API, safely wraps the Pushover REST API and tries to handle all corner cases and error conditions, for reliable push message sending to any Pushover client device (iOS, Android, desktop, Open Client). It also supports linking message bodies too long for Pushover as supplementary URLs sent with the rest of the notification.

`po` is a utility that uses libpo to provide a multi-platform, POSIX-y, command-line interface for push notification sending. It exposes all of the features of libpo, and also supports API token and user key storage. It can receive message body input as a command line argument, or from standard input.

## Compiling
You will need libcurl; everything else is pulled in by Cargo.
Run `cargo build` to compile both the libpo library and the `po` command-line pusher; they will each be in the `target/debug` subdirectory. Run `cargo build --release` to compile optimized builds of each, which end up in `target/release`. If you want, copy `po` to someplace in your path.

## Using `po`
First, run `po --setup` to receive instructions on how to store a Pushover API token and user key for use by the command-line client.

To push a simple message with its title set to the hostname of the sending computer:

```po --title `hostname` "Hello"```

To do the same but with the message body taken from standard input:

```echo Hello | po --title `hostname` ```

To push the output of `ls -la` and link the full output as a Gist if it exceeds Pushover's maximum length (1024 characters):

```ls -la | po --gist```

## Using libpo

Add `po = "*"` to the `[dependencies]` section of your project's Cargo.toml.

Use `po::send_basic` to send a basic message with API token, user key, and message body strings. `po::send` supports more complicated messages with optional parameters, and `po::send_gist` posts the full text of each message to GitHub Gist and links it as a supplementary URL (useful if messages exceed the maximum Pushover length).

For a more flexible (and simpler if you have many parameters) API, use `po::push`. This function accepts strings representing an API token, user key, and message body string just like the `po::send_*` family of functions, but accepts as its last parameter an array of `po::Parameters` types representing every optional parameter supported by the library.

libpo also exposes a `po::gist` function which uploads a string to GitHub Gist with an optional title.

## Todo
* More complete error handling
* Make emergency priority messaging actually work
* Verify the stored API token and user key
* Retry sending the notification if a transient error is encountered
* Markdown output support for Gists
* Asynchronous sending for `po` via daemonization
* More testing

## To be done as Rust features stabilize
* Use the tee trait to pipe input to output as it's received
* Re-add proper exit statuses
* Work on config file read/write error reporting
* Better error handling for config file/directory creation
* Actually use semantic versions for dependencies as they leave 0.*.*

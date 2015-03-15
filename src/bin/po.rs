#![feature(core)]
#![feature(io)]
#![feature(path_ext)]
#![feature(exit_status)]

extern crate po;
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use std::io::prelude::*;
use std::path::Path;

static USAGE: &'static str = "
Usage: po [options]
       po [options] <message>
       po --setup <token> <user>
       po --setup

Options:
    -h, --help                      Display this information.
    --setup                         Setup po with a given Pushover API token
                                    and user key. If neither are provided,
                                    then --setup prints setup instructions.
    -t <title>, --title <title>     The title to give the notification.
    -p <priority>                   A priority for the notification,
                                    from -2 to 2 [default: 0].
    -d <device>, --device <device>  Specify which device should receive
                                    the notification.
    -s <sound>, --sound <sound>     Specify a notification sound from the API
                                    sound list.
    -g, --gist                      If the message is too long to send
                                    (>1024 bytes), then upload it to GitHub
                                    Gist and link it in the notification.
    --always-gist                   Always upload the message to GitHub Gist
                                    and link it in the notification.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_message: Option<String>,
    arg_token: String,
    arg_user: String,
    flag_p: i8,
    flag_title: Option<String>,
    flag_device: Option<String>,
    flag_sound: Option<String>,
    flag_setup: bool,
    flag_gist: bool,
    flag_always_gist: bool
}

fn optional_arg<'a>(arg:  &'a Option<String>) -> Option<&'a str> {
    match *arg {
        Some(ref s) => Some(s.as_slice()),
        None => None
    }
}

fn send(token: &str, user: &str, message: &str, priority: i8,
        title: Option<&str>, device: Option<&str>, sound: Option<&str>,
        gist: bool) {
    if gist {
        match po::send_gist(token, user, message, priority, title,
                            device, sound) {
            Ok(()) => {},
            Err(errors) => {
                println!("po: {:?}", errors);
                std::env::set_exit_status(1)
            }
        }
    }
    else {
        match po::send(token, user, message, priority, title, device, sound) {
            Ok(()) => {},
            Err(errors) => {
                println!("po: {:?}", errors);
                std::env::set_exit_status(1)
            }
        }
    }
}

fn setup(config: &Path, token: &str, user: &str) {
    match po::config::write(token, user, config) {
        Ok(()) => {},
        Err(po::config::WriteError::InvalidApiToken(s)) => {
            println!("Invalid API token {}", s);
            std::env::set_exit_status(2)
        },
        Err(po::config::WriteError::InvalidUserKey(s)) => {
            println!("Invalid user key {}", s);
            std::env::set_exit_status(2)
        },
        Err(po::config::WriteError::FileError(e)) => {
            println!("Config write error: {}", e.to_string());
            std::env::set_exit_status(1)
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                        .and_then(|d| d.decode())
                        .unwrap_or_else(|e| e.exit());
    let mut config_path = std::env::home_dir().unwrap();
    config_path.push(".config");
    if !config_path.exists() {
        std::fs::create_dir(&config_path).unwrap();
    }
    config_path.push("po");
    if !config_path.exists() {
        std::fs::create_dir(&config_path).unwrap();
    }
    config_path.push("tokens");
    config_path.set_extension("json");

    if args.flag_setup {
        if args.arg_token == "" || args.arg_user == "" {
            println!("
To setup po, you'll need a Pushover API token and a user key. First,
sign up for a Pushover account (if you don't already have one) at
https://pushover.net/login. Then, get an API token at
https://pushover.net/apps/build. You can set the application name to
something like the hostname of the computer that will be sending
notifications, or use the default. Next, access your account dashboard
at https://pushover.net to get your user key. Finally, run the command:
`po --setup <API token> <user key>`");
        }
        else {
            setup(&config_path, args.arg_token.as_slice(), args.arg_user.as_slice());
        }
        return;
    }

    let config = po::config::read(&config_path);

    if config == Err(po::config::ReadError::NoConfig) {
        println!("po: Please run po --setup to configure your Pushover API token & user key.");
        std::env::set_exit_status(2);
    }
    else if let Err(e) = config {
        println!("po: Config read error: {:?}", e);
        std::env::set_exit_status(1);
    }
    else if let Some(message) = args.arg_message {
        let (token, user) = config.unwrap();
        let gist = (args.flag_gist && message.len() > 1024) ||
            args.flag_always_gist;

        send(token.as_slice(),
             user.as_slice(),
             message.as_slice(),
             args.flag_p,
             optional_arg(&args.flag_title),
             optional_arg(&args.flag_device),
             optional_arg(&args.flag_sound),
             gist);
    }
    else {
        let (token, user) = config.unwrap();
        let mut input = std::io::stdin().tee(std::io::stdout());
        let mut message = String::new();

        input.read_to_string(&mut message).unwrap();
        let gist = (args.flag_gist && message.len() > 1024) ||
            args.flag_always_gist;

        send(token.as_slice(),
             user.as_slice(),
             message.as_slice(),
             args.flag_p,
             optional_arg(&args.flag_title),
             optional_arg(&args.flag_device),
             optional_arg(&args.flag_sound),
             gist);
    }
}

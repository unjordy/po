extern crate po;
extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use std::io::prelude::*;
use std::path::Path;
use po::Parameters;

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
    --debug                         Print debugging information.
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
    flag_always_gist: bool,
    flag_debug: bool
}

// Consume our arguments struct and produce a vector of Parameters for our
// po send function
fn parse_parameters(args: Args) -> Vec<Parameters> {
    let mut parameters: Vec<Parameters> = Vec::new();

    if args.flag_p != 0 {
        parameters.push(Parameters::Priority(args.flag_p));
    }
    if let Some(title) = args.flag_title {
        parameters.push(Parameters::Title(title));
    }
    if let Some(device) = args.flag_device {
        parameters.push(Parameters::Device(device));
    }
    if let Some(sound) = args.flag_sound {
        parameters.push(Parameters::Sound(sound));
    }
    if args.flag_always_gist {
        parameters.push(Parameters::Gist);
    }
    if args.flag_debug {
        parameters.push(Parameters::Debug);
    }
    parameters
}

fn setup(config: &Path, token: &str, user: &str) {
    match po::config::write(token, user, config) {
        Ok(()) => {},
        Err(po::config::WriteError::InvalidApiToken(s)) => {
            println!("Invalid API token {}", s);
            // TODO: setting exit status isn't stable yet
            // std::env::set_exit_status(2)
        },
        Err(po::config::WriteError::InvalidUserKey(s)) => {
            println!("Invalid user key {}", s);
            // TODO: setting exit status isn't stable yet
            // std::env::set_exit_status(2)
        },
        Err(po::config::WriteError::FileError) => {
            println!("Config write error");
            // TODO: setting exit status isn't stable yet
            // std::env::set_exit_status(1)
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                        .and_then(|d| d.decode())
                        .unwrap_or_else(|e| e.exit());
    let mut config_path = std::env::home_dir().unwrap();
    config_path.push(".config");
    std::fs::create_dir(&config_path).unwrap_or_else(|_| ());
    config_path.push("po");
    std::fs::create_dir(&config_path).unwrap_or_else(|_| ());
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
            setup(&config_path, args.arg_token.as_ref(), args.arg_user.as_ref());
        }
        return;
    }

    let config = po::config::read(&config_path);

    if config == Err(po::config::ReadError::NoConfig) {
        println!("po: Please run po --setup to configure your Pushover API token & user key.");
        // TODO: setting exit status isn't stable yet
        // std::env::set_exit_status(2);
    }
    else if let Err(e) = config {
        println!("po: Config read error: {:?}", e);
        // TODO: setting exit status isn't stable yet
        // std::env::set_exit_status(1);
    }
    else if let Some(message) = args.arg_message.clone() {
        let (token, user) = config.unwrap();
        let arg_gist = args.flag_gist;
        let mut parameters = parse_parameters(args);
        if arg_gist && message.len() > 1024 {
            parameters.push(Parameters::Gist);
        }

        match po::push(token.as_ref(),
                       user.as_ref(),
                       message.as_ref(),
                       parameters.as_ref()) {
            Ok(()) => {},
            Err(errors) => {
                println!("po: {:?}", errors);
                // TODO: setting exit status isn't stable yet
                // std::env::set_exit_status(1);
            }
        }
    }
    else {
        let (token, user) = config.unwrap();
        let mut input = std::io::stdin();
        let mut message = String::new();

        input.read_to_string(&mut message).unwrap();
        print!("{}", message); // TODO: use tee instead when that stabilizes
        let arg_gist = args.flag_gist;
        let mut parameters = parse_parameters(args);
        if arg_gist && message.len() > 1024 {
            parameters.push(Parameters::Gist);
        }

        match po::push(token.as_ref(),
                       user.as_ref(),
                       message.as_ref(),
                       parameters.as_ref()) {
            Ok(()) => {},
            Err(errors) => {
                println!("po: {:?}", errors);
                // TODO: setting exit status isn't stable yet
                // std::env::set_exit_status(1);
            }
        }
    }
}

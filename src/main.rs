use std::cmp::{min};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use discord::Discord;
use discord::model::{ChannelId};
use clap::{Parser, CommandFactory};

/// Simple Discord script to automatically generate mass invites
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// User token: https://www.online-tech-tips.com/computer-tips/what-is-a-discord-token-and-how-to-get-one/
    #[clap(short, long)]
    token: String,

    /// Delete all non-accepted invites contained in the specified file
    #[clap(short, long)]
    delete: Option<String>,

    /// Invite channel ID.
    #[clap(short, long)]
    id: u64,

    /// Amount of invites to generate (max = 1000).
    #[clap(short, long, default_value_t = 1000)]
    amount: u16,

    /// Invite lifetime in seconds (0 = unlimited, max = 604800).
    #[clap(short, long, default_value_t = 604800)]
    lifetime: u32,

    /// Max uses per invite (0 = unlimited, max = 100).
    #[clap(short, long, default_value_t = 1)]
    max_uses: u8,

    /// Number of operations per batch.
    #[clap(short, long, default_value_t = 5)]
    batch_size: u8,

    /// Seconds to wait between batches of operations.
    #[clap(short, long, default_value_t = 15)]
    timeout: u8
}

struct RateLimit {
    timeout: u8,
    batch_size: u8
}

impl RateLimit {
    fn execute<T, F>(&self, args: Vec<T>, mut op: F) where F: FnMut(T){
        let mut rate_limiter = 0;
        for arg in args {
            rate_limiter += 1;
            op(arg);
            rate_limiter %= self.batch_size;
            if rate_limiter == 0 {
                sleep(Duration::from_secs(self.timeout as u64));
            }
        }
    }
}

fn main() {
    let _ = Args::command().term_width(0).get_matches();
    let args = Args::parse();
    let Args { token, delete, id, amount, lifetime, max_uses, timeout, batch_size } = args;

    let rate_limit = RateLimit { timeout, batch_size };

    let channel = ChannelId(id);
    let count = min(amount, 100);

    let ds = Discord::from_user_token(&token).unwrap();

    println!();

    match delete {
        Some(d) => delete_invites(ds, d, channel, rate_limit),
        None => create_invites(ds, count, id, lifetime, max_uses, rate_limit)
    }
}

fn delete_invites(ds: Discord, invites_file: String, channel: ChannelId, rate_limit: RateLimit) {
    let invite_codes: Vec<String> = match invites_file.as_str() {
        "" => {
            println!("No invite file specified, deleting all channel invites");
            ds.get_channel_invites(channel).expect("Error getting all channel invites to delete").iter().map(|ri| ri.code.clone()).collect()
        },
        _ => BufReader::new(File::open(invites_file).expect("Error opening file with invites to delete")).lines().into_iter().map(|l| l.unwrap()).collect()
    };
    rate_limit.execute(invite_codes, |code| {
        if let Err(e) = ds.delete_invite(&code) {
            println!("Error while deleting invite {code}, continuing with next: {e}");
        } else {
            println!("Successfully deleted {code}.");
        }
    })
}

fn create_invites(ds: Discord, max_invites: u16, id: u64, max_age: u32, max_uses: u8, rate_limit: RateLimit) {
    let file_name = format!("invites_{:?}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()).to_owned();
    let mut file = OpenOptions::new().append(true).create(true).open(file_name).unwrap();
    rate_limit.execute((1..=max_invites).into_iter().collect(), move |invite_count| {
        println!("{invite_count} / {max_invites}");
        let code = ds.create_invite(ChannelId(id), max_age, max_uses, false, true).unwrap().code;
        let invite = format!("discord.gg/{code}\n");
        println!("{invite}");
        file.write_all(invite.as_bytes()).unwrap();
    });
}

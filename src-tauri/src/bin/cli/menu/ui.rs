use std::io::{self, Write};

use futures::Future;

pub fn print_frame(title: &str, messages_to_print: Vec<String>, emojis_in_title: bool) {
    let gap_length = if emojis_in_title {
        title.len()
    } else {
        title.len() + 2
    };
    let one_side_of_dashes = (0..12).map(|_| "-").collect::<String>();
    let fill_gap_from_title = (0..gap_length).map(|_| "-").collect::<String>();

    println!("\n{one_side_of_dashes} {title} {one_side_of_dashes}\n");

    for message in messages_to_print {
        println!("{message}");
    }

    println!("\n{one_side_of_dashes}{fill_gap_from_title}{one_side_of_dashes}\n");
}

pub async fn loader<T, E>(text: &str, callback: impl Future<Output = Result<T, E>>) -> Result<T, E>
where
    E: std::fmt::Debug,
{
    print!("\r⏳ {text}");
    io::stdout().flush().expect("failed to flush stdout");
    let result = callback.await;
    println!("\r\x1B[1A\x1B[2K");
    io::stdout().flush().expect("failed to flush stdout");
    result
}

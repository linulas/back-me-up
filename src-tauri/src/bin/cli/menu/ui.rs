pub fn print_frame(title: &str, messages_to_print: Vec<String>) {
    let one_side_of_dashes = (0..12).map(|_| "-").collect::<String>();
    let fill_gap_from_title = (0..title.len()).map(|_| "-").collect::<String>();

    println!("\n{one_side_of_dashes} {title} {one_side_of_dashes}\n");

    for message in messages_to_print {
        println!("{message}");
    }

    println!("\n{one_side_of_dashes}{fill_gap_from_title}{one_side_of_dashes}\n");
}


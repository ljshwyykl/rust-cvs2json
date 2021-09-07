extern crate getopts;
#[macro_use]
extern crate json;
extern crate csv;

use csv::Reader;
use csv::StringRecord;
use getopts::Options;
use json::JsonValue;
use std::env;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
struct Args {
    input: String,
    output: Option<String>,
}

fn get_args(arg_strings: &[String]) -> Option<Args> {
    let mut opts: Options = Options::new();
    opts.optopt(
        "o",
        "",
        "The path of the output file including the file extension.",
        "TARGET_FILE_NAME",
    );

    opts.optflag("n", "null", "Empty strings are set to null.");
    opts.optflag("h", "help", "Prints this help menu.");

    let matches = match opts.parse(&arg_strings[1..]) {
        Ok(m) => m,
        Err(err) => panic!("{}", err),
    };

    let program: String = arg_strings[0].clone();

    if matches.opt_present("h") {
        println_usage(&program, opts);
        return None;
    }

    let output: Option<String> = matches.opt_str("o");

    let input: String = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        println_usage(&program, opts);
        return None;
    };

    Some(Args { input, output })
}

fn println_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} SOURCE_FILE_NAME [options]", program);
    print!("{}", opts.usage(&brief));
}

fn get_file_names(input: String, output: Option<String>) -> (String, String) {
    if !input.contains(".csv") {
        panic!("src file is invalid. Should be specified and should contain the .csv extension!");
    }

    let src_file_name: String = input;

    let dest_file_name: String = {
        match output {
            Some(output_string) => {
                if !output_string.contains(".json") {
                    panic!("destination file is invalid. Should be specified and should contain the .json extension!");
                }

                output_string
            }

            None => {
                let splitted: Vec<&str> = src_file_name.split(".").collect();
                let mut dest_name = splitted[0].to_string();
                dest_name.push_str(".json");
                dest_name.to_owned()
            }
        }
    };

    (src_file_name, dest_file_name)
}

fn json_with_record_row(
    mut json: JsonValue,
    record: StringRecord,
    headers: &StringRecord,
) -> JsonValue {
    let record: StringRecord = record;

    let mut element = object! {};
    for index in 0..headers.len() {
        if index >= record.len() {
            break;
        }

        let header = &headers[index][..];
        println!("header:{}", header);
        let value: &str = &record[index];

        // let key: &str = &record[0];
        if value.is_empty() {
            element[header] = json::Null;
        } else {
            element[header] = value.into();
        }

        // if index == 0 {
        //     json[key] = object! {};
        // } else {
        //     if value.is_empty() {
        //         json[key][header] = json::Null;
        //     } else {
        //         json[key][header] = value.into();
        //     }
        // }
    }

    json.push(element.clone())
        .expect("Error pushing element to json");
    json
}

fn main() {
    let arg_strings: Vec<String> = env::args().collect();

    let args: Args = match get_args(&arg_strings) {
        Some(args) => args,
        None => return,
    };
    println!("args:{:?}", args);

    let (src_file_name, dest_file_name) =
        get_file_names(args.input.to_owned(), args.output.to_owned());

    let mut src_file: File = File::open(src_file_name).expect("File not exist");
    let mut contents: String = String::new();

    src_file.read_to_string(&mut contents).expect("read error");

    let mut data: JsonValue = array![];

    let mut rdr: Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    // let headers: StringRecord = rdr.headers().clone().expect("headers error");
    let headers = rdr.headers().expect("headers error").clone();

    let mut record_iter = rdr.records();
    while let Some(record) = record_iter.next() {
        data = json_with_record_row(data, record.unwrap(), &headers); // ,
    }

    let mut dest_file: File = File::create(&dest_file_name)
        .expect(&format!("Error creating the file: {}", dest_file_name)[..]);
    dest_file
        .write_all(data.to_string().as_bytes())
        .expect(&format!("Error writing to file: {}", dest_file_name)[..]);

    println!("Successfully wrote to file {}", dest_file_name);
}

extern crate clap;
use clap::{App, Arg};

use std::thread;
use std::io::{self, Write, Error, BufReader, BufRead};
use std::net::TcpStream;
//use openssl::ssl::{SslMethod, SslConnector, SslVerifyMode, SslStream};
use bytes::{Bytes};
use std::process;


struct Router {
    ip: String,
    username: String,
    password: String,
    port: String,

}

impl Router {
    fn new(ip: String, username: String, password: String, port: String) -> Router {

        Router {
            ip,
            username,
            password,
            port,
        }
    }


    fn login(&self, stream: &mut TcpStream) -> Result<(), Error> {
        println!("Logging in..");

        let bytes = Bytes::from("/login");
        let length = bytes.len();

        stream.write(&[length as u8])?;
        stream.write(&bytes[..])?;

        let bytes = Bytes::from(format!("=name={}", self.username));
        let length = bytes.len();

        stream.write(&[length as u8])?;
        stream.write(&bytes[..])?;

        let bytes = Bytes::from(format!("=password={}", self.password));
        let length = bytes.len();

        stream.write(&[length as u8])?;
        stream.write(&bytes[..])?;
        stream.write(&[0])?;


        Ok(())
    }

}


fn main() {

    let matches = App::new("RouterOsCli")
                        .version("0.1.0")
                        .about("RouterOS API CLI client")
                        .arg(Arg::with_name("ip")
                            .help("IP Address")
                            .index(1)
                            .required(true))
                        .arg(Arg::with_name("port")
                            .help("API Port")
                            .index(2)
                            .required(true))
                        .arg(Arg::with_name("username")
                            .help("Username")
                            .index(3)
                            .required(true))
                        .arg(Arg::with_name("password")
                            .help("Password")
                            .index(4)
                            .required(true))
                        .get_matches();

    let ip = matches.value_of("ip").unwrap();
    let port = matches.value_of("port").unwrap();
    let username = matches.value_of("username").unwrap();
    let password = matches.value_of("password").unwrap();


    let r = Router::new(ip.to_string(), username.to_string(),
                        password.to_string(), port.to_string());



    let mut stream = TcpStream::connect(format!("{}:{}", r.ip, r.port))
        .expect("Can not connect to router");


    println!("Enter your command: (e To exit)");

    let input_stream = stream.try_clone().unwrap();

    thread::spawn(move || {



//        input_stream.read(&mut buffer).unwrap();

        let mut reader = BufReader::new(&input_stream);

        loop {
            let mut buffer = Vec::new();
            match reader.read_until(0u8, &mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        process::exit(0)
                    } else {


                        let reply = process_su8(&mut buffer[..]).unwrap();

                        for r in reply {
                            io::stdout().write(b">>> ").unwrap();
                            io::stdout().write(r).unwrap();
                            io::stdout().write(b"\n").unwrap();
                        }
                        io::stdout().write(b"\n").unwrap();
                        buffer.clear();
                    }
                } ,
                Err(e) => eprintln!("Error: {}", e),
            }

        }





    });

    let mut output_stream = &mut stream;

    match r.login(&mut output_stream) {
        Ok(_) => println!("Logged in..."),
        Err(_) => eprintln!("Can not log in...")
    }

    loop {
        let mut command = vec![];


        loop {
            let mut input = String::new();
//            io::stdout().write(b"<<< ").unwrap();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    if input == "\n" {
                        break;
                    } else if input == "e\n" {
                        process::exit(0);
                    }
                    command.push(input);
                },
                Err(_) => println!("Invalid command")
            };

        };



        match send_command(&mut output_stream, command) {
            Ok(_) => println!("Waiting for reply..."),
            Err(_) => eprintln!("Error running command...")
        };

    }

}


fn send_command(stream: &mut TcpStream, command: Vec<String>) -> Result<(), Error> {
    for cmd in command {
        let bytes = Bytes::from(cmd.trim());
        let length = bytes.len();
        stream.write(&[length as u8]).unwrap();
        stream.write(&bytes[..]).unwrap();

    };
    stream.write(&[0]).unwrap();

    Ok(())
}

fn process_su8(sen: &[u8]) -> Result<Vec<&[u8]>, Error> {

    let mut words: Vec<&[u8]> = Vec::new();

    let mut acc = 0;
    loop {
        let n = sen[acc] as usize;
        let word: &[u8];
        if n + acc + 1 > sen.len() {
            word = &sen[..];

        } else {
            word = &sen[acc..n+acc+1];

        }
        words.push(&word[1..]);
        acc = sen[acc] as usize + acc + 1;
        if acc > sen.len() {break;}
        if sen[acc] == 0 {break;}
    };

    Ok(words)

}



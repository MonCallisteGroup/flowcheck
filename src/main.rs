use std::net::TcpStream;
use std::fs::File;
use std::io::{BufRead,BufReader};
use std::net::Shutdown;
use std::time::Duration;
use std::net::ToSocketAddrs;
use std::process;

extern crate clap;
extern crate hostname;

use clap::{Arg,App};


fn main() {

    /* set up the arguments */
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("path")
            .short("p")
            .long("path")
            .value_name("PATH")
            .help("path containing flows files, one for each hostname")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("timeout")
            .short("t")
            .long("timeout")
            .value_name("TIMEOUT")
            .help("time in seconds to wait for connection (default 10s)")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("hostname")
                 .short("n")
                 .long("hostname")
                 .value_name("HOSTNAME")
                 .help("use hostname (default is hostname returned by 'hostname' command)")
                 .takes_value(true)
                 .required(false)
        )
        .get_matches();

    /* get our machine name - this is what our system is configured to - returned by uname -n or 'hostname'*/
    let myhostname = hostname::get_hostname().unwrap();
    /* if we are executed it -n hostname, then we use that name instead of the system name */
    let filehost = matches.value_of("hostname").unwrap_or(&myhostname);


    /* path is where we look for flow config files
       the flow files are in path/hostname
     */
    let path = matches.value_of("path").unwrap();
    let mut filename: String = path.to_owned();
    if !filename.ends_with("/") {
        filename.push_str("/");
    }
    filename.push_str(filehost);

    /* we need a connection timeout, to detect missing flows, default is 10s, but 1s is probably enough */
    let arg_timeout = matches.value_of("timeout").unwrap_or("10");
    let timeout = arg_timeout.parse::<u64>().unwrap_or(10);

    /* open the file */
//    let file = File::open(filename).unwrap();
    let file = File::open(&filename);
    let myfile = match file {
        Ok(f) => f,
        Err(_e) => {
            println!("unable to open file {:?}",filename);

            process::exit(1);
        }

    };

    /* set up for buffered reading */
    let reader = BufReader::new(myfile);

    /* read every line until EOF */
    for (_index, line) in reader.lines().enumerate() {
        let line = line.unwrap();

        /* remove the whitespace at start and end */
        let line = line.trim();

        /* skip comment lines */
        if line.starts_with("#") {
            continue;
        }

        /* split the line into elements, each line is like;
        ** id=host:port
        ** FLOW0001=google.com:80
        ** FLOW0002=google.com:514
        **
        ** if we don't get 2 elements skip
        */
        let v: Vec<&str> = line.split('=').collect();
        if v.len() != 2 {
            continue;
        }

        /* print the flow id and host:port pair, later we add the result */
        print!("{} {} ", v[0],v[1]);

        /* we try to resolve the supplied name, if we can't the print error and skip, otherwise continue to connecting */
        let resolved_addr = v[1].to_socket_addrs();
        let mut x = match resolved_addr {
            Ok(v) => v,
            Err(_e) => {
                println!("HostnameUnresolved");
                continue;
            }
        };

        let first_addr = x.next().unwrap();
        let _client = TcpStream::connect_timeout(&first_addr, Duration::from_secs(timeout))
            .and_then(|stream| {
                println!("OK");

                /* be nice and close the connection */
                stream.shutdown(Shutdown::Both).unwrap();
                Ok(())
            })
            .map_err(|err| {
                println!("{:?}",err.kind());
            });
    }
}

/*
connection err: Os { code: 61, kind: ConnectionRefused, message: "Connection refused" }

connection err: Os { code: 13, kind: PermissionDenied, message: "Permission denied" }
*/

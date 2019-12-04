use std::net::TcpStream;
use std::fs::File;
use std::io::{BufRead,BufReader};
use std::net::Shutdown;
use std::time::Duration;
use std::net::ToSocketAddrs;
use std::fmt;

extern crate clap;
extern crate hostname;

use clap::{Arg,App};
use std::fmt::{Formatter, Error};

// small and simple stats struct to count some results
struct Stats {
    lines: u64,
    flow_ok: u64,
    unresolved: u64,
    other: u64,
}
impl fmt::Display for Stats {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f,"(Lines:{}, OK:{}, Unresolved:{}, Other: {})",self.lines,self.flow_ok,self.unresolved,self.other)
    }
}
impl Stats {
    fn add_flow(&mut self) {
        self.lines += 1;
    }
    fn add_unresolved(&mut self) {
        self.unresolved += 1;
    }
    fn add_flowok(&mut self) {
        self.flow_ok += 1;
    }
    fn add_other(&mut self) {
        self.other += 1;
    }
}



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
        ).get_matches();

    /* we need a connection timeout, to detect missing flows, default is 10s, but 1s is probably enough */
    let arg_timeout = matches.value_of("timeout").unwrap_or("10");
    let timeout = arg_timeout.parse::<u64>().unwrap_or(10);

    /* get our machine name - this is what our system is configured to - returned by uname -n or 'hostname'*/
    let myhostname = hostname::get_hostname().unwrap();
    /* if we are executed it -n hostname, then we use that name instead of the system name */
    let filehost = matches.value_of("hostname").unwrap_or(&myhostname);


    /* path is where we look for flow config files
       the flow files are in path/hostname
     */
    let path = matches.value_of("path").unwrap();
    let filename: String = build_file_path(path.to_string(), filehost.to_string());

    match process(filename, timeout) {
        Ok(r) => print!("{}",r),
        Err(e) => print!("{}",e)
    }
}

fn process(filename: String, timeout: u64) -> Result<Stats,&'static str> {
    /* open the file */
//    let file = File::open(filename).unwrap();
    let mut stats = Stats { lines: 0, flow_ok: 0, unresolved: 0, other: 0};
    let file = File::open(&filename);
    let myfile = match file {
        Ok(f) => f,
        Err(_e) => {
            return Err("error opening file")
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

        stats.add_flow();
        /* print the flow id and host:port pair, later we add the result */
        print!("{} {} ", v[0],v[1]);

        /* we try to resolve the supplied name, if we can't the print error and skip, otherwise continue to connecting */
        let resolved_addr = v[1].to_socket_addrs();
        let mut x = match resolved_addr {
            Ok(v) => v,
            Err(_e) => {
                println!("HostnameUnresolved");
                stats.add_unresolved();
                continue;
            }
        };

        let first_addr = x.next().unwrap();
        let _client = TcpStream::connect_timeout(&first_addr, Duration::from_secs(timeout))
            .and_then(|stream| {
                println!("OK");
                stats.add_flowok();

                /* be nice and close the connection */
                stream.shutdown(Shutdown::Both).unwrap();
                Ok(())
            })
            .map_err(|err| {
                stats.add_other();
                println!("{:?}",err.kind());
            });
    }
    Ok(stats)
}

fn build_file_path(path: String, filehost: String) -> String {
    let mut fullpath: String = path;
    if !fullpath.ends_with("/") {   // assuming a unix type filesystem
        fullpath.push_str("/");
    }
    fullpath.push_str(&filehost);
    fullpath
}

/*
connection err: Os { code: 61, kind: ConnectionRefused, message: "Connection refused" }
connection err: Os { code: 13, kind: PermissionDenied, message: "Permission denied" }
*/

/* this is such a simple command line tool, we don't really
   need tests. So these are just basically an experiment.
*/
#[cfg(test)]
mod tests {
    #[test]
    fn build_file_path() {
        let x = super::build_file_path(String::from("/var/tmp/"), String::from("a_silly_test"));
        assert_eq!(x, "/var/tmp/a_silly_test");
    }

    #[test]
    fn process() {
        let x = super::process(String::from("./example/flowcheck.conf"),1);

        match x {
            Ok(_t) => assert!(true),
            Err(_e) => assert!(false)
        }
    }

    #[test]
    fn process_bad_file() {
        let x = super::process(String::from("./example/flowcheck.conf_bad"),1);

        match x {
            Ok(_t) => assert!(false),
            Err(_e) => assert!(true)
        }
    }

    #[test]
    fn stats() {
        let x = super::process(String::from("./example/flowcheck.conf"),1);

        match x {
            Ok(t) => {
               assert_eq!(t.lines + t.flow_ok + t.unresolved + t.other, 6);
            }
            Err(_e) => assert!(false)
        }
    }

}

# Flowcheck

![](https://github.com/MonCallisteGroup/flowcheck/workflows/Build/badge.svg?branch=master)

A simple rust tool to test TCP connectivity to a specified list of host:port definitions.

It is designed to be distributed to a server and executed remotely.  The
default configuration file is the name of the server, ie. on a unix host
it is the name returned by 'hostname -s'. A different configuration file
can be specified by -n (--hostname). The configuration file should be 
contained in a path specified by -p (--path).

The configuration file contains a list of host:port pairs, and a flow
name.

On execution each flow configured is output with this data: flow name,
host:port pair and a result is displayed, finally a few metrics are
given.

## Getting Started

### Prerequsites

The following crates are used
```
clap = "2.33.0"
hostname = "0.1.5"
```
### Installing

## Usage

### Create flow config file

A flow config file has the format;
FLOWNAME=hostname:port

```
FLOW0001=google.com:443
FLOW0002=qerqewrqwerwer.com:45
FLOW0003=127.0.0.1:7878
```

### Execute

```
dellatronic% flowcheck  -p example/ -n flowcheck.conf
FLOW0001 google.com:443 OK
FLOW0002 qerqewrqwerwer.com:45 HostnameUnresolved
FLOW0003 127.0.0.1:7878 ConnectionRefused
Lines processed: 3
Flows OK: 1
Unresolved hosts: 1
Other network errors: 1
```

### Arguments

```
dellatronic% flowcheck  --help                          
flowcheck 0.2.0
Kent Ibbetson <kent.ibbetson@moncalliste.com>
A tool to check network connectivity

USAGE:
    flowcheck [OPTIONS] --path <PATH>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -n, --hostname <HOSTNAME>    use hostname (default is hostname returned by 'hostname' command)
    -p, --path <PATH>            path containing flows files, one for each hostname
    -t, --timeout <TIMEOUT>      time in seconds to wait for connection (default 10s)
```

## Contributing

## Versioning

```
0.1.0 - Initial work
0.2.0 - Split functions, and add tests
```

## Authors

* **Kent Ibbetson** - *Initial work* - [MonCallisteGroup](https://github.com/kibbet)

## Licence

## Acknowledgements

## Notes

This is basically the first 'production' ready tool I've written in Rust,
one that fills an actual requirement we had. Originally I had written a Perl
script to do the same work, but it was quite slow - I'm sure no fault of
perl, but rather my coding. The rust version is vastly superior; faster
and more stable. Yes, there are other (existing) way to handle this task,
but it was a interesting project to test the Rust development waters.

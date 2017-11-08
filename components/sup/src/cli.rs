use std::path::Path;
use std::result;
use std::str::FromStr;

use clap::{App, Arg};
use hcore::service::ServiceGroup;
use url::Url;

use config::GossipListenAddr;
use http_gateway::ListenAddr;
use manager::service::{Topology, UpdateStrategy};


pub fn start<'a, 'b>() -> App<'a, 'b> {
    let base = clap_app!(@subcommand start =>
        (about: "Start a loaded, but stopped, Habitat service or a transient service from \
            a package or artifact. If the Habitat Supervisor is not already running this \
            will additionally start one for you.")
        (aliases: &["sta", "star"])
        (@arg LISTEN_GOSSIP: --("listen-gossip") +takes_value {valid_listen_gossip}
            "The listen address for the gossip system [default: 0.0.0.0:9638]")
        (@arg LISTEN_HTTP: --("listen-http") +takes_value {valid_listen_http}
            "The listen address for the HTTP gateway [default: 0.0.0.0:9631]")
        (@arg NAME: --("override-name") +takes_value
            "The name for the state directory if launching more than one Supervisor \
            [default: default]")
        (@arg ORGANIZATION: --org +takes_value
            "The organization that the Supervisor and it's subsequent services are part of \
            [default: default]")
        (@arg PEER: --peer +takes_value +multiple
            "The listen address of an initial peer (IP[:PORT])")
        (@arg PERMANENT_PEER: --("permanent-peer") -I "If this Supervisor is a permanent peer")
        (@arg PEER_WATCH_FILE: --("peer-watch-file") +takes_value conflicts_with[peer]
            "Watch this file for connecting to the ring"
        )
        (@arg RING: --ring -r +takes_value "Ring key name")
        (@arg PKG_IDENT_OR_ARTIFACT: +required +takes_value
            "A Habitat package identifier (ex: core/redis) or filepath to a Habitat Artifact \
            (ex: /home/core-redis-3.0.7-21120102031201-x86_64-linux.hart)")
        (@arg APPLICATION: --application -a +takes_value requires[ENVIRONMENT]
            "Application name; [default: not set].")
        (@arg ENVIRONMENT: --environment -e +takes_value requires[APPLICATION]
            "Environment name; [default: not set].")
        (@arg CHANNEL: --channel +takes_value
            "Receive package updates from the specified release channel [default: stable]")
        (@arg GROUP: --group +takes_value
            "The service group; shared config and topology [default: default]")
        (@arg BLDR_URL: --url -u +takes_value {valid_url}
            "Receive package updates from Builder at the specified URL \
            [default: https://bldr.habitat.sh]")
        (@arg TOPOLOGY: --topology -t +takes_value {valid_topology}
            "Service topology; [default: none]")
        (@arg STRATEGY: --strategy -s +takes_value {valid_update_strategy}
            "The update strategy; [default: none] [values: none, at-once, rolling]")
        (@arg BIND: --bind +takes_value +multiple
            "One or more service groups to bind to a configuration")
        (@arg CONFIG_DIR: --("config-from") +takes_value {dir_exists}
            "Use package config from this path, rather than the package itself")
        (@arg AUTO_UPDATE: --("auto-update") -A "Enable automatic updates for the Supervisor \
            itself")
        (@arg EVENTS: --events -n +takes_value {valid_service_group} "Name of the service \
            group running a Habitat EventSrv to forward Supervisor and service event data to")
    );

    if cfg!(any(target_os = "linux", target_os = "macos")) {
        base
    } else if cfg!(target_os = "windows") {
        base.arg(Arg::with_name("PASSWORD")
            .long("password")
            .takes_value(true)
            .help("Password of the service user"))
    } else {
        unreachable!()
    }
}

// CLAP Validation Functions
////////////////////////////////////////////////////////////////////////

fn dir_exists(val: String) -> result::Result<(), String> {
    if Path::new(&val).is_dir() {
        Ok(())
    } else {
        Err(format!("Directory: '{}' cannot be found", &val))
    }
}

fn valid_service_group(val: String) -> result::Result<(), String> {
    match ServiceGroup::validate(&val) {
        Ok(()) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

fn valid_topology(val: String) -> result::Result<(), String> {
    match Topology::from_str(&val) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Service topology: '{}' is not valid", &val)),
    }
}

fn valid_listen_gossip(val: String) -> result::Result<(), String> {
    match GossipListenAddr::from_str(&val) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Listen gossip address should include both IP and port, eg: '0.0.0.0:9700'"))
    }
}

fn valid_listen_http(val: String) -> result::Result<(), String> {
    match ListenAddr::from_str(&val) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Listen http address should include both IP and port, eg: '0.0.0.0:9700'"))
    }
}

fn valid_update_strategy(val: String) -> result::Result<(), String> {
    match UpdateStrategy::from_str(&val) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Update strategy: '{}' is not valid", &val)),
    }
}

fn valid_url(val: String) -> result::Result<(), String> {
    match Url::parse(&val) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("URL: '{}' is not valid", &val)),
    }
}

////////////////////////////////////////////////////////////////////////
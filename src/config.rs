use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser)]
pub struct Config {
    /// One or more domain names to serve. Specify multiple times for multiple domains.
    #[clap(
        long,
        num_args = 1..,
        env = "LNADDRD_DOMAINS",
        value_delimiter = ',',
    )]
    pub domains: Vec<String>,

    /// The address to bind the server to
    #[clap(long, default_value = "127.0.0.1:8080", env = "LNADDRD_BIND")]
    pub bind: SocketAddr,

    /// The database URL
    #[clap(
        long,
        env = "LNADDRD_DATABASE_URL",
        default_value = "postgres://localhost:5432/lnaddrd"
    )]
    pub database: String,
}

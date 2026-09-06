//! GraphQL server for the Scryr manifest system.
//!
//! This server provides a GraphQL API to query and manage components defined in Python manifests.
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::useless_let_if_seq)]

use clap::Parser;
use crystal_server::state::Args;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    crystal_server::server::run(args.server).await
}

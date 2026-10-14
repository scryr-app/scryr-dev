//! Export the server schema without starting HTTP or opening a database.

fn main() {
    print!("{}", crystal_server::graphql_schema_sdl());
}

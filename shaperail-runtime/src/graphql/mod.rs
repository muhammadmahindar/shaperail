//! GraphQL support (M15). Dynamic schema from resources, query/mutation resolvers, optional playground.

mod handler;
mod schema;

pub use handler::{graphql_handler, playground_handler};
pub use schema::{build_schema, GqlContext, GraphQLSchema};

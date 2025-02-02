use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_rocket::{GraphQLQuery, GraphQLRequest, GraphQLResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use rocket::{response::content, routes, State};
use std::env;

mod models;
use models::{create_schema, DatabaseConnection, GrahpQLSchema};
mod validate;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
mod schema;

/// Creates a new pool for connecting to the database
fn establish_connection() -> DatabaseConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::new(&database_url);
    Pool::new(manager).expect("Could not connect to database")
}

/// Playground for making graphql requests
#[rocket::get("/")]
fn graphql_playground() -> content::Html<String> {
    content::Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

/// Endpoint for all graphql queries
#[rocket::get("/graphql?<query..>")]
async fn graphql_query(schema: &State<GrahpQLSchema>, query: GraphQLQuery) -> GraphQLResponse {
    query.execute(schema).await
}

/// Endpoint for all graphql requests
#[rocket::post("/graphql", data = "<request>", format = "application/json")]
async fn graphql_request(
    schema: &State<GrahpQLSchema>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    request.execute(schema).await
}

embed_migrations!();

/// Entrypoint of this binary crate that initializes the webserver
#[rocket::launch]
fn rocket() -> _ {
    let db_connection_pool = establish_connection();
    let connection = db_connection_pool
        .get()
        .expect("Recieved no connection from pool");
    embedded_migrations::run(&connection).expect("Migrations could not be applied successfully");

    rocket::build()
        .manage(create_schema(db_connection_pool))
        .mount(
            "/",
            routes![graphql_query, graphql_request, graphql_playground],
        )
}

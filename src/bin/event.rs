use mongodb::bson::{doc, Document};
use mongodb::bson::Regex;
use mongodb::bson::Bson;
use mongodb::options::ClientOptions;
use mongodb::options::ResolverConfig;
use std::env;
use std::error::Error;
use tokio;
use tokio::runtime::Runtime;
use serde::{Deserialize, Serialize};
use futures::stream::TryStreamExt;
use mongodb::options::FindOptions;
use structopt::StructOpt;
use toml;

#[derive(Deserialize)]
struct Config {
    database: String,
    collections: Collections
}
#[derive(Deserialize)]
struct Collections {
    users: String,
    events: String,
}


fn true_or_false(s: &str) -> Result<bool, &'static str> {
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("expected `true` or `false`"),
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "event", about = "Event management")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
    #[structopt(short, help = "verbose")]
    verbose: bool,
}
#[derive(StructOpt, Debug)]
enum Command {
    Add {
        name: String,
        point: i32,
//        #[structopt(parse(try_from_str = true_or_false))]
//        twitter: bool,
    }
}

async fn run(opt: Opt) -> Result<(), Box<dyn Error>> {
   match opt.cmd {
        Command::Add { name, point } => {
            add(&name, point).await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    run(opt).await?;
    Ok(())
}

async fn add(name: &str, point: i32) -> Result<(), Box<dyn Error>> {

    let config: Config = toml::from_str(r#"
        database = "churi"

        [collections]
        users = "users"
        events = "events"
    "#).unwrap();

    let client_url =
        env::var("MONGODB_URL").expect("You must set the MONGODB_URL environment var!");
    let mut options =
        ClientOptions::parse_with_resolver_config(&client_url, ResolverConfig::cloudflare())
            .await?;
            options.app_name = Some("event".to_string());
    let client = mongodb::Client::with_options(options)?;

    let users = client.database(&config.database).collection::<Document>(&config.collections.users);

    let regex = Regex {
        pattern: name.to_string(),
        options: "g".to_string(),
    };

    let query = doc! {"name": regex.clone()};

    // let find_options = FindOptions::builder().sort(doc! { "created_at": 1 }).build();
    let mut cursor = users.find(query.clone(), None).await?;

    // Iterate over the results of the cursor.
    while let Some(user) = cursor.try_next().await? {
        println!("id: {:?}", user);
    }
    let find_result = users
        .find_one(Some(query.clone()), None)
        .await?;

    let user_tuple = if find_result.is_some() {
        let user_json: serde_json::Value = Bson::from(find_result).into();
        (user_json["id"].as_str().unwrap().to_string(), true)
    } else {
        ("0000000000".to_string(), false)
    };

    println!("JSON: {}", user_tuple.0);

    let events = client.database(&config.database).collection(&config.collections.events);
    let bson_dt = bson::DateTime::now();
    let (name, twitter) = user_tuple;
    let insert_result = events
        .insert_one(
            doc! {
                "event_id": 2,
                "user_id": bson::to_bson(&name).unwrap_or_default(),
                "point": bson::to_bson(&point).unwrap(),
                "twitter": bson::to_bson(&twitter).unwrap(),
                "created_at": bson_dt,
             },
            None,
        )
        .await?;
    println!("Insert: {:?}", insert_result);

    Ok(())
}

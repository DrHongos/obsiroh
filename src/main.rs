pub mod helpers;

use std::str::FromStr;
use anyhow::Result;
use iroh::sync::{Author, CapabilityKind};
use futures::{TryStreamExt, StreamExt};
use dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    println!("Starting ObsIroh");
    let doc_locations = helpers::init()?;

    let node = helpers::create_node(doc_locations.clone()).await?;
    let client = node.client();
    
    let author_id = match dotenv::var("AUTHOR_SECRET") {
        Ok(author_id) => {
            author_id
        },
        Err(_) => {
            let nauthor = client.authors.create().await?;
            // should store
            nauthor.to_string()            
        }          
    };
    let author = Author::from_str(&author_id).expect("Author loading failed");
    println!("Author: {:#?}", author);  // maybe instructions to persist author?
    
    // loads all documents 
    for (_p, t) in doc_locations {
        let imported_doc = client.docs.import(t.clone()).await?;
        let _ = imported_doc.start_sync(t.nodes.clone()).await.expect("Cannot start sync");
        
    }
    // from a list of them, starts management (?)
    let mut doc_list = client.docs.list().await?;
    while let Some((doc_id, cap)) = doc_list.try_next().await.expect("Next doc is erroneous") {
        println!("Opening document {} with {} permissions", doc_id, cap);
        if let Some(doc) = client.docs.open(doc_id).await? {
            let tt = doc.status().await?;
            println!("Status: {:#?}", tt);                       
            // WIP
        
            let mut events = doc.subscribe().await?;
            let mut just_once = false;
            let _events_handle = tokio::spawn(async move {
                while let Some(Ok(event)) = events.next().await {
                    match event {
                        iroh::client::LiveEvent::InsertRemote { content_status, entry, .. } => {
                            // Only update if the we already have the content. Likely to happen when a remote user toggles "done".
                            if content_status == iroh::sync::ContentStatus::Complete {
                                println!("Completed {:#?}", entry);
                            }
                        },
                        iroh::client::LiveEvent::InsertLocal { entry } => {
                            println!("Inserted local: {:#?}", entry);
                        },
                        iroh::client::LiveEvent::ContentReady { hash } => {
                            println!("Content ready: {}", hash);
                            // i need to launch this here or it wont find entries.. 
                            // inside and in the same loop, need to download them 
                            // locally, so i get the data i want
                            if !just_once {
                                let mut doc_q = doc.get_many(iroh::sync::store::Query::single_latest_per_key()).await.expect("query went wrong");
                                while let Some(en) = doc_q.try_next().await.expect("nope") {
                                    println!("Entry {:#?}", en);                            
                                    // needs node (DocTicket.nodes.first()) && path 
 //                                   let _ = helpers::download_entry(client, node, en, path).await.expect("Downloading entry");
                                };
                                // if i have write permissions
                                match cap {
                                    CapabilityKind::Read => {
                                        println!("Only read mode");
                                    },
                                    CapabilityKind::Write => {
                                        println!("Write mode");
 
                                    }
                                }                                            
                                just_once = true;
                            }
    
                        },
                            _ => {}
                        }
                }
            });    
        };
    }

    node.await?;
    Ok(())
}

use std::{
    io::Write,
    path::PathBuf,
    collections::HashMap,
};
use anyhow::Result;
use iroh::{
    client::BlobDownloadProgress,
    sync::CapabilityKind,
    ticket::DocTicket,
    rpc_protocol::BlobDownloadRequest,
    bytes::store::bao_tree::blake3::Hash,
};
use tokio_util::task::LocalPoolHandle;
use futures::{TryStreamExt, StreamExt};
use dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    println!("Starting ObsIroh");
    // move this block to a init() function ------------
    let paths = dotenv::var("PATHS").expect("No PATHS in .env");
    let tickets = dotenv::var("TICKETS").expect("No tickets in .env");
    let vpaths: Vec<String> = paths.split(",").into_iter().map(|p| p.trim().to_string()).collect();
    let vtickets: Vec<String> = tickets.split(",").into_iter().map(|t| t.trim().to_string()).collect();
    if vpaths.len() != vtickets.len() {
        panic!("Select one path for ticket");
    }
    let mut documents_locations = HashMap::new();
    let _: Vec<_> = vpaths.into_iter().zip(vtickets).map(|(p, t)| {
        documents_locations.insert(t, p)
    }).collect();
    println!("{:#?}", documents_locations);
    // -------------------------------------------------

    // now it needs to start working!
/* 
    // Path is in config? where is this program runned? 
    let path = PathBuf::from(path); 
    let path = {
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()?.join(path)
        }
    };
//                     let dir_flattened = iroh::util::fs::scan_dir(path.clone(), iroh::rpc_protocol::WrapOption::NoWrap)?;
    let doc_store = iroh::sync::store::memory::Store::default();
    let db = iroh::bytes::store::mem::Store::default();
    let lp = LocalPoolHandle::new(2);

    // Create node
    let node = iroh::node::Node::builder(db, doc_store)
        .local_pool(&lp)
        .spawn()
        .await?;
    println!("Node id: {}", node.node_id());
    // 
    let client = node.client();
    // Author (check local key to persist?)
    let author = client.authors.create().await?;            

    // ticket is in config
    let tp = DocTicket::from_str(&ticket_str)?;
    // connect to provider and sync
    let imported_doc = client.docs.import(tp.clone()).await?;
    println!("importing {:?}", imported_doc);                    
    
    let _ = imported_doc.start_sync(tp.nodes.clone()).await.expect("Cannot start sync");
    
    let mut doc_list = client.docs.list().await?;
    // Warning!! This is for just 1 document!! I plan to manage many
    let (doc_id, cap) = doc_list.next().await.unwrap().unwrap();                    
    println!("Opening doc: {} with {} permission", doc_id, cap);
 */    /*         
    if let Some(doc) = client.docs.open(doc_id).await? {
        println!("Opened {:#?}", doc);

        let t = doc.get_download_policy().await?;
        println!("Download policy is: {:#?}", t);

        let tt = doc.status().await?;
        println!("Status: {:#?}", tt);                       



        // CLEAN THIS SHIT OUT!
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
                                
                                let dr = client.blobs.download(BlobDownloadRequest{
                                    hash: en.content_hash(),
                                    format: BlobFormat::Raw,
                                    peer: tp.nodes.first().unwrap().clone(),
                                    tag: iroh::rpc_protocol::SetTagOption::Auto,
                                    out: iroh::rpc_protocol::DownloadLocation::External { 
                                        path: path.join(String::from_utf8(
                                        en.key().to_vec(),
                                        //    hash.to_string().as_bytes().to_vec()
                                        ).unwrap()), 
                                        in_place: true 
                                    },
                                }).await.expect("Cannot get download request");
                                let outcome = dr
                                    .finish()
                                    .await
                                    .expect("unable to download hash");

                                println!(
                                    "\ndownloaded {} bytes",
                                    outcome.downloaded_size,
                                );
                            };
                            // if i have write permissions
                            match cap {
                                CapabilityKind::Read => {
                                    println!("Only read mode");
                                },
                                CapabilityKind::Write => {
                                    // create a file, store it
                                    let nkey = "hello_new_node.md";
                                    println!("Creating a new file {}", nkey);
                                    let npath = path.join(nkey);
                                    let mut nfile = std::fs::File::create(npath.clone()).expect("Cannot create new file");
                                    nfile.write_all(b"Hello, i am a new node syncing\nshould send data back?").expect("Cannot write file");
                                    // add it to the document
                                    println!("And loading it to the document");
                                    let import_outcome = doc
                                        .import_file(
                                            author,
                                            nkey.into(),
                                            npath,
                                            true,
                                        )
                                        .await
                                        .context("import file")
                                        .expect("Cannot import file")
                                        .finish()
                                        .await
                                        .context("import finish");
                                    println!("Loading outcome {:#?}", import_outcome);
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
     */
//    node.await?;
    Ok(())
}

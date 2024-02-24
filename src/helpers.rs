use anyhow::{Result, Context};
use std::{
    path::PathBuf,
    collections::HashMap,
    str::FromStr,
    io::Write,
};
use iroh::{
    bytes::{store::Store as ByteStore, util::progress::IgnoreProgressSender}, client::{mem::{Doc, Iroh}, Entry}, net::NodeAddr, node::Node, rpc_protocol::{BlobDownloadRequest, BlobFormat}, sync::Author, ticket::DocTicket
};
use tokio_util::task::LocalPoolHandle;

/// Reads env config and sets initial map of information
/// paths must be given as absolute otherwise will be created in running folder
pub fn init() -> Result<HashMap::<PathBuf, DocTicket >> {
    let paths = dotenv::var("PATHS").expect("No PATHS in .env");
    let tickets = dotenv::var("TICKETS").expect("No tickets in .env");
    let vpaths: Vec<String> = paths.split(",").into_iter().map(|p| p.trim().to_string()).collect();
    let vtickets: Vec<String> = tickets.split(",").into_iter().map(|t| t.trim().to_string()).collect();
    // if no input, len is still 1.. 
    if vpaths.len() == 0 || vtickets.len() == 0 || vpaths.len() != vtickets.len() {
        panic!("Select one path for ticket");
    }
    let mut documents_locations = HashMap::new();
    let _: Vec<_> = vpaths.into_iter().zip(vtickets).map(|(p, t)| {
        // convert tickets
        let tp = DocTicket::from_str(&t).expect("Serialize ticket from str");
        // and paths
        let path = PathBuf::from(p); 
        let path = {
            if path.is_absolute() {
                path
            } else {
                std::env::current_dir().expect("Getting current dir").join(path)
            }
        };
        documents_locations.insert(path, tp)
    }).collect();
    //println!("{:#?}", documents_locations);
    Ok(documents_locations)
}

pub async fn create_node(doc_location: HashMap<PathBuf, DocTicket>) -> Result<Node<iroh::bytes::store::mem::Store>> {
    let doc_store = iroh::sync::store::memory::Store::default();
    let db = iroh::bytes::store::mem::Store::default();
    let lp = LocalPoolHandle::new(1);

    for (p, _t) in doc_location {
        let dir_flattened = iroh::util::fs::scan_dir(p.clone(), iroh::rpc_protocol::WrapOption::NoWrap)?;
        for ds in dir_flattened.into_iter() {
            println!("Loading {}", ds.name());
            let (_tt, _prog) = db.import_file(
                ds.path().to_path_buf(), 
                iroh::bytes::store::ImportMode::TryReference, 
                iroh::bytes::BlobFormat::Raw, 
                IgnoreProgressSender::default()
            ).await?;
            // should be loaded in doc store too?            
        }
        // or how to create the replicas of the loaded data?
     }    

    // TODO: persist
    // let keypair = SecretKey::generate();
    // can create node with this keypair

    // Create node
    let node = iroh::node::Node::builder(db, doc_store)
        .local_pool(&lp)
        .spawn()
        .await?;
    println!("Node id: {}", node.node_id());

    Ok(node)
}

pub async fn download_entry(client: Iroh, node: NodeAddr, en: Entry, path: PathBuf) -> Result<bool> {    
    let dr = client.blobs.download(BlobDownloadRequest{
        hash: en.content_hash(),
        format: BlobFormat::Raw,
        peer: node,
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
    
    Ok(true)
}

/// used for tests
pub async fn create_store_file(doc: Doc, author: Author, path: PathBuf) -> Result<()> {
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
            author.id(),
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
    Ok(())
}
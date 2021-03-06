use clap::ArgMatches;
use protobuf::{NodeListRequest, NodeLocateRequest, NodeManagementClient};
use tonic::Request;

use std::{error, io};

pub fn process(matches: &ArgMatches, cluster_matches: &ArgMatches) {
    let result: Result<(), Box<dyn error::Error>> 
            = match cluster_matches.subcommand() {
        ("list", Some(list_matches)) =>
            list(&matches, &cluster_matches, &list_matches),
        ("locate", Some(locate_matches)) =>
            locate(&matches, &cluster_matches, &locate_matches),
        (cmd, _) => Err(Box::new(io::Error::new(io::ErrorKind::Other,
            format!("unknown subcommand '{}'", cmd)))),
    };

    if let Err(e) = result {
        println!("{}", e);
    }
}

#[tokio::main]
async fn list(matches: &ArgMatches, _: &ArgMatches,
        _list_matches: &ArgMatches) -> Result<(), Box<dyn error::Error>> {
    // initialize grpc client
    let ip_address = matches.value_of("ip_address").unwrap();
    let port = matches.value_of("port").unwrap().parse::<u16>()?;
    let mut client = NodeManagementClient::connect(
        format!("http://{}:{}", ip_address, port)).await?;

    // initialize request
    let request = Request::new(NodeListRequest {});

    // retrieve reply
    let reply = client.list(request).await?;
    let reply = reply.get_ref();

    // print information
    println!("{:<8}{:<24}{:<24}", "id", "rpc_addr", "xfer_addr");
    println!("------------------------------------------------");
    for node in reply.nodes.iter() {
        println!("{:<8}{:<24}{:<24}", node.id,
            node.rpc_addr, node.xfer_addr);
    }

    Ok(())
}

#[tokio::main]
async fn locate(matches: &ArgMatches, _: &ArgMatches,
        locate_matches: &ArgMatches) -> Result<(), Box<dyn error::Error>> {
    // initialize grpc client
    let ip_address = matches.value_of("ip_address").unwrap();
    let port = matches.value_of("port").unwrap().parse::<u16>()?;
    let mut client = NodeManagementClient::connect(
        format!("http://{}:{}", ip_address, port)).await?;

    // initialize request
    let request = Request::new(NodeLocateRequest {
        geocode: locate_matches.value_of("GEOCODE").unwrap().to_string(),
    });

    // retrieve reply
    let reply = client.locate(request).await?;
    let reply = reply.get_ref();

    // print information
    match &reply.node {
        Some(node) => println!("node: {}\nrpcAddr: {}\nxferAddr: {}",
            node.id, node.rpc_addr, node.xfer_addr),
        None => println!("node not found"),
    }

    Ok(())
}

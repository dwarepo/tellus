use petgraph::algo::dijkstra::dijkstra;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use petgraph::prelude::*;
use petgraph::Graph;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Clone, Debug, Default)]
struct Network {
    name: String,
    bandwidth: u64,
    cost: f64,
    available_datacenters: HashSet<String>,
}

impl Network {
    fn new(
        name: String,
        bandwidth: u64,
        cost: f64,
        available_datacenters: HashSet<String>,
    ) -> Self {
        Network {
            name,
            bandwidth,
            cost,
            available_datacenters,
        }
    }
}

struct DatacenterGraph {
    graph: Graph<String, Network>,
    node_indices: HashMap<String, NodeIndex>,
    networks: Vec<Network>,
}

impl DatacenterGraph {
    fn new() -> Self {
        DatacenterGraph {
            graph: Graph::new(),
            node_indices: HashMap::new(),
            networks: Vec::new(),
        }
    }

    fn add_network(&mut self, network: Network) {
        // Add the network to the list of networks in the graph.
        self.networks.push(network.clone());

        // Add the available datacenters to the graph if they don't already exist.
        for dc_id in network.available_datacenters.iter() {
            if !self.node_indices.contains_key(dc_id) {
                let node_index = self.graph.add_node(dc_id.clone());
                self.node_indices.insert(dc_id.clone(), node_index);
            }
        }

        // Find the indices of the nodes that correspond to the available datacenters.
        let node_indices: Vec<NodeIndex> = network
            .available_datacenters
            .iter()
            .filter_map(|dc_id| self.node_indices.get(dc_id).cloned())
            .collect();

        // Add an edge to the graph to represent the network between the datacenters.
        for i in 0..node_indices.len() {
            for j in (i + 1)..node_indices.len() {
                self.graph
                    .add_edge(node_indices[i], node_indices[j], network.clone());
            }
        }
    }

    fn shortest_path(&self, start: &str, end: &str, optimize: &str) -> Option<Vec<Network>> {
        // Get NodeIndex from ```start``` String
        let start_node = self
            .graph
            .node_indices()
            .find(|i| self.graph[*i] == start)
            .unwrap();
        // Get NodeIndex from ```end``` String
        let end_node = self
            .graph
            .node_indices()
            .find(|i| self.graph[*i] == end)
            .unwrap();

        // Run dijkstra's shortest path for weight
        let path = match optimize {
            "cost" => dijkstra(&self.graph, start_node, Some(end_node), |e| {
                e.weight().cost as u64
            }),
            "bandwidth" => dijkstra(&self.graph, start_node, Some(end_node), |e| {
                e.weight().bandwidth as u64 // / self.graph[e.source()].bandwidth as f64,
            }),
            _ => return None,
        };

        // Get Edges from shortest path
        if let Some(path) = Some(path) {
            let mut edges = vec![];
            for i in 0..path.len() - 1 {
                let node1 = NodeIndex::new(i);
                let node2 = NodeIndex::new(i + 1);
                match self.graph.find_edge(node1, node2) {
                    Some(ex) => edges.push(
                        self.graph
                            .edge_references()
                            .find(|x| x.id() == ex)
                            .unwrap()
                            .weight()
                            .clone(),
                    ),
                    None => {}
                };
            }
            return Some(edges);
        }
        None
    }

    // Load networks into DataCenterGraph from CSV
    fn load_networks_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            let name = parts[0].parse::<String>()?;
            let bandwidth = parts[1].parse::<u64>()?;
            let cost = parts[2].parse::<f64>()?;
            let available_datacenters: HashSet<String> =
                parts[3].split('|').map(|s| s.to_string()).collect();

            let network = Network::new(name, bandwidth, cost, available_datacenters);
            self.add_network(network);
        }

        Ok(())
    }
}

fn main() {
    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();
    // Start point
    let start = args[1].clone();
    // End point
    let end = args[2].clone();
    // Get prio string
    let prio = args[3].clone();

    // Instantiate the Datacenter Graph and load from csv
    let mut datacenter_graph = DatacenterGraph::new();
    datacenter_graph
        .load_networks_from_csv("test_networks.csv")
        .expect("No such file");

    // Print graph as graphviz file to screen
    println!("{:?}\n", Dot::with_config(&datacenter_graph.graph, &[]));

    // Get the shortest path with prio
    let path = datacenter_graph.shortest_path(&start, &end, &prio);

    // Print result on screen
    println!(
        "Query: \t({} to {} prio {})\nBW: \t{} Gbit/s,\nHops: \t{}\nCosts: \t{}€,\nPath: \t{}",
        start,
        end,
        prio,
        path.clone()
            .unwrap()
            .iter()
            .map(|x| x.bandwidth)
            .min()
            .unwrap() as f64
            / 100.0,
        path.clone().unwrap().len(),
        path.clone().unwrap().iter().map(|x| x.cost).sum::<f64>(),
        path.unwrap()
            .iter()
            .map(|x| format!("\n\t{}\n", x.name.clone()))
            .collect::<String>()
    );
}

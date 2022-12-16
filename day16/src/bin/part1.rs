use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
    str::FromStr,
};

use clap::Parser;
use petgraph::{prelude::DiGraph, stable_graph::NodeIndex};
use regex::Regex;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = "AA")]
    starting_room: String,
    #[clap(short, long, default_value_t = 30)]
    time: u64,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let stdin = std::io::stdin().lock();
    let tunnel_scans = stdin
        .lines()
        .map(|line| Result::<TunnelScan, eyre::Error>::Ok(line?.parse()?))
        .collect::<eyre::Result<Vec<_>>>()?;

    let tunnels = Tunnels::from_scans(&tunnel_scans);

    let best_path = find_best_path(&tunnels, &args.starting_room, args.time, 0);

    println!("Found best path:");
    for step in &best_path.steps {
        let (step, room) = match step {
            Step::Open { room } => ("open", *room),
            Step::Go { room } => ("go", *room),
        };
        println!("  {step} {}", room.valve);
    }

    println!();
    println!("Score: {}", best_path.score(args.time));

    Ok(())
}

struct TunnelScan {
    valve: String,
    flow_rate: u64,
    paths: Vec<String>,
}

impl FromStr for TunnelScan {
    type Err = eyre::Error;

    fn from_str(s: &str) -> eyre::Result<Self> {
        lazy_static::lazy_static! {
            // Regex with match groups named value and paths
            static ref TUNNEL_SCAN_REGEX: Regex = Regex::new(r#"^Valve (?P<valve>[A-Z]+) has flow rate=(?P<flow_rate>\d+); (tunnel leads to valve|tunnels lead to valves) (?P<paths>[A-Z, ]+)$"#).unwrap();
        }

        let captures = TUNNEL_SCAN_REGEX
            .captures(s)
            .ok_or_else(|| eyre::eyre!("invalid tunnel scan: {s:?}"))?;
        let valve = captures.name("valve").unwrap().as_str().to_string();
        let flow_rate = captures.name("flow_rate").unwrap().as_str().parse()?;
        let paths = captures
            .name("paths")
            .unwrap()
            .as_str()
            .split(", ")
            .map(|s| s.to_string())
            .collect();

        Ok(Self {
            valve,
            flow_rate,
            paths,
        })
    }
}

struct Tunnels {
    room_nodes: HashMap<String, NodeIndex>,
    room_graph: DiGraph<Room, ()>,
}

impl Tunnels {
    fn from_scans(scans: &[TunnelScan]) -> Self {
        let mut room_nodes: HashMap<String, NodeIndex> = HashMap::new();
        let mut room_graph = DiGraph::new();
        for scan in scans {
            let node = room_graph.add_node(Room {
                valve: scan.valve.clone(),
                flow_rate: scan.flow_rate,
            });
            room_nodes.insert(scan.valve.clone(), node);
        }

        for scan in scans {
            let node = room_nodes.get(&scan.valve).unwrap();
            for path in &scan.paths {
                let path_node = room_nodes.get(path).unwrap();
                room_graph.add_edge(*node, *path_node, ());
            }
        }

        Self {
            room_nodes,
            room_graph,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Room {
    valve: String,
    flow_rate: u64,
}

#[derive(Debug, Clone)]
enum Step<'a> {
    Open { room: &'a Room },
    Go { room: &'a Room },
}

#[derive(Debug, Clone)]
struct Path<'a> {
    steps: Vec<Step<'a>>,
}

impl<'a> Path<'a> {
    fn empty() -> Self {
        Path { steps: vec![] }
    }

    fn add(&mut self, step: Step<'a>) {
        self.steps.push(step);
    }

    fn score(&self, mut time: u64) -> u64 {
        let mut score = 0;
        let mut open_valves: HashSet<&Room> = HashSet::new();
        let mut steps = self.steps.iter();
        while time > 0 {
            if let Some(step) = steps.next() {
                match step {
                    Step::Open { room } => {
                        open_valves.insert(room);
                    }
                    Step::Go { .. } => {}
                }
            }

            let current_flow_rate: u64 = open_valves.iter().map(|room| room.flow_rate).sum();
            score += current_flow_rate;
            time -= 1;
        }

        score
    }
}

fn find_best_path<'a>(
    tunnels: &'a Tunnels,
    starting_room: &str,
    time: u64,
    depth: usize,
) -> Path<'a> {
    let node = tunnels.room_nodes.get(starting_room).unwrap();
    let room = &tunnels.room_graph[*node];

    if time == 0 {
        return Path::empty();
    }

    let candidate_steps = tunnels
        .room_graph
        .neighbors(*node)
        .map(|node| Step::Go {
            room: &tunnels.room_graph[node],
        })
        .chain(std::iter::once(Step::Open { room }));

    let best_path = candidate_steps
        .map(|step| {
            let room = match step {
                Step::Open { room } => room,
                Step::Go { room } => room,
            };
            let mut path = find_best_path(tunnels, &room.valve, time - 1, depth + 1);
            path.add(step);
            path
        })
        .max_by_key(|path| path.score(time))
        .unwrap_or_else(|| Path::empty());
    // let padding = std::iter::once("  ")
    //     .cycle()
    //     .take(depth)
    //     .collect::<String>();
    // println!(
    //     "{padding}[find_best_path] room:{starting_room} ({}) time:{time} = {}",
    //     tunnels.room_graph[*node].flow_rate,
    //     best_path.score(time),
    // );

    best_path
}

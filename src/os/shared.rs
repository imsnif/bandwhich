use std::{
    collections::HashMap,
    io::{self, ErrorKind, Write},
    net::Ipv4Addr,
    time,
};

use crossterm::event::{read, Event};
use eyre::{bail, eyre};
use itertools::Itertools;
use log::{debug, warn};
use pnet::datalink::{self, Channel::Ethernet, Config, DataLinkReceiver, NetworkInterface};
use tokio::runtime::Runtime;

use crate::{network::dns, os::errors::GetInterfaceError, OsInputOutput};

#[cfg(any(target_os = "android", target_os = "linux"))]
use crate::os::linux::get_open_sockets;
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
use crate::os::lsof::get_open_sockets;
#[cfg(target_os = "windows")]
use crate::os::windows::get_open_sockets;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub parent_pid: Option<u32>,
}

impl ProcessInfo {
    pub fn new(name: &str, pid: u32) -> Self {
        Self {
            name: name.to_string(),
            pid,
            parent_pid: None,
        }
    }

    pub fn with_parent(name: &str, pid: u32, parent_pid: Option<u32>) -> Self {
        Self {
            name: name.to_string(),
            pid,
            parent_pid,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProcessTreeNode {
    pub process_info: ProcessInfo,
    pub children: Vec<ProcessTreeNode>,
    #[allow(dead_code)]
    pub depth: usize,
}

impl ProcessTreeNode {
    pub fn new(process_info: ProcessInfo, depth: usize) -> Self {
        Self {
            process_info,
            children: Vec::new(),
            depth,
        }
    }

    pub fn add_child(&mut self, child: ProcessTreeNode) {
        self.children.push(child);
    }

    #[allow(dead_code)]
    pub fn find_node_mut(&mut self, pid: u32) -> Option<&mut ProcessTreeNode> {
        if self.process_info.pid == pid {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(node) = child.find_node_mut(pid) {
                return Some(node);
            }
        }

        None
    }

    pub fn iter_depth_first(&self) -> ProcessTreeIterator {
        ProcessTreeIterator::new(self)
    }
}

pub struct ProcessTreeIterator<'a> {
    stack: Vec<(&'a ProcessTreeNode, usize)>,
}

impl<'a> ProcessTreeIterator<'a> {
    fn new(root: &'a ProcessTreeNode) -> Self {
        let stack = vec![(root, 0)];
        Self { stack }
    }
}

impl<'a> Iterator for ProcessTreeIterator<'a> {
    type Item = (&'a ProcessInfo, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((node, depth)) = self.stack.pop() {
            // Add children to stack in reverse order for depth-first traversal
            for child in node.children.iter().rev() {
                self.stack.push((child, depth + 1));
            }

            Some((&node.process_info, depth))
        } else {
            None
        }
    }
}

pub fn build_process_trees(processes: Vec<ProcessInfo>) -> Vec<ProcessTreeNode> {
    let mut process_map: HashMap<u32, ProcessInfo> = HashMap::new();
    let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut roots = Vec::new();

    // First pass: build maps
    for process in processes {
        let pid = process.pid;
        process_map.insert(pid, process.clone());

        if let Some(parent_pid) = process.parent_pid {
            children_map.entry(parent_pid).or_default().push(pid);
        } else {
            // Process without parent is a root
            roots.push(pid);
        }
    }

    // Second pass: build trees
    fn build_tree_recursive(
        pid: u32,
        process_map: &HashMap<u32, ProcessInfo>,
        children_map: &HashMap<u32, Vec<u32>>,
        depth: usize,
    ) -> Option<ProcessTreeNode> {
        let process_info = process_map.get(&pid)?.clone();
        let mut node = ProcessTreeNode::new(process_info, depth);

        if let Some(child_pids) = children_map.get(&pid) {
            for &child_pid in child_pids {
                if let Some(child_node) =
                    build_tree_recursive(child_pid, process_map, children_map, depth + 1)
                {
                    node.add_child(child_node);
                }
            }
        }

        Some(node)
    }

    // Build root trees
    let mut result = Vec::new();
    for root_pid in roots {
        if let Some(tree) = build_tree_recursive(root_pid, &process_map, &children_map, 0) {
            result.push(tree);
        }
    }

    // Handle orphaned processes (parent not in our process list)
    for (pid, process_info) in &process_map {
        if let Some(parent_pid) = process_info.parent_pid {
            if !process_map.contains_key(&parent_pid) {
                // Parent not found, treat as root
                if let Some(tree) = build_tree_recursive(*pid, &process_map, &children_map, 0) {
                    result.push(tree);
                }
            }
        }
    }

    result
}

pub fn aggregate_bandwidth_by_tree(
    process_trees: &[ProcessTreeNode],
    processes_map: &HashMap<ProcessInfo, crate::display::NetworkData>,
) -> HashMap<ProcessInfo, crate::display::NetworkData> {
    use crate::display::{Bandwidth, NetworkData};

    let mut aggregated = HashMap::new();

    fn aggregate_recursive(
        node: &ProcessTreeNode,
        processes_map: &HashMap<ProcessInfo, NetworkData>,
        aggregated: &mut HashMap<ProcessInfo, NetworkData>,
    ) -> NetworkData {
        // Get this process's own bandwidth data
        let mut total_data = processes_map
            .get(&node.process_info)
            .cloned()
            .unwrap_or_default();

        // Aggregate data from all children
        for child in &node.children {
            let child_data = aggregate_recursive(child, processes_map, aggregated);
            total_data.combine_bandwidth(&child_data);
        }

        // Store the aggregated data for this process
        aggregated.insert(node.process_info.clone(), total_data.clone());
        total_data
    }

    for tree in process_trees {
        aggregate_recursive(tree, processes_map, &mut aggregated);
    }

    aggregated
}

pub struct TerminalEvents;

impl Iterator for TerminalEvents {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        read().ok()
    }
}

pub(crate) fn get_datalink_channel(
    interface: &NetworkInterface,
) -> Result<Box<dyn DataLinkReceiver>, GetInterfaceError> {
    let config = Config {
        read_timeout: Some(time::Duration::new(1, 0)),
        read_buffer_size: 65536,
        ..Default::default()
    };

    match datalink::channel(interface, config) {
        Ok(Ethernet(_tx, rx)) => Ok(rx),
        Ok(_) => Err(GetInterfaceError::OtherError(format!(
            "{}: Unsupported interface type",
            interface.name
        ))),
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(GetInterfaceError::PermissionError(
                interface.name.to_owned(),
            )),
            _ => Err(GetInterfaceError::OtherError(format!(
                "{}: {e}",
                &interface.name
            ))),
        },
    }
}

fn get_interface(interface_name: &str) -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
}

fn create_write_to_stdout() -> Box<dyn FnMut(&str) + Send> {
    let mut stdout = io::stdout();
    Box::new({
        move |output| match writeln!(stdout, "{output}") {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                // A process that was listening to bandwhich stdout has exited
                // We can't do much here, lets just exit as well
                std::process::exit(0)
            }
            Err(e) => panic!("Failed to write to stdout: {e}"),
        }
    })
}

pub fn get_input(
    interface_name: Option<&str>,
    resolve: bool,
    dns_server: Option<Ipv4Addr>,
) -> eyre::Result<OsInputOutput> {
    // get the user's requested interface, if any
    // IDEA: allow requesting multiple interfaces
    let requested_interfaces = interface_name
        .map(|name| get_interface(name).ok_or_else(|| eyre!("Cannot find interface {name}")))
        .transpose()?
        .map(|interface| vec![interface]);

    // take the user's requested interfaces (or all interfaces), and filter for up ones
    let available_interfaces = requested_interfaces
        .unwrap_or_else(datalink::interfaces)
        .into_iter()
        .filter(|interface| {
            // see https://github.com/libpnet/libpnet/issues/564
            let keep = if cfg!(target_os = "windows") {
                !interface.ips.is_empty()
            } else {
                interface.is_up() && !interface.ips.is_empty()
            };
            if !keep {
                debug!("{} is down. Skipping it.", interface.name);
            }
            keep
        })
        .collect_vec();

    // bail if no interfaces are up
    if available_interfaces.is_empty() {
        bail!("Failed to find any network interface to listen on.");
    }

    // try to get a frame receiver for each interface
    let interfaces_with_frames_res = available_interfaces
        .into_iter()
        .map(|interface| {
            let frames_res = get_datalink_channel(&interface);
            (interface, frames_res)
        })
        .collect_vec();

    // warn for all frame receivers we failed to acquire
    interfaces_with_frames_res
        .iter()
        .filter_map(|(interface, frames_res)| frames_res.as_ref().err().map(|err| (interface, err)))
        .for_each(|(interface, err)| {
            warn!(
                "Failed to acquire a frame receiver for {}: {err}",
                interface.name
            )
        });

    // bail if all of them fail
    // note that `Iterator::all` returns `true` for an empty iterator, so it is important to handle
    // that failure mode separately, which we already have
    if interfaces_with_frames_res
        .iter()
        .all(|(_, frames)| frames.is_err())
    {
        let (permission_err_interfaces, other_errs) = interfaces_with_frames_res.iter().fold(
            (vec![], vec![]),
            |(mut perms, mut others), (_, res)| {
                match res {
                    Ok(_) => (),
                    Err(GetInterfaceError::PermissionError(interface)) => {
                        perms.push(interface.as_str())
                    }
                    Err(GetInterfaceError::OtherError(err)) => others.push(err.as_str()),
                }
                (perms, others)
            },
        );

        let err_msg = match (permission_err_interfaces.is_empty(), other_errs.is_empty()) {
            (false, false) => format!(
                "\n\n{}: {}\nAdditional errors:\n{}",
                permission_err_interfaces.join(", "),
                eperm_message(),
                other_errs.join("\n")
            ),
            (false, true) => format!(
                "\n\n{}: {}",
                permission_err_interfaces.join(", "),
                eperm_message()
            ),
            (true, false) => format!("\n\n{}", other_errs.join("\n")),
            (true, true) => unreachable!("Found no errors in error handling code path."),
        };
        bail!(err_msg);
    }

    // filter out interfaces for which we failed to acquire a frame receiver
    let interfaces_with_frames = interfaces_with_frames_res
        .into_iter()
        .filter_map(|(interface, res)| res.ok().map(|frames| (interface, frames)))
        .collect();

    let dns_client = if resolve {
        let runtime = Runtime::new()?;
        let resolver = runtime
            .block_on(dns::Resolver::new(dns_server))
            .map_err(|err| {
                eyre!("Could not initialize the DNS resolver. Are you offline?\n\nReason: {err}")
            })?;
        let dns_client = dns::Client::new(resolver, runtime)?;
        Some(dns_client)
    } else {
        None
    };

    let write_to_stdout = create_write_to_stdout();

    Ok(OsInputOutput {
        interfaces_with_frames,
        get_open_sockets,
        terminal_events: Box::new(TerminalEvents),
        dns_client,
        write_to_stdout,
    })
}

#[inline]
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
fn eperm_message() -> &'static str {
    "Insufficient permissions to listen on network interface(s). Try running with sudo."
}

#[inline]
#[cfg(any(target_os = "android", target_os = "linux"))]
fn eperm_message() -> &'static str {
    r#"
    Insufficient permissions to listen on network interface(s). You can work around
    this issue like this:

    * Try running `bandwhich` with `sudo`

    * Build a `setcap(8)` wrapper for `bandwhich` with the following rules:
        `cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep`
    "#
}

#[inline]
#[cfg(target_os = "windows")]
fn eperm_message() -> &'static str {
    "Insufficient permissions to listen on network interface(s). Try running with administrator rights."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_info_new() {
        let proc_info = ProcessInfo::new("test_process", 1234);
        assert_eq!(proc_info.name, "test_process");
        assert_eq!(proc_info.pid, 1234);
        assert_eq!(proc_info.parent_pid, None);
    }

    #[test]
    fn test_process_info_with_parent() {
        let proc_info = ProcessInfo::with_parent("test_process", 1234, Some(5678));
        assert_eq!(proc_info.name, "test_process");
        assert_eq!(proc_info.pid, 1234);
        assert_eq!(proc_info.parent_pid, Some(5678));
    }

    #[test]
    fn test_build_process_trees_simple() {
        let processes = vec![
            ProcessInfo::with_parent("parent", 1, None),
            ProcessInfo::with_parent("child", 2, Some(1)),
        ];

        let trees = build_process_trees(processes);
        assert_eq!(trees.len(), 1);

        let root = &trees[0];
        assert_eq!(root.process_info.name, "parent");
        assert_eq!(root.process_info.pid, 1);
        assert_eq!(root.children.len(), 1);

        let child = &root.children[0];
        assert_eq!(child.process_info.name, "child");
        assert_eq!(child.process_info.pid, 2);
        assert_eq!(child.children.len(), 0);
    }

    #[test]
    fn test_build_process_trees_multiple_roots() {
        let processes = vec![
            ProcessInfo::with_parent("root1", 1, None),
            ProcessInfo::with_parent("root2", 2, None),
            ProcessInfo::with_parent("child1", 3, Some(1)),
        ];

        let trees = build_process_trees(processes);
        assert_eq!(trees.len(), 2);

        // Find the root with children
        let tree_with_child = trees.iter().find(|t| !t.children.is_empty()).unwrap();
        assert_eq!(tree_with_child.process_info.name, "root1");
        assert_eq!(tree_with_child.children.len(), 1);
        assert_eq!(tree_with_child.children[0].process_info.name, "child1");

        // Find the root without children
        let tree_without_child = trees.iter().find(|t| t.children.is_empty()).unwrap();
        assert_eq!(tree_without_child.process_info.name, "root2");
    }

    #[test]
    fn test_build_process_trees_orphaned_processes() {
        let processes = vec![
            ProcessInfo::with_parent("orphan", 1, Some(999)), // Parent 999 doesn't exist
            ProcessInfo::with_parent("child", 2, Some(1)),
        ];

        let trees = build_process_trees(processes);
        assert_eq!(trees.len(), 1);

        let root = &trees[0];
        assert_eq!(root.process_info.name, "orphan");
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].process_info.name, "child");
    }

    #[test]
    fn test_process_tree_iterator() {
        let processes = vec![
            ProcessInfo::with_parent("root", 1, None),
            ProcessInfo::with_parent("child1", 2, Some(1)),
            ProcessInfo::with_parent("child2", 3, Some(1)),
            ProcessInfo::with_parent("grandchild", 4, Some(2)),
        ];

        let trees = build_process_trees(processes);
        assert_eq!(trees.len(), 1);

        let root = &trees[0];
        let items: Vec<_> = root.iter_depth_first().collect();

        // Should visit root, then child1, then grandchild, then child2
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].0.name, "root");
        assert_eq!(items[0].1, 0); // depth 0
        assert_eq!(items[1].0.name, "child1");
        assert_eq!(items[1].1, 1); // depth 1
        assert_eq!(items[2].0.name, "grandchild");
        assert_eq!(items[2].1, 2); // depth 2
        assert_eq!(items[3].0.name, "child2");
        assert_eq!(items[3].1, 1); // depth 1
    }

    #[test]
    fn test_aggregate_bandwidth_by_tree() {
        use crate::display::NetworkData;
        use std::collections::HashMap;

        // Create test processes
        let processes = vec![
            ProcessInfo::with_parent("parent", 1, None),
            ProcessInfo::with_parent("child", 2, Some(1)),
        ];

        // Create bandwidth data
        let mut bandwidth_map = HashMap::new();
        bandwidth_map.insert(
            ProcessInfo::with_parent("parent", 1, None),
            NetworkData {
                total_bytes_downloaded: 100,
                total_bytes_uploaded: 50,
                connection_count: 1,
            },
        );
        bandwidth_map.insert(
            ProcessInfo::with_parent("child", 2, Some(1)),
            NetworkData {
                total_bytes_downloaded: 200,
                total_bytes_uploaded: 100,
                connection_count: 2,
            },
        );

        let trees = build_process_trees(processes);
        let aggregated = aggregate_bandwidth_by_tree(&trees, &bandwidth_map);

        // Parent should have its own bandwidth + child's bandwidth
        let parent_data = aggregated
            .get(&ProcessInfo::with_parent("parent", 1, None))
            .unwrap();
        assert_eq!(parent_data.total_bytes_downloaded, 300); // 100 + 200
        assert_eq!(parent_data.total_bytes_uploaded, 150); // 50 + 100
        assert_eq!(parent_data.connection_count, 2); // child's count (combined_bandwidth doesn't add connection_count for parent)

        // Child should have only its own bandwidth
        let child_data = aggregated
            .get(&ProcessInfo::with_parent("child", 2, Some(1)))
            .unwrap();
        assert_eq!(child_data.total_bytes_downloaded, 200);
        assert_eq!(child_data.total_bytes_uploaded, 100);
        assert_eq!(child_data.connection_count, 2);
    }
}

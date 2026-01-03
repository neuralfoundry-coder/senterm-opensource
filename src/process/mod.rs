use sysinfo::{Pid, Process, ProcessStatus, System, ProcessRefreshKind};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

/// Maximum history points to keep (60 seconds at 1Hz refresh)
const MAX_HISTORY_POINTS: usize = 60;

/// Process filter type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessFilter {
    All,
    System,
    User,
    CurrentUser,
}

impl ProcessFilter {
    pub fn next(&self) -> Self {
        match self {
            ProcessFilter::All => ProcessFilter::System,
            ProcessFilter::System => ProcessFilter::User,
            ProcessFilter::User => ProcessFilter::CurrentUser,
            ProcessFilter::CurrentUser => ProcessFilter::All,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessFilter::All => "All",
            ProcessFilter::System => "System",
            ProcessFilter::User => "User",
            ProcessFilter::CurrentUser => "Current User",
        }
    }
}

/// Process sort field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSort {
    Pid,
    Name,
    Cpu,
    Memory,
    StartTime,
}

impl ProcessSort {
    pub fn next(&self) -> Self {
        match self {
            ProcessSort::Pid => ProcessSort::Name,
            ProcessSort::Name => ProcessSort::Cpu,
            ProcessSort::Cpu => ProcessSort::Memory,
            ProcessSort::Memory => ProcessSort::StartTime,
            ProcessSort::StartTime => ProcessSort::Pid,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessSort::Pid => "PID",
            ProcessSort::Name => "Name",
            ProcessSort::Cpu => "CPU%",
            ProcessSort::Memory => "MEM%",
            ProcessSort::StartTime => "Time",
        }
    }
}

/// Process information with history
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub cmd: Vec<String>,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub memory_percent: f32,
    pub status: String,
    pub user: Option<String>,
    pub start_time: u64,
    pub children: Vec<u32>,
    pub is_expanded: bool,
    // History for charts
    pub cpu_history: VecDeque<f32>,
    pub mem_history: VecDeque<f32>,
}

impl ProcessInfo {
    pub fn from_process(pid: Pid, process: &Process, total_memory: u64) -> Self {
        let memory_percent = if total_memory > 0 {
            (process.memory() as f64 / total_memory as f64 * 100.0) as f32
        } else {
            0.0
        };
        
        let status_str = match process.status() {
            ProcessStatus::Run => "Running",
            ProcessStatus::Sleep => "Sleeping",
            ProcessStatus::Stop => "Stopped",
            ProcessStatus::Zombie => "Zombie",
            ProcessStatus::Idle => "Idle",
            _ => "Unknown",
        };
        
        ProcessInfo {
            pid: pid.as_u32(),
            parent_pid: process.parent().map(|p| p.as_u32()),
            name: process.name().to_string_lossy().to_string(),
            cmd: process.cmd().iter().map(|s| s.to_string_lossy().to_string()).collect(),
            cpu_usage: process.cpu_usage(),
            memory_bytes: process.memory(),
            memory_percent,
            status: status_str.to_string(),
            user: process.user_id().map(|u| u.to_string()),
            start_time: process.start_time(),
            children: Vec::new(),
            is_expanded: true,
            cpu_history: VecDeque::with_capacity(MAX_HISTORY_POINTS),
            mem_history: VecDeque::with_capacity(MAX_HISTORY_POINTS),
        }
    }
    
    pub fn update_history(&mut self) {
        // Add current values to history
        if self.cpu_history.len() >= MAX_HISTORY_POINTS {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(self.cpu_usage);
        
        if self.mem_history.len() >= MAX_HISTORY_POINTS {
            self.mem_history.pop_front();
        }
        self.mem_history.push_back(self.memory_percent);
    }
    
    pub fn format_memory(&self) -> String {
        if self.memory_bytes >= 1024 * 1024 * 1024 {
            format!("{:.1}G", self.memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if self.memory_bytes >= 1024 * 1024 {
            format!("{:.1}M", self.memory_bytes as f64 / (1024.0 * 1024.0))
        } else if self.memory_bytes >= 1024 {
            format!("{:.1}K", self.memory_bytes as f64 / 1024.0)
        } else {
            format!("{}B", self.memory_bytes)
        }
    }
}

/// Process tree viewer state
pub struct ProcessViewer {
    pub sys: System,
    pub processes: HashMap<u32, ProcessInfo>,
    pub tree_order: Vec<u32>,  // PIDs in tree display order
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub visible_height: usize,  // Cached visible height for scroll calculations
    pub filter: ProcessFilter,
    pub sort_by: ProcessSort,
    pub sort_ascending: bool,
    pub search_query: String,
    pub search_mode: bool,
    pub show_details: bool,
    pub last_refresh: Instant,
    pub current_user_id: Option<String>,
}

impl ProcessViewer {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        
        // Need to refresh twice for accurate CPU usage
        sys.refresh_all();
        std::thread::sleep(std::time::Duration::from_millis(100));
        sys.refresh_all();
        
        // Get current user ID
        let current_user_id = std::env::var("USER").ok()
            .or_else(|| std::env::var("USERNAME").ok());
        
        let mut viewer = ProcessViewer {
            sys,
            processes: HashMap::new(),
            tree_order: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            visible_height: 20,  // Default, will be updated by UI
            filter: ProcessFilter::All,
            sort_by: ProcessSort::Cpu,
            sort_ascending: false,
            search_query: String::new(),
            search_mode: false,
            show_details: true,
            last_refresh: Instant::now(),
            current_user_id,
        };
        
        viewer.refresh();
        viewer
    }
    
    /// Refresh process list
    pub fn refresh(&mut self) {
        // Refresh CPU info first for accurate readings
        self.sys.refresh_cpu_all();
        
        self.sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        
        let total_memory = self.sys.total_memory();
        
        // Update existing processes and add new ones
        let mut new_processes: HashMap<u32, ProcessInfo> = HashMap::new();
        
        for (pid, process) in self.sys.processes() {
            let pid_u32 = pid.as_u32();
            
            let mut info = if let Some(existing) = self.processes.get(&pid_u32) {
                let mut updated = ProcessInfo::from_process(*pid, process, total_memory);
                updated.cpu_history = existing.cpu_history.clone();
                updated.mem_history = existing.mem_history.clone();
                updated.is_expanded = existing.is_expanded;
                updated
            } else {
                ProcessInfo::from_process(*pid, process, total_memory)
            };
            
            info.update_history();
            new_processes.insert(pid_u32, info);
        }
        
        // Build parent-child relationships
        let pids: Vec<u32> = new_processes.keys().copied().collect();
        for pid in &pids {
            if let Some(info) = new_processes.get(pid) {
                if let Some(parent_pid) = info.parent_pid {
                    if let Some(parent) = new_processes.get_mut(&parent_pid) {
                        parent.children.push(*pid);
                    }
                }
            }
        }
        
        self.processes = new_processes;
        self.rebuild_tree_order();
        self.last_refresh = Instant::now();
    }
    
    /// Rebuild the tree display order based on filter and sort
    fn rebuild_tree_order(&mut self) {
        let mut filtered: Vec<u32> = self.processes.keys()
            .filter(|pid| self.filter_process(**pid))
            .filter(|pid| self.search_filter(**pid))
            .copied()
            .collect();
        
        // Sort
        filtered.sort_by(|a, b| {
            let pa = self.processes.get(a);
            let pb = self.processes.get(b);
            
            let cmp = match (pa, pb) {
                (Some(pa), Some(pb)) => match self.sort_by {
                    ProcessSort::Pid => pa.pid.cmp(&pb.pid),
                    ProcessSort::Name => pa.name.to_lowercase().cmp(&pb.name.to_lowercase()),
                    ProcessSort::Cpu => pa.cpu_usage.partial_cmp(&pb.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
                    ProcessSort::Memory => pa.memory_bytes.cmp(&pb.memory_bytes),
                    ProcessSort::StartTime => pa.start_time.cmp(&pb.start_time),
                },
                _ => std::cmp::Ordering::Equal,
            };
            
            if self.sort_ascending { cmp } else { cmp.reverse() }
        });
        
        // Build tree order (for tree view, we need hierarchical ordering)
        self.tree_order = self.build_tree_order_recursive(&filtered);
    }
    
    fn build_tree_order_recursive(&self, pids: &[u32]) -> Vec<u32> {
        // Find root processes (no parent or parent not in list)
        let mut roots: Vec<u32> = pids.iter()
            .filter(|pid| {
                self.processes.get(*pid)
                    .and_then(|p| p.parent_pid)
                    .map(|parent| !pids.contains(&parent))
                    .unwrap_or(true)
            })
            .copied()
            .collect();
        
        // Sort roots using the current sort settings
        self.sort_pids(&mut roots);
        
        let mut result = Vec::new();
        for root in roots {
            self.add_to_tree_order(&mut result, root, pids, 0);
        }
        result
    }
    
    /// Sort a list of PIDs based on current sort settings
    fn sort_pids(&self, pids: &mut Vec<u32>) {
        let sort_by = self.sort_by;
        let ascending = self.sort_ascending;
        
        pids.sort_by(|a, b| {
            let pa = self.processes.get(a);
            let pb = self.processes.get(b);
            
            let cmp = match (pa, pb) {
                (Some(pa), Some(pb)) => match sort_by {
                    ProcessSort::Pid => pa.pid.cmp(&pb.pid),
                    ProcessSort::Name => pa.name.to_lowercase().cmp(&pb.name.to_lowercase()),
                    ProcessSort::Cpu => pa.cpu_usage.partial_cmp(&pb.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
                    ProcessSort::Memory => pa.memory_bytes.cmp(&pb.memory_bytes),
                    ProcessSort::StartTime => pa.start_time.cmp(&pb.start_time),
                },
                _ => std::cmp::Ordering::Equal,
            };
            
            if ascending { cmp } else { cmp.reverse() }
        });
    }
    
    fn add_to_tree_order(&self, result: &mut Vec<u32>, pid: u32, all_pids: &[u32], depth: usize) {
        if let Some(info) = self.processes.get(&pid) {
            result.push(pid);
            
            if info.is_expanded {
                let mut children: Vec<u32> = info.children.iter()
                    .filter(|c| all_pids.contains(c))
                    .copied()
                    .collect();
                
                // Sort children using current sort settings
                self.sort_pids(&mut children);
                
                for child in children {
                    self.add_to_tree_order(result, child, all_pids, depth + 1);
                }
            }
        }
    }
    
    fn filter_process(&self, pid: u32) -> bool {
        if let Some(info) = self.processes.get(&pid) {
            match self.filter {
                ProcessFilter::All => true,
                ProcessFilter::System => {
                    // System processes typically have low PIDs or specific users
                    info.pid < 1000 || info.user.as_ref().map(|u| u == "root" || u == "0").unwrap_or(false)
                },
                ProcessFilter::User => {
                    info.pid >= 1000 && info.user.as_ref().map(|u| u != "root" && u != "0").unwrap_or(true)
                },
                ProcessFilter::CurrentUser => {
                    if let (Some(current), Some(process_user)) = (&self.current_user_id, &info.user) {
                        process_user.contains(current)
                    } else {
                        false
                    }
                },
            }
        } else {
            false
        }
    }
    
    fn search_filter(&self, pid: u32) -> bool {
        if self.search_query.is_empty() {
            return true;
        }
        
        if let Some(info) = self.processes.get(&pid) {
            let query = self.search_query.to_lowercase();
            info.name.to_lowercase().contains(&query)
                || info.pid.to_string().contains(&query)
                || info.cmd.iter().any(|c| c.to_lowercase().contains(&query))
        } else {
            false
        }
    }
    
    /// Get the depth of a process in the tree
    pub fn get_depth(&self, pid: u32) -> usize {
        let mut depth = 0;
        let mut current = pid;
        
        while let Some(info) = self.processes.get(&current) {
            if let Some(parent) = info.parent_pid {
                if self.processes.contains_key(&parent) {
                    depth += 1;
                    current = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        depth
    }
    
    /// Navigation
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.ensure_visible();
        }
    }
    
    pub fn move_down(&mut self) {
        if self.selected_index < self.tree_order.len().saturating_sub(1) {
            self.selected_index += 1;
            self.ensure_visible();
        }
    }
    
    pub fn move_to_top(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }
    
    pub fn move_to_bottom(&mut self) {
        self.selected_index = self.tree_order.len().saturating_sub(1);
        self.ensure_visible();
    }
    
    pub fn page_up(&mut self, page_size: usize) {
        self.selected_index = self.selected_index.saturating_sub(page_size);
        self.ensure_visible();
    }
    
    pub fn page_down(&mut self, page_size: usize) {
        self.selected_index = (self.selected_index + page_size).min(self.tree_order.len().saturating_sub(1));
        self.ensure_visible();
    }
    
    /// Ensure selected item is visible by adjusting scroll
    fn ensure_visible(&mut self) {
        if self.visible_height == 0 {
            return;
        }
        
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.visible_height {
            self.scroll_offset = self.selected_index - self.visible_height + 1;
        }
    }
    
    /// Set visible height (called by UI)
    pub fn set_visible_height(&mut self, height: usize) {
        self.visible_height = height;
    }
    
    /// Get selected process
    pub fn selected_process(&self) -> Option<&ProcessInfo> {
        self.tree_order.get(self.selected_index)
            .and_then(|pid| self.processes.get(pid))
    }
    
    /// Toggle expand/collapse for selected process
    pub fn toggle_expand(&mut self) {
        if let Some(pid) = self.tree_order.get(self.selected_index).copied() {
            if let Some(info) = self.processes.get_mut(&pid) {
                info.is_expanded = !info.is_expanded;
                self.rebuild_tree_order();
            }
        }
    }
    
    /// Move to parent process
    pub fn move_to_parent(&mut self) {
        if let Some(pid) = self.tree_order.get(self.selected_index).copied() {
            if let Some(info) = self.processes.get(&pid) {
                if let Some(parent_pid) = info.parent_pid {
                    if let Some(idx) = self.tree_order.iter().position(|p| *p == parent_pid) {
                        self.selected_index = idx;
                        self.ensure_visible();
                    }
                }
            }
        }
    }
    
    /// Kill selected process
    pub fn kill_selected(&mut self, force: bool) -> Result<(), String> {
        if let Some(pid) = self.tree_order.get(self.selected_index).copied() {
            let pid_sysinfo = Pid::from_u32(pid);
            
            if let Some(process) = self.sys.process(pid_sysinfo) {
                if force {
                    process.kill();
                    Ok(())
                } else {
                    // SIGTERM
                    process.kill();
                    Ok(())
                }
            } else {
                Err(format!("Process {} not found", pid))
            }
        } else {
            Err("No process selected".to_string())
        }
    }
    
    /// Cycle filter
    pub fn cycle_filter(&mut self) {
        self.filter = self.filter.next();
        self.rebuild_tree_order();
        self.selected_index = 0;
    }
    
    /// Cycle sort
    pub fn cycle_sort(&mut self) {
        self.sort_by = self.sort_by.next();
        self.rebuild_tree_order();
    }
    
    /// Toggle sort order
    pub fn toggle_sort_order(&mut self) {
        self.sort_ascending = !self.sort_ascending;
        self.rebuild_tree_order();
    }
    
    /// Search
    pub fn set_search(&mut self, query: String) {
        self.search_query = query;
        self.rebuild_tree_order();
        self.selected_index = 0;
    }
    
    /// Get visible processes for rendering
    pub fn visible_processes(&self, height: usize) -> Vec<(usize, &ProcessInfo)> {
        let start = self.scroll_offset;
        let end = (start + height).min(self.tree_order.len());
        
        self.tree_order[start..end]
            .iter()
            .enumerate()
            .filter_map(|(i, pid)| {
                self.processes.get(pid).map(|p| (start + i, p))
            })
            .collect()
    }
    
}

impl std::fmt::Debug for ProcessViewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessViewer")
            .field("process_count", &self.processes.len())
            .field("selected_index", &self.selected_index)
            .field("filter", &self.filter)
            .field("sort_by", &self.sort_by)
            .finish()
    }
}


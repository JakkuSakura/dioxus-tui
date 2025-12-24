pub mod cpu_panel;
pub mod disk_panel;
pub mod mem_panel;
pub mod net_panel;
pub mod proc_panel_bottom;
pub mod proc_panel_top;
pub mod topbar;

pub struct ComponentBlock {
    pub x: usize,
    pub y: usize,
    pub lines: Vec<String>,
}

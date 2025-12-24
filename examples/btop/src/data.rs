#[derive(Clone, Copy)]
pub struct TopbarData {
    pub active_tab: &'static str,
    pub tabs: [&'static str; 3],
    pub time: &'static str,
    pub interval_ms: u32,
}

#[derive(Clone, Copy)]
pub struct CpuCoreData {
    pub idx: u8,
    pub percent: u8,
    pub bar: &'static str,
}

#[derive(Clone, Copy)]
pub struct CpuData {
    pub model: &'static str,
    pub freq: &'static str,
    pub total_percent: u8,
    pub temp_c: u8,
    pub uptime: &'static str,
    pub load: (u8, u8, u8),
    pub cores: &'static [CpuCoreData],
}

#[derive(Clone, Copy)]
pub struct MemData {
    pub total_gib: &'static str,
    pub used_gib: &'static str,
    pub used_pct: u8,
    pub available_gib: &'static str,
    pub available_pct: u8,
    pub cached_gib: &'static str,
    pub cached_pct: u8,
    pub free_gib: &'static str,
    pub free_pct: u8,
}

#[derive(Clone, Copy)]
pub struct DiskData {
    pub root_used: &'static str,
    pub root_total: &'static str,
    pub root_used_pct: u8,
    pub root_used_gib: &'static str,
    pub swap_total: &'static str,
    pub swap_used_pct: i32,
    pub swap_used: &'static str,
    pub proc_used: i32,
    pub efi_total: &'static str,
}

#[derive(Clone, Copy)]
pub struct ProcRow {
    pub pid: u32,
    pub name: &'static str,
    pub cmd: &'static str,
    pub user: &'static str,
    pub mem: &'static str,
    pub cpu: &'static str,
    pub bar: &'static str,
    pub tail: &'static str,
}

#[derive(Clone, Copy)]
pub struct ProcData {
    pub rows_top: &'static [ProcRow],
    pub rows_bottom: &'static [ProcRow],
}

#[derive(Clone, Copy)]
pub struct NetData {
    pub interface: &'static str,
    pub down_rate: &'static str,
    pub down_rate_mib: &'static str,
    pub down_total: &'static str,
    pub up_rate: &'static str,
    pub up_rate_mib: &'static str,
    pub up_total: &'static str,
    pub graph_top: &'static str,
    pub graph_mid: &'static str,
    pub graph_solid: &'static str,
    pub graph_bottom: &'static str,
    pub graph_footer: &'static str,
}

#[derive(Clone, Copy)]
pub struct MockData {
    pub topbar: TopbarData,
    pub cpu: CpuData,
    pub mem: MemData,
    pub disk: DiskData,
    pub proc: ProcData,
    pub net: NetData,
}

pub static CPU_CORES: [CpuCoreData; 19] = [
    CpuCoreData {
        idx: 0,
        percent: 1,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 1,
        percent: 4,
        bar: "⣀⣀⣀⣀⣀⡀",
    },
    CpuCoreData {
        idx: 2,
        percent: 2,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 3,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 4,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 5,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 6,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 7,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 8,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 9,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 10,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 11,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 12,
        percent: 1,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 13,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 14,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 15,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 16,
        percent: 1,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
    CpuCoreData {
        idx: 17,
        percent: 1,
        bar: "⣀⣀⣀⣀⣀⡀",
    },
    CpuCoreData {
        idx: 18,
        percent: 0,
        bar: "⣀⣀⣀⣀⣀⣀",
    },
];

pub static PROC_TOP: [ProcRow; 9] = [
    ProcRow {
        pid: 9050,
        name: "kvm",
        cmd: "/usr/bin/kvm -id 102",
        user: "root",
        mem: "66G",
        cpu: "0.2",
        bar: "⣀⣀⣀⣀⣀",
        tail: "█",
    },
    ProcRow {
        pid: 6762,
        name: "tailscal",
        cmd: "/usr/sbin/tailscaled",
        user: "root",
        mem: "12M",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 9719,
        name: "kvm",
        cmd: "/usr/bin/kvm -id 105",
        user: "root",
        mem: "6.8G",
        cpu: "0.0",
        bar: "⡀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 222345,
        name: "kvm",
        cmd: "/usr/bin/kvm -id 106",
        user: "root",
        mem: "5.2G",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 9438,
        name: "kvm",
        cmd: "/usr/bin/kvm -id 103",
        user: "root",
        mem: "12G",
        cpu: "0.1",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 9610,
        name: "kvm",
        cmd: "/usr/bin/kvm -id 104",
        user: "root",
        mem: "1.9G",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 7893,
        name: "kvm",
        cmd: "/usr/bin/kvm -id 100",
        user: "root",
        mem: "16G",
        cpu: "0.0",
        bar: "⣀⢀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 6750,
        name: "rrdcache",
        cmd: "/usr/bin/rrdcached -g",
        user: "root",
        mem: "3.0M",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 7533,
        name: "pmxcfs",
        cmd: "/usr/bin/pmxcfs",
        user: "root",
        mem: "269M",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
];

pub static PROC_BOTTOM: [ProcRow; 7] = [
    ProcRow {
        pid: 6746,
        name: "pve-lxc-",
        cmd: "/usr/lib/x86_64-linux",
        user: "root",
        mem: "3.0M",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 6766,
        name: "zed",
        cmd: "/usr/sbin/zed -F",
        user: "root",
        mem: "3.1M",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 4026,
        name: "dmeventd",
        cmd: "/usr/sbin/dmeventd -f",
        user: "root",
        mem: "24M",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 6866,
        name: "lxcfs",
        cmd: "/usr/bin/lxcfs /var/l",
        user: "root",
        mem: "0B",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 3081602,
        name: "pvefw-lo",
        cmd: "/usr/sbin/pvefw-logge",
        user: "root",
        mem: "0B",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 3549518,
        name: "btop",
        cmd: "btop",
        user: "jakku",
        mem: "6.0M",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: " ",
    },
    ProcRow {
        pid: 7818,
        name: "pveproxy",
        cmd: "pveproxy",
        user: "www-+",
        mem: "168M",
        cpu: "0.0",
        bar: "⣀⣀⣀⣀⣀",
        tail: "↓",
    },
];

pub static MOCK_DATA: MockData = MockData {
    topbar: TopbarData {
        active_tab: "cpu",
        tabs: ["cpu", "menu", "preset"],
        time: "18:32:29",
        interval_ms: 2000,
    },
    cpu: CpuData {
        model: "EPYC 9965 192",
        freq: "1.8 GHz",
        total_percent: 1,
        temp_c: 36,
        uptime: "up 5d 07:35",
        load: (3, 4, 4),
        cores: &CPU_CORES,
    },
    mem: MemData {
        total_gib: "376",
        used_gib: "117",
        used_pct: 31,
        available_gib: "259",
        available_pct: 69,
        cached_gib: "197",
        cached_pct: 52,
        free_gib: "59.8",
        free_pct: 16,
    },
    disk: DiskData {
        root_used: "93.9",
        root_total: "GiB",
        root_used_pct: 87,
        root_used_gib: "82.1",
        swap_total: "7.99",
        swap_used_pct: 0,
        swap_used: "0",
        proc_used: -214,
        efi_total: "1021",
    },
    proc: ProcData {
        rows_top: &PROC_TOP,
        rows_bottom: &PROC_BOTTOM,
    },
    net: NetData {
        interface: "ens10f0np0",
        down_rate: "1.44 MiB/s",
        down_rate_mib: "11.5 Mibps",
        down_total: "1014 GiB",
        up_rate: "510 KiB/s",
        up_rate_mib: "3.98 Mibps",
        up_total: "252 GiB",
        graph_top: "4.8M  ⡀                  ",
        graph_mid: " ⢀⣰⣤⣄⣄⣧⣧⢠  ⣄⣇⢠ ⡀⣇⣀⣠⣄⡀⡄⢀ ⡆",
        graph_solid: "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣿⣿⣿",
        graph_bottom: "⠛⠉⠉⠛⠙⠛⠟⠏⠙⠛⠋⠛⠛⠛⠉⠛⠛⠉⠙⠙⠉⠉⠋⠙⠉",
        graph_footer: "4.8M                     ",
    },
};

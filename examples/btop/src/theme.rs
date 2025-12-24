use dioxus_tui::builders::Style;

// Colors derived from btop++ Default_theme in src/btop_theme.cpp.
pub const MAIN_BG: &str = "#000000";
pub const MAIN_FG: &str = "#cccccc";
pub const TITLE: &str = "#eeeeee";
pub const HI_FG: &str = "#b54040";
pub const SELECTED_BG: &str = "#6a2f2f";
pub const SELECTED_FG: &str = "#eeeeee";
pub const INACTIVE_FG: &str = "#404040";
pub const GRAPH_TEXT: &str = "#606060";
pub const METER_BG: &str = "#404040";
pub const PROC_MISC: &str = "#0de756";
pub const CPU_BOX: &str = "#556d59";
pub const MEM_BOX: &str = "#6c6c4b";
pub const NET_BOX: &str = "#5c588d";
pub const PROC_BOX: &str = "#805252";
pub const DIV_LINE: &str = "#303030";
pub const TEMP_START: &str = "#4897d4";
pub const TEMP_MID: &str = "#5474e8";
pub const TEMP_END: &str = "#ff40b6";
pub const CPU_START: &str = "#77ca9b";
pub const CPU_MID: &str = "#cbc06c";
pub const CPU_END: &str = "#dc4c4c";
pub const FREE_START: &str = "#384f21";
pub const FREE_MID: &str = "#b5e685";
pub const FREE_END: &str = "#dcff85";
pub const CACHED_START: &str = "#163350";
pub const CACHED_MID: &str = "#74e6fc";
pub const CACHED_END: &str = "#26c5ff";
pub const AVAILABLE_START: &str = "#4e3f0e";
pub const AVAILABLE_MID: &str = "#ffd77a";
pub const AVAILABLE_END: &str = "#ffb814";
pub const USED_START: &str = "#592b26";
pub const USED_MID: &str = "#d9626d";
pub const USED_END: &str = "#ff4769";
pub const DOWNLOAD_START: &str = "#291f75";
pub const DOWNLOAD_MID: &str = "#4f43a3";
pub const DOWNLOAD_END: &str = "#b0a9de";
pub const UPLOAD_START: &str = "#620665";
pub const UPLOAD_MID: &str = "#7d4180";
pub const UPLOAD_END: &str = "#dcafde";
pub const PROCESS_START: &str = "#80d0a3";
pub const PROCESS_MID: &str = "#dcd179";
pub const PROCESS_END: &str = "#d45454";
pub const PROC_PAUSE_BG: &str = "#b54040";

pub fn fg(color: &str) -> Style {
    Style {
        fg: Some(color.to_string()),
        ..Default::default()
    }
}

pub fn fg_bold(color: &str) -> Style {
    Style {
        fg: Some(color.to_string()),
        bold: true,
        ..Default::default()
    }
}

pub fn fg_dim(color: &str) -> Style {
    Style {
        fg: Some(color.to_string()),
        dim: true,
        ..Default::default()
    }
}

pub fn fg_bg(fg: &str, bg: &str) -> Style {
    Style {
        fg: Some(fg.to_string()),
        bg: Some(bg.to_string()),
        ..Default::default()
    }
}

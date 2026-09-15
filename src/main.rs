// Crate-wide until the remaining modules are cleaned (Sprint 4 removed the
// per-file copies from core/server/ui modules; dump, parser, routes, cli,
// cronus_ui*, vm, hydra, scripting still rely on this).
#![allow(dead_code, unused_imports, unused_variables)]
mod access;
mod actions;
mod animations;
mod api_crud;
#[cfg(test)]
mod api_security_tests;
#[cfg(test)]
mod api_validation_tests;
mod ast_diff;
mod audit;
mod auth;
mod auth_entity;
mod authz;
mod binding;
mod block_explorer;
mod board;
mod brain;
mod cli;
mod command_palette;
mod components;
mod constitution_check;
mod contracts;
mod cronus_ui;
mod cronus_ui_accordion;
mod cronus_ui_alert;
mod cronus_ui_alert_dialog;
mod cronus_ui_animated_button;
mod cronus_ui_animated_checkbox;
mod cronus_ui_animated_list;
mod cronus_ui_animated_number;
mod cronus_ui_app_shell;
mod cronus_ui_area_chart;
mod cronus_ui_aspect_ratio;
mod cronus_ui_aurora_background;
mod cronus_ui_autocomplete;
mod cronus_ui_avatar;
mod cronus_ui_avatar_group;
mod cronus_ui_badge;
mod cronus_ui_banner;
mod cronus_ui_bar_chart;
mod cronus_ui_border_beam;
mod cronus_ui_bouncy_accordion;
mod cronus_ui_breadcrumb;
mod cronus_ui_button_group;
mod cronus_ui_calendar;
mod cronus_ui_candlestick_chart;
mod cronus_ui_card;
mod cronus_ui_card_stack;
mod cronus_ui_carousel;
mod cronus_ui_chart;
mod cronus_ui_checkbox;
mod cronus_ui_chip;
mod cronus_ui_choropleth_chart;
mod cronus_ui_click_spark;
mod cronus_ui_code_block;
mod cronus_ui_code_tabs;
mod cronus_ui_collapsible;
mod cronus_ui_color_picker;
mod cronus_ui_combobox;
mod cronus_ui_command;
mod cronus_ui_comparison_slider;
mod cronus_ui_composed_chart;
mod cronus_ui_confetti;
mod cronus_ui_confirmation_dialog;
mod cronus_ui_context_menu;
mod cronus_ui_copy_button;
mod cronus_ui_countdown;
mod cronus_ui_credit_card_input;
mod cronus_ui_css;
mod cronus_ui_currency_input;
mod cronus_ui_data;
mod cronus_ui_data_table;
mod cronus_ui_date_picker;
mod cronus_ui_date_range_picker;
mod cronus_ui_description_list;
mod cronus_ui_dialog;
mod cronus_ui_dock;
mod cronus_ui_dot_pattern;
mod cronus_ui_drawer;
mod cronus_ui_dropdown_menu;
mod cronus_ui_dynamic_island;
mod cronus_ui_empty;
mod cronus_ui_expandable_tabs;
mod cronus_ui_fab;
mod cronus_ui_field;
mod cronus_ui_file_dropzone;
mod cronus_ui_flickering_grid;
mod cronus_ui_flip_card;
mod cronus_ui_floating_label_input;
mod cronus_ui_form;
mod cronus_ui_frame;
mod cronus_ui_funnel_chart;
mod cronus_ui_gauge_chart;
mod cronus_ui_glare_hover;
mod cronus_ui_glass_card;
mod cronus_ui_gradient_border;
mod cronus_ui_gradient_text;
mod cronus_ui_grid_pattern;
mod cronus_ui_heatmap;
mod cronus_ui_heatmap_chart;
mod cronus_ui_highlighter;
mod cronus_ui_hover_card;
mod cronus_ui_image_zoom;
mod cronus_ui_input;
mod cronus_ui_input_group;
mod cronus_ui_input_otp;
mod cronus_ui_invite_dialog;
mod cronus_ui_json_viewer;
mod cronus_ui_kanban;
mod cronus_ui_kbd;
mod cronus_ui_kit;
mod cronus_ui_label;
mod cronus_ui_light_rays;
mod cronus_ui_lightbox;
mod cronus_ui_line_chart;
mod cronus_ui_live_line_chart;
mod cronus_ui_loader;
mod cronus_ui_logo_carousel;
mod cronus_ui_magnetic;
mod cronus_ui_marquee;
mod cronus_ui_masonry;
mod cronus_ui_menubar;
mod cronus_ui_metric;
mod cronus_ui_mode_toggle;
mod cronus_ui_morphing_popover;
mod cronus_ui_motion_presets;
mod cronus_ui_multi_select;
mod cronus_ui_navigation_menu;
mod cronus_ui_noise;
mod cronus_ui_notification_center;
mod cronus_ui_number_flow;
mod cronus_ui_number_input;
mod cronus_ui_orbit;
#[cfg(test)]
mod cronus_ui_output_gate;
mod cronus_ui_pagination;
mod cronus_ui_particles;
mod cronus_ui_password_input;
mod cronus_ui_phone_input;
mod cronus_ui_pie_chart;
mod cronus_ui_pill_nav;
mod cronus_ui_popover;
mod cronus_ui_profit_loss_chart;
mod cronus_ui_progress;
mod cronus_ui_progressive_blur;
mod cronus_ui_radar_chart;
mod cronus_ui_radio_group;
mod cronus_ui_rating;
mod cronus_ui_resizable;
mod cronus_ui_retro_grid;
mod cronus_ui_reveal;
mod cronus_ui_rich_text_editor;
mod cronus_ui_ring_chart;
mod cronus_ui_ripple;
mod cronus_ui_scatter_chart;
mod cronus_ui_scheduler;
mod cronus_ui_scramble_text;
mod cronus_ui_scroll_area;
mod cronus_ui_scroll_nav;
mod cronus_ui_scroll_progress;
mod cronus_ui_segmented_control;
mod cronus_ui_select;
mod cronus_ui_separator;
mod cronus_ui_sheet;
mod cronus_ui_shimmer;
mod cronus_ui_shiny_text;
mod cronus_ui_sidebar;
mod cronus_ui_signature_pad;
mod cronus_ui_skeleton;
mod cronus_ui_slide_up_text;
mod cronus_ui_slider;
mod cronus_ui_sonner;
mod cronus_ui_sparkles_text;
mod cronus_ui_sparkline;
mod cronus_ui_spinner;
mod cronus_ui_spinning_text;
mod cronus_ui_split_button;
mod cronus_ui_spotlight_card;
mod cronus_ui_star_border;
mod cronus_ui_status_dot;
mod cronus_ui_stepper;
mod cronus_ui_sunburst_chart;
mod cronus_ui_switch;
mod cronus_ui_table;
mod cronus_ui_table_of_contents;
mod cronus_ui_tabs;
mod cronus_ui_tags_input;
mod cronus_ui_terminal;
mod cronus_ui_text_effect;
mod cronus_ui_text_shimmer;
mod cronus_ui_textarea;
mod cronus_ui_tilt_card;
mod cronus_ui_time_picker;
mod cronus_ui_timeline;
mod cronus_ui_toast;
mod cronus_ui_toggle;
mod cronus_ui_toggle_group;
mod cronus_ui_toolbar;
mod cronus_ui_tooltip;
mod cronus_ui_tree_view;
mod cronus_ui_typing_text;
mod cronus_ui_usage_meter;
mod cronus_ui_video_player;
mod cronus_ui_widgets;
mod cronus_ui_word_rotate;
mod cronus_ui_workspace_switcher;
// Sprint 5 C2 — AI suite (alphabetical).
mod cronus_ui_conversation;
mod cronus_ui_inline_citation;
mod cronus_ui_message;
mod cronus_ui_prompt_input;
mod cronus_ui_reasoning;
mod cronus_ui_sources;
mod cronus_ui_suggestion;
mod cronus_ui_tool;
mod data_table;
mod database;
mod deploy;
#[cfg(feature = "dump")]
mod dump;
mod effects;
mod env_schema;
mod error;
mod export;
mod feedback;
mod files;
mod graph;
mod graphql;
mod hardcode_lint;
mod hmr;
#[cfg(test)]
mod http_dispatch_tests;
mod http_guard;
mod hydra;
mod layout_system;
mod lint;
mod memory;
mod navigation;
mod overlays;
mod parser;
mod payments;
mod promote;
mod rate_limit;
mod relations;
mod render;
mod resolve;
mod routes;
mod runtime_js;
mod scripting;
mod security;
mod server;
mod session;
mod sse;
mod tabs;
mod tailwind;
mod testing;
mod theme;
mod trust;
mod ui;
mod validation;
mod vm;
mod voodoo;
mod webhook;
mod zeus;

use cli::brief::cmd_brief;
use cli::build::cmd_build;
use cli::changelog::cmd_changelog;
use cli::compose::cmd_compose;
use cli::context::cmd_context;
use cli::deploy_cmd::cmd_deploy;
use cli::doctor::cmd_doctor;
use cli::drift::cmd_drift;
use cli::dump_cmd::cmd_dump;
use cli::export_cmd::cmd_export;
use cli::generate::cmd_generate;
use cli::graph_cmd::cmd_graph;
use cli::handoff::cmd_handoff;
use cli::help::print_help;
use cli::lease::cmd_lease;
use cli::memory_cmd::cmd_memory;
use cli::new::cmd_new;
use cli::parse_cmd::cmd_parse;
use cli::reconcile::cmd_reconcile;
use cli::review::cmd_review;
use cli::run::{cmd_debug, cmd_run};
use cli::seed::cmd_seed;
use cli::segment::cmd_segment;
use cli::spec::cmd_spec;
use cli::stats::cmd_stats;
use cli::status_cmd::cmd_status;
use cli::sync_cmd::cmd_sync;
use cli::test_cmd::cmd_test;
use cli::timeline::cmd_timeline;
use cli::validate::{cmd_validate, cmd_validate_mission};
use cli::verify::cmd_verify_audit;

use std::env;
use std::fs;

use std::sync::atomic::{AtomicBool, Ordering};
pub static STRICT_MODE: AtomicBool = AtomicBool::new(false);
pub static STRICT_AI_MODE: AtomicBool = AtomicBool::new(false);
pub static DEBUG_MODE: AtomicBool = AtomicBool::new(false);
/// `cronus run --audit-canvas` / `CRONUS_AUDIT=1`. Exclusive `/audit/*` path.
pub static AUDIT_CANVAS: AtomicBool = AtomicBool::new(false);

/// Cache for the last `--ai` build result, served by `GET /api/_errors`.
/// Written by `cmd_build` when `--ai` flag is used, read by the server.
use std::sync::{LazyLock, Mutex};
pub static LAST_AI_ERRORS: LazyLock<Mutex<Option<serde_json::Value>>> =
    LazyLock::new(|| Mutex::new(None));

// ══════════════════════════════════════════════════
// MAIN
// ══════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let strict = args.iter().any(|a| a == "--strict");
    let strict_ai = args.iter().any(|a| a == "--strict-ai");
    if strict || strict_ai {
        STRICT_MODE.store(true, Ordering::Relaxed);
    }
    STRICT_AI_MODE.store(strict_ai, Ordering::Relaxed);
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "run" => cmd_run(&args).await,
        "debug" => cmd_debug(&args).await,
        "build" => cmd_build(&args),
        "parse" => cmd_parse(&args),
        "new" => cmd_new(&args),
        "seed" => cmd_seed(&args),
        "deploy" => cmd_deploy(&args),
        "doctor" => cmd_doctor(&args),
        "stats" => cmd_stats(&args),
        "export" => cmd_export(&args),
        "test" => cmd_test(&args),
        "compose" => cmd_compose(&args),
        "generate" | "gen" => cmd_generate(&args),
        "dump" => cmd_dump(&args),
        "clone" => cmd_dump(&args), // clone is an alias for dump
        "validate" => {
            if args.iter().any(|a| a == "--mission") {
                cmd_validate_mission();
            } else {
                cmd_validate(&args);
            }
        }
        "graph" => cmd_graph(&args),
        "brief" => cmd_brief(),
        "context" => cmd_context(&args),
        "mcp" => cli::mcp::cmd_mcp(&args),
        "sync" => cmd_sync(),
        "handoff" => cmd_handoff(&args),
        "lease" => cmd_lease(&args),
        "drift" => cmd_drift(&args),
        "spec" => cmd_spec(&args),
        "segment" => cmd_segment(&args),
        "reconcile" => cmd_reconcile(&args),
        "review" => cmd_review(&args),
        "timeline" => cmd_timeline(),
        "status" => cmd_status(),
        "changelog" => cmd_changelog(),
        "memory" => cmd_memory(&args),
        "verify-audit" => cmd_verify_audit(&args),
        "audit" => cli::cronus_audit::cmd_audit(&args),
        "version" | "-v" | "-V" | "--version" => cli::version::cmd_version(),
        "help" | "--help" | "-h" | _ => print_help(),
    }
}

// print_help moved to cli::help

// ══════════════════════════════════════════════════
// cmd_brief moved to cli::brief

// cmd_graph moved to cli::graph_cmd
// ══════════════════════════════════════════════════
// CMD: CONTEXT
// ══════════════════════════════════════════════════

// cmd_context moved to cli/context.rs

// ══════════════════════════════════════════════════
// FIND .cronus FILE
// ══════════════════════════════════════════════════

pub(crate) fn find_cronus_file() -> Option<String> {
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".cronus") && entry.path().is_file() {
                return Some(name);
            }
        }
    }
    None
}

/// Find ALL .cronus files in current directory (multi-agent mode).
pub(crate) fn find_all_cronus_files() -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".cronus") && entry.path().is_file() {
                files.push(name);
            }
        }
    }
    files.sort();
    files
}

// cmd_dump moved to cli::dump_cmd

// cmd_build and build_ai_error_json moved to cli/build.rs
// save_ast_snapshot moved to cli/build.rs

// cmd_validate moved to cli::validate

// cmd_new moved to cli::new

// ══════════════════════════════════════════════════
// SEED COMMAND
// ══════════════════════════════════════════════════

// cmd_seed moved to cli::seed

// cmd_parse moved to cli::parse_cmd

// cmd_deploy moved to cli::deploy_cmd

// cmd_doctor moved to cli::doctor

// cmd_stats moved to cli::stats

// cmd_export moved to cli::export_cmd

// cmd_test moved to cli::test_cmd

// cmd_compose moved to cli::compose

// cmd_generate and generate system moved to cli::generate

/// Auto-generated documentation page — derived 100% from the parsed .cronus AST.
/// Follows the Synthetic Docs design (obsidian dark, glass panels, code blocks).

// ══════════════════════════════════════════════════
// SEMANTIC MEMORY
// ══════════════════════════════════════════════════

pub(crate) fn open_memory_db() -> Result<memory::SemanticMemory, String> {
    let _ = fs::create_dir_all(".cronus");
    memory::SemanticMemory::open(".cronus/memory.db")
}

// cmd_verify_audit moved to cli::verify

// cmd_memory + find_flag_value moved to cli/memory_cmd.rs

// ══════════════════════════════════════════════════
// AST SNAPSHOT + CHANGELOG
// ══════════════════════════════════════════════════

// cmd_verify moved to cli::verify

// cmd_changelog moved to cli/changelog.rs

//! Dedicated DataTable renderer. DOM matches React (`data-table.tsx` over
//! `table.tsx`): `<div data-slot="data-table">` → optional
//! `data-table-toolbar` → `data-table-container` (rounded border) →
//! `<section data-slot="table-container">` → `<table data-slot="table">` with
//! `table-header` / `table-body`, `table-row`, `table-head` and `table-cell`,
//! then the optional pagination footer.
//!
//! Columns are `columns "Title"` lines; their config styles the column:
//! `sort:true` (React `DataTableColumnHeader`: ghost button, chevrons glyph,
//! sort aria-label) or `sort:button` (the docs' plain ghost `Button` with the
//! ArrowUpDown glyph), `font:medium|mono`, `color:secondary`,
//! `transform:capitalize|lowercase`, `badge:<variant>` (every cell a Badge) or
//! `badge:tone` with `tones:"value=variant,…"` (variant by cell value). Rows are
//! `item "First cell"` lines whose config carries the other cells by column key
//! (the title slug: `email:"…" role:Owner`); plain `text` lines still fill rows
//! positionally, and bound rows (keys become headers) win over both. The
//! `label` names the table (never a header or cell); label-only tables show a
//! Name/Value dummy row.
//!
//! Opt-ins, as props: `searchable:true` + `search-placeholder:`,
//! `filter:<column>` + `filter-icons:"a,b,c"` (faceted filter with live facet
//! counts), `column-visibility:true`, `density-toggle:true`,
//! `pagination:true` + `page-size:` + `page-sizes:"5,10,20"`,
//! `selection:true` (checkbox column, selected-of-total readout) with `action`
//! items as the bulk-actions bar (`icon:`, `variant:`, `count:true` appends the
//! selected count), `loading:true` + `loading-rows:`, `error:"…"` + `retry:true`,
//! and the empty state (`empty-icon:`, `empty-title:`, `empty-description:`;
//! plain "No results." without them).
//!
//! Zero JS: row checkboxes, the column-visibility menu and the density switch
//! are native inputs whose `:checked` state drives CSS (the bulk bar and the
//! selected counts use CSS counters, hidden columns `:has()`); popovers open
//! natively. Sortable headers carry `data-sort` (page runtime sorts the table).
//! Search is an enabled `data-table-search` input (`data-search` + row
//! `data-search-item`). Paging buttons carry `data-table-page`. Facet filters
//! are native checkboxes. Retry is an enabled button. Not interact
//! `table("data-table")`.

use std::collections::HashMap;

use crate::cronus_ui_kit::{attr_nonempty, attr_num, esc, flag, label_of, widget_id};
use crate::parser::{ComponentItemNode, ComponentNode};

/// Item kinds that name the widget rather than carry table content.
const NAME_KINDS: &[&str] = &["label", "title", "columns", "action"];

/// Columns the visibility menu can hide with CSS (one rule per index).
const MAX_HIDEABLE: usize = 8;

const SVG_OPEN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">";
/// lucide `CirclePlus` (PlusCircle).
const CIRCLE_PLUS: &str = "<circle cx=\"12\" cy=\"12\" r=\"10\"></circle><path d=\"M8 12h8\"></path><path d=\"M12 8v8\"></path></svg>";
/// lucide `ChevronsLeft` / `ChevronsRight`.
const CHEVRONS_LEFT: &str =
    "<path d=\"m11 17-5-5 5-5\"></path><path d=\"m18 17-5-5 5-5\"></path></svg>";
const CHEVRONS_RIGHT: &str =
    "<path d=\"m6 17 5-5-5-5\"></path><path d=\"m13 17 5-5-5-5\"></path></svg>";

#[derive(Clone, Copy, PartialEq)]
enum Sort {
    None,
    Header,
    Button,
}

struct Column {
    title: String,
    key: String,
    sort: Sort,
    classes: Vec<&'static str>,
    badge: Option<String>,
    tones: HashMap<String, String>,
}

struct Cell {
    text: String,
    tone: Option<String>,
}

pub fn render(comp: &ComponentNode) -> String {
    let bound = crate::cronus_ui_data::rows();
    if !bound.is_empty() {
        return from_bound(&bound);
    }
    let columns = columns_of(comp);
    if columns.is_empty() {
        let (headers, rows) = fallback_headers_and_rows(comp);
        let columns: Vec<Column> = headers.into_iter().map(plain_column).collect();
        let rows: Vec<Vec<Cell>> = rows
            .into_iter()
            .map(|r| {
                r.into_iter()
                    .map(|t| Cell {
                        text: t,
                        tone: None,
                    })
                    .collect()
            })
            .collect();
        return Table::new(comp, columns, rows).render();
    }
    let rows = rows_of(comp, &columns);
    Table::new(comp, columns, rows).render()
}

fn plain_column(title: String) -> Column {
    Column {
        key: slug(&title),
        title,
        sort: Sort::None,
        classes: Vec::new(),
        badge: None,
        tones: HashMap::new(),
    }
}

fn from_bound(bound: &[serde_json::Value]) -> String {
    let keys: Vec<String> = bound
        .first()
        .and_then(|row| row.as_object())
        .map(|o| o.keys().filter(|k| !k.starts_with('_')).cloned().collect())
        .unwrap_or_default();
    let columns: Vec<Column> = keys.iter().map(|k| plain_column(esc(k))).collect();
    let rows: Vec<Vec<Cell>> = bound
        .iter()
        .map(|row| {
            keys.iter()
                .map(|k| Cell {
                    text: row
                        .get(k)
                        .map(|x| esc(&crate::relations::display_value(x)))
                        .unwrap_or_default(),
                    tone: None,
                })
                .collect()
        })
        .collect();
    let stub = ComponentNode {
        name: "data-table".into(),
        layout: None,
        style: Some("data-table".into()),
        items: vec![],
        props: HashMap::new(),
        params: vec![],
        template: None,
        sections: vec![],
        state: vec![],
        tests: vec![],
        binding: None,
        span: Default::default(),
    };
    Table::new(&stub, columns, rows).render()
}

fn slug(title: &str) -> String {
    let mut out = String::new();
    for ch in title.chars() {
        if ch.is_alphanumeric() {
            out.extend(ch.to_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

fn columns_of(comp: &ComponentNode) -> Vec<Column> {
    comp.items
        .iter()
        .filter(|i| i.item_type == "columns" && !i.text.is_empty())
        .map(|i| {
            let get = |k: &str| i.config.get(k).map(String::as_str);
            let mut classes = Vec::new();
            match get("font") {
                Some("medium") => classes.push("font-medium"),
                Some("mono") => classes.push("font-mono"),
                _ => {}
            }
            if get("color") == Some("secondary") {
                classes.push("text-secondary");
            }
            match get("transform") {
                Some("capitalize") => classes.push("capitalize"),
                Some("lowercase") => classes.push("lowercase"),
                _ => {}
            }
            if get("align") == Some("end") {
                classes.push("text-end");
            }
            let tones = get("tones")
                .map(|t| {
                    t.split(',')
                        .filter_map(|p| p.split_once('='))
                        .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string()))
                        .collect()
                })
                .unwrap_or_default();
            Column {
                key: slug(&i.text),
                title: esc(&i.text),
                sort: match get("sort") {
                    Some("button") => Sort::Button,
                    Some(v) if crate::cronus_ui_kit::truthy(v) => Sort::Header,
                    _ => Sort::None,
                },
                classes,
                badge: get("badge").filter(|b| !b.is_empty()).map(str::to_string),
                tones,
            }
        })
        .collect()
}

/// Rows: keyed `item` lines (config by column key) when any item carries a
/// column key, else the content texts wrapped to the column count.
fn rows_of(comp: &ComponentNode, columns: &[Column]) -> Vec<Vec<Cell>> {
    let keyed: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| {
            i.item_type == "item"
                && !i.text.is_empty()
                && columns
                    .iter()
                    .skip(1)
                    .any(|c| i.config.contains_key(&c.key))
        })
        .collect();
    if !keyed.is_empty() {
        return keyed
            .iter()
            .map(|i| {
                columns
                    .iter()
                    .enumerate()
                    .map(|(n, c)| {
                        let raw = if n == 0 {
                            i.text.as_str()
                        } else {
                            i.config.get(&c.key).map(String::as_str).unwrap_or("")
                        };
                        Cell {
                            text: esc(raw),
                            tone: if n == 0 {
                                i.tone.clone()
                            } else {
                                i.config.get(&format!("{}-tone", c.key)).cloned()
                            },
                        }
                    })
                    .collect()
            })
            .collect();
    }
    let cells: Vec<Cell> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .map(|i| Cell {
            text: esc(&i.text),
            tone: i.tone.clone(),
        })
        .collect();
    let n = columns.len();
    if cells.is_empty() {
        return Vec::new();
    }
    cells
        .chunks(n)
        .map(|chunk| {
            (0..n)
                .map(|i| {
                    chunk.get(i).map_or(
                        Cell {
                            text: "—".into(),
                            tone: None,
                        },
                        |c| Cell {
                            text: c.text.clone(),
                            tone: c.tone.clone(),
                        },
                    )
                })
                .collect()
        })
        .collect()
}

/// Headers/rows when no `columns` line exists: the first two content texts are
/// headers and the rest fill two-column rows; label-only is a Name/Value dummy.
fn fallback_headers_and_rows(comp: &ComponentNode) -> (Vec<String>, Vec<Vec<String>>) {
    let cells: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .map(|i| esc(&i.text))
        .collect();
    match cells.len() {
        0 => (
            vec!["Name".into(), "Value".into()],
            vec![vec![label_of(comp), "—".into()]],
        ),
        1 => (
            vec!["Name".into(), "Value".into()],
            vec![vec![cells[0].clone(), "—".into()]],
        ),
        _ => {
            let (head, rest) = cells.split_at(2);
            let rows = rest
                .chunks(2)
                .map(|chunk| {
                    (0..2)
                        .map(|i| chunk.get(i).cloned().unwrap_or_else(|| "—".into()))
                        .collect()
                })
                .collect();
            (head.to_vec(), rows)
        }
    }
}

struct Table<'a> {
    comp: &'a ComponentNode,
    columns: Vec<Column>,
    rows: Vec<Vec<Cell>>,
    selection: bool,
    pagination: bool,
    page_size: usize,
    loading: bool,
    error: Option<String>,
}

impl<'a> Table<'a> {
    fn new(comp: &'a ComponentNode, columns: Vec<Column>, rows: Vec<Vec<Cell>>) -> Self {
        let pagination = flag(comp, "pagination");
        Table {
            comp,
            columns,
            rows,
            selection: flag(comp, "selection") || flag(comp, "enable-row-selection"),
            pagination,
            page_size: attr_num::<usize>(comp, "page-size")
                .filter(|n| *n > 0)
                .unwrap_or(10),
            loading: flag(comp, "loading"),
            error: attr_nonempty(comp, "error").map(esc),
        }
    }

    fn opt(&self, key: &str) -> bool {
        flag(self.comp, key)
    }

    fn id(&self, part: &str) -> String {
        widget_id(self.comp, part)
    }

    fn col_span(&self) -> usize {
        (self.columns.len() + usize::from(self.selection)).max(1)
    }

    fn render(&self) -> String {
        let toolbar = self.toolbar();
        let bulk = self.bulk_bar();
        let busy = if self.loading {
            " aria-busy=\"true\""
        } else {
            ""
        };
        let loading = if self.loading {
            "<span role=\"status\" data-slot=\"data-table-loading\" class=\"sr-only\">Loading rows…</span>"
        } else {
            ""
        };
        let search_target = if self.opt("searchable") {
            format!(" data-search-target=\"{}\"", self.id("search"))
        } else {
            String::new()
        };
        let table = format!(
            "<div data-slot=\"data-table-container\"{busy}>{loading}<section data-slot=\"table-container\" tabindex=\"0\" aria-label=\"Table\"><table data-slot=\"table\"{search_target}><thead data-slot=\"table-header\"><tr data-slot=\"table-row\">{}</tr></thead><tbody data-slot=\"table-body\">{}</tbody></table></section></div>",
            self.head(),
            self.body()
        );
        let footer = self.footer();
        format!("<div data-slot=\"data-table\">{toolbar}{table}{bulk}{footer}</div>")
    }

    fn head(&self) -> String {
        let mut out = String::new();
        if self.selection {
            out.push_str(&format!(
                "<th data-slot=\"table-head\" colspan=\"1\" scope=\"col\">{}</th>",
                checkbox("Select all rows on this page", &self.id("select-all"))
            ));
        }
        for (i, c) in self.columns.iter().enumerate() {
            let class = class_attr(&c.classes);
            let idx = i + usize::from(self.selection);
            let sort_attr = if c.sort == Sort::None {
                String::new()
            } else {
                format!(" data-sort=\"{idx}\"")
            };
            let inner = match c.sort {
                Sort::Header => format!(
                    "<button type=\"button\" data-slot=\"data-table-column-header\" data-variant=\"ghost\" data-size=\"sm\" class=\"cui-btn\" data-sort=\"{idx}\" aria-label=\"Sort by {t}. Activate to sort ascending.\"><span>{t}</span>{}</button>",
                    icon_class("chevrons-up-down", "opacity-60"),
                    t = c.title
                ),
                Sort::Button => format!(
                    "<button type=\"button\" data-slot=\"button\" data-variant=\"ghost\" data-size=\"sm\" class=\"cui-btn cui-dt-sort\" data-sort=\"{idx}\">{}{}</button>",
                    c.title,
                    crate::cronus_ui_icons::svg_or_empty("arrow-up-down")
                ),
                Sort::None => c.title.clone(),
            };
            out.push_str(&format!(
                "<th data-slot=\"table-head\" colspan=\"1\" scope=\"col\" aria-sort=\"none\"{sort_attr}{class}>{inner}</th>"
            ));
        }
        out
    }

    fn body(&self) -> String {
        let span = self.col_span();
        if self.loading {
            let count = attr_num::<usize>(self.comp, "loading-rows")
                .filter(|n| *n > 0)
                .unwrap_or(if self.pagination { self.page_size } else { 5 });
            let cells = (0..span)
                .map(|_| "<td data-slot=\"table-cell\"><div data-slot=\"skeleton\" aria-hidden=\"true\" class=\"h-4 w-full\"></div></td>")
                .collect::<String>();
            return (0..count)
                .map(|_| format!("<tr data-slot=\"table-row\">{cells}</tr>"))
                .collect();
        }
        if let Some(error) = &self.error {
            let retry = if self.opt("retry") {
                format!(
                    "<button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\">{}Retry</button>",
                    crate::cronus_ui_icons::svg_or_empty("refresh-cw")
                )
            } else {
                String::new()
            };
            return format!(
                "<tr data-slot=\"table-row\"><td data-slot=\"table-cell\" colspan=\"{span}\" class=\"cui-dt-error\"><div>{}<div role=\"alert\">{error}</div>{retry}</div></td></tr>",
                icon_class("circle-alert", "text-error")
            );
        }
        if self.rows.is_empty() {
            let inner = match attr_nonempty(self.comp, "empty-title") {
                Some(title) => {
                    let icon = attr_nonempty(self.comp, "empty-icon")
                        .and_then(crate::cronus_ui_icons::svg)
                        .map(|svg| format!("<div data-slot=\"empty-icon\">{svg}</div>"))
                        .unwrap_or_default();
                    let desc = attr_nonempty(self.comp, "empty-description")
                        .map(|d| format!("<div data-slot=\"empty-description\">{}</div>", esc(d)))
                        .unwrap_or_default();
                    format!(
                        "<div data-slot=\"empty\">{icon}<div data-slot=\"empty-title\">{}</div>{desc}</div>",
                        esc(title)
                    )
                }
                None => "No results.".to_string(),
            };
            return format!(
                "<tr data-slot=\"table-row\"><td data-slot=\"table-cell\" colspan=\"{span}\" class=\"cui-dt-empty\">{inner}</td></tr>"
            );
        }
        let search_item = if self.opt("searchable") {
            " data-search-item"
        } else {
            ""
        };
        let visible: &[Vec<Cell>] = if self.pagination {
            &self.rows[..self.rows.len().min(self.page_size)]
        } else {
            &self.rows
        };
        visible
            .iter()
            .enumerate()
            .map(|(n, row)| {
                let mut cells = String::new();
                if self.selection {
                    cells.push_str(&format!(
                        "<td data-slot=\"table-cell\">{}</td>",
                        checkbox(
                            &format!("Select row {}", n + 1),
                            &self.id(&format!("row-{}", n + 1))
                        )
                    ));
                }
                for (c, cell) in self.columns.iter().zip(row) {
                    let class = class_attr(&c.classes);
                    let variant = cell.tone.clone().or_else(|| match c.badge.as_deref() {
                        Some("tone") => c.tones.get(&cell.text.to_lowercase()).cloned(),
                        Some(v) => Some(v.to_string()),
                        None => None,
                    });
                    let inner = match variant {
                        Some(v) => crate::cronus_ui_badge::badge_html(&cell.text, &v),
                        None => cell.text.clone(),
                    };
                    cells.push_str(&format!("<td data-slot=\"table-cell\"{class}>{inner}</td>"));
                }
                format!("<tr data-slot=\"table-row\"{search_item}>{cells}</tr>")
            })
            .collect()
    }

    fn toolbar(&self) -> String {
        let searchable = self.opt("searchable");
        let filter = attr_nonempty(self.comp, "filter");
        let visibility = self.opt("column-visibility");
        let density = self.opt("density-toggle");
        if !(searchable || filter.is_some() || visibility || density) {
            return String::new();
        }
        let mut out = String::from(
            "<div role=\"group\" aria-label=\"Table controls\" data-slot=\"data-table-toolbar\">",
        );
        if searchable {
            let placeholder =
                esc(attr_nonempty(self.comp, "search-placeholder").unwrap_or("Search…"));
            let id = self.id("search");
            out.push_str(&format!(
                "<div class=\"cui-dt-search\">{}<label for=\"{id}\" class=\"sr-only\">{placeholder}</label><input data-slot=\"data-table-search\" type=\"search\" id=\"{id}\" placeholder=\"{placeholder}\" class=\"h-9 ps-9\" data-search=\"{id}\" /></div>",
                icon_class("search", "text-muted")
            ));
        }
        if let Some(title) = filter {
            out.push_str(&self.faceted_filter(title));
        }
        out.push_str("<div class=\"cui-dt-toolbar-end\">");
        if density {
            let id = self.id("density");
            out.push_str(&format!(
                "<label class=\"cui-dt-density\"><input type=\"checkbox\" id=\"{id}\" aria-label=\"Compact\"><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\" aria-pressed=\"false\" tabindex=\"-1\" aria-hidden=\"true\">{}<span class=\"when-off\">Compact</span><span class=\"when-on\">Comfortable</span></button></label>",
                crate::cronus_ui_icons::svg_or_empty("sliders-horizontal")
            ));
        }
        if visibility {
            out.push_str(&self.visibility_menu());
        }
        out.push_str("</div></div>");
        out
    }

    fn faceted_filter(&self, title: &str) -> String {
        let key = slug(title);
        let index = self.columns.iter().position(|c| c.key == key);
        let icons: Vec<&str> = attr_nonempty(self.comp, "filter-icons")
            .map(|s| s.split(',').map(str::trim).collect())
            .unwrap_or_default();
        // Distinct values in first-seen order, with facet counts.
        let mut options: Vec<(String, usize)> = Vec::new();
        if let Some(i) = index {
            for row in &self.rows {
                if let Some(cell) = row.get(i) {
                    match options.iter_mut().find(|(v, _)| *v == cell.text) {
                        Some(o) => o.1 += 1,
                        None => options.push((cell.text.clone(), 1)),
                    }
                }
            }
        }
        let trigger_id = self.id("filter");
        let pop_id = self.id("filter-menu");
        let items: String = options
            .iter()
            .enumerate()
            .map(|(n, (value, count))| {
                let icon = icons
                    .get(n)
                    .map(|i| icon_class(i, "text-tertiary"))
                    .unwrap_or_default();
                let mut label = value.clone();
                if let Some(first) = label.get(..1) {
                    label = format!("{}{}", first.to_uppercase(), &label[1..]);
                }
                format!(
                    "<label data-slot=\"dropdown-menu-checkbox-item\" role=\"menuitemcheckbox\"><input type=\"checkbox\" value=\"{value}\"><span class=\"cui-dt-check\">{}</span>{icon}<span>{label}</span><span class=\"cui-dt-facet\">{count}</span></label>",
                    crate::cronus_ui_icons::svg_or_empty("check")
                )
            })
            .collect();
        format!(
            "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"data-table-faceted-filter\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\" popovertarget=\"{pop_id}\" aria-haspopup=\"menu\">{SVG_OPEN}{CIRCLE_PLUS}{title}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"dropdown-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{trigger_id}\" class=\"cui-dt-filter-menu\"><div data-slot=\"dropdown-menu-label\">{title}</div><div data-slot=\"dropdown-menu-separator\" role=\"separator\"></div>{items}</div>",
            title = esc(title)
        )
    }

    fn visibility_menu(&self) -> String {
        let trigger_id = self.id("view");
        let pop_id = self.id("view-menu");
        let offset = usize::from(self.selection);
        let items: String = self
            .columns
            .iter()
            .enumerate()
            .map(|(n, c)| {
                let column = n + 1 + offset;
                let data = if column <= MAX_HIDEABLE {
                    format!(" data-column=\"{column}\"")
                } else {
                    String::new()
                };
                format!(
                    "<label data-slot=\"dropdown-menu-checkbox-item\" role=\"menuitemcheckbox\" class=\"capitalize\"{data}><input type=\"checkbox\" checked><span class=\"cui-dt-check\">{}</span>{}</label>",
                    crate::cronus_ui_icons::svg_or_empty("check"),
                    c.key
                )
            })
            .collect();
        format!(
            "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\" popovertarget=\"{pop_id}\" aria-haspopup=\"menu\">{}View{}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"dropdown-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{trigger_id}\" class=\"cui-dt-view-menu\"><div data-slot=\"dropdown-menu-label\">Toggle columns</div><div data-slot=\"dropdown-menu-separator\" role=\"separator\"></div>{items}</div>",
            crate::cronus_ui_icons::svg_or_empty("columns-3"),
            icon_class("chevron-down", "opacity-60")
        )
    }

    fn bulk_bar(&self) -> String {
        if !self.selection {
            return String::new();
        }
        let actions: String = self
            .comp
            .items
            .iter()
            .filter(|i| i.item_type == "action" && !i.text.is_empty())
            .map(|a| {
                let mut inner = crate::cronus_ui_kit::item_icon(a);
                inner.push_str(&esc(&a.text));
                if a.config
                    .get("count")
                    .is_some_and(|c| crate::cronus_ui_kit::truthy(c))
                {
                    inner.push_str(" <span class=\"cui-dt-count\"></span>");
                }
                let variant = a
                    .config
                    .get("variant")
                    .map(String::as_str)
                    .unwrap_or("outline");
                crate::cronus_ui::button_html(&inner, variant, "sm", a.link.as_deref(), false, None)
            })
            .collect();
        if actions.is_empty() {
            return String::new();
        }
        format!(
            "<section aria-label=\"Bulk actions\" data-slot=\"data-table-bulk-actions\"><span aria-live=\"polite\"><span class=\"cui-dt-count\"></span> selected</span><div>{actions}</div></section>"
        )
    }

    fn footer(&self) -> String {
        if self.loading || self.error.is_some() {
            return String::new();
        }
        let total = self.rows.len();
        let selected = format!(
            "<p aria-live=\"polite\" class=\"cui-dt-selected\"><span class=\"cui-dt-count\"></span> of {total} row(s) selected.</p>"
        );
        if !self.pagination {
            return if self.selection {
                selected
            } else {
                String::new()
            };
        }
        let sizes: Vec<usize> = attr_nonempty(self.comp, "page-sizes")
            .map(|s| s.split(',').filter_map(|p| p.trim().parse().ok()).collect())
            .filter(|v: &Vec<usize>| !v.is_empty())
            .unwrap_or_else(|| vec![10, 20, 30, 50]);
        let page_size = self.page_size;
        let page_count = total
            .div_ceil(page_size)
            .max(if total == 0 { 0 } else { 1 });
        let first = if total == 0 { 0 } else { 1 };
        let last = page_size.min(total);
        let can_next = page_count > 1;
        let size_id = self.id("page-size");
        let pop_id = self.id("page-size-menu");
        let name = self.id("page-size-value");
        let labels: String = sizes
            .iter()
            .take(12)
            .enumerate()
            .map(|(i, s)| format!(" data-o{}=\"{s}\"", i + 1))
            .collect();
        let options: String = sizes
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let checked = if *s == page_size { " checked" } else { "" };
                format!(
                    "<label data-slot=\"select-item\" data-option=\"{}\"><input type=\"radio\" name=\"{name}\" value=\"{s}\"{checked}><span>{s}</span></label>",
                    i + 1
                )
            })
            .collect();
        let nav_button = |label: &str, glyph: String, enabled: bool, page: Option<&str>| {
            let disabled = if enabled { "" } else { " disabled" };
            let page_attr = page
                .map(|p| format!(" data-table-page=\"{p}\""))
                .unwrap_or_default();
            format!(
                "<button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"icon-sm\" class=\"cui-btn\"{page_attr} aria-label=\"{label}\"{disabled}>{glyph}</button>"
            )
        };
        let nav = format!(
            "<nav aria-label=\"Pagination\">{}{}{}{}</nav>",
            nav_button(
                "Go to first page",
                format!("{SVG_OPEN}{CHEVRONS_LEFT}"),
                false,
                None
            ),
            nav_button(
                "Go to previous page",
                crate::cronus_ui_icons::svg_or_empty("chevron-left"),
                true,
                Some("prev")
            ),
            nav_button(
                "Go to next page",
                crate::cronus_ui_icons::svg_or_empty("chevron-right"),
                true,
                Some("next")
            ),
            nav_button(
                "Go to last page",
                format!("{SVG_OPEN}{CHEVRONS_RIGHT}"),
                can_next,
                None
            ),
        );
        let pagination = format!(
            "<div data-slot=\"data-table-pagination\"><p aria-live=\"polite\" class=\"cui-dt-range\">{first}–{last} of {total}</p><div class=\"cui-dt-pager\"><div class=\"cui-dt-page-size\"><label for=\"{size_id}\">Rows per page</label><button type=\"button\" id=\"{size_id}\" role=\"combobox\" aria-expanded=\"false\" aria-autocomplete=\"none\" aria-label=\"Rows per page\" data-state=\"closed\" data-slot=\"select-trigger\" class=\"h-8\" popovertarget=\"{pop_id}\" aria-controls=\"{pop_id}\"><span{labels}>{page_size}</span>{}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"select-content\" role=\"radiogroup\" aria-label=\"Rows per page\" anchor=\"{size_id}\">{options}</div></div><p class=\"cui-dt-page\">Page {} of {page_count}</p>{nav}</div></div>",
            icon_class("chevron-down", "opacity-60"),
            if page_count == 0 { 0 } else { 1 }
        );
        let start = if self.selection {
            selected
        } else {
            "<span></span>".to_string()
        };
        format!("<div class=\"cui-dt-foot\">{start}<div class=\"ms-auto\">{pagination}</div></div>")
    }
}

fn class_attr(classes: &[&str]) -> String {
    if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    }
}

/// A vendored lucide glyph with an extra class (React `className` on the icon).
fn icon_class(name: &str, class: &str) -> String {
    crate::cronus_ui_icons::svg_or_empty(name).replacen(
        "<svg ",
        &format!("<svg class=\"{class}\" "),
        1,
    )
}

/// React `Checkbox` in a cell: the native input carries the state (zero JS),
/// the Radix button is decorative (same pattern as the checkbox family).
fn checkbox(label: &str, id: &str) -> String {
    format!(
        "<label data-control=\"checkbox\"><input type=\"checkbox\" id=\"{id}\" aria-label=\"{label}\"><button type=\"button\" role=\"checkbox\" aria-checked=\"false\" data-state=\"unchecked\" value=\"on\" data-slot=\"checkbox\" tabindex=\"-1\" aria-hidden=\"true\"><span data-state=\"checked\">{}</span></button></label>",
        crate::cronus_ui_icons::svg_or_empty("check")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn extra(item_type: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: item_type.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("text-align: start;padding:0.5rem 0.75rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("zinc-"));
    }

    fn th(t: &str) -> String {
        format!(
            "<th data-slot=\"table-head\" colspan=\"1\" scope=\"col\" aria-sort=\"none\">{t}</th>"
        )
    }

    fn column(title: &str, pairs: &[(&str, &str)]) -> ComponentItemNode {
        let mut c = extra("columns", title);
        for (k, v) in pairs {
            c.config.insert(k.to_string(), v.to_string());
        }
        c
    }

    fn row(first: &str, pairs: &[(&str, &str)]) -> ComponentItemNode {
        let mut r = extra("item", first);
        for (k, v) in pairs {
            r.config.insert(k.to_string(), v.to_string());
        }
        r
    }

    /// The docs' members table: five sortable columns and two keyed rows.
    fn members() -> ComponentNode {
        let mut c = stub("data-table", "Members");
        c.items = vec![
            column("Name", &[("sort", "true"), ("font", "medium")]),
            column("Email", &[("sort", "true"), ("color", "secondary")]),
            column("Role", &[("sort", "true"), ("badge", "outline")]),
            column(
                "Status",
                &[
                    ("sort", "true"),
                    ("badge", "tone"),
                    ("transform", "capitalize"),
                    ("tones", "active=success,invited=warning,suspended=error"),
                ],
            ),
            column(
                "Seats",
                &[("sort", "true"), ("font", "mono"), ("color", "secondary")],
            ),
            row(
                "Ada Lovelace",
                &[
                    ("email", "ada@cronus.dev"),
                    ("role", "Owner"),
                    ("status", "active"),
                    ("seats", "5"),
                ],
            ),
            row(
                "Katherine Johnson",
                &[
                    ("email", "katherine@cronus.dev"),
                    ("role", "Member"),
                    ("status", "invited"),
                    ("seats", "1"),
                ],
            ),
        ];
        c
    }

    #[test]
    fn label_only_is_name_value_dummy_in_react_table_dom() {
        let html = render(&stub("data-table", "People"));
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"data-table\"><div data-slot=\"data-table-container\"><section data-slot=\"table-container\" tabindex=\"0\" aria-label=\"Table\"><table data-slot=\"table\"><thead data-slot=\"table-header\"><tr data-slot=\"table-row\">{}{}</tr></thead><tbody data-slot=\"table-body\"><tr data-slot=\"table-row\"><td data-slot=\"table-cell\">People</td><td data-slot=\"table-cell\">—</td></tr></tbody></table></section></div></div>",
                th("Name"),
                th("Value")
            )
        );
        reject_interact(&html);
    }

    #[test]
    fn columns_are_headers_remaining_are_cells() {
        let mut c = stub("data-table", "People");
        c.items = vec![
            extra("columns", "Name"),
            extra("columns", "Role"),
            extra("item", "Ada"),
            extra("item", "Engineer"),
            extra("item", "Grace"),
            extra("item", "Admiral"),
        ];
        let html = render(&c);
        assert!(html.contains(&format!("{}{}", th("Name"), th("Role"))));
        assert!(html.contains("<tr data-slot=\"table-row\"><td data-slot=\"table-cell\">Ada</td><td data-slot=\"table-cell\">Engineer</td></tr>"));
        assert!(html.contains(
            "<td data-slot=\"table-cell\">Grace</td><td data-slot=\"table-cell\">Admiral</td>"
        ));
        reject_interact(&html);
    }

    /// `label` + `text` items (the emitter's shape): the label names the table
    /// and the texts fill it — texts used to be dropped as "field" kinds.
    #[test]
    fn text_items_fill_table_and_label_is_not_content() {
        let mut c = stub("data-table", "Members");
        for t in ["Name", "Role", "Ada", "Admin", "Linus", "Editor"] {
            c.items.push(extra("text", t));
        }
        let html = render(&c);
        assert!(html.contains(&format!(
            "<tr data-slot=\"table-row\">{}{}</tr>",
            th("Name"),
            th("Role")
        )));
        assert!(html.contains(
            "<td data-slot=\"table-cell\">Ada</td><td data-slot=\"table-cell\">Admin</td>"
        ));
        assert!(html.contains(
            "<td data-slot=\"table-cell\">Linus</td><td data-slot=\"table-cell\">Editor</td>"
        ));
        assert!(!html.contains("Members"));
        assert_eq!(html.matches("data-slot=\"table-row\"").count(), 3);
        reject_interact(&html);
    }

    #[test]
    fn bound_rows_fill_cells_without_voodoo() {
        use crate::binding::ResolvedData;
        let rows = vec![serde_json::json!({"name": "Ada", "role": "Eng"})];
        crate::cronus_ui_data::with_binding("Lead", &ResolvedData::Rows(rows), || {
            crate::voodoo::with_enabled(true, || {
                let html = render(&stub("data-table", "People"));
                assert!(html.contains(&th("name")));
                assert!(html.contains(&th("role")));
                assert!(html.contains("<td data-slot=\"table-cell\">Ada</td>"));
                reject_interact(&html);
            });
        });
    }

    /// Docs "Sortable": keyed rows land under their column, sortable headers
    /// are React's `DataTableColumnHeader` (idle ghost button + chevrons),
    /// column config styles cells and badges follow the tone map.
    #[test]
    fn keyed_rows_sort_headers_and_column_styles() {
        let html = render(&members());
        assert!(html.contains("<th data-slot=\"table-head\" colspan=\"1\" scope=\"col\" aria-sort=\"none\" data-sort=\"0\" class=\"font-medium\"><button type=\"button\" data-slot=\"data-table-column-header\" data-variant=\"ghost\" data-size=\"sm\" class=\"cui-btn\" data-sort=\"0\" aria-label=\"Sort by Name. Activate to sort ascending.\"><span>Name</span><svg class=\"opacity-60\" "));
        assert!(html.contains("data-icon=\"chevrons-up-down\""));
        assert!(html.contains("<td data-slot=\"table-cell\" class=\"font-medium\">Ada Lovelace</td><td data-slot=\"table-cell\" class=\"text-secondary\">ada@cronus.dev</td><td data-slot=\"table-cell\"><span data-slot=\"badge\" data-variant=\"outline\">Owner</span></td><td data-slot=\"table-cell\" class=\"capitalize\"><span data-slot=\"badge\" data-variant=\"success\">active</span></td><td data-slot=\"table-cell\" class=\"font-mono text-secondary\">5</td>"));
        assert!(html.contains("<span data-slot=\"badge\" data-variant=\"warning\">invited</span>"));
        assert_eq!(html.matches("<tr data-slot=\"table-row\">").count(), 3);
        assert!(!html.contains("data-table-toolbar"));
        assert!(!html.contains("data-table-pagination"));
        reject_interact(&html);
    }

    /// Docs "Basic": `sort:button` is the plain ghost Button with ArrowUpDown,
    /// `transform:` classes wrap the cell text.
    #[test]
    fn basic_payments_sort_button_and_transforms() {
        let mut c = stub("data-table", "Payments");
        c.items = vec![
            column("Status", &[("transform", "capitalize")]),
            column("Email", &[("transform", "lowercase")]),
            column("Amount", &[("sort", "button"), ("font", "mono")]),
            row(
                "success",
                &[("email", "ken99@example.com"), ("amount", "$316.00")],
            ),
        ];
        let html = render(&c);
        assert!(html.contains("<th data-slot=\"table-head\" colspan=\"1\" scope=\"col\" aria-sort=\"none\" data-sort=\"2\" class=\"font-mono\"><button type=\"button\" data-slot=\"button\" data-variant=\"ghost\" data-size=\"sm\" class=\"cui-btn cui-dt-sort\" data-sort=\"2\">Amount<svg "));
        assert!(html.contains("data-icon=\"arrow-up-down\""));
        assert!(html.contains("<td data-slot=\"table-cell\" class=\"capitalize\">success</td><td data-slot=\"table-cell\" class=\"lowercase\">ken99@example.com</td><td data-slot=\"table-cell\" class=\"font-mono\">$316.00</td>"));
        reject_interact(&html);
    }

    /// Docs "Search & filter": the toolbar carries the search input (sr-only
    /// label, Search glyph) and the faceted filter (dashed outline trigger,
    /// native popover menu with facet counts).
    #[test]
    fn searchable_and_faceted_filter_toolbar() {
        let mut c = members();
        c.props.insert("searchable".into(), "true".into());
        c.props
            .insert("search-placeholder".into(), "Search members…".into());
        c.props.insert("filter".into(), "Status".into());
        c.props
            .insert("filter-icons".into(), "check,circle-dashed".into());
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"data-table\"><div role=\"group\" aria-label=\"Table controls\" data-slot=\"data-table-toolbar\"><div class=\"cui-dt-search\"><svg class=\"text-muted\" "));
        assert!(html.contains("<label for=\"cui-data-table-search\" class=\"sr-only\">Search members…</label><input data-slot=\"data-table-search\" type=\"search\" id=\"cui-data-table-search\" placeholder=\"Search members…\" class=\"h-9 ps-9\" data-search=\"cui-data-table-search\" />"));
        assert!(html
            .contains("<table data-slot=\"table\" data-search-target=\"cui-data-table-search\">"));
        assert!(html.contains("<tr data-slot=\"table-row\" data-search-item>"));
        assert!(html.contains("<button type=\"button\" id=\"cui-data-table-filter\" data-slot=\"data-table-faceted-filter\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\" popovertarget=\"cui-data-table-filter-menu\" aria-haspopup=\"menu\"><svg "));
        assert!(html.contains("</svg>Status</button><div id=\"cui-data-table-filter-menu\" popover=\"auto\" data-slot=\"dropdown-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"cui-data-table-filter\" class=\"cui-dt-filter-menu\"><div data-slot=\"dropdown-menu-label\">Status</div><div data-slot=\"dropdown-menu-separator\" role=\"separator\"></div><label data-slot=\"dropdown-menu-checkbox-item\" role=\"menuitemcheckbox\"><input type=\"checkbox\" value=\"active\">"));
        assert!(html.contains("data-icon=\"circle-dashed\""));
        assert!(html.contains("<span>Active</span><span class=\"cui-dt-facet\">1</span></label>"));
        assert!(html.contains("<span>Invited</span><span class=\"cui-dt-facet\">1</span></label>"));
        assert!(html.contains("<div class=\"cui-dt-toolbar-end\"></div></div><div data-slot=\"data-table-container\">"));
        reject_interact(&html);
    }

    /// Docs "Pagination": five rows per page, the range readout, the native
    /// rows-per-page select and the four icon buttons (first/previous dimmed).
    #[test]
    fn pagination_footer_shows_first_page() {
        let mut c = members();
        for n in 0..10 {
            c.items.push(row(
                &format!("Member {n}"),
                &[
                    ("email", "m@cronus.dev"),
                    ("role", "Member"),
                    ("status", "active"),
                    ("seats", "1"),
                ],
            ));
        }
        c.props.insert("pagination".into(), "true".into());
        c.props.insert("page-size".into(), "5".into());
        c.props.insert("page-sizes".into(), "5,10,20".into());
        let html = render(&c);
        assert_eq!(html.matches("<tr data-slot=\"table-row\">").count(), 6);
        assert!(html.contains("<div class=\"cui-dt-foot\"><span></span><div class=\"ms-auto\"><div data-slot=\"data-table-pagination\"><p aria-live=\"polite\" class=\"cui-dt-range\">1–5 of 12</p>"));
        assert!(html.contains("<label for=\"cui-data-table-page-size\">Rows per page</label><button type=\"button\" id=\"cui-data-table-page-size\" role=\"combobox\" aria-expanded=\"false\" aria-autocomplete=\"none\" aria-label=\"Rows per page\" data-state=\"closed\" data-slot=\"select-trigger\" class=\"h-8\" popovertarget=\"cui-data-table-page-size-menu\" aria-controls=\"cui-data-table-page-size-menu\"><span data-o1=\"5\" data-o2=\"10\" data-o3=\"20\">5</span>"));
        assert!(html.contains("<label data-slot=\"select-item\" data-option=\"1\"><input type=\"radio\" name=\"cui-data-table-page-size-value\" value=\"5\" checked><span>5</span></label>"));
        assert!(html.contains("<p class=\"cui-dt-page\">Page 1 of 3</p><nav aria-label=\"Pagination\"><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"icon-sm\" class=\"cui-btn\" aria-label=\"Go to first page\" disabled>"));
        assert!(html.contains("data-table-page=\"prev\" aria-label=\"Go to previous page\">"));
        assert!(html.contains("data-table-page=\"next\" aria-label=\"Go to next page\">"));
        assert!(html.contains("data-icon=\"chevron-left\""));
        reject_interact(&html);
    }

    /// Docs "Selection & bulk actions": a checkbox column (native inputs),
    /// the bulk bar with the selected counter and the selected-of-total line.
    #[test]
    fn selection_column_bulk_bar_and_counts() {
        let mut c = members();
        c.props.insert("selection".into(), "true".into());
        let mut mail = extra("action", "Email");
        mail.config.insert("icon".into(), "mail".into());
        mail.config.insert("variant".into(), "outline".into());
        mail.config.insert("count".into(), "true".into());
        c.items.push(mail);
        let mut remove = extra("action", "Remove");
        remove.config.insert("variant".into(), "destructive".into());
        c.items.push(remove);
        let html = render(&c);
        assert!(html.contains("<tr data-slot=\"table-row\"><th data-slot=\"table-head\" colspan=\"1\" scope=\"col\"><label data-control=\"checkbox\"><input type=\"checkbox\" id=\"cui-data-table-select-all\" aria-label=\"Select all rows on this page\"><button type=\"button\" role=\"checkbox\" aria-checked=\"false\" data-state=\"unchecked\" value=\"on\" data-slot=\"checkbox\" tabindex=\"-1\" aria-hidden=\"true\">"));
        assert!(html.contains("<td data-slot=\"table-cell\"><label data-control=\"checkbox\"><input type=\"checkbox\" id=\"cui-data-table-row-2\" aria-label=\"Select row 2\">"));
        assert!(html.contains("</div><section aria-label=\"Bulk actions\" data-slot=\"data-table-bulk-actions\"><span aria-live=\"polite\"><span class=\"cui-dt-count\"></span> selected</span><div><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\"><svg "));
        assert!(html.contains("</svg>Email <span class=\"cui-dt-count\"></span></button><button type=\"button\" data-slot=\"button\" data-variant=\"destructive\" data-size=\"sm\" class=\"cui-btn\">Remove</button></div></section><p aria-live=\"polite\" class=\"cui-dt-selected\"><span class=\"cui-dt-count\"></span> of 2 row(s) selected.</p></div>"));
        c.props.insert("pagination".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(
            "<div class=\"cui-dt-foot\"><p aria-live=\"polite\" class=\"cui-dt-selected\">"
        ));
        reject_interact(&html);
        let css = include_str!("cronus_ui_css/data-table.css");
        assert!(css.contains("[data-slot=\"data-table\"] { counter-reset: cui-selected; }"));
        assert!(css.contains("[data-slot=\"data-table\"]:has([data-slot=\"table-body\"] input:checked) > [data-slot=\"data-table-bulk-actions\"] { display: flex; }"));
        assert!(css.contains(".cui-dt-count::before { content: counter(cui-selected); }"));
    }

    /// Docs "Column visibility" and "Density": the View menu lists every
    /// column as a checked checkbox item; the density switch is a native
    /// checkbox behind React's `aria-pressed` outline button.
    #[test]
    fn view_menu_and_density_switch() {
        let mut c = members();
        c.props.insert("column-visibility".into(), "true".into());
        c.props.insert("density-toggle".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("<div class=\"cui-dt-toolbar-end\"><label class=\"cui-dt-density\"><input type=\"checkbox\" id=\"cui-data-table-density\" aria-label=\"Compact\"><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\" aria-pressed=\"false\" tabindex=\"-1\" aria-hidden=\"true\"><svg "));
        assert!(html.contains("</svg><span class=\"when-off\">Compact</span><span class=\"when-on\">Comfortable</span></button></label><button type=\"button\" id=\"cui-data-table-view\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\" popovertarget=\"cui-data-table-view-menu\" aria-haspopup=\"menu\"><svg "));
        assert!(html.contains("</svg>View<svg class=\"opacity-60\" "));
        assert!(html.contains("<div data-slot=\"dropdown-menu-label\">Toggle columns</div><div data-slot=\"dropdown-menu-separator\" role=\"separator\"></div><label data-slot=\"dropdown-menu-checkbox-item\" role=\"menuitemcheckbox\" class=\"capitalize\" data-column=\"1\"><input type=\"checkbox\" checked>"));
        assert!(html.contains(
            "data-column=\"5\"><input type=\"checkbox\" checked><span class=\"cui-dt-check\">"
        ));
        assert!(html.contains("</span>seats</label></div>"));
        let css = include_str!("cronus_ui_css/data-table.css");
        assert!(css.contains("[data-slot=\"data-table\"]:has([data-column=\"3\"] > input:not(:checked)) :is([data-slot=\"table-head\"], [data-slot=\"table-cell\"]):nth-child(3) { display: none; }"));
        assert!(css.contains("[data-slot=\"data-table\"]:has(.cui-dt-density > input:checked) [data-slot=\"table-cell\"] { padding-top: 0.375rem; padding-bottom: 0.375rem; }"));
        reject_interact(&html);
    }

    /// Docs "Loading", "Empty", "Error": skeleton rows behind a busy region,
    /// the Empty composition in one centered cell, the alert row with Retry.
    #[test]
    fn loading_empty_and_error_states() {
        let mut c = members();
        c.props.insert("loading".into(), "true".into());
        c.props.insert("loading-rows".into(), "4".into());
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"data-table-container\" aria-busy=\"true\"><span role=\"status\" data-slot=\"data-table-loading\" class=\"sr-only\">Loading rows…</span>"));
        assert_eq!(html.matches("<tr data-slot=\"table-row\">").count(), 5);
        assert_eq!(
            html.matches(
                "<div data-slot=\"skeleton\" aria-hidden=\"true\" class=\"h-4 w-full\"></div>"
            )
            .count(),
            20
        );
        assert!(!html.contains("Ada Lovelace"));

        let mut e = members();
        e.items.truncate(5);
        e.props.insert("empty-icon".into(), "inbox".into());
        e.props
            .insert("empty-title".into(), "No members yet".into());
        e.props
            .insert("empty-description".into(), "Invite teammates.".into());
        let html = render(&e);
        assert!(html.contains("<tbody data-slot=\"table-body\"><tr data-slot=\"table-row\"><td data-slot=\"table-cell\" colspan=\"5\" class=\"cui-dt-empty\"><div data-slot=\"empty\"><div data-slot=\"empty-icon\"><svg "));
        assert!(html.contains("<div data-slot=\"empty-title\">No members yet</div><div data-slot=\"empty-description\">Invite teammates.</div></div></td></tr></tbody>"));
        e.props.remove("empty-title");
        assert!(render(&e).contains("class=\"cui-dt-empty\">No results.</td>"));

        let mut f = members();
        f.items.truncate(5);
        f.props
            .insert("error".into(), "Couldn’t load <team>.".into());
        f.props.insert("retry".into(), "true".into());
        let html = render(&f);
        assert!(html.contains("<td data-slot=\"table-cell\" colspan=\"5\" class=\"cui-dt-error\"><div><svg class=\"text-error\" "));
        assert!(html.contains("<div role=\"alert\">Couldn’t load &lt;team&gt;.</div><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\"><svg "));
        assert!(html.contains("</svg>Retry</button></div></td>"));
        assert!(!html.contains("data-table-pagination"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_inline_th_styles() {
        let c = stub("data-table", "People");
        let html = render(&c);
        assert!(!html.contains("style="));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("data-table", "People"));
            reject_interact(&html);
            assert!(html.contains("<table data-slot=\"table\">"));
        });
    }

    /// Geometry parity (Wave 1t): rounded bordered container, `text-sm`
    /// 14px/20px table, 40px `font-medium` heads, `p-3` cells, row dividers
    /// with none under the last body row.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = include_str!("cronus_ui_css/data-table.css");
        assert!(css.contains("[data-slot=\"data-table-container\"] {\n  overflow: hidden; border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border);\n}"));
        assert!(css.contains("[data-slot=\"data-table\"] [data-slot=\"table-head\"] {\n  height: 2.5rem; padding: 0 0.75rem; text-align: start; vertical-align: middle;\n  font-size: inherit; font-weight: 500; color: var(--cronus-fg-secondary); white-space: nowrap; border: 0;\n}"));
        assert!(css.contains("[data-slot=\"data-table\"] [data-slot=\"table-body\"] > [data-slot=\"table-row\"]:last-child { border-bottom: 0; }"));
        assert!(css.contains("[data-slot=\"data-table-column-header\"] {"));
        assert!(css.contains("[data-slot=\"data-table-toolbar\"] { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; }"));
        assert!(!css.contains("zinc-"));
    }
}

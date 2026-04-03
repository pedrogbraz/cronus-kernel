//! Form section renderer
use crate::parser::SectionNode;

pub(super) fn render_form_section(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
    let title = section.title.as_deref().unwrap_or("Form");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let entity = section.config.get("entity").map(|s| s.as_str()).unwrap_or("");

    // Check if we have a bound record (edit mode)
    let bound_record = match bound_data {
        crate::binding::ResolvedData::Record(Some(record)) => Some(record),
        _ => None,
    };
    let is_edit = bound_record.is_some();
    let record_id = bound_record
        .and_then(|r| r.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Check if on submit has "update entity" instruction
    let has_update_action = section.actions.iter().any(|a| {
        a.event == "submit" && a.instructions.iter().any(|i| i.verb == "update")
    });

    // Edit mode: use PATCH + include record ID in action URL
    let action = section.config.get("action").map(|s| s.to_string())
        .unwrap_or_else(|| {
            if !entity.is_empty() {
                if (is_edit || has_update_action) && !record_id.is_empty() {
                    format!("/api/{}s/{}", entity.to_lowercase(), record_id)
                } else {
                    format!("/api/{}s", entity.to_lowercase())
                }
            } else {
                "#".to_string()
            }
        });
    let method = if is_edit || has_update_action {
        "PATCH"
    } else {
        section.config.get("method").map(|s| s.as_str()).unwrap_or("POST")
    };

    let mut fields_html = String::new();
    let mut actions_html = String::new();
    let mut links_html = String::new();

    // Count short fields (text/email/tel/url/password) for grid layout
    let short_field_types = ["text", "email", "tel", "url", "password", "number"];
    let field_items: Vec<&std::collections::HashMap<String, String>> = section.items.iter()
        .filter(|i| i.get("_type").map(|s| s.as_str()) == Some("field"))
        .collect();
    let total_fields = field_items.len();
    let first_two_short = total_fields >= 2
        && field_items.get(0).and_then(|i| i.get("type")).map(|t| short_field_types.contains(&t.as_str())).unwrap_or(true)
        && field_items.get(1).and_then(|i| i.get("type")).map(|t| short_field_types.contains(&t.as_str())).unwrap_or(true);
    let mut field_index: usize = 0;

    for item in &section.items {
        let itype = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        let item_title = item.get("title").map(|s| s.as_str()).unwrap_or("");

        if itype == "field" {
            // Grid row: open wrapper before first field, close after second field
            if first_two_short && field_index == 0 {
                fields_html.push_str(r#"<div style="display:grid;grid-template-columns:1fr 1fr;gap:16px">"#);
            }

            let ftype = item.get("type").map(|s| s.as_str()).unwrap_or("text");
            let name_lower = item_title.to_lowercase().replace(' ', "_");
            let placeholder = item.get("placeholder").map(|s| s.as_str()).unwrap_or("");
            let required = if item.get("required").map(|s| s == "true").unwrap_or(false) { "required" } else { "" };
            let disabled = if item.get("disabled").map(|s| s == "true").unwrap_or(false) { "disabled" } else { "" };
            let readonly = if item.get("readonly").map(|s| s == "true").unwrap_or(false) { "readonly" } else { "" };
            // Pre-fill from bound record if in edit mode, otherwise use static value
            let static_value = item.get("value").map(|s| s.as_str()).unwrap_or("");
            let bound_value_owned: String;
            let value = if let Some(record) = bound_record {
                let field_val = record.get(&name_lower)
                    .or_else(|| record.get(item_title));
                match field_val {
                    Some(serde_json::Value::String(s)) => s.as_str(),
                    Some(serde_json::Value::Number(n)) => {
                        bound_value_owned = n.to_string();
                        bound_value_owned.as_str()
                    }
                    Some(serde_json::Value::Bool(b)) => {
                        bound_value_owned = b.to_string();
                        bound_value_owned.as_str()
                    }
                    Some(v) if !v.is_null() => {
                        bound_value_owned = v.to_string();
                        bound_value_owned.as_str()
                    }
                    _ => static_value,
                }
            } else {
                static_value
            };

            let label_style = "display:block;font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;color:var(--cronus-text-muted);margin-bottom:8px";
            let input_style = "width:100%;padding:12px 16px;border:1px solid var(--cronus-border);border-radius:var(--cronus-radius);font-size:14px;outline:none;font-family:var(--cronus-font);transition:border-color 0.2s;box-sizing:border-box;background:var(--cronus-surface);color:var(--cronus-text)";

            match ftype {
                "text" | "email" | "password" | "url" | "tel" | "number" => {
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><input type="{ftype}" name="{name}" placeholder="{placeholder}" value="{value}" {required} {disabled} {readonly} style="{input_style}" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'"></div>"#,
                        label_style = label_style, label = item_title, ftype = ftype, name = name_lower,
                        placeholder = placeholder, value = value, required = required,
                        disabled = disabled, readonly = readonly, input_style = input_style,
                    ));
                }
                "select" => {
                    let options_raw = item.get("options").map(|s| s.as_str()).unwrap_or("");
                    let options: Vec<&str> = if options_raw.is_empty() { vec![] } else { options_raw.split("||").collect() };
                    let mut opts_html = format!(r#"<option value="">Select {}...</option>"#, item_title);
                    for opt in &options {
                        let selected = if *opt == value { " selected" } else { "" };
                        opts_html.push_str(&format!(r#"<option value="{v}"{sel}>{v}</option>"#, v = opt, sel = selected));
                    }
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><select name="{name}" {required} {disabled} style="{input_style};appearance:none;background:#fff url('data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 width=%2212%22 height=%2212%22 viewBox=%220 0 12 12%22><path d=%22M2 4l4 4 4-4%22 fill=%22none%22 stroke=%22%2371717a%22 stroke-width=%221.5%22/></svg>') no-repeat right 12px center">{options}</select></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        required = required, disabled = disabled, input_style = input_style, options = opts_html,
                    ));
                }
                "textarea" => {
                    let rows = item.get("rows").map(|s| s.as_str()).unwrap_or("4");
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><textarea name="{name}" rows="{rows}" placeholder="{placeholder}" {required} {disabled} {readonly} style="{input_style};resize:vertical" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'">{textarea_val}</textarea></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        rows = rows, placeholder = placeholder, required = required,
                        disabled = disabled, readonly = readonly, input_style = input_style,
                        textarea_val = value,
                    ));
                }
                "checkbox" => {
                    fields_html.push_str(&format!(
                        r#"<label style="display:flex;align-items:center;gap:12px;cursor:pointer"><input type="checkbox" name="{name}" {disabled} style="width:18px;height:18px;accent-color:#000"><span style="font-size:14px">{label}</span></label>"#,
                        name = name_lower, disabled = disabled, label = item_title,
                    ));
                }
                "radio" => {
                    let options_raw = item.get("options").map(|s| s.as_str()).unwrap_or("");
                    let options: Vec<&str> = if options_raw.is_empty() { vec![] } else { options_raw.split("||").collect() };
                    let mut radio_html = String::new();
                    for opt in &options {
                        radio_html.push_str(&format!(
                            r#"<label style="display:flex;align-items:center;gap:8px;cursor:pointer"><input type="radio" name="{name}" value="{val}" {disabled} style="accent-color:#000"><span style="font-size:14px">{val}</span></label>"#,
                            name = name_lower, val = opt, disabled = disabled,
                        ));
                    }
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><div style="display:flex;flex-direction:column;gap:8px">{radios}</div></div>"#,
                        label_style = label_style, label = item_title, radios = radio_html,
                    ));
                }
                "file" => {
                    let accept = item.get("accept").map(|s| s.as_str()).unwrap_or("");
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><input type="file" name="{name}" accept="{accept}" {required} {disabled} style="width:100%;padding:10px;border:1px dashed #e5e7eb;border-radius:8px;font-size:14px;cursor:pointer;box-sizing:border-box"></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        accept = accept, required = required, disabled = disabled,
                    ));
                }
                "date" => {
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><input type="date" name="{name}" value="{value}" {required} {disabled} {readonly} style="{input_style}" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'"></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        value = value, required = required, disabled = disabled,
                        readonly = readonly, input_style = input_style,
                    ));
                }
                "money" => {
                    let prefix = item.get("prefix").map(|s| s.as_str()).unwrap_or("$");
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><div style="position:relative"><span style="position:absolute;left:16px;top:50%;transform:translateY(-50%);color:#71717a;font-weight:600">{prefix}</span><input type="number" step="0.01" name="{name}" placeholder="{placeholder}" value="{value}" {required} {disabled} {readonly} style="{input_style};padding-left:32px" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'"></div></div>"#,
                        label_style = label_style, label = item_title, prefix = prefix,
                        name = name_lower, placeholder = placeholder, value = value,
                        required = required, disabled = disabled, readonly = readonly,
                        input_style = input_style,
                    ));
                }
                "color" => {
                    let default_color = if value.is_empty() { "#000000" } else { value };
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><div style="display:flex;align-items:center;gap:12px"><input type="color" name="{name}" value="{color}" {disabled} style="width:48px;height:48px;border:none;border-radius:8px;cursor:pointer" oninput="document.getElementById('{name}-label').textContent=this.value"><span style="font-size:14px;color:#71717a" id="{name}-label">{color}</span></div></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        color = default_color, disabled = disabled,
                    ));
                }
                "tags" => {
                    let tag_placeholder = if placeholder.is_empty() { "Type and press Enter..." } else { placeholder };
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><div style="display:flex;flex-wrap:wrap;gap:8px;padding:8px 12px;border:1px solid #e5e7eb;border-radius:8px;min-height:48px;align-items:center;transition:border-color 0.2s" onfocusin="this.style.borderColor='#000'" onfocusout="this.style.borderColor='#e5e7eb'"><input type="text" placeholder="{tag_placeholder}" data-name="{name}" style="border:none;outline:none;flex:1;min-width:100px;font-size:14px;font-family:Inter,sans-serif" onkeydown="if(event.key==='Enter'){{event.preventDefault();cronusAddTag(this)}}"></div><input type="hidden" name="{name}" value=""></div>"#,
                        label_style = label_style, label = item_title,
                        tag_placeholder = tag_placeholder, name = name_lower,
                    ));
                }
                "range" => {
                    let min_val = item.get("min").map(|s| s.as_str()).unwrap_or("0");
                    let max_val = item.get("max").map(|s| s.as_str()).unwrap_or("100");
                    let default_val = if value.is_empty() { "50" } else { value };
                    let step = item.get("step").map(|s| s.as_str()).unwrap_or("1");
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><div><div style="display:flex;justify-content:space-between;margin-bottom:8px"><span style="font-size:12px;color:#71717a">{min}</span><span style="font-size:14px;font-weight:600" id="{name}-value">{default}</span><span style="font-size:12px;color:#71717a">{max}</span></div><input type="range" min="{min}" max="{max}" value="{default}" step="{step}" name="{name}" {disabled} style="width:100%;accent-color:#000" oninput="document.getElementById('{name}-value').textContent=this.value"></div></div>"#,
                        label_style = label_style, label = item_title,
                        min = min_val, max = max_val, default = default_val,
                        step = step, name = name_lower, disabled = disabled,
                    ));
                }
                "search" => {
                    let search_entity = item.get("entity").map(|s| s.as_str())
                        .or_else(|| section.config.get("entity").map(|s| s.as_str()))
                        .unwrap_or("item");
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><div style="position:relative"><span class="material-symbols-outlined" style="position:absolute;left:12px;top:50%;transform:translateY(-50%);color:#71717a;font-size:18px">search</span><input type="text" name="{name}" placeholder="Search {entity}..." data-entity="{entity}" style="{input_style};padding-left:40px" oninput="cronusSearch(this,'{entity}')" onfocus="this.style.borderColor='#000'" onblur="setTimeout(()=>{{this.style.borderColor='#e5e7eb';document.getElementById('{name}-results').style.display='none'}},200)"><div id="{name}-results" style="display:none;position:absolute;top:100%;left:0;right:0;background:#fff;border:1px solid #e5e7eb;border-radius:8px;margin-top:4px;max-height:200px;overflow-y:auto;box-shadow:0 8px 24px rgba(0,0,0,0.1);z-index:10"></div></div></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        entity = search_entity, input_style = input_style,
                    ));
                }
                _ => {
                    // Fallback: treat as text
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><input type="text" name="{name}" placeholder="{placeholder}" value="{value}" {required} {disabled} {readonly} style="{input_style}" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'"></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        placeholder = placeholder, value = value, required = required,
                        disabled = disabled, readonly = readonly, input_style = input_style,
                    ));
                }
            }

            // Inline validation: error message div
            if let Some(error_msg) = item.get("error") {
                let err_name = item_title.to_lowercase().replace(' ', "_");
                fields_html.push_str(&format!(
                    r#"<p style="display:none;font-size:12px;color:#dc2626;margin:4px 0 0" data-error="{name}">{msg}</p>"#,
                    name = err_name, msg = error_msg,
                ));
            }

            // Close grid row wrapper after second field
            if first_two_short && field_index == 1 {
                fields_html.push_str("</div>");
            }
            field_index += 1;
        } else if itype == "action" {

            let variant = item.get("variant").map(|s| s.as_str())
                .or_else(|| item.get("style").map(|s| s.as_str()))
                .unwrap_or("primary");
            let (bg, color) = match variant {
                "secondary" | "outline" => ("var(--cronus-surface)", "var(--cronus-text)"),
                _ => ("var(--cronus-accent)", "#fff"),
            };
            let border = if variant == "outline" || variant == "secondary" { "1px solid var(--cronus-border)" } else { "none" };
            actions_html.push_str(&format!(
                r#"<button type="submit" data-label="{label}" style="width:100%;padding:14px;border:{border};border-radius:999px;background:{bg};color:{color};font-size:16px;font-weight:700;cursor:pointer;font-family:var(--cronus-font)" class="btn-hover">{label}</button>"#,
                label = item_title, bg = bg, color = color, border = border,
            ));
        } else if itype == "link" {
            let link = item.get("link").map(|s| s.as_str()).unwrap_or("#");
            links_html.push_str(&format!(
                r#"<a href="{link}" style="text-align:center;font-size:14px;color:var(--cronus-accent);text-decoration:none">{text}</a>"#,
                link = link, text = item_title,
            ));
        }
    }

    // If no explicit action item, add a default submit button
    if actions_html.is_empty() {
        actions_html = r#"<button type="submit" data-label="Save" style="width:100%;padding:14px;border:none;border-radius:999px;background:var(--cronus-accent);color:#fff;font-size:16px;font-weight:700;cursor:pointer;font-family:var(--cronus-font)" class="btn-hover">Save</button>"#.to_string();
    }

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(r#"<p style="font-size:14px;color:#5e5e5e;margin:0" class="anim-slide-up d2">{}</p>"#, subtitle)
    };

    let data_entity = if !entity.is_empty() { format!(r#" data-entity="{}""#, entity) } else { String::new() };
    let data_cronus_entity = if !entity.is_empty() { format!(r#" data-cronus-entity="{}""#, entity) } else { String::new() };

    // Edit mode: add data-cronus-id and data-cronus-method for the JS submit handler
    let edit_attrs = if is_edit && !record_id.is_empty() {
        format!(r#" data-cronus-id="{}" data-cronus-method="PATCH""#, record_id)
    } else {
        String::new()
    };

    // Hidden field for record ID in edit mode
    let hidden_id = if is_edit && !record_id.is_empty() {
        format!(r#"<input type="hidden" name="_id" value="{}">"#, record_id)
    } else {
        String::new()
    };

    format!(
        r##"<section style="padding:48px 24px">
  <div style="max-width:var(--cronus-max-w, 1120px);margin:0 auto">
  <div style="margin-bottom:32px">
    <h2 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;margin:0 0 8px" class="anim-slide-up d1">{title}</h2>
    {subtitle_html}
  </div>
  <form id="cronus-form" action="{action}" method="{method}"{data_entity}{data_cronus_entity} data-cronus-form data-cronus-section="{section_type}"{edit_attrs} style="display:flex;flex-direction:column;gap:20px" class="anim-slide-up d3">
    {hidden_id}
    {fields}
    <div id="form-msg" style="display:none;padding:12px 16px;border-radius:8px;font-size:14px;font-weight:500"></div>
    {actions}
    {links}
  </form>
  </div>
</section>"##,
        title = title, subtitle_html = subtitle_html, action = action, method = method,
        data_entity = data_entity, data_cronus_entity = data_cronus_entity,
        section_type = section.section_type, edit_attrs = edit_attrs,
        hidden_id = hidden_id, fields = fields_html, actions = actions_html, links = links_html,
    )
}


use crate::parser::SectionNode;

// ── Alert ──────────────────────────────────────────────────────────

pub fn render_alert(section: &SectionNode) -> String {
    let style = section
        .config
        .get("style")
        .map(|s| s.as_str())
        .unwrap_or("info");
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    let (bg, border, text, icon_color, icon_name) = match style {
        "success" => (
            "bg-emerald-50",
            "border-emerald-200",
            "text-emerald-800",
            "text-emerald-600",
            "check_circle",
        ),
        "warning" => (
            "bg-amber-50",
            "border-amber-200",
            "text-amber-800",
            "text-amber-600",
            "warning",
        ),
        "error" => (
            "bg-red-50",
            "border-red-200",
            "text-red-800",
            "text-red-600",
            "error",
        ),
        _ => (
            "bg-blue-50",
            "border-blue-200",
            "text-blue-800",
            "text-blue-600",
            "info",
        ),
    };

    let close_btn = format!(
        r#"<button onclick="this.closest('.cronus-alert').remove()" class="ml-auto opacity-60 hover:opacity-100 transition-opacity cursor-pointer">
            <span class="material-symbols-rounded {icon_color}" style="font-size:20px">close</span>
        </button>"#,
        icon_color = icon_color,
    );

    let mut html = String::new();
    html.push_str(&format!(
        r#"<div class="cronus-alert flex items-start gap-3 p-4 rounded-xl border {bg} {border} {text}" role="alert">"#,
        bg = bg, border = border, text = text,
    ));

    // Icon
    html.push_str(&format!(
        r#"<span class="material-symbols-rounded {icon_color} shrink-0" style="font-size:24px">{icon_name}</span>"#,
        icon_color = icon_color, icon_name = icon_name,
    ));

    // Content
    html.push_str(r#"<div class="flex-1 min-w-0">"#);
    if !title.is_empty() {
        html.push_str(&format!(r#"<p class="font-medium text-sm">{}</p>"#, title,));
    }
    if !subtitle.is_empty() {
        html.push_str(&format!(
            r#"<p class="text-sm opacity-80 mt-0.5">{}</p>"#,
            subtitle,
        ));
    }
    html.push_str("</div>");

    // Close button
    html.push_str(&close_btn);

    html.push_str("</div>");
    html
}

// ── Accordion ──────────────────────────────────────────────────────

pub fn render_accordion(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let id_base = format!("cronus-accordion-{}", title.len());

    let mut html = String::new();

    // Container
    html.push_str(r#"<div class="cronus-accordion">"#);

    // Optional heading
    if !title.is_empty() {
        html.push_str(&format!(
            r#"<h3 class="text-base font-semibold text-neutral-900 mb-3">{}</h3>"#,
            title,
        ));
    }

    html.push_str(r#"<div class="border rounded-xl overflow-hidden divide-y divide-neutral-200">"#);

    for (i, item) in section.items.iter().enumerate() {
        let question = item
            .get("name")
            .or_else(|| item.get("title"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let answer = item.get("description").map(|s| s.as_str()).unwrap_or("");
        let panel_id = format!("{}-panel-{}", id_base, i);

        // Header
        html.push_str(&format!(
            r#"<div class="cronus-accordion-item">
<div class="flex justify-between items-center px-4 py-3.5 cursor-pointer select-none hover:bg-neutral-50 transition-colors" onclick="cronusToggleAccordion(this)" role="button" aria-expanded="false" aria-controls="{panel_id}">
  <span class="text-sm font-medium text-neutral-900">{question}</span>
  <span class="material-symbols-rounded cronus-accordion-icon text-neutral-400 transition-transform duration-200" style="font-size:20px">expand_more</span>
</div>
<div id="{panel_id}" class="px-4 pb-4 text-sm text-neutral-600 overflow-hidden" style="max-height:0;opacity:0;transition:max-height 0.25s ease,opacity 0.2s ease;padding-top:0;padding-bottom:0">
  <div class="pt-0 pb-1">{answer}</div>
</div>
</div>"#,
            panel_id = panel_id,
            question = question,
            answer = answer,
        ));
    }

    html.push_str("</div>"); // border container
    html.push_str("</div>"); // cronus-accordion

    // JavaScript (inline, idempotent via window check)
    html.push_str("<script");
    html.push_str(crate::security::script_nonce_attr());
    html.push_str(
        r#">
if(!window._cronusAccordionInit){window._cronusAccordionInit=true;
window.cronusToggleAccordion=function(el){
  var panel=el.nextElementSibling;
  var icon=el.querySelector('.cronus-accordion-icon');
  var isOpen=panel.style.maxHeight&&panel.style.maxHeight!=='0px';
  if(isOpen){
    panel.style.maxHeight='0px';
    panel.style.opacity='0';
    panel.style.paddingTop='0';
    panel.style.paddingBottom='0';
    icon.style.transform='rotate(0deg)';
    el.setAttribute('aria-expanded','false');
  }else{
    panel.style.maxHeight=panel.scrollHeight+'px';
    panel.style.opacity='1';
    panel.style.paddingTop='0';
    panel.style.paddingBottom='16px';
    icon.style.transform='rotate(180deg)';
    el.setAttribute('aria-expanded','true');
  }
};
}
</script>"#,
    );

    html
}

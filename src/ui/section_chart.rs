//! Chart section renderers (bar, line, donut)
use crate::parser::SectionNode;

pub(super) fn render_chart_section(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
    let chart_type = section.config.get("type").map(|s| s.as_str()).unwrap_or("bar");
    let title = section.title.as_deref().unwrap_or("Chart");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    // When bound_data has Rows, extract chart data from DB rows.
    // Uses "label"/"name"/"title" field for label, "value"/"amount"/"count" for numeric value.
    let bound_chart_data: Vec<(String, f64)> = if let crate::binding::ResolvedData::Rows(rows) = bound_data {
        rows.iter().filter_map(|row| {
            let label = row.get("label").or_else(|| row.get("name")).or_else(|| row.get("title"))
                .and_then(|v| v.as_str())
                .map(crate::security::html_escape)
                .unwrap_or_default();
            if label.is_empty() { return None; }
            let val = row.get("value").or_else(|| row.get("amount")).or_else(|| row.get("count"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            Some((label, val))
        }).collect()
    } else {
        Vec::new()
    };

    let data: Vec<(String, f64)> = if !bound_chart_data.is_empty() {
        bound_chart_data
    } else {
        // Fallback: parse from static items
        section.items.iter().filter_map(|item| {
            let label = item.get("title")?.clone();
            let val_str = item.get("description")?;
            let val: f64 = val_str.trim().parse().ok()?;
            Some((label, val))
        }).collect()
    };

    if data.is_empty() {
        // No data — render placeholder chart with title (dark dashboard style)
        let periods_raw = section.config.get("periods").map(|s| s.as_str()).unwrap_or("7 Days,30 Days");
        let period_items: Vec<&str> = periods_raw.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let mut period_html = String::from(r#"<div style="display:flex;gap:8px">"#);
        for (pi, p) in period_items.iter().enumerate() {
            if pi == period_items.len() - 1 {
                period_html.push_str(&format!(
                    r#"<button style="padding:4px 12px;font-size:10px;font-weight:700;text-transform:uppercase;background:linear-gradient(135deg,#87adff,#d277ff);border:none;border-radius:4px;color:#fff;cursor:pointer">{p}</button>"#
                ));
            } else {
                period_html.push_str(&format!(
                    r#"<button style="padding:4px 12px;font-size:10px;font-weight:700;text-transform:uppercase;background:#2a2a2a;border:none;border-radius:4px;color:rgba(226,226,226,0.6);cursor:pointer">{p}</button>"#
                ));
            }
        }
        period_html.push_str("</div>");

        let y_axis_raw = section.config.get("y_axis").map(|s| s.as_str()).unwrap_or("1.5M,1.0M,0.5M,0.0");
        let y_labels: Vec<&str> = y_axis_raw.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let y_spans: String = y_labels.iter().map(|l| format!("<span>{l}</span>")).collect::<Vec<_>>().join("");
        let y_axis_html = format!(
            r#"<div style="display:flex;flex-direction:column;justify-content:space-between;font-size:10px;color:rgba(226,226,226,0.3);width:40px;padding:8px 0">{y_spans}</div>"#
        );

        let x_axis_raw = section.config.get("x_axis").map(|s| s.as_str()).unwrap_or("Oct 01,Oct 08,Oct 15,Oct 22,Oct 29");
        let x_items: Vec<&str> = x_axis_raw.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let x_spans: String = x_items.iter().map(|l| format!("<span>{l}</span>")).collect::<Vec<_>>().join("");
        let x_labels = format!(
            r#"<div style="display:flex;justify-content:space-between;padding:16px 48px;font-size:10px;color:rgba(226,226,226,0.4);text-transform:uppercase;letter-spacing:0.15em;background:#1b1b1b">{x_spans}</div>"#
        );

        // SVG area chart with purple-blue gradient
        let is_area = chart_type == "area";
        let chart_html = if is_area {
            // Smooth area chart with gradient fill
            let points = [
                (0,160),(30,140),(60,120),(90,135),(120,100),(150,110),(180,80),
                (210,90),(240,60),(270,70),(300,45),(330,55),(360,35),(390,50),
                (420,30),(450,40),(480,25),(510,45),(540,55),(570,40),(600,50)
            ];
            let mut path_line = String::new();
            let mut path_area = String::new();
            for (i, (x, y)) in points.iter().enumerate() {
                if i == 0 {
                    path_line.push_str(&format!("M{},{}", x, y));
                    path_area.push_str(&format!("M{},{}", x, y));
                } else {
                    let prev = points[i - 1];
                    let cx = (prev.0 + x) / 2;
                    path_line.push_str(&format!(" C{},{} {},{} {},{}", cx, prev.1, cx, y, x, y));
                    path_area.push_str(&format!(" C{},{} {},{} {},{}", cx, prev.1, cx, y, x, y));
                }
            }
            path_area.push_str(" L600,200 L0,200 Z");

            format!(
                r##"<div style="position:relative;height:200px;overflow:hidden;padding:8px 4px">
<svg viewBox="0 0 600 200" preserveAspectRatio="none" style="width:100%;height:100%">
  <defs>
    <linearGradient id="area-grad" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#d277ff" stop-opacity="0.3"/>
      <stop offset="50%" stop-color="#87adff" stop-opacity="0.1"/>
      <stop offset="100%" stop-color="#87adff" stop-opacity="0.02"/>
    </linearGradient>
    <linearGradient id="line-grad" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#87adff"/>
      <stop offset="100%" stop-color="#d277ff"/>
    </linearGradient>
  </defs>
  <path d="{area}" fill="url(#area-grad)" opacity="0" style="animation:cronusFadeIn 1.2s ease-out 0.3s forwards"/>
  <path d="{line}" fill="none" stroke="url(#line-grad)" stroke-width="2" stroke-linecap="round"
    stroke-dasharray="1200" stroke-dashoffset="1200" style="animation:cronusDrawLine 2s ease-out 0.5s forwards"/>
  <line x1="0" y1="50" x2="600" y2="50" stroke="rgba(226,226,226,0.05)" stroke-dasharray="4"/>
  <line x1="0" y1="100" x2="600" y2="100" stroke="rgba(226,226,226,0.05)" stroke-dasharray="4"/>
  <line x1="0" y1="150" x2="600" y2="150" stroke="rgba(226,226,226,0.05)" stroke-dasharray="4"/>
</svg>
<div class="anim-breathe" style="position:absolute;bottom:30%;left:50%;width:60%;height:1px;background:linear-gradient(90deg,transparent,rgba(210,119,255,0.15),transparent);transform:translateX(-50%)"></div>
<style>@keyframes cronusFadeIn{{from{{opacity:0}}to{{opacity:1}}}}@keyframes cronusDrawLine{{to{{stroke-dashoffset:0}}}}</style>
</div>"##,
                area = path_area, line = path_line
            )
        } else {
            // Bar chart — matching Nova Core reference design
            // Heights as percentages, colors alternate primary/secondary
            let bars: Vec<(&str, &str)> = vec![
                ("50%", "rgba(135,173,255,0.2)"), ("66%", "rgba(135,173,255,0.2)"),
                ("75%", "rgba(135,173,255,0.3)"), ("50%", "rgba(135,173,255,0.2)"),
                ("80%", "rgba(210,119,255,0.4)"), ("66%", "rgba(135,173,255,0.2)"),
                ("60%", "rgba(135,173,255,0.2)"), ("50%", "rgba(135,173,255,0.2)"),
                ("75%", "rgba(135,173,255,0.3)"), ("80%", "rgba(135,173,255,0.2)"),
                ("100%","rgba(135,173,255,0.4)"),
            ];
            let mut bars_html = String::new();
            for (h, bg) in &bars {
                bars_html.push_str(&format!(
                    r##"<div style="flex:1;height:{h};background:{bg};border-radius:2px 2px 0 0;transition:background 0.2s" onmouseover="this.style.background='rgba(135,173,255,0.4)'" onmouseout="this.style.background='{bg}'"></div>"##,
                    h = h, bg = bg
                ));
            }
            format!(
                r##"<div style="position:relative;height:260px;overflow:hidden;background:#000;border-radius:8px;border-top:0.5px solid rgba(135,173,255,0.2);box-shadow:0 0 60px rgba(135,173,255,0.04)"><div style="position:absolute;inset:0;opacity:0.2;background:linear-gradient(135deg,rgba(135,173,255,1),rgba(210,119,255,1));filter:blur(48px)"></div><div style="position:relative;width:100%;height:100%;padding:16px 16px 32px;display:flex;align-items:flex-end;justify-content:space-between;gap:4px">{bars}</div></div>"##,
                bars = bars_html
            )
        };

        return format!(
            r#"<div style="margin-bottom:48px;background:#0e0e0e;border-radius:12px;border:0.5px solid rgba(76,69,70,0.15);overflow:hidden"><div style="padding:24px 32px;display:flex;justify-content:space-between;align-items:center;background:#1b1b1b"><div><h3 style="font-size:18px;font-weight:600;margin:0;color:#e2e2e2">{}</h3><p style="font-size:12px;color:rgba(226,226,226,0.4);margin:4px 0 0">{}</p></div>{}</div><div style="display:flex;padding:24px 32px;background:#0e0e0e">{}{}</div>{}</div>"#,
            title, subtitle, period_html, y_axis_html, chart_html, x_labels
        );
    }

    match chart_type {
        "area" => render_chart_line(title, subtitle, &data),
        "line" => render_chart_line(title, subtitle, &data),
        "donut" => render_chart_donut(title, subtitle, &data),
        _ => render_chart_bar(title, subtitle, &data),
    }
}

pub(super) fn render_chart_bar(title: &str, subtitle: &str, data: &[(String, f64)]) -> String {
    let max_val = data.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
    if max_val == 0.0 {
        return String::new();
    }

    let bars: Vec<String> = data.iter().enumerate().map(|(i, (label, value))| {
        let percent = (value / max_val) * 100.0;
        let delay = format!("{:.2}s", 0.1 + i as f64 * 0.08);
        format!(
            r##"<div style="flex:1;display:flex;flex-direction:column;align-items:center;gap:8px">
        <span class="anim-count" style="font-size:11px;font-weight:600;color:#1a1c1c;animation-delay:{delay}">{value}</span>
        <div style="width:100%;background:#000;border-radius:4px 4px 0 0;height:{percent}%" class="chart-bar-anim" style="animation-delay:{delay}"></div>
        <span style="font-size:11px;color:#71717a">{label}</span>
      </div>"##,
            value = *value as i64,
            percent = percent as i64,
            label = label,
            delay = delay,
        )
    }).collect();

    format!(
        r##"<div class="anim-slide-up card-hover" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">
  <div style="margin-bottom:20px">
    <h3 style="font-size:16px;font-weight:700;margin:0 0 4px">{title}</h3>
    <p style="font-size:13px;color:#71717a;margin:0">{subtitle}</p>
  </div>
  <div style="display:flex;align-items:flex-end;gap:8px;height:200px;padding-top:16px">
    {bars}
  </div>
</div>"##,
        title = title,
        subtitle = subtitle,
        bars = bars.join("\n    "),
    )
}

pub(super) fn render_chart_line(title: &str, subtitle: &str, data: &[(String, f64)]) -> String {
    let max_val = data.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
    if max_val == 0.0 || data.len() < 2 {
        return String::new();
    }

    let width = 600;
    let height = 200;
    let padding = 20;
    let usable_w = width - 2 * padding;
    let usable_h = height - 2 * padding;
    let n = data.len();

    let points: Vec<(i32, i32)> = data.iter().enumerate().map(|(i, (_, v))| {
        let x = padding + (i as i32 * usable_w / (n as i32 - 1));
        let y = padding + (usable_h as f64 * (1.0 - v / max_val)) as i32;
        (x, y)
    }).collect();

    let polyline_pts: String = points.iter().map(|(x, y)| format!("{},{}", x, y)).collect::<Vec<_>>().join(" ");
    let fill_pts = format!("{} {},{} {},{}",
        polyline_pts,
        points.last().unwrap().0, height,
        points.first().unwrap().0, height
    );

    let circles: String = points.iter().map(|(x, y)| {
        format!(r##"<circle cx="{}" cy="{}" r="4" fill="#000"/>"##, x, y)
    }).collect::<Vec<_>>().join("\n    ");

    let labels: String = data.iter().map(|(label, _)| {
        format!(r##"<span style="font-size:11px;color:#71717a">{}</span>"##, label)
    }).collect::<Vec<_>>().join("\n    ");

    format!(
        r##"<div class="anim-slide-up" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">
  <div style="margin-bottom:20px">
    <h3 style="font-size:16px;font-weight:700;margin:0 0 4px">{title}</h3>
    <p style="font-size:13px;color:#71717a;margin:0">{subtitle}</p>
  </div>
  <svg viewBox="0 0 {width} {height}" style="width:100%;height:200px">
    <defs><linearGradient id="lineFill" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="rgba(0,0,0,0.08)"/><stop offset="100%" stop-color="rgba(0,0,0,0)"/></linearGradient></defs>
    <polyline points="{fill_pts}" fill="url(#lineFill)" stroke="none" style="opacity:0;animation:fadeIn 1s ease 0.8s both"/>
    <polyline points="{polyline_pts}" fill="none" stroke="#000" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="chart-line-draw" style="--line-len:2000;stroke-dasharray:2000"/>
    {circles}
  </svg>
  <div style="display:flex;justify-content:space-between;padding-top:8px">
    {labels}
  </div>
</div>"##,
        title = title,
        subtitle = subtitle,
        width = width,
        height = height,
        polyline_pts = polyline_pts,
        fill_pts = fill_pts,
        circles = circles,
        labels = labels,
    )
}

pub(super) fn render_chart_donut(title: &str, subtitle: &str, data: &[(String, f64)]) -> String {
    let total: f64 = data.iter().map(|(_, v)| *v).sum();
    if total == 0.0 {
        return String::new();
    }

    let circumference: f64 = 2.0 * std::f64::consts::PI * 50.0;
    let colors = ["#000", "#71717a", "#a1a1aa", "#d4d4d8", "#e5e7eb", "#f3f4f6"];

    let mut offset = 0.0_f64;
    let segments: Vec<String> = data.iter().enumerate().map(|(i, (_, v))| {
        let arc = (v / total) * circumference;
        let color = colors[i % colors.len()];
        let delay = format!("{:.2}", 0.2 + i as f64 * 0.15);
        let seg = format!(
            r##"<circle cx="60" cy="60" r="50" fill="none" stroke="{color}" stroke-width="10" class="chart-donut-draw" style="--arc:{arc};stroke-dashoffset:{offset};animation-delay:{delay}s"/>"##,
            color = color,
            arc = arc,
            offset = -offset,
            delay = delay,
        );
        offset += arc;
        seg
    }).collect();

    let legend: String = data.iter().enumerate().map(|(i, (label, value))| {
        let color = colors[i % colors.len()];
        format!(
            r##"<div style="display:flex;align-items:center;gap:8px">
      <div style="width:12px;height:12px;border-radius:3px;background:{color};flex-shrink:0"></div>
      <span style="font-size:13px;color:#1a1c1c">{label}</span>
      <span style="font-size:13px;color:#71717a;margin-left:auto">{value}</span>
    </div>"##,
            color = color,
            label = label,
            value = *value as i64,
        )
    }).collect::<Vec<_>>().join("\n    ");

    format!(
        r##"<div class="anim-slide-up" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">
  <div style="margin-bottom:20px">
    <h3 style="font-size:16px;font-weight:700;margin:0 0 4px">{title}</h3>
    <p style="font-size:13px;color:#71717a;margin:0">{subtitle}</p>
  </div>
  <div style="display:flex;align-items:center;gap:32px">
    <svg viewBox="0 0 120 120" style="width:120px;height:120px;transform:rotate(-90deg)">
      <circle cx="60" cy="60" r="50" fill="none" stroke="#e5e7eb" stroke-width="10"/>
      {segments}
    </svg>
    <div style="display:flex;flex-direction:column;gap:12px">
      {legend}
    </div>
  </div>
</div>"##,
        title = title,
        subtitle = subtitle,
        segments = segments.join("\n      "),
        legend = legend,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::ResolvedData;
    use serde_json::json;
    use std::collections::HashMap;

    fn chart(kind: &str) -> SectionNode {
        let mut config = HashMap::new();
        config.insert("type".to_string(), kind.to_string());
        SectionNode {
            section_type: "chart".into(),
            title: None,
            subtitle: None,
            config,
            items: vec![],
            plans: vec![],
            binding: None,
            actions: vec![],
            visibility: None,
            template: None,
            style_block: None,
            doc: None,
        }
    }

    #[test]
    fn chart_labels_from_rows_are_escaped_for_every_chart_type() {
        let rows = ResolvedData::Rows(vec![
            json!({"label": "<svg onload=alert(1)>", "value": 3}),
            json!({"name": "\"><script>c()</script>", "value": 5}),
        ]);
        for kind in ["bar", "line", "donut"] {
            let html = render_chart_section(&chart(kind), &rows);
            assert!(!html.is_empty(), "{kind}");
            assert!(!html.contains("<svg onload"), "{kind}: {html}");
            assert!(!html.contains("<script>c()"), "{kind}: {html}");
            assert!(html.contains("&lt;svg onload=alert(1)&gt;"), "{kind}: {html}");
        }
    }
}


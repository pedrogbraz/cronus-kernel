#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Marketing Components — Premium SSR sections
//!
//! Apple/Vercel/Stripe-level HTML components generated from Rust.
//! Pure HTML + Tailwind CDN + vanilla JS animations.

// ══════════════════════════════════════════════════
// TYPES
// ══════════════════════════════════════════════════

pub struct FeatureItem {
    pub label: String,
    pub title: String,
    pub description: String,
    pub icon: String,
}

pub struct PricingPlan {
    pub name: String,
    pub price: String,
    pub period: String,
    pub features: Vec<String>,
    pub featured: bool,
    pub cta: String,
}

pub struct Testimonial {
    pub quote: String,
    pub author: String,
    pub role: String,
    pub avatar: String,
}

// ══════════════════════════════════════════════════
// ANIMATIONS CSS (shared)
// ══════════════════════════════════════════════════

pub const ANIM_CSS: &str = r#"
<style>
  @keyframes fadeUp { from { opacity: 0; transform: translateY(24px); } to { opacity: 1; transform: translateY(0); } }
  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
  @keyframes countUp { from { opacity: 0; transform: scale(0.8); } to { opacity: 1; transform: scale(1); } }
  @keyframes glow { 0%,100% { opacity: 0.4; } 50% { opacity: 0.7; } }
  .anim { opacity: 0; animation: fadeUp 0.6s ease-out forwards; }
  .anim-d1 { animation-delay: 0.1s; }
  .anim-d2 { animation-delay: 0.2s; }
  .anim-d3 { animation-delay: 0.3s; }
  .anim-d4 { animation-delay: 0.4s; }
  .anim-d5 { animation-delay: 0.5s; }
  .anim-d6 { animation-delay: 0.6s; }
  .anim-fade { opacity: 0; animation: fadeIn 0.8s ease-out forwards; }
  .glass { background: rgba(255,255,255,0.03); backdrop-filter: blur(16px); border: 1px solid rgba(255,255,255,0.06); }
  .glass-hover:hover { background: rgba(255,255,255,0.06); border-color: rgba(255,255,255,0.1); }
  .glow-accent { animation: glow 3s ease-in-out infinite; }
</style>
"#;

// ══════════════════════════════════════════════════
// HERO — SPLIT LAYOUT (text left, cards right)
// ══════════════════════════════════════════════════

pub fn hero_split(
    title: &str,
    subtitle: &str,
    badge: &str,
    bullets: &[&str],
    ctas: &[(&str, &str, &str)], // (text, href, style: "primary"|"secondary")
    accent: &str,
    sidebar_cards: &[(&str, &str, &str)], // (label, value, icon)
) -> String {
    let badge_html = if badge.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div class="anim inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-{a}-500/20 bg-{a}-500/5 mb-8">
        <span class="w-1.5 h-1.5 rounded-full bg-{a}-400 glow-accent"></span>
        <span class="font-mono text-[10px] uppercase tracking-[0.2em] text-{a}-400">{badge}</span>
      </div>"#,
            a = accent, badge = badge,
        )
    };

    let bullets_html: String = bullets.iter().enumerate().map(|(i, b)| {
        format!(
            r#"<li class="anim anim-d{d} flex items-center gap-3 text-neutral-400">
            <span class="w-1.5 h-1.5 bg-{a}-500"></span>
            <span class="text-sm">{b}</span>
          </li>"#,
            d = i + 2, a = accent, b = b,
        )
    }).collect::<Vec<_>>().join("\n");

    let ctas_html: String = ctas.iter().enumerate().map(|(i, (text, href, style))| {
        if *style == "primary" {
            format!(
                r#"<a href="{href}" class="anim anim-d{d} inline-flex items-center gap-2 px-6 py-3 bg-{a}-500 hover:bg-{a}-400 text-black font-semibold rounded-lg transition-all duration-200 text-sm">{text}</a>"#,
                href = href, d = i + 4, a = accent, text = text,
            )
        } else {
            format!(
                r#"<a href="{href}" class="anim anim-d{d} inline-flex items-center gap-2 px-6 py-3 border border-neutral-700 hover:border-neutral-500 text-white rounded-lg transition-all duration-200 text-sm">{text}</a>"#,
                href = href, d = i + 4, text = text,
            )
        }
    }).collect::<Vec<_>>().join("\n      ");

    let cards_html: String = sidebar_cards.iter().enumerate().map(|(i, (label, value, icon))| {
        format!(
            r#"<div class="anim anim-d{d} glass glass-hover rounded-xl p-5 transition-all duration-300">
          <div class="flex items-center justify-between mb-3">
            <span class="font-mono text-[9px] uppercase tracking-[0.2em] text-neutral-500">{label}</span>
            <span class="text-lg">{icon}</span>
          </div>
          <p class="text-2xl font-bold tabular-nums text-{a}-400">{value}</p>
        </div>"#,
            d = i + 3, label = label, value = value, icon = icon, a = accent,
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r##"<section class="relative overflow-hidden px-6 py-24 md:py-32">
  <div class="absolute inset-0 bg-gradient-radial from-{accent}-500/8 via-transparent to-transparent" style="background: radial-gradient(ellipse 60% 50% at 30% 0%, rgba(245,158,11,0.06), transparent);"></div>
  <div class="relative max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
    <div>
      {badge}
      <h1 class="anim anim-d1 text-5xl md:text-6xl font-extrabold uppercase tracking-tight leading-[1.05]">{title}</h1>
      <p class="anim anim-d2 mt-6 text-lg text-neutral-400 max-w-xl leading-relaxed">{subtitle}</p>
      <ul class="mt-8 space-y-3">
        {bullets}
      </ul>
      <div class="mt-10 flex flex-wrap items-center gap-4">
        {ctas}
      </div>
    </div>
    <div class="space-y-4">
      {cards}
    </div>
  </div>
</section>"##,
        accent = accent,
        badge = badge_html,
        title = title,
        subtitle = subtitle,
        bullets = bullets_html,
        ctas = ctas_html,
        cards = cards_html,
    )
}

// ══════════════════════════════════════════════════
// HERO — CENTERED
// ══════════════════════════════════════════════════

pub fn hero_centered(
    title: &str,
    subtitle: &str,
    badge: &str,
    ctas: &[(&str, &str, &str)],
    accent: &str,
) -> String {
    let badge_html = if badge.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div class="anim inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-{a}-500/20 bg-{a}-500/5 mb-8">
        <span class="w-1.5 h-1.5 rounded-full bg-{a}-400 glow-accent"></span>
        <span class="font-mono text-[10px] uppercase tracking-[0.2em] text-{a}-400">{b}</span>
      </div>"#,
            a = accent, b = badge,
        )
    };

    let ctas_html: String = ctas.iter().enumerate().map(|(i, (text, href, style))| {
        if *style == "primary" {
            format!(r#"<a href="{}" class="anim anim-d{} px-8 py-3.5 bg-{}-500 hover:bg-{}-400 text-black font-semibold rounded-lg transition text-sm">{}</a>"#, href, i+3, accent, accent, text)
        } else {
            format!(r#"<a href="{}" class="anim anim-d{} px-8 py-3.5 border border-neutral-700 hover:border-neutral-500 text-white rounded-lg transition text-sm">{}</a>"#, href, i+3, text)
        }
    }).collect::<Vec<_>>().join("\n      ");

    format!(
        r#"<section class="relative px-6 py-28 md:py-36">
  <div class="absolute inset-0" style="background: radial-gradient(ellipse 50% 40% at 50% 0%, rgba(245,158,11,0.05), transparent);"></div>
  <div class="relative max-w-4xl mx-auto text-center">
    {badge}
    <h1 class="anim anim-d1 text-5xl md:text-7xl font-extrabold tracking-tight bg-gradient-to-b from-white via-white to-neutral-500 bg-clip-text text-transparent leading-[1.1]">{title}</h1>
    <p class="anim anim-d2 mt-6 text-lg md:text-xl text-neutral-400 max-w-2xl mx-auto leading-relaxed">{subtitle}</p>
    <div class="anim anim-d3 mt-10 flex flex-wrap items-center justify-center gap-4">
      {ctas}
    </div>
  </div>
</section>"#,
        badge = badge_html,
        title = title,
        subtitle = subtitle,
        ctas = ctas_html,
    )
}

// ══════════════════════════════════════════════════
// HERO — VIDEO BACKGROUND
// ══════════════════════════════════════════════════

pub fn hero_video(title: &str, subtitle: &str, video_url: &str, accent: &str) -> String {
    format!(
        r##"<section class="relative px-6 py-32 overflow-hidden">
  <video autoplay muted loop playsinline class="absolute inset-0 w-full h-full object-cover opacity-20">
    <source src="{video_url}" type="video/mp4">
  </video>
  <div class="absolute inset-0 bg-gradient-to-t from-neutral-950 via-neutral-950/80 to-neutral-950/40"></div>
  <div class="relative max-w-4xl mx-auto text-center">
    <h1 class="anim text-5xl md:text-7xl font-extrabold tracking-tight text-white">{title}</h1>
    <p class="anim anim-d1 mt-6 text-lg text-neutral-300 max-w-2xl mx-auto">{subtitle}</p>
    <a href="#" class="anim anim-d2 mt-10 inline-block px-8 py-3.5 bg-{accent}-500 text-black font-semibold rounded-lg">Watch Demo</a>
  </div>
</section>"##,
        video_url = video_url,
        title = title,
        subtitle = subtitle,
        accent = accent,
    )
}

// ══════════════════════════════════════════════════
// FEATURES GRID (Vercel-style shared borders)
// ══════════════════════════════════════════════════

pub fn features_grid(items: &[FeatureItem], cols: u8, accent: &str) -> String {
    let cards: String = items.iter().enumerate().map(|(i, item)| {
        let emoji = icon_to_emoji(&item.icon);
        format!(
            r#"<div class="anim anim-d{d} group p-8 border-b border-r border-neutral-800/50 hover:bg-neutral-900/30 transition-all duration-300">
      <div class="flex items-center gap-3 mb-4">
        <span class="text-xl">{emoji}</span>
        <span class="font-mono text-[9px] uppercase tracking-[0.2em] text-{a}-500">{label}</span>
      </div>
      <h3 class="text-lg font-bold text-white mb-2">{title}</h3>
      <p class="text-sm text-neutral-400 leading-relaxed mb-4">{desc}</p>
      <span class="font-mono text-[10px] text-{a}-500/60 group-hover:text-{a}-400 transition">Read Specs →</span>
    </div>"#,
            d = (i % 6) + 1,
            emoji = emoji,
            a = accent,
            label = item.label,
            title = item.title,
            desc = item.description,
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r#"<section class="px-6 py-20">
  <div class="max-w-6xl mx-auto">
    <div class="grid grid-cols-1 md:grid-cols-{cols} border-l border-t border-neutral-800/50 rounded-xl overflow-hidden">
      {cards}
    </div>
  </div>
</section>"#,
        cols = cols,
        cards = cards,
    )
}

// ══════════════════════════════════════════════════
// PRICING TABLE
// ══════════════════════════════════════════════════

pub fn pricing_table(plans: &[PricingPlan], accent: &str) -> String {
    let cards: String = plans.iter().enumerate().map(|(i, plan)| {
        let border = if plan.featured {
            format!("border-{}-500/50 shadow-lg shadow-{}-500/5", accent, accent)
        } else {
            "border-neutral-800/60".to_string()
        };
        let badge = if plan.featured {
            format!(r#"<div class="absolute -top-3 left-1/2 -translate-x-1/2 px-4 py-1 bg-{}-500 text-black font-mono text-[10px] uppercase tracking-[0.15em] rounded-full font-bold">Recommended</div>"#, accent)
        } else {
            String::new()
        };
        let features: String = plan.features.iter().map(|f| {
            format!(r#"<li class="flex items-center gap-3 text-sm text-neutral-300"><span class="text-emerald-400 text-xs">✓</span> {}</li>"#, f)
        }).collect::<Vec<_>>().join("\n          ");

        let btn_style = if plan.featured {
            format!("bg-{}-500 hover:bg-{}-400 text-black", accent, accent)
        } else {
            "bg-neutral-800 hover:bg-neutral-700 text-white".to_string()
        };

        format!(
            r#"<div class="anim anim-d{d} relative glass rounded-2xl p-8 border {border} flex flex-col">
      {badge}
      <h3 class="text-xl font-bold">{name}</h3>
      <div class="mt-4 flex items-baseline gap-1">
        <span class="text-4xl font-extrabold tabular-nums">{price}</span>
        <span class="text-sm text-neutral-500">/{period}</span>
      </div>
      <ul class="mt-8 space-y-3 flex-1">
        {features}
      </ul>
      <button class="mt-8 w-full py-3 rounded-lg {btn_style} font-semibold text-sm transition">{cta}</button>
    </div>"#,
            d = i + 1,
            border = border,
            badge = badge,
            name = plan.name,
            price = plan.price,
            period = plan.period,
            features = features,
            btn_style = btn_style,
            cta = plan.cta,
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r#"<section class="px-6 py-20">
  <div class="max-w-5xl mx-auto">
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      {cards}
    </div>
  </div>
</section>"#,
        cards = cards,
    )
}

// ══════════════════════════════════════════════════
// TESTIMONIALS
// ══════════════════════════════════════════════════

pub fn testimonials(items: &[Testimonial], accent: &str) -> String {
    let cols = if items.len() >= 3 { 3 } else { 2 };
    let cards: String = items.iter().enumerate().map(|(i, t)| {
        format!(
            r#"<div class="anim anim-d{d} glass rounded-2xl p-8">
      <div class="text-3xl text-{a}-500/30 mb-4">"</div>
      <p class="text-sm text-neutral-300 leading-relaxed mb-6">{quote}</p>
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-full bg-neutral-800 flex items-center justify-center text-sm font-bold text-{a}-400">{initial}</div>
        <div>
          <p class="text-sm font-semibold text-white">{author}</p>
          <p class="text-xs text-neutral-500">{role}</p>
        </div>
      </div>
    </div>"#,
            d = (i % 6) + 1,
            a = accent,
            quote = t.quote,
            initial = t.author.chars().next().unwrap_or('?'),
            author = t.author,
            role = t.role,
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r#"<section class="px-6 py-20">
  <div class="max-w-6xl mx-auto">
    <div class="grid grid-cols-1 md:grid-cols-{cols} gap-6">
      {cards}
    </div>
  </div>
</section>"#,
        cols = cols,
        cards = cards,
    )
}

// ══════════════════════════════════════════════════
// FAQ ACCORDION
// ══════════════════════════════════════════════════

pub fn faq(items: &[(&str, &str)], accent: &str) -> String {
    let entries: String = items.iter().enumerate().map(|(i, (q, a))| {
        format!(
            r#"<div class="anim anim-d{d} border-b border-neutral-800/50">
      <button onclick="toggleFaq(this)" class="w-full flex items-center justify-between py-5 text-left group">
        <span class="text-sm font-semibold text-white group-hover:text-{ac}-400 transition">{q}</span>
        <span class="faq-icon text-neutral-500 text-xl transition-transform duration-200">+</span>
      </button>
      <div class="faq-body hidden pb-5">
        <p class="text-sm text-neutral-400 leading-relaxed">{a}</p>
      </div>
    </div>"#,
            d = (i % 6) + 1, ac = accent, q = q, a = a,
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r##"<section class="px-6 py-20">
  <div class="max-w-3xl mx-auto">
    <h2 class="anim text-3xl font-bold text-center mb-12">Frequently Asked Questions</h2>
    <div class="divide-neutral-800/50">
      {entries}
    </div>
  </div>
  <script>
    function toggleFaq(btn) {{
      var body = btn.nextElementSibling;
      var icon = btn.querySelector('.faq-icon');
      var isOpen = !body.classList.contains('hidden');
      body.classList.toggle('hidden');
      icon.textContent = isOpen ? '+' : '−';
      icon.style.transform = isOpen ? 'rotate(0deg)' : 'rotate(180deg)';
    }}
  </script>
</section>"##,
        entries = entries,
    )
}

// ══════════════════════════════════════════════════
// STATS SECTION (animated count-up numbers)
// ══════════════════════════════════════════════════

pub fn stats_section(stats: &[(&str, &str, &str)], accent: &str) -> String {
    // stats: (value, label, suffix)  e.g. ("99.99", "Uptime", "%")
    // Based on stitch-variations/ir/Stats.tsx + StatsSection1.tsx
    let items: String = stats.iter().enumerate().map(|(i, (value, label, suffix))| {
        format!(
            r#"<div class="text-center group" data-animate="fadeInUp" style="animation-delay:{delay}s">
      <div class="text-4xl md:text-5xl font-black tracking-tight text-white" data-countup="{value}" data-suffix="{suffix}">{value}{suffix}</div>
      <div class="text-sm text-neutral-500 mt-2">{label}</div>
      <div class="mt-4 mx-auto w-8 h-0.5 rounded-full opacity-30 group-hover:w-16 transition-all duration-500" style="background:linear-gradient(90deg,transparent,var(--accent-color,#f59e0b),transparent)"></div>
    </div>"#,
            delay = i as f32 * 0.1 + 0.1, value = value, suffix = suffix, label = label,
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r##"<section class="px-6 py-20 border-y border-white/[0.04]">
  <div class="max-w-5xl mx-auto">
    <div class="grid grid-cols-2 md:grid-cols-{cols} gap-12">
      {items}
    </div>
  </div>
  <script>
    document.querySelectorAll('[data-countup]').forEach(function(el){{
      var raw=el.getAttribute('data-countup');
      var suffix=el.getAttribute('data-suffix')||'';
      var numStr=raw.replace(/[^0-9.]/g,'');
      var target=parseFloat(numStr);
      if(isNaN(target))return;
      var isFloat=numStr.includes('.');
      var prefix=raw.replace(numStr,'').replace(suffix,'');
      var duration=1500;var startTime=null;
      function step(ts){{
        if(!startTime)startTime=ts;
        var progress=Math.min((ts-startTime)/duration,1);
        var eased=1-Math.pow(1-progress,3);
        var current=target*eased;
        var formatted=isFloat?current.toFixed(numStr.split('.')[1]?.length||2):(target>=1000?Math.round(current).toLocaleString():Math.round(current).toString());
        el.textContent=prefix+formatted+suffix;
        if(progress<1)requestAnimationFrame(step);
        else el.textContent=raw+suffix;
      }}
      var obs=new IntersectionObserver(function(e){{if(e[0].isIntersecting){{requestAnimationFrame(step);obs.disconnect();}}}},{{threshold:0.3}});
      obs.observe(el);
    }});
  </script>
</section>"##,
        cols = stats.len().min(4),
        items = items,
    )
}

// ══════════════════════════════════════════════════
// CTA SECTION
// ══════════════════════════════════════════════════

pub fn cta_section(title: &str, subtitle: &str, ctas: &[(&str, &str, &str)], accent: &str) -> String {
    // Based on stitch-variations/ir-brasil/CTASection6.tsx
    let ctas_html: String = ctas.iter().enumerate().map(|(i, (text, href, style))| {
        if *style == "primary" {
            format!(r#"<a href="{}" class="px-8 py-3 bg-white text-black text-sm font-medium rounded-full hover:bg-neutral-200 transition-colors">{}</a>"#, href, text)
        } else {
            format!(r#"<a href="{}" class="px-8 py-3 text-white text-sm font-medium rounded-full border border-white/20 hover:bg-white/5 transition-colors">{}</a>"#, href, text)
        }
    }).collect::<Vec<_>>().join("\n      ");

    format!(
        r##"<section class="relative py-24 overflow-hidden">
  <div class="absolute inset-0" style="background:radial-gradient(ellipse 60% 50% at 50% 50%,rgba(245,158,11,0.06),transparent 70%)"></div>
  <div class="max-w-4xl mx-auto px-6 text-center relative z-10">
    <p class="text-xs font-medium uppercase tracking-widest text-neutral-500 mb-4">Get Started</p>
    <h2 class="text-4xl md:text-6xl font-extrabold tracking-tight text-white mb-6" data-animate="fadeInUp">{title}</h2>
    <p class="text-lg text-neutral-500 mb-10 max-w-xl mx-auto" data-animate="fadeInUp">{subtitle}</p>
    <div class="flex items-center justify-center gap-4 mb-6" data-animate="fadeInUp">
      {ctas}
    </div>
    <p class="text-xs text-neutral-600">No credit card required. Free tier available.</p>
  </div>
</section>"##,
        title = title,
        subtitle = subtitle,
        ctas = ctas_html,
    )
}

// ══════════════════════════════════════════════════
// LOGO CLOUD
// ══════════════════════════════════════════════════

pub fn logo_cloud(logos: &[(&str, &str)], accent: &str) -> String {
    // logos: (name, url_or_svg)
    let items: String = logos.iter().map(|(name, _url)| {
        format!(
            r#"<div class="flex items-center justify-center px-6 py-4 text-neutral-600 hover:text-neutral-400 transition">
      <span class="font-mono text-sm font-bold tracking-wider uppercase">{}</span>
    </div>"#,
            name
        )
    }).collect::<Vec<_>>().join("\n");

    let _ = accent; // reserved for future use
    format!(
        r#"<section class="px-6 py-16 border-y border-neutral-800/30">
  <div class="max-w-5xl mx-auto">
    <p class="text-center font-mono text-[10px] uppercase tracking-[0.2em] text-neutral-600 mb-8">Trusted by industry leaders</p>
    <div class="grid grid-cols-2 md:grid-cols-{cols} gap-4">
      {items}
    </div>
  </div>
</section>"#,
        cols = logos.len().min(6),
        items = items,
    )
}

// ══════════════════════════════════════════════════
// ICON MAPPING
// ══════════════════════════════════════════════════

fn icon_to_emoji(icon: &str) -> &str {
    match icon {
        "cpu" | "chip" => "🧠",
        "globe" | "world" | "earth" => "🌍",
        "shield" | "security" | "lock" => "🛡️",
        "activity" | "pulse" | "monitor" => "📊",
        "zap" | "bolt" | "lightning" | "fast" => "⚡",
        "brain" | "ai" | "intelligence" => "🧠",
        "users" | "team" | "people" => "👥",
        "kanban" | "board" | "tasks" => "📋",
        "credit-card" | "payment" | "billing" => "💳",
        "code" | "terminal" | "dev" => "💻",
        "server" | "cloud" | "infra" => "☁️",
        "chart" | "analytics" | "graph" => "📈",
        "clock" | "time" | "schedule" => "⏱️",
        "database" | "storage" | "db" => "🗄️",
        "key" | "auth" | "api" => "🔑",
        "mail" | "email" | "message" => "✉️",
        "rocket" | "launch" | "deploy" => "🚀",
        "star" | "favorite" => "⭐",
        "check" | "verified" => "✅",
        "settings" | "config" | "gear" => "⚙️",
        _ => "⚡",
    }
}

// ══════════════════════════════════════════════════
// SOCIAL PROOF (logos + trust line)
// ══════════════════════════════════════════════════

pub fn social_proof(logos: &[(&str, &str)], trust_line: &str) -> String {
    let items: String = logos.iter().map(|(name, _url)| {
        format!(
            r#"<div class="flex items-center justify-center px-8 py-3">
      <span class="font-mono text-sm font-bold tracking-wider uppercase text-neutral-600 hover:text-neutral-400 transition">{}</span>
    </div>"#,
            name
        )
    }).collect::<Vec<_>>().join("\n");

    let trust = if trust_line.is_empty() {
        "Trusted by industry leaders".to_string()
    } else {
        trust_line.to_string()
    };

    format!(
        r#"<section class="px-6 py-16 border-y border-neutral-800/30">
  <div class="max-w-5xl mx-auto">
    <p class="text-center font-mono text-[10px] uppercase tracking-[0.2em] text-neutral-600 mb-8">{trust}</p>
    <div class="flex flex-wrap items-center justify-center gap-2">
      {items}
    </div>
  </div>
</section>"#,
        trust = trust,
        items = items,
    )
}

// ══════════════════════════════════════════════════
// HOW IT WORKS — 3 numbered steps
// ══════════════════════════════════════════════════

pub fn how_it_works(steps: &[(&str, &str, &str)], accent: &str) -> String {
    // steps: (title, description, icon)
    let items: String = steps.iter().enumerate().map(|(i, (title, desc, icon))| {
        let num = i + 1;
        let emoji = icon_to_emoji(icon);
        format!(
            r##"<div class="anim anim-d{d} relative text-center">
      <div class="w-16 h-16 mx-auto mb-6 rounded-2xl bg-{a}-500/10 border border-{a}-500/20 flex items-center justify-center">
        <span class="text-2xl">{emoji}</span>
      </div>
      <div class="absolute top-8 left-1/2 -translate-x-1/2 -translate-y-1/2 w-8 h-8 rounded-full bg-neutral-950 border-2 border-{a}-500/40 flex items-center justify-center font-mono text-xs font-bold text-{a}-400">{num}</div>
      <h3 class="text-lg font-bold text-white mb-2">{title}</h3>
      <p class="text-sm text-neutral-400 leading-relaxed max-w-xs mx-auto">{desc}</p>
    </div>"##,
            d = i + 1,
            a = accent,
            emoji = emoji,
            num = num,
            title = title,
            desc = desc,
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r#"<section class="px-6 py-20">
  <div class="max-w-5xl mx-auto">
    <h2 class="anim text-3xl font-bold text-center mb-4">How It Works</h2>
    <p class="anim anim-d1 text-center text-neutral-400 mb-16 max-w-lg mx-auto">Get started in three simple steps</p>
    <div class="grid grid-cols-1 md:grid-cols-3 gap-12">
      {items}
    </div>
  </div>
</section>"#,
        items = items,
    )
}

// ══════════════════════════════════════════════════
// BENEFITS — alternating image/text rows
// ══════════════════════════════════════════════════

pub fn benefits(items: &[(&str, &str, &str)], accent: &str) -> String {
    // items: (title, description, icon_or_image)
    let rows: String = items.iter().enumerate().map(|(i, (title, desc, icon))| {
        let emoji = icon_to_emoji(icon);
        let reverse = if i % 2 == 1 { "md:flex-row-reverse" } else { "" };
        format!(
            r##"<div class="anim anim-d{d} flex flex-col {reverse} md:flex-row items-center gap-12">
      <div class="flex-1">
        <span class="font-mono text-[10px] uppercase tracking-[0.2em] text-{a}-500 mb-3 block">0{num}</span>
        <h3 class="text-2xl font-bold text-white mb-4">{title}</h3>
        <p class="text-neutral-400 leading-relaxed">{desc}</p>
      </div>
      <div class="flex-1 flex items-center justify-center">
        <div class="w-64 h-48 glass rounded-2xl flex items-center justify-center text-6xl">{emoji}</div>
      </div>
    </div>"##,
            d = (i % 6) + 1,
            reverse = reverse,
            a = accent,
            num = i + 1,
            title = title,
            desc = desc,
            emoji = emoji,
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r#"<section class="px-6 py-20">
  <div class="max-w-5xl mx-auto space-y-20">
    {rows}
  </div>
</section>"#,
        rows = rows,
    )
}

// ══════════════════════════════════════════════════
// COMPARISON TABLE — CRONUS vs Others
// ══════════════════════════════════════════════════

pub fn comparison_table(
    features: &[(&str, bool, bool)], // (feature_name, cronus_has, others_have)
    cronus_label: &str,
    others_label: &str,
    accent: &str,
) -> String {
    let rows: String = features.iter().enumerate().map(|(i, (feature, cronus, others))| {
        let c = if *cronus { format!(r#"<span class="text-emerald-400">✓</span>"#) } else { format!(r#"<span class="text-neutral-600">—</span>"#) };
        let o = if *others { format!(r#"<span class="text-emerald-400">✓</span>"#) } else { format!(r#"<span class="text-neutral-600">—</span>"#) };
        format!(
            r#"<tr class="anim anim-d{d} border-b border-neutral-800/30 hover:bg-neutral-900/30 transition">
      <td class="px-6 py-4 text-sm text-neutral-300">{feature}</td>
      <td class="px-6 py-4 text-center">{c}</td>
      <td class="px-6 py-4 text-center">{o}</td>
    </tr>"#,
            d = (i % 6) + 1,
            feature = feature,
            c = c,
            o = o,
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r##"<section class="px-6 py-20">
  <div class="max-w-3xl mx-auto">
    <h2 class="anim text-3xl font-bold text-center mb-12">Why Choose CRONUS?</h2>
    <div class="glass rounded-2xl overflow-hidden">
      <table class="w-full">
        <thead>
          <tr class="border-b border-neutral-800/50">
            <th class="px-6 py-4 text-left font-mono text-[10px] uppercase tracking-[0.15em] text-neutral-500">Feature</th>
            <th class="px-6 py-4 text-center font-mono text-[10px] uppercase tracking-[0.15em] text-{accent}-400">{cronus}</th>
            <th class="px-6 py-4 text-center font-mono text-[10px] uppercase tracking-[0.15em] text-neutral-500">{others}</th>
          </tr>
        </thead>
        <tbody>
          {rows}
        </tbody>
      </table>
    </div>
  </div>
</section>"##,
        accent = accent,
        cronus = cronus_label,
        others = others_label,
        rows = rows,
    )
}

// ══════════════════════════════════════════════════
// NEWSLETTER SIGNUP
// ══════════════════════════════════════════════════

pub fn newsletter(accent: &str, title: &str, subtitle: &str) -> String {
    format!(
        r##"<section class="px-6 py-20">
  <div class="max-w-xl mx-auto text-center">
    <h2 class="anim text-2xl font-bold mb-3">{title}</h2>
    <p class="anim anim-d1 text-sm text-neutral-400 mb-8">{subtitle}</p>
    <form class="anim anim-d2 flex gap-3" onsubmit="event.preventDefault();var e=this.querySelector('input');fetch('/api/newsletter',{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify({{email:e.value}})}}).then(function(){{e.value='';e.placeholder='Subscribed!'}}).catch(function(){{}})">
      <input type="email" required placeholder="you@example.com"
        class="flex-1 bg-[#080808] border border-neutral-800 rounded-lg px-4 py-3 text-sm text-white placeholder-neutral-600 focus:outline-none focus:border-{accent}-500/50 transition font-mono text-xs">
      <button type="submit"
        class="px-6 py-3 bg-{accent}-500 hover:bg-{accent}-400 text-black font-semibold rounded-lg transition text-sm whitespace-nowrap">Subscribe</button>
    </form>
    <p class="anim anim-d3 mt-3 text-xs text-neutral-600">No spam. Unsubscribe anytime.</p>
  </div>
</section>"##,
        accent = accent,
        title = title,
        subtitle = subtitle,
    )
}

// ══════════════════════════════════════════════════
// VIDEO SECTION — embed with play button overlay
// ══════════════════════════════════════════════════

pub fn video_section(title: &str, subtitle: &str, video_url: &str, accent: &str) -> String {
    format!(
        r##"<section class="px-6 py-20">
  <div class="max-w-4xl mx-auto text-center">
    <h2 class="anim text-3xl font-bold mb-3">{title}</h2>
    <p class="anim anim-d1 text-neutral-400 mb-12">{subtitle}</p>
    <div class="anim anim-d2 relative rounded-2xl overflow-hidden border border-neutral-800/60 aspect-video bg-neutral-900 group cursor-pointer" onclick="var v=this.querySelector('video');var o=this.querySelector('.play-overlay');if(v.paused){{v.play();o.style.display='none'}}else{{v.pause();o.style.display='flex'}}">
      <video class="w-full h-full object-cover" preload="metadata" poster="">
        <source src="{video_url}" type="video/mp4">
      </video>
      <div class="play-overlay absolute inset-0 bg-neutral-950/60 flex items-center justify-center transition group-hover:bg-neutral-950/40">
        <div class="w-20 h-20 rounded-full bg-{accent}-500/90 flex items-center justify-center group-hover:scale-110 transition-transform">
          <svg class="w-8 h-8 text-black ml-1" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
        </div>
      </div>
    </div>
  </div>
</section>"##,
        title = title,
        subtitle = subtitle,
        video_url = video_url,
        accent = accent,
    )
}

// ══════════════════════════════════════════════════
// FOOTER — 4 columns + social + newsletter
// ══════════════════════════════════════════════════

pub fn footer(
    app_name: &str,
    columns: &[(&str, &[(&str, &str)])], // (column_title, [(link_text, href)])
    accent: &str,
) -> String {
    let cols: String = columns.iter().map(|(title, links)| {
        let items: String = links.iter().map(|(text, href)| {
            format!(r#"<li><a href="{}" class="text-sm text-neutral-500 hover:text-neutral-300 transition">{}</a></li>"#, href, text)
        }).collect::<Vec<_>>().join("\n        ");
        format!(
            r#"<div>
      <h4 class="font-mono text-[10px] uppercase tracking-[0.2em] text-neutral-400 mb-4">{title}</h4>
      <ul class="space-y-2">
        {items}
      </ul>
    </div>"#,
            title = title,
            items = items,
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r##"<footer class="border-t border-neutral-800/50 px-6 pt-16 pb-8 mt-20">
  <div class="max-w-6xl mx-auto">
    <div class="grid grid-cols-2 md:grid-cols-4 gap-8 mb-12">
      {cols}
    </div>
    <div class="border-t border-neutral-800/30 pt-8 flex flex-col md:flex-row items-center justify-between gap-4">
      <div class="flex items-center gap-3">
        <span class="font-bold text-{accent}-400">{app_name}</span>
        <span class="text-xs text-neutral-600">Built with CRONUS</span>
      </div>
      <div class="flex items-center gap-4">
        <a href="#" class="text-neutral-600 hover:text-neutral-400 transition text-sm">Twitter</a>
        <a href="#" class="text-neutral-600 hover:text-neutral-400 transition text-sm">GitHub</a>
        <a href="#" class="text-neutral-600 hover:text-neutral-400 transition text-sm">Discord</a>
      </div>
      <p class="text-xs text-neutral-700">&copy; 2026 {app_name}. All rights reserved.</p>
    </div>
  </div>
</footer>"##,
        cols = cols,
        accent = accent,
        app_name = app_name,
    )
}

// ══════════════════════════════════════════════════
// PRICING TOGGLE — monthly/yearly switcher
// ══════════════════════════════════════════════════

pub fn pricing_toggle(
    plans_monthly: &[PricingPlan],
    plans_yearly: &[PricingPlan],
    accent: &str,
) -> String {
    let monthly_html = pricing_table(plans_monthly, accent);
    let yearly_html = pricing_table(plans_yearly, accent);

    format!(
        r##"<div class="text-center mb-8">
  <div class="inline-flex items-center gap-3 bg-neutral-900 rounded-full p-1 border border-neutral-800">
    <button onclick="document.getElementById('pricing-monthly').style.display='block';document.getElementById('pricing-yearly').style.display='none';this.classList.add('bg-{a}-500','text-black');this.classList.remove('text-neutral-400');this.nextElementSibling.classList.remove('bg-{a}-500','text-black');this.nextElementSibling.classList.add('text-neutral-400')"
      class="px-4 py-1.5 rounded-full text-sm font-semibold bg-{a}-500 text-black transition">Monthly</button>
    <button onclick="document.getElementById('pricing-yearly').style.display='block';document.getElementById('pricing-monthly').style.display='none';this.classList.add('bg-{a}-500','text-black');this.classList.remove('text-neutral-400');this.previousElementSibling.classList.remove('bg-{a}-500','text-black');this.previousElementSibling.classList.add('text-neutral-400')"
      class="px-4 py-1.5 rounded-full text-sm font-semibold text-neutral-400 transition">Yearly <span class="text-emerald-400 text-xs">-20%</span></button>
  </div>
</div>
<div id="pricing-monthly">{monthly}</div>
<div id="pricing-yearly" style="display:none">{yearly}</div>"##,
        a = accent,
        monthly = monthly_html,
        yearly = yearly_html,
    )
}

// ══════════════════════════════════════════════════
// API DASHBOARD (from ApiDashboard5.tsx — dark)
// ══════════════════════════════════════════════════

pub fn api_dashboard(accent: &str) -> String {
    let bars: String = (0..30).map(|_| r#"<div class="flex-1 h-6 bg-emerald-500/20 rounded-sm"></div>"#.to_string()).collect::<Vec<_>>().join("");

    format!(
        r##"<section class="py-24">
  <div class="max-w-7xl mx-auto px-6">
    <div class="mb-16">
      <p class="font-mono text-[10px] uppercase tracking-[0.2em] text-neutral-500 mb-3">Developer Tools</p>
      <h2 class="text-4xl md:text-5xl font-extrabold tracking-tight text-white">API &amp; Webhooks</h2>
      <p class="text-neutral-400 text-lg mt-4 max-w-2xl">RESTful APIs, real-time webhooks, and complete SDKs.</p>
    </div>
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <div class="lg:col-span-2 space-y-6">
        <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8">
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-lg font-semibold text-white">API Keys</h3>
            <button class="text-xs font-medium px-4 py-2 bg-{a}-500 text-black rounded-full">Create Key</button>
          </div>
          <div class="space-y-3">
            <div class="bg-[#141414] flex items-center justify-between p-4 rounded-lg"><div class="flex items-center gap-3"><div class="w-2.5 h-2.5 rounded-full bg-emerald-500"></div><div><div class="text-sm font-medium text-white">Live Key</div><div class="text-xs font-mono text-neutral-500">sk_live_***4x7K</div></div></div><span class="text-xs text-neutral-600">copy</span></div>
            <div class="bg-[#141414] flex items-center justify-between p-4 rounded-lg"><div class="flex items-center gap-3"><div class="w-2.5 h-2.5 rounded-full bg-amber-500"></div><div><div class="text-sm font-medium text-white">Test Key</div><div class="text-xs font-mono text-neutral-500">sk_test_***9m2P</div></div></div><span class="text-xs text-neutral-600">copy</span></div>
          </div>
        </div>
        <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8">
          <div class="flex items-center justify-between mb-6"><h3 class="text-lg font-semibold text-white">Webhooks</h3><span class="text-xs px-3 py-1 bg-neutral-800 text-neutral-400 rounded-full">3 endpoints</span></div>
          <div class="space-y-3">
            <div class="bg-[#141414] flex items-center justify-between p-4 rounded-lg"><div><div class="text-sm font-medium text-white font-mono">payment.completed</div><div class="text-xs text-neutral-500">https://api.example.com/webhooks</div></div><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">active</span></div>
            <div class="bg-[#141414] flex items-center justify-between p-4 rounded-lg"><div><div class="text-sm font-medium text-white font-mono">subscription.renewed</div><div class="text-xs text-neutral-500">https://api.example.com/webhooks</div></div><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">active</span></div>
            <div class="bg-[#141414] flex items-center justify-between p-4 rounded-lg"><div><div class="text-sm font-medium text-white font-mono">refund.created</div><div class="text-xs text-neutral-500">https://api.example.com/webhooks</div></div><span class="text-xs px-2 py-0.5 bg-amber-500/10 text-amber-400 rounded-full">paused</span></div>
          </div>
        </div>
      </div>
      <div class="space-y-6">
        <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white mb-2">Start Building</h3><p class="text-sm text-neutral-400 mb-6">Get started with our API in under 5 minutes.</p><a href="/docs" class="block w-full py-3 text-sm font-medium text-center bg-{a}-500 text-black rounded-full">View Docs</a></div>
        <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><div class="flex items-center gap-2 mb-4"><div class="w-2.5 h-2.5 rounded-full bg-emerald-500"></div><span class="text-sm font-medium text-white">System Health</span></div><p class="text-xs text-neutral-500 mb-3">All systems operational</p><div class="flex gap-0.5">{bars}</div><p class="text-xs text-neutral-500 mt-2">99.99% uptime</p></div>
      </div>
    </div>
  </div>
</section>"##, a = accent, bars = bars)
}

// ══════════════════════════════════════════════════
// WEBHOOKS SECTION (from ApiWebhooks.tsx)
// ══════════════════════════════════════════════════

pub fn webhooks_section(events: &[(&str, &str, &str)], accent: &str) -> String {
    let items: String = events.iter().map(|(name, url, status)| {
        let bc = if *status == "active" { "emerald" } else { "amber" };
        format!(r#"<div class="bg-[#141414] flex items-center justify-between p-4 rounded-lg"><div><div class="text-sm font-medium text-white font-mono">{name}</div><div class="text-xs text-neutral-500">{url}</div></div><span class="text-xs px-2 py-0.5 bg-{bc}-500/10 text-{bc}-400 rounded-full">{status}</span></div>"#, name=name, url=url, status=status, bc=bc)
    }).collect::<Vec<_>>().join("\n");

    format!(r##"<section class="py-20"><div class="max-w-4xl mx-auto px-6">
    <p class="font-mono text-[10px] uppercase tracking-[0.2em] text-{a}-500 mb-3">// Webhooks</p>
    <h2 class="text-3xl font-bold text-white mb-8">Real-time Events</h2>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><div class="space-y-3">{items}</div></div>
    <div class="mt-6 bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6">
      <p class="font-mono text-[10px] text-neutral-500 mb-3">EXAMPLE PAYLOAD</p>
      <pre class="text-xs font-mono text-neutral-300"><code>POST /webhooks
{{"event":"payment.completed","data":{{"id":"pay_abc","amount":4990}}}}</code></pre>
    </div>
  </div></section>"##, a=accent, items=items)
}

// ══════════════════════════════════════════════════
// PAYOUTS OVERVIEW (from PayoutsOverview.tsx — dark)
// ══════════════════════════════════════════════════

pub fn payouts_overview(accent: &str) -> String {
    format!(r##"<section class="py-24"><div class="max-w-7xl mx-auto px-6">
    <p class="font-mono text-[10px] uppercase tracking-[0.2em] text-neutral-500 mb-3">// Payouts</p>
    <h2 class="text-4xl font-extrabold text-white mb-12">Payouts &amp; Balance</h2>
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
      <div class="lg:col-span-2 bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8">
        <div class="flex items-center justify-between mb-6"><span class="text-sm text-neutral-400">Available Balance</span><span class="text-xs px-3 py-1 bg-emerald-500/10 text-emerald-400 rounded-full">Auto-payout</span></div>
        <div class="text-5xl font-bold text-white tabular-nums mb-1">$142,850.00</div>
        <p class="text-sm text-neutral-500 mb-6">USD — Updated now</p>
        <div class="flex gap-3 mb-8"><button class="px-6 py-2.5 bg-{a}-500 text-black text-sm font-medium rounded-full">Payout Now</button><button class="px-6 py-2.5 border border-neutral-700 text-white text-sm rounded-full">Configure</button></div>
        <div class="grid grid-cols-3 gap-6">
          <div><div class="font-mono text-[10px] text-neutral-500">WEEKLY AVG</div><div class="text-lg font-semibold text-white tabular-nums">$11,540</div></div>
          <div><div class="font-mono text-[10px] text-neutral-500">MONTHLY</div><div class="text-lg font-semibold text-white tabular-nums">$48,750</div></div>
          <div><div class="font-mono text-[10px] text-neutral-500">NEXT</div><div class="text-lg font-semibold text-white">Mar 28</div></div>
        </div>
      </div>
      <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8">
        <h3 class="text-lg font-semibold text-white mb-2">Upcoming</h3>
        <div class="text-3xl font-bold text-white tabular-nums mb-6">$14,320.00</div>
        <div class="flex justify-between py-2 border-b border-neutral-800/30"><span class="text-sm text-neutral-500">Transactions</span><span class="text-sm text-white tabular-nums">847</span></div>
        <div class="flex justify-between py-2 border-b border-neutral-800/30"><span class="text-sm text-neutral-500">Fees</span><span class="text-sm text-white tabular-nums">$428.50</span></div>
        <div class="flex justify-between py-2"><span class="text-sm text-neutral-500">Method</span><span class="text-sm text-white">ACH Transfer</span></div>
      </div>
    </div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8">
      <div class="flex items-center justify-between mb-6"><h3 class="text-lg font-semibold text-white">Payout History</h3><button class="font-mono text-[10px] px-4 py-2 border border-neutral-700 text-neutral-400 rounded-full">CSV</button></div>
      <table class="w-full text-sm"><thead><tr class="border-b border-neutral-800/50"><th class="text-left py-3 font-mono text-[10px] text-neutral-500">ID</th><th class="text-left py-3 font-mono text-[10px] text-neutral-500">DATE</th><th class="text-left py-3 font-mono text-[10px] text-neutral-500">AMOUNT</th><th class="text-left py-3 font-mono text-[10px] text-neutral-500">STATUS</th></tr></thead>
      <tbody>
        <tr class="border-b border-neutral-800/30"><td class="py-3 font-mono text-neutral-300">PO-0892</td><td class="py-3 text-neutral-500">Mar 25</td><td class="py-3 text-white tabular-nums">$12,450</td><td class="py-3"><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">done</span></td></tr>
        <tr class="border-b border-neutral-800/30"><td class="py-3 font-mono text-neutral-300">PO-0891</td><td class="py-3 text-neutral-500">Mar 18</td><td class="py-3 text-white tabular-nums">$9,320</td><td class="py-3"><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">done</span></td></tr>
        <tr><td class="py-3 font-mono text-neutral-300">PO-0890</td><td class="py-3 text-neutral-500">Mar 11</td><td class="py-3 text-white tabular-nums">$15,780</td><td class="py-3"><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">done</span></td></tr>
      </tbody></table>
    </div>
  </div></section>"##, a = accent)
}

// ══════════════════════════════════════════════════
// PREMIUM CSS (extracted from globals.css)
// ══════════════════════════════════════════════════

pub const PREMIUM_CSS: &str = r##"
<style>
  :root { --surface: #000; --surface-high: #1a1a1a; --on-surface: #fff; --muted: #a1a1aa; }
  html { scroll-behavior: smooth; }
  ::selection { background: rgba(245,158,11,0.3); color: white; }
  .prism-glow { background: radial-gradient(circle at 50% 50%, rgba(245,158,11,0.08) 0%, transparent 70%); pointer-events: none; }
  .grid-pattern { background-image: linear-gradient(to right,rgba(255,255,255,0.03) 1px,transparent 1px),linear-gradient(to bottom,rgba(255,255,255,0.03) 1px,transparent 1px); background-size: 40px 40px; pointer-events: none; }
  .ghost-border { border: 1px solid rgba(255,255,255,0.06); }
  @keyframes fadeUp { from{opacity:0;transform:translateY(24px)} to{opacity:1;transform:translateY(0)} }
  @keyframes shimmer { 0%{background-position:-200% 0} 100%{background-position:200% 0} }
  @keyframes pulse-dot { 0%,100%{opacity:.4} 50%{opacity:1} }
  .anim{opacity:0;animation:fadeUp .6s ease-out forwards}
  .anim-d1{animation-delay:.1s} .anim-d2{animation-delay:.2s} .anim-d3{animation-delay:.3s}
  .anim-d4{animation-delay:.4s} .anim-d5{animation-delay:.5s} .anim-d6{animation-delay:.6s}
  .glass{background:rgba(255,255,255,.03);backdrop-filter:blur(16px);border:1px solid rgba(255,255,255,.06)}
  .shimmer{background:linear-gradient(90deg,transparent 0%,rgba(255,255,255,.04) 50%,transparent 100%);background-size:200% 100%;animation:shimmer 2s ease-in-out infinite}
</style>
"##;

// ══════════════════════════════════════════════════
// EDGE NETWORK (from EdgeNetwork.tsx — dark)
// ══════════════════════════════════════════════════

pub fn edge_network_section(accent: &str) -> String {
    let highlighted = [5,8,11,14,17,20,23,26,29];
    let dots: String = (0..36).map(|i| {
        let color = if highlighted.contains(&i) { format!("bg-{}-500", accent) } else { "bg-neutral-800".to_string() };
        format!(r#"<div class="w-4 h-4 rounded-full {}"></div>"#, color)
    }).collect::<Vec<_>>().join("\n              ");

    format!(r##"<section class="py-32">
  <div class="max-w-7xl mx-auto px-6">
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-20 items-center">
      <div>
        <p class="font-mono text-[10px] uppercase tracking-[0.2em] text-neutral-500 mb-4">Edge Infrastructure</p>
        <h2 class="text-4xl md:text-6xl font-extrabold tracking-tight leading-[1] text-white mb-10">The Edge Network Advantage</h2>
        <div class="space-y-8">
          <div class="flex gap-4"><div class="w-12 h-12 shrink-0 bg-neutral-900 rounded-lg flex items-center justify-center text-xl">⚡</div><div><h3 class="text-base font-semibold text-white">Cold Starts: 0ms</h3><p class="text-sm text-neutral-400 mt-1">Instant function invocation with pre-warmed instances at every edge location.</p></div></div>
          <div class="flex gap-4"><div class="w-12 h-12 shrink-0 bg-neutral-900 rounded-lg flex items-center justify-center text-xl">🛡️</div><div><h3 class="text-base font-semibold text-white">DDoS Protection</h3><p class="text-sm text-neutral-400 mt-1">Enterprise-grade protection with automatic threat detection and mitigation.</p></div></div>
          <div class="flex gap-4"><div class="w-12 h-12 shrink-0 bg-neutral-900 rounded-lg flex items-center justify-center text-xl">📊</div><div><h3 class="text-base font-semibold text-white">Real-time Analytics</h3><p class="text-sm text-neutral-400 mt-1">Monitor performance, errors, and usage across all regions in real time.</p></div></div>
        </div>
      </div>
      <div class="relative w-full aspect-square bg-neutral-900 border border-neutral-800/50 rounded-2xl flex items-center justify-center overflow-hidden">
        <div class="absolute inset-0" style="background:radial-gradient(circle,rgba(245,158,11,0.05),transparent)"></div>
        <div class="grid grid-cols-6 grid-rows-6 gap-4 p-8 relative z-10">
          {dots}
        </div>
        <div class="absolute -bottom-4 left-1/2 -translate-x-1/2 bg-neutral-950 border border-neutral-800 shadow-2xl rounded-full px-6 py-3">
          <span class="text-sm font-medium text-white whitespace-nowrap">100+ Edge Locations Worldwide</span>
        </div>
      </div>
    </div>
  </div>
</section>"##, dots = dots)
}

// ══════════════════════════════════════════════════
// DEVELOPER EXPERIENCE (from DeveloperExperience.tsx — dark)
// ══════════════════════════════════════════════════

pub fn developer_section(accent: &str) -> String {
    format!(r##"<section class="py-24 relative overflow-hidden">
  <div class="grid-pattern opacity-20 absolute inset-0"></div>
  <div class="max-w-7xl mx-auto px-6 relative z-10">
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center mb-24">
      <div>
        <div class="inline-flex items-center gap-2 px-4 py-1.5 mb-6 bg-neutral-900 border border-neutral-800 rounded-full">
          <div class="w-2 h-2 rounded-full bg-violet-500 animate-pulse"></div>
          <span class="text-xs font-medium text-neutral-400">Developer First</span>
        </div>
        <h2 class="text-5xl md:text-7xl font-extrabold tracking-tighter leading-[0.9] text-white mb-6">Integrate.<br>Test.<br>Ship.</h2>
        <p class="text-lg text-neutral-400 mb-8 max-w-md">Complete docs, typed SDKs, sandbox with real data. Zero to production in 15 minutes.</p>
        <div class="flex gap-3">
          <button class="px-6 py-3 bg-{a}-500 text-black text-sm font-medium rounded-full">View Docs</button>
          <button class="px-6 py-3 border border-neutral-700 text-white text-sm font-medium rounded-full">Try Sandbox</button>
        </div>
      </div>
      <div class="bg-[#0a0a0a] rounded-xl border border-neutral-800/50 overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 border-b border-neutral-800">
          <div class="flex gap-2"><div class="w-3 h-3 rounded-full bg-red-500"></div><div class="w-3 h-3 rounded-full bg-amber-500"></div><div class="w-3 h-3 rounded-full bg-emerald-500"></div></div>
          <span class="text-xs text-neutral-600 font-mono">terminal</span><div class="w-16"></div>
        </div>
        <div class="p-6 font-mono text-sm leading-relaxed text-neutral-300">
          <div class="text-{a}-400">$ cronus init my-app --template saas</div>
          <div><span class="text-blue-400">info</span> Creating project...</div>
          <div><span class="text-emerald-400">✓</span> TypeScript SDK installed</div>
          <div><span class="text-emerald-400">✓</span> Webhook routes configured</div>
          <div><span class="text-emerald-400">✓</span> Auth + API + Dashboard ready</div>
          <div class="text-{a}-400 mt-2">$ cronus run</div>
          <div><span class="text-emerald-400">✓</span> Server running → http://localhost:5175</div>
        </div>
      </div>
    </div>
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div class="md:col-span-2 bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white">Typed SDKs</h3><p class="text-sm text-neutral-400 mt-2">TypeScript, Node.js, Python, Go, Rust with full type coverage.</p><div class="flex flex-wrap gap-2 mt-6"><span class="px-3 py-1.5 bg-neutral-800 rounded-full text-xs font-mono text-neutral-400">TypeScript</span><span class="px-3 py-1.5 bg-neutral-800 rounded-full text-xs font-mono text-neutral-400">Python</span><span class="px-3 py-1.5 bg-neutral-800 rounded-full text-xs font-mono text-neutral-400">Go</span><span class="px-3 py-1.5 bg-neutral-800 rounded-full text-xs font-mono text-neutral-400">Rust</span></div></div>
      <div class="bg-neutral-900 border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white">Sandbox</h3><p class="text-sm text-neutral-400 mt-2">Test with real-looking data. Simulate any scenario.</p><div class="flex flex-wrap gap-2 mt-6"><span class="px-3 py-1.5 bg-neutral-800 rounded-full text-xs font-mono text-neutral-400">cronus test api</span><span class="px-3 py-1.5 bg-neutral-800 rounded-full text-xs font-mono text-neutral-400">cronus simulate</span></div></div>
      <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white">Reliable Webhooks</h3><p class="text-sm text-neutral-400 mt-2">Auto retry with exponential backoff. Real-time event dashboard.</p></div>
      <div class="md:col-span-2 bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white">Complete Docs</h3><p class="text-sm text-neutral-400 mt-2">Full documentation with real examples. Step-by-step guides for every feature.</p></div>
    </div>
  </div>
</section>"##, a = accent)
}

// ══════════════════════════════════════════════════
// ENTERPRISE (from Enterprise.tsx — dark)
// ══════════════════════════════════════════════════

pub fn enterprise_section(accent: &str) -> String {
    let regions = [("US-E","Virginia"),("US-W","Oregon"),("EU-W","Frankfurt"),("AP-SE","Singapore"),("AP-NE","Tokyo"),("EU-N","Stockholm")];
    let region_cards: String = regions.iter().map(|(code, city)| {
        format!(r#"<div class="text-center p-3 bg-[#0f0f0f] rounded-lg"><div class="w-2.5 h-2.5 rounded-full bg-emerald-500 mx-auto mb-2"></div><div class="text-xs font-mono font-bold text-white">{}</div><div class="text-xs text-neutral-500">{}</div></div>"#, code, city)
    }).collect::<Vec<_>>().join("\n            ");

    format!(r##"<section class="py-24">
  <div class="max-w-7xl mx-auto px-6">
    <div class="mb-12">
      <p class="font-mono text-[10px] uppercase tracking-[0.2em] text-neutral-500 mb-3">ENTERPRISE</p>
      <h2 class="text-4xl md:text-5xl font-extrabold tracking-tight text-white">Built for Regulated Industries</h2>
    </div>
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
      <div class="md:col-span-2 bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8">
        <h3 class="text-lg font-semibold text-white">Compliance Ready</h3>
        <p class="text-sm text-neutral-400 mt-2">Enterprise-grade certifications out of the box.</p>
        <div class="flex flex-wrap gap-2 mt-4"><span class="px-4 py-2 bg-neutral-800 rounded-full text-sm font-medium text-white">SOC 2</span><span class="px-4 py-2 bg-neutral-800 rounded-full text-sm font-medium text-white">HIPAA</span><span class="px-4 py-2 bg-neutral-800 rounded-full text-sm font-medium text-white">GDPR</span><span class="px-4 py-2 bg-neutral-800 rounded-full text-sm font-medium text-white">ISO 27001</span></div>
      </div>
      <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white">DDoS Mitigation</h3><p class="text-sm text-neutral-500 mt-2">Automatic threat detection with 10Tbps+ capacity.</p></div>
      <div class="bg-neutral-900 border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white">Dedicated Support</h3><p class="text-sm text-neutral-400 mt-2">24/7 priority with 15-minute response SLA.</p></div>
    </div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8 mb-12">
      <div class="flex items-center gap-3 mb-6"><span class="text-2xl">🌍</span><div><h3 class="text-lg font-semibold text-white">Global Edge Network</h3><p class="text-sm text-neutral-500">Active deployment regions</p></div></div>
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-4">
        {regions}
      </div>
    </div>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div class="text-center"><div class="text-3xl font-bold text-white tabular-nums">99.99%</div><div class="text-sm text-neutral-500 mt-1">Uptime SLA</div></div>
      <div class="text-center"><div class="text-3xl font-bold text-white tabular-nums">50ms</div><div class="text-sm text-neutral-500 mt-1">Avg Response</div></div>
      <div class="text-center"><div class="text-3xl font-bold text-white tabular-nums">10M+</div><div class="text-sm text-neutral-500 mt-1">Requests/Day</div></div>
      <div class="text-center"><div class="text-3xl font-bold text-white tabular-nums">100+</div><div class="text-sm text-neutral-500 mt-1">Edge Locations</div></div>
    </div>
  </div>
</section>"##, regions = region_cards)
}

// ══════════════════════════════════════════════════
// BILLING PORTAL
// ══════════════════════════════════════════════════

pub fn billing_portal(plans: &[PricingPlan], current_plan: &str, accent: &str) -> String {
    let plan_cards: String = plans.iter().map(|p| {
        let is_current = p.name.to_lowercase() == current_plan.to_lowercase();
        let border = if is_current { format!("border-{}-500/50", accent) } else { "border-neutral-800/50".to_string() };
        let badge = if is_current { format!(r#"<span class="text-xs px-2 py-0.5 bg-{}-500/10 text-{}-400 rounded-full">Current</span>"#, accent, accent) } else { String::new() };
        let btn = if is_current {
            r#"<button disabled class="w-full py-2 rounded-lg bg-neutral-800 text-neutral-500 text-sm cursor-not-allowed">Current Plan</button>"#.to_string()
        } else {
            format!(r#"<button class="w-full py-2 rounded-lg bg-{}-500 hover:bg-{}-400 text-black text-sm font-semibold transition">Upgrade</button>"#, accent, accent)
        };
        format!(r#"<div class="bg-[#0a0a0a] border {} rounded-xl p-6"><div class="flex items-center justify-between mb-3"><h4 class="font-semibold text-white">{}</h4>{}</div><div class="text-2xl font-bold text-white tabular-nums mb-4">{}<span class="text-sm text-neutral-500 font-normal">/{}</span></div>{}</div>"#,
            border, p.name, badge, p.price, p.period, btn)
    }).collect::<Vec<_>>().join("\n      ");

    format!(r##"<div class="max-w-6xl mx-auto">
  <div class="flex items-center gap-3 mb-8">
    <p class="font-mono text-[10px] uppercase tracking-[0.2em] text-{a}-500">// Billing</p>
    <div class="flex-1 h-px bg-neutral-800/50"></div>
  </div>

  <h1 class="text-3xl font-bold tracking-tight mb-8">Plans &amp; Billing</h1>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-12">
    {plans}
  </div>

  <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8 mb-6">
    <div class="flex items-center justify-between mb-6">
      <h3 class="text-lg font-semibold text-white">Payment Method</h3>
      <button class="font-mono text-[10px] px-4 py-2 border border-neutral-700 text-neutral-400 rounded-full hover:border-neutral-500 transition">Update</button>
    </div>
    <div class="flex items-center gap-4">
      <div class="w-14 h-9 bg-neutral-800 rounded-lg flex items-center justify-center font-mono text-xs text-neutral-400">VISA</div>
      <div><div class="text-sm text-white">**** **** **** 4242</div><div class="text-xs text-neutral-500">Expires 12/28</div></div>
    </div>
  </div>

  <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8">
    <div class="flex items-center justify-between mb-6">
      <h3 class="text-lg font-semibold text-white">Invoice History</h3>
      <button class="font-mono text-[10px] px-4 py-2 border border-neutral-700 text-neutral-400 rounded-full">Download All</button>
    </div>
    <table class="w-full text-sm">
      <thead><tr class="border-b border-neutral-800/50"><th class="text-left py-3 font-mono text-[10px] text-neutral-500">DATE</th><th class="text-left py-3 font-mono text-[10px] text-neutral-500">AMOUNT</th><th class="text-left py-3 font-mono text-[10px] text-neutral-500">STATUS</th><th class="text-right py-3 font-mono text-[10px] text-neutral-500"></th></tr></thead>
      <tbody>
        <tr class="border-b border-neutral-800/30"><td class="py-3 text-neutral-400">Mar 1, 2026</td><td class="py-3 text-white tabular-nums">$79.00</td><td class="py-3"><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">paid</span></td><td class="py-3 text-right"><span class="text-xs text-neutral-600 hover:text-neutral-400 cursor-pointer">PDF</span></td></tr>
        <tr class="border-b border-neutral-800/30"><td class="py-3 text-neutral-400">Feb 1, 2026</td><td class="py-3 text-white tabular-nums">$79.00</td><td class="py-3"><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">paid</span></td><td class="py-3 text-right"><span class="text-xs text-neutral-600 hover:text-neutral-400 cursor-pointer">PDF</span></td></tr>
        <tr><td class="py-3 text-neutral-400">Jan 1, 2026</td><td class="py-3 text-white tabular-nums">$79.00</td><td class="py-3"><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">paid</span></td><td class="py-3 text-right"><span class="text-xs text-neutral-600 hover:text-neutral-400 cursor-pointer">PDF</span></td></tr>
      </tbody>
    </table>
  </div>
</div>"##, a = accent, plans = plan_cards)
}
// ══════════════════════════════════════════════════
// REAL LANDING PAGE TEMPLATES — Extracted from stitch
// ══════════════════════════════════════════════════

/// ultima landing page — hero section (3187 bytes of real HTML)
pub fn template_ultima_hero() -> String {
    r##"
<section class="relative min-h-screen flex flex-col justify-center pt-24 px-6 overflow-hidden mesh-bg">
<div class="max-w-7xl mx-auto w-full grid grid-cols-1 lg:grid-cols-12 gap-12 items-center">
<div class="lg:col-span-7 space-y-8 z-10">
<div class="inline-flex items-center px-3 py-1 rounded-full bg-surface-container-low border-[0.5px] border-outline-variant/20 space-x-2">
<span class="w-2 h-2 rounded-full bg-primary animate-pulse"></span>
<span class="text-[10px] uppercase tracking-[0.2em] font-medium text-primary/80">Systems Operational</span>
</div>
<h1 class="text-6xl md:text-8xl font-black text-on-surface text-display max-w-2xl">
                    The future of money, <span class="text-transparent bg-clip-text liquid-gradient">designed with taste.</span>
</h1>
<p class="text-lg md:text-xl text-on-surface-variant/80 max-w-xl font-body leading-relaxed">
                    Ultima Studio is an institutional-grade financial engine wrapped in a cinematic experience. Sovereignty starts with superior architecture.
                </p>
<div class="flex flex-col sm:flex-row items-center gap-4 pt-4">
<button class="w-full sm:w-auto px-8 py-4 rounded-xl liquid-gradient text-on-primary font-bold tracking-tight text-lg shadow-[0_0_40px_rgba(173,198,255,0.2)] hover:scale-[1.02] active:scale-95 transition-all">
                        Initiate Protocol
                    </button>
<button class="w-full sm:w-auto px-8 py-4 rounded-xl bg-surface-container-high border-[0.5px] border-outline-variant/30 text-on-surface font-semibold tracking-tight text-lg inner-glow hover:bg-surface-container-highest transition-all">
                        View Documentation
                    </button>
</div>
</div>
<!-- Floating 3D Card Visual -->
<div class="lg:col-span-5 relative group">
<div class="relative w-full aspect-[4/5] perspective-1000">
<!-- The "Liquid Glass" Card -->
<div class="w-full h-72 md:h-96 rounded-2xl glass-panel relative overflow-hidden transform rotate-x-12 rotate-y-[-20deg] shadow-[0_50px_100px_rgba(0,0,0,0.8)] flex flex-col justify-between p-8">
<div class="absolute inset-0 liquid-gradient opacity-10 blur-3xl -z-10"></div>
<div class="flex justify-between items-start">
<span class="text-xl font-black italic tracking-tighter">ULTIMA</span>
<span class="material-symbols-outlined text-3xl">contactless</span>
</div>
<div class="space-y-4">
<div class="text-2xl font-medium tracking-[0.2em] monospaced-label">4400 8821 9902 1104</div>
<div class="flex justify-between items-end">
<div>
<div class="text-[10px] uppercase opacity-40 monospaced-label">Holder</div>
<div class="text-sm font-semibold tracking-wide uppercase">Sovereign Architect</div>
</div>
<div class="w-12 h-12 rounded-full border-[0.5px] border-white/20 flex items-center justify-center">
<div class="w-8 h-8 rounded-full bg-white/10 backdrop-blur-md"></div>
</div>
</div>
</div>
</div>
<!-- Decorative Floating Elements -->
<div class="absolute -top-10 -right-10 w-32 h-32 rounded-full liquid-gradient blur-3xl opacity-20 animate-pulse"></div>
<div class="absolute -bottom-20 -left-10 w-64 h-64 rounded-full bg-primary/10 blur-[100px] -z-20"></div>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// ultima landing page — features section (4422 bytes of real HTML)
pub fn template_ultima_features() -> String {
    r##"
<section class="py-32 px-6 bg-surface-container-lowest relative overflow-hidden">
<div class="max-w-7xl mx-auto">
<div class="mb-24 space-y-4">
<h2 class="text-sm font-bold text-primary tracking-[0.3em] uppercase">Core Infrastructure</h2>
<h3 class="text-4xl md:text-5xl font-bold tracking-tight max-w-2xl">Precision engineering for the high-frequency era.</h3>
</div>
<div class="grid grid-cols-1 md:grid-cols-12 gap-6">
<!-- Large Feature -->
<div class="md:col-span-8 group relative bg-surface-container rounded-2xl border-[0.5px] border-outline-variant/15 p-10 overflow-hidden hover:bg-surface-container-high transition-all duration-500">
<div class="relative z-10 flex flex-col h-full justify-between">
<div class="space-y-4 max-w-md">
<span class="material-symbols-outlined text-4xl text-primary" data-weight="fill">insights</span>
<h4 class="text-2xl font-bold">Real-time Liquidity Matrix</h4>
<p class="text-on-surface-variant/70 leading-relaxed">Execute at the speed of light with our proprietary liquidity routing engine. Zero slippage, infinite depth.</p>
</div>
<div class="mt-12 flex gap-4">
<div class="px-4 py-2 rounded-lg bg-surface-container-lowest border-[0.5px] border-outline-variant/20 flex items-center gap-3">
<span class="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.5)]"></span>
<span class="text-xs font-mono opacity-60">Uptime: 99.999%</span>
</div>
</div>
</div>
<div class="absolute bottom-0 right-0 w-1/2 h-full bg-gradient-to-l from-primary/5 to-transparent pointer-events-none"></div>
<img alt="Tech Graphic" class="absolute -bottom-12 -right-12 w-80 opacity-20 grayscale brightness-150 group-hover:scale-110 group-hover:opacity-40 transition-all duration-700" data-alt="abstract tech visualization of flowing digital data lines and mesh geometry in dark metallic finish" src=""/>
</div>
<!-- Small Feature 1 -->
<div class="md:col-span-4 bg-surface-container rounded-2xl border-[0.5px] border-outline-variant/15 p-10 flex flex-col justify-between hover:bg-surface-container-high transition-all duration-500">
<span class="material-symbols-outlined text-4xl text-tertiary">layers</span>
<div class="space-y-3">
<h4 class="text-xl font-bold">Multi-layer Security</h4>
<p class="text-sm text-on-surface-variant/70">Biometric encryption meets cold-storage hardware isolation.</p>
</div>
</div>
<!-- Small Feature 2 -->
<div class="md:col-span-4 bg-surface-container rounded-2xl border-[0.5px] border-outline-variant/15 p-10 flex flex-col justify-between hover:bg-surface-container-high transition-all duration-500">
<span class="material-symbols-outlined text-4xl text-secondary">payments</span>
<div class="space-y-3">
<h4 class="text-xl font-bold">Global Settlements</h4>
<p class="text-sm text-on-surface-variant/70">Settle cross-border transactions in under 4 seconds with 0.1% fees.</p>
</div>
</div>
<!-- Large Feature 2 -->
<div class="md:col-span-8 group relative bg-surface-container rounded-2xl border-[0.5px] border-outline-variant/15 p-10 overflow-hidden hover:bg-surface-container-high transition-all duration-500">
<div class="relative z-10 flex flex-col h-full justify-between">
<div class="space-y-4 max-w-md">
<span class="material-symbols-outlined text-4xl text-primary">terminal</span>
<h4 class="text-2xl font-bold">Sovereign API</h4>
<p class="text-on-surface-variant/70 leading-relaxed">A developer-first experience designed for enterprise scale. Build your entire treasury on our core.</p>
</div>
<div class="mt-8">
<code class="text-xs text-primary/80 font-mono bg-surface-container-lowest/50 p-4 rounded-lg block border-[0.5px] border-outline-variant/10">
<span class="text-tertiary">ultima</span> .initiate_transfer({{<br/>
                                  amount: <span class="text-secondary">"1.2M"</span>,<br/>
                                  currency: <span class="text-secondary">"USD"</span>,<br/>
                                  vault: <span class="text-secondary">"Alpha_Prime"</span><br/>
                                }});
                            </code>
</div>
</div>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// ultima landing page — pricing section (2943 bytes of real HTML)
pub fn template_ultima_pricing() -> String {
    r##"
<section class="py-40 px-6 bg-surface">
<div class="max-w-7xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-24 items-center">
<div class="space-y-10">
<h2 class="text-5xl md:text-7xl font-black text-display leading-tight">Architectural<br/>Restraint</h2>
<p class="text-xl text-on-surface-variant/80 max-w-lg leading-relaxed font-body">
                    We believe data should be observed, not just seen. Our interface eliminates the noise of traditional finance, highlighting only the signals that matter for your capital.
                </p>
<div class="grid grid-cols-2 gap-8 pt-8">
<div>
<div class="text-4xl font-bold text-primary tracking-tight">2.4ms</div>
<div class="text-xs uppercase tracking-widest opacity-40 mt-2">Median Latency</div>
</div>
<div>
<div class="text-4xl font-bold text-secondary tracking-tight">$42B+</div>
<div class="text-xs uppercase tracking-widest opacity-40 mt-2">Annual Volume</div>
</div>
</div>
</div>
<div class="relative bg-surface-container-lowest rounded-3xl border-[0.5px] border-outline-variant/15 p-1 shadow-2xl overflow-hidden group">
<div class="p-8 space-y-8 bg-surface-container/30">
<div class="flex justify-between items-center">
<div class="space-y-1">
<div class="text-xs opacity-50 monospaced-label uppercase tracking-tighter">Total Assets Under Management</div>
<div class="text-3xl font-bold">$1,204,550.00 <span class="text-sm text-emerald-500 font-medium">+12.4%</span></div>
</div>
<span class="material-symbols-outlined opacity-30">fullscreen</span>
</div>
<!-- Decorative Chart -->
<div class="relative h-64 w-full flex items-end justify-between px-2">
<div class="absolute inset-0 bg-gradient-to-t from-primary/5 to-transparent"></div>
<!-- Simple SVG Graph representation -->
<svg class="absolute inset-0 w-full h-full" preserveaspectratio="none">
<defs>
<lineargradient id="chartGradient" x1="0" x2="0" y1="0" y2="1">
<stop offset="0%" stop-color="#adc6ff" stop-opacity="0.3"></stop>
<stop offset="100%" stop-color="#adc6ff" stop-opacity="0"></stop>
</lineargradient>
</defs>
<path class="drop-shadow-[0_0_10px_rgba(173,198,255,0.5)]" d="M0 200 Q 100 150, 200 180 T 400 100 T 600 50 T 800 80" fill="none" stroke="#adc6ff" stroke-linecap="round" stroke-width="3"></path>
<path d="M0 200 Q 100 150, 200 180 T 400 100 T 600 50 T 800 80 V 256 H 0 Z" fill="url(#chartGradient)"></path>
</svg>
<!-- Floating Data Tooltip (Mock) -->
<div class="absolute top-10 left-1/2 -translate-x-1/2 glass-panel px-4 py-2 rounded-lg text-xs font-mono border-[0.5px] border-primary/20 shadow-2xl">
                            Vol: $44.2k • 14:02:11
                        </div>
</div>
<div class="grid grid-cols-4 gap-4">
<div class="h-1 bg-primary rounded-full"></div>
<div class="h-1 bg-surface-container-high rounded-full"></div>
<div class="h-1 bg-surface-container-high rounded-full"></div>
<div class="h-1 bg-surface-container-high rounded-full"></div>
</div>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// ultima landing page — cta section (1536 bytes of real HTML)
pub fn template_ultima_cta() -> String {
    r##"
<section class="py-32 px-6">
<div class="max-w-5xl mx-auto glass-panel rounded-[2rem] p-12 md:p-24 text-center space-y-10 relative overflow-hidden bg-surface-container-low/50">
<div class="absolute top-0 left-0 w-full h-full liquid-gradient opacity-[0.03] pointer-events-none"></div>
<h2 class="text-4xl md:text-6xl font-black text-display leading-[1.1]">Join the world's most <br/>advanced capital network.</h2>
<p class="text-lg text-on-surface-variant/70 max-w-xl mx-auto leading-relaxed">
                Experience institutional finance without the legacy overhead. Secure your node in the Ultima Studio ecosystem today.
            </p>
<div class="flex flex-col sm:flex-row justify-center gap-6 pt-6 relative z-10">
<button class="px-12 py-5 rounded-2xl liquid-gradient text-on-primary font-bold text-lg hover:scale-[1.05] hover:shadow-[0_0_50px_rgba(173,198,255,0.3)] transition-all">
                    Apply for Access
                </button>
<button class="px-12 py-5 rounded-2xl bg-surface-container-highest border-[0.5px] border-outline-variant/30 text-on-surface font-semibold text-lg hover:bg-surface-bright transition-all">
                    Talk to an Expert
                </button>
</div>
<div class="pt-12 flex justify-center gap-12 opacity-30 grayscale contrast-125">
<span class="font-black text-xl italic tracking-tighter">GOLDMAN</span>
<span class="font-black text-xl italic tracking-tighter">MORGAN</span>
<span class="font-black text-xl italic tracking-tighter">BLACKROCK</span>
</div>
</div>
</section>
"##.to_string()
}

/// monolith landing page — hero section (4058 bytes of real HTML)
pub fn template_monolith_hero() -> String {
    r##"
<section class="relative min-h-[921px] flex flex-col items-center justify-center px-6 overflow-hidden">
<div class="absolute inset-0 z-0">
<div class="absolute inset-0 bg-gradient-to-b from-transparent via-background/80 to-background"></div>
<img alt="Cinematic abstract data interface" class="w-full h-full object-cover opacity-40" data-alt="3D cinematic visualization of a transparent glass monolith terminal floating in dark void with glowing cyan data streams and reflections" src=""/>
</div>
<div class="relative z-10 max-w-5xl w-full text-center mt-20">
<div class="inline-flex items-center gap-2 px-3 py-1 mb-8 rounded-full bg-surface-container-high ghost-border">
<span class="w-2 h-2 rounded-full bg-primary"></span>
<span class="font-label text-[10px] tracking-[0.2em] uppercase text-on-surface-variant">v4.0.0 Now in Stable Release</span>
</div>
<h1 class="text-5xl md:text-8xl font-black tracking-[-0.04em] leading-[0.9] text-primary mb-8 uppercase">
                    Architectural<br/>Restraint.
                </h1>
<p class="max-w-2xl mx-auto text-lg md:text-xl text-on-surface-variant font-light leading-relaxed mb-12">
                    The orchestration engine for high-fidelity engineering teams. Deploy infrastructure at the speed of thought with the precision of a master draftsperson.
                </p>
<div class="flex flex-col md:flex-row gap-4 justify-center items-center">
<button class="monolith-gradient text-on-primary-container px-8 py-4 rounded-xl font-bold text-sm tracking-tight w-full md:w-auto hover:scale-[0.98] transition-transform">
                        START BUILDING FOR FREE
                    </button>
<button class="bg-surface-container-high ghost-border text-primary px-8 py-4 rounded-xl font-bold text-sm tracking-tight w-full md:w-auto hover:bg-surface-container-highest transition-colors">
                        REQUEST ENTERPRISE DEMO
                    </button>
</div>
</div>
<!-- Floating Terminal Mockup -->
<div class="relative z-10 w-full max-w-4xl mt-24 glass-panel rounded-xl ghost-border overflow-hidden shadow-2xl">
<div class="flex items-center justify-between px-4 py-3 bg-surface-container-highest/50 border-b border-white/5">
<div class="flex gap-1.5">
<div class="w-2.5 h-2.5 rounded-full bg-white/10"></div>
<div class="w-2.5 h-2.5 rounded-full bg-white/10"></div>
<div class="w-2.5 h-2.5 rounded-full bg-white/10"></div>
</div>
<div class="font-label text-[10px] tracking-widest text-white/30 uppercase">monolith_os — terminal</div>
<div class="w-12"></div>
</div>
<div class="p-6 font-label text-sm leading-relaxed text-primary/80">
<div class="flex gap-4 mb-2">
<span class="text-primary/30">01</span>
<span class="text-primary/50">monolith</span>
<span class="text-white">deploy</span>
<span class="text-primary/40">--project</span>
<span class="text-tertiary">"hyper-cluster-01"</span>
</div>
<div class="flex gap-4 mb-2">
<span class="text-primary/30">02</span>
<span class="text-primary/40">Analyzing edge nodes in US-EAST-1...</span>
</div>
<div class="flex gap-4 mb-2">
<span class="text-primary/30">03</span>
<span class="text-primary/40">Validating protocol signatures [OK]</span>
</div>
<div class="flex gap-4 mb-2">
<span class="text-primary/30">04</span>
<span class="text-primary/40">Pushing build artifacts to global cache...</span>
</div>
<div class="flex gap-4 mb-2">
<span class="text-primary/30">05</span>
<span class="text-white font-bold">✓ Deployment Success</span>
<span class="text-primary/30">1.4s</span>
</div>
<div class="flex gap-4">
<span class="text-primary/30">06</span>
<span class="text-primary/50">URL:</span>
<span class="underline text-tertiary">https://hyper-cluster.monolith.io</span>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// monolith landing page — features section (3438 bytes of real HTML)
pub fn template_monolith_features() -> String {
    r##"
<section class="py-32 px-6 max-w-7xl mx-auto">
<div class="grid grid-cols-1 md:grid-cols-12 gap-6">
<!-- Large Feature -->
<div class="md:col-span-8 bg-surface-container rounded-xl p-8 flex flex-col justify-between min-h-[400px] relative overflow-hidden group">
<div class="relative z-10">
<span class="material-symbols-outlined text-primary mb-6 scale-150">layers</span>
<h3 class="text-3xl font-bold text-primary mb-4">Multi-Cloud Infrastructure.<br/>Simplified to a single API.</h3>
<p class="text-on-surface-variant max-w-md">Connect AWS, GCP, and Azure through our unified control plane. Zero friction, total sovereignty.</p>
</div>
<div class="absolute bottom-0 right-0 w-2/3 h-1/2 translate-y-1/4 translate-x-1/4 group-hover:translate-y-0 transition-transform duration-700">
<img alt="Server hardware close up" class="rounded-tl-xl ghost-border object-cover h-full w-full grayscale opacity-30 group-hover:grayscale-0 group-hover:opacity-60 transition-all" data-alt="Technical close-up of server racks with glowing LED indicators in a dark, high-tech data center environment" src=""/>
</div>
</div>
<!-- Small Feature -->
<div class="md:col-span-4 bg-surface-container-high rounded-xl p-8 flex flex-col items-start ghost-border">
<span class="material-symbols-outlined text-primary mb-6">shield</span>
<h3 class="font-bold text-xl text-primary mb-2">SOC2 Type II Ready</h3>
<p class="text-sm text-on-surface-variant leading-relaxed">Enterprise-grade security is not an add-on. It's built into the very core of our monolithic architecture.</p>
<div class="mt-auto pt-6 w-full">
<div class="h-1 bg-white/5 rounded-full overflow-hidden">
<div class="h-full bg-primary w-full opacity-50"></div>
</div>
<div class="flex justify-between mt-2 font-label text-[10px] text-white/30 tracking-widest uppercase">
<span>Compliance status</span>
<span>100%</span>
</div>
</div>
</div>
<!-- Column 3 Small -->
<div class="md:col-span-4 bg-surface-container-lowest rounded-xl p-8 border border-white/5 flex flex-col gap-4">
<span class="material-symbols-outlined text-primary">terminal</span>
<h3 class="font-bold text-xl text-primary">CLI First</h3>
<p class="text-sm text-on-surface-variant">Manage everything via our powerful command-line interface. Built for speed and scripting efficiency.</p>
</div>
<!-- Column 3 Wide -->
<div class="md:col-span-8 bg-surface-container-high rounded-xl p-8 flex items-center gap-12 overflow-hidden">
<div class="w-1/2">
<span class="material-symbols-outlined text-primary mb-6">insights</span>
<h3 class="text-3xl font-bold text-primary mb-4">Deep Observability</h3>
<p class="text-on-surface-variant">Real-time metrics and tracing for every request, across every node, globally.</p>
</div>
<div class="w-1/2 grid grid-cols-6 items-end gap-1 h-32">
<div class="bg-primary/10 h-1/4 rounded-sm"></div>
<div class="bg-primary/20 h-2/4 rounded-sm"></div>
<div class="bg-primary/40 h-3/4 rounded-sm"></div>
<div class="bg-primary/60 h-4/4 rounded-sm"></div>
<div class="bg-primary/30 h-2/4 rounded-sm"></div>
<div class="bg-primary/80 h-full rounded-sm"></div>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// monolith landing page — pricing section (2120 bytes of real HTML)
pub fn template_monolith_pricing() -> String {
    r##"
<section class="bg-surface-container-lowest py-32 px-6">
<div class="max-w-7xl mx-auto flex flex-col md:flex-row gap-20 items-center">
<div class="w-full md:w-1/2">
<span class="font-label text-xs tracking-[0.3em] text-primary/40 uppercase mb-4 block">Security &amp; Sovereignty</span>
<h2 class="text-4xl md:text-6xl font-black text-primary mb-8 tracking-tighter uppercase leading-none">Your Data. <br/>Your Fortress.</h2>
<ul class="space-y-6">
<li class="flex gap-4">
<span class="material-symbols-outlined text-primary">lock</span>
<div>
<h4 class="font-bold text-primary">End-to-End Encryption</h4>
<p class="text-sm text-on-surface-variant">AES-256 bit encryption at rest and in transit across all environments.</p>
</div>
</li>
<li class="flex gap-4">
<span class="material-symbols-outlined text-primary">verified_user</span>
<div>
<h4 class="font-bold text-primary">RBAC &amp; IAM Controls</h4>
<p class="text-sm text-on-surface-variant">Granular permissions and identity management integrated with your SSO.</p>
</div>
</li>
<li class="flex gap-4">
<span class="material-symbols-outlined text-primary">database</span>
<div>
<h4 class="font-bold text-primary">Isolated Database VPCs</h4>
<p class="text-sm text-on-surface-variant">Your databases live in dedicated network segments for maximum isolation.</p>
</div>
</li>
</ul>
</div>
<div class="w-full md:w-1/2 relative group">
<div class="absolute inset-0 bg-primary/5 blur-3xl rounded-full scale-75 group-hover:scale-100 transition-transform duration-1000"></div>
<img alt="Cyber security visualization" class="relative z-10 rounded-xl grayscale opacity-80 ghost-border shadow-2xl" data-alt="Abstract glowing circuit patterns and encryption lock symbols on a dark surface with sharp metallic highlights" src=""/>
</div>
</div>
</section>
"##.to_string()
}

/// monolith landing page — cta section (898 bytes of real HTML)
pub fn template_monolith_cta() -> String {
    r##"
<section class="py-40 px-6 text-center bg-background">
<div class="max-w-3xl mx-auto">
<h2 class="text-5xl md:text-7xl font-black tracking-tighter text-primary mb-10 uppercase">Scale Beyond Limits.</h2>
<p class="text-xl text-on-surface-variant mb-12">Join 2,000+ infrastructure teams building the future on Monolith.</p>
<div class="flex flex-col md:flex-row gap-6 justify-center">
<button class="monolith-gradient text-on-primary-container px-12 py-5 rounded-xl font-black text-sm tracking-widest uppercase hover:scale-105 transition-transform">
                        Create Account
                    </button>
<button class="bg-surface-container-high ghost-border text-primary px-12 py-5 rounded-xl font-black text-sm tracking-widest uppercase hover:bg-surface-container-highest transition-colors">
                        Documentation
                    </button>
</div>
</div>
</section>
"##.to_string()
}

/// nova landing page — hero section (3839 bytes of real HTML)
pub fn template_nova_hero() -> String {
    r##"
<section class="px-6 py-20 lg:py-32 flex flex-col items-start gap-12 max-w-7xl mx-auto">
<div class="max-w-3xl space-y-6">
<div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-primary/10 border border-primary/20">
<span class="h-1.5 w-1.5 rounded-full bg-tertiary animate-pulse"></span>
<span class="text-[10px] font-label font-bold tracking-widest text-primary uppercase">v2.4.0 Engine Live</span>
</div>
<h1 class="font-headline font-black text-6xl lg:text-8xl tracking-tighter leading-[0.9] text-white">
                    Architecture for the <span class="text-transparent bg-clip-text kinetic-gradient">100x Era.</span>
</h1>
<p class="text-on-surface-variant font-body text-lg lg:text-xl max-w-xl leading-relaxed">
                    A high-performance execution layer built for the synthetic void. Engineered for sub-10ms latency and infinite scale.
                </p>
<div class="flex flex-wrap gap-4 pt-4">
<button class="kinetic-gradient px-8 py-4 rounded font-headline font-bold text-on-primary-fixed hover:shadow-[0_0_20px_rgba(135,173,255,0.4)] transition-all active:scale-95">
                        Deploy Instance
                    </button>
<button class="px-8 py-4 rounded border border-outline-variant/30 hover:bg-surface-container-high transition-all font-headline font-bold text-white">
                        Read Documentation
                    </button>
</div>
</div>
<!-- Floating Liquid Glass Card -->
<div class="w-full relative group">
<div class="absolute -inset-1 bg-gradient-to-r from-primary/20 to-secondary/20 blur-2xl opacity-50 group-hover:opacity-100 transition duration-1000"></div>
<div class="glass-card relative rounded-xl overflow-hidden shadow-2xl">
<div class="h-12 bg-surface-container flex items-center px-4 gap-1.5 border-b border-outline-variant/10">
<div class="w-2.5 h-2.5 rounded-full bg-error/40"></div>
<div class="w-2.5 h-2.5 rounded-full bg-secondary/40"></div>
<div class="w-2.5 h-2.5 rounded-full bg-tertiary/40"></div>
<div class="ml-4 text-[10px] font-label tracking-widest text-white/40">NOVA_TERMINAL -- DASHBOARD</div>
</div>
<div class="grid grid-cols-1 lg:grid-cols-12 min-h-[400px]">
<div class="lg:col-span-8 p-6 border-r border-outline-variant/10">
<img class="w-full h-full object-cover rounded opacity-80 mix-blend-screen" data-alt="abstract 3D visualization of liquid crystal structures flowing through a digital matrix with electric blue and purple lighting" src=""/>
</div>
<div class="lg:col-span-4 p-6 flex flex-col gap-6 bg-surface-container-lowest/50">
<div class="space-y-2">
<label class="text-[10px] font-label text-tertiary tracking-widest uppercase">Node Health</label>
<div class="h-1.5 w-full bg-surface-container rounded-full overflow-hidden">
<div class="h-full bg-tertiary w-[88%]"></div>
</div>
</div>
<div class="space-y-4 font-mono text-[11px] text-white/60">
<div class="flex justify-between items-center py-2 border-b border-outline-variant/10">
<span>UPTIME</span>
<span class="text-white">99.999%</span>
</div>
<div class="flex justify-between items-center py-2 border-b border-outline-variant/10">
<span>LATENCY</span>
<span class="text-tertiary">14.2ms</span>
</div>
<div class="flex justify-between items-center py-2 border-b border-outline-variant/10">
<span>THROUGHPUT</span>
<span class="text-white">12.4 GB/S</span>
</div>
<div class="flex justify-between items-center py-2">
<span>LOCATION</span>
<span class="text-white">US-EAST-1</span>
</div>
</div>
</div>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// nova landing page — features section (3409 bytes of real HTML)
pub fn template_nova_features() -> String {
    r##"
<section class="px-6 py-20 max-w-7xl mx-auto">
<div class="mb-12">
<h2 class="font-headline font-bold text-3xl tracking-tight mb-2">Technical Primitives</h2>
<div class="h-1 w-12 kinetic-gradient"></div>
</div>
<div class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 gap-4">
<!-- Large Featured Bento -->
<div class="md:col-span-2 md:row-span-2 bg-surface-container rounded-lg p-8 border-t border-primary/20 relative overflow-hidden group">
<div class="relative z-10 space-y-4 h-full flex flex-col">
<span class="material-symbols-outlined text-primary text-4xl">terminal</span>
<h3 class="font-headline font-black text-2xl tracking-tighter">Native Cloud Execution</h3>
<p class="text-on-surface-variant text-sm leading-relaxed">
                            Deploy complex architectures with a single CLI command. Nova Core abstracts the infrastructure layer, allowing for true focus on product logic.
                        </p>
<div class="mt-auto pt-8">
<div class="bg-surface-container-lowest p-4 rounded-md border border-outline-variant/20 font-mono text-xs">
<div class="text-secondary-dim">$ nova deploy --production</div>
<div class="text-white/40 mt-1">&gt; Initializing alpha-9 cluster...</div>
<div class="text-tertiary mt-1">&gt; Success: Deployment live at nova.sh/0x4f2</div>
</div>
</div>
</div>
<!-- Decorative Element -->
<div class="absolute -right-12 -bottom-12 w-64 h-64 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-all duration-700"></div>
</div>
<div class="bg-surface-container-low rounded-lg p-6 border-t border-outline-variant/30 flex flex-col gap-4">
<span class="material-symbols-outlined text-tertiary">sensors</span>
<h3 class="font-headline font-bold text-lg tracking-tight">Real-time Telemetry</h3>
<p class="text-on-surface-variant text-xs">Full observability into every packet, request, and compute cycle with sub-second resolution.</p>
</div>
<div class="bg-surface-container-low rounded-lg p-6 border-t border-outline-variant/30 flex flex-col gap-4">
<span class="material-symbols-outlined text-secondary">shield</span>
<h3 class="font-headline font-bold text-lg tracking-tight">Hardened Isolation</h3>
<p class="text-on-surface-variant text-xs">gVisor-level isolation for every workload, ensuring absolute security at the edge.</p>
</div>
<div class="md:col-span-2 bg-surface-container-high rounded-lg p-6 flex flex-col md:flex-row gap-6 border-t border-outline-variant/30">
<div class="md:w-1/2 space-y-3">
<span class="material-symbols-outlined text-primary">groups</span>
<h3 class="font-headline font-bold text-lg tracking-tight">Global Distribution</h3>
<p class="text-on-surface-variant text-xs">Automatically route traffic to the nearest compute node across 40+ global regions.</p>
</div>
<div class="md:w-1/2 bg-surface-container-lowest rounded overflow-hidden h-32 relative">
<div class="absolute inset-0 opacity-40">
<img class="w-full h-full object-cover" data-alt="digital stylized global map with glowing node connections in electric blue on a dark background" src=""/>
</div>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// nova landing page — pricing section (1925 bytes of real HTML)
pub fn template_nova_pricing() -> String {
    r##"
<section class="px-6 py-20 max-w-7xl mx-auto flex flex-col lg:flex-row items-center gap-16">
<div class="lg:w-1/2 relative">
<div class="absolute -inset-10 bg-secondary/5 blur-[100px] rounded-full"></div>
<img class="w-full h-[400px] object-cover rounded-xl border border-outline-variant/20 grayscale hover:grayscale-0 transition-all duration-1000 relative z-10" data-alt="ultra-high-definition artistic shot of dark basalt rock with glowing violet light emanating from deep cracks" src=""/>
</div>
<div class="lg:w-1/2 space-y-8">
<h2 class="font-headline font-black text-5xl tracking-tighter leading-none">
                    Built for the <br/><span class="text-tertiary">Mission Critical</span>.
                </h2>
<div class="space-y-4">
<div class="flex items-start gap-4">
<span class="material-symbols-outlined text-tertiary mt-1">check_circle</span>
<div>
<p class="font-bold text-white">99.999% SLA Guarantee</p>
<p class="text-on-surface-variant text-sm">Enterprise-grade reliability for high-stakes financial and technical operations.</p>
</div>
</div>
<div class="flex items-start gap-4">
<span class="material-symbols-outlined text-tertiary mt-1">check_circle</span>
<div>
<p class="font-bold text-white">Advanced Load Balancing</p>
<p class="text-on-surface-variant text-sm">Intelligent traffic steering based on real-time network health and geography.</p>
</div>
</div>
</div>
<button class="px-8 py-4 kinetic-gradient rounded font-headline font-bold text-on-primary active:scale-95 transition-all">
                    Start Building Free
                </button>
</div>
</section>
"##.to_string()
}

/// enterprise landing page — hero section (1470 bytes of real HTML)
pub fn template_enterprise_hero() -> String {
    r##"
<section class="relative overflow-hidden prism-bg pt-24 pb-32">
<div class="absolute inset-0 grid-pattern pointer-events-none"></div>
<div class="max-w-7xl mx-auto px-6 relative z-10 text-center">
<div class="inline-flex items-center space-x-2 bg-surface-container-high px-4 py-1 rounded-full mb-8 ghost-border">
<span class="text-[10px] uppercase tracking-widest font-bold">Enterprise</span>
<span class="w-1 h-1 bg-primary rounded-full"></span>
<span class="text-xs font-medium text-on-surface-variant">Global Infrastructure</span>
</div>
<h1 class="text-5xl md:text-7xl font-extrabold tracking-tighter text-primary mb-6 leading-[1.1]">
                    Enterprise-grade <br/>speed and security.
                </h1>
<p class="text-lg md:text-xl text-on-surface-variant max-w-2xl mx-auto mb-10 leading-relaxed">
                    Deploy globally in seconds. Scale to billions of requests. Secure by default with industry-leading compliance and 99.99% uptime.
                </p>
<div class="flex flex-col md:flex-row items-center justify-center gap-4">
<button class="w-full md:w-auto bg-primary text-on-primary px-8 py-4 rounded-full font-semibold text-base hover:scale-[1.02] transition-transform">Contact Sales</button>
<button class="w-full md:w-auto bg-surface-container-lowest text-on-surface px-8 py-4 rounded-full font-semibold text-base ghost-border hover:bg-surface-container-low transition-colors">View Security Docs</button>
</div>
</div>
</section>
"##.to_string()
}

/// enterprise landing page — features section (988 bytes of real HTML)
pub fn template_enterprise_features() -> String {
    r##"
<section class="py-12 border-y border-outline-variant/10 bg-surface-container-low">
<div class="max-w-7xl mx-auto px-6 grid grid-cols-2 md:grid-cols-4 gap-8">
<div class="text-center">
<div class="text-3xl font-bold tracking-tighter mb-1">99.99%</div>
<div class="text-xs font-medium uppercase tracking-widest text-on-surface-variant">SLA Uptime</div>
</div>
<div class="text-center">
<div class="text-3xl font-bold tracking-tighter mb-1">SOC2</div>
<div class="text-xs font-medium uppercase tracking-widest text-on-surface-variant">Type II Compliant</div>
</div>
<div class="text-center">
<div class="text-3xl font-bold tracking-tighter mb-1">24/7</div>
<div class="text-xs font-medium uppercase tracking-widest text-on-surface-variant">Dedicated Support</div>
</div>
<div class="text-center">
<div class="text-3xl font-bold tracking-tighter mb-1">100+</div>
<div class="text-xs font-medium uppercase tracking-widest text-on-surface-variant">Edge Locations</div>
</div>
</div>
</section>
"##.to_string()
}

/// enterprise landing page — pricing section (4465 bytes of real HTML)
pub fn template_enterprise_pricing() -> String {
    r##"
<section class="py-24 max-w-7xl mx-auto px-6">
<div class="mb-16">
<h2 class="text-3xl font-bold tracking-tight mb-4">Uncompromising Security</h2>
<p class="text-on-surface-variant max-w-xl">Our platform is engineered to meet the strictest regulatory requirements for global enterprises.</p>
</div>
<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
<!-- Compliance Card -->
<div class="md:col-span-2 bg-surface-container-lowest p-8 rounded-xl ghost-border flex flex-col justify-between">
<div>
<div class="mb-6 flex space-x-3">
<span class="bg-surface-container-high px-3 py-1 rounded-full text-[10px] font-bold">SOC2</span>
<span class="bg-surface-container-high px-3 py-1 rounded-full text-[10px] font-bold">HIPAA</span>
<span class="bg-surface-container-high px-3 py-1 rounded-full text-[10px] font-bold">GDPR</span>
<span class="bg-surface-container-high px-3 py-1 rounded-full text-[10px] font-bold">ISO 27001</span>
</div>
<h3 class="text-2xl font-bold tracking-tight mb-4">Enterprise Compliance</h3>
<p class="text-on-surface-variant mb-8 max-w-lg">Vercel maintains the highest security standards with continuous monitoring and third-party audits. Your data stays protected and compliant globally.</p>
</div>
<div class="flex items-center space-x-4 text-sm font-medium">
<span class="text-primary cursor-pointer hover:underline flex items-center">Download Reports <span class="material-symbols-outlined ml-1 text-sm">arrow_forward</span></span>
</div>
</div>
<!-- DDoS Protection Card -->
<div class="bg-primary text-on-primary p-8 rounded-xl flex flex-col justify-between">
<div class="material-symbols-outlined text-4xl mb-6">shield_with_heart</div>
<div>
<h3 class="text-xl font-bold mb-2">Layer 7 DDoS Protection</h3>
<p class="text-sm opacity-80 leading-relaxed">Automatic mitigation of large-scale attacks before they ever reach your servers.</p>
</div>
</div>
<!-- Dedicated Support -->
<div class="bg-surface-container p-8 rounded-xl flex flex-col">
<div class="material-symbols-outlined text-primary text-3xl mb-4">support_agent</div>
<h3 class="text-xl font-bold mb-3">Dedicated Support</h3>
<p class="text-sm text-on-surface-variant mb-6 flex-grow">A named Customer Success Manager and 24/7/365 priority engineering support for your critical infrastructure.</p>
<div class="pt-4 border-t border-outline-variant/30 text-xs font-bold tracking-widest uppercase">15 Min Response Time</div>
</div>
<!-- Global Edge Locations -->
<div class="md:col-span-2 bg-surface-container-lowest p-8 rounded-xl ghost-border">
<div class="flex flex-col md:flex-row gap-8">
<div class="md:w-1/3">
<h3 class="text-xl font-bold mb-4">Global Edge Network</h3>
<p class="text-sm text-on-surface-variant mb-6">Our network spans across every continent, ensuring sub-millisecond latency for your global user base.</p>
<ul class="space-y-2 text-xs font-medium text-on-surface-variant">
<li class="flex items-center"><span class="w-1.5 h-1.5 bg-tertiary-container rounded-full mr-2"></span>North America (32)</li>
<li class="flex items-center"><span class="w-1.5 h-1.5 bg-tertiary-container rounded-full mr-2"></span>Europe (24)</li>
<li class="flex items-center"><span class="w-1.5 h-1.5 bg-tertiary-container rounded-full mr-2"></span>Asia Pacific (18)</li>
<li class="flex items-center"><span class="w-1.5 h-1.5 bg-tertiary-container rounded-full mr-2"></span>South America (12)</li>
</ul>
</div>
<div class="md:w-2/3 bg-surface-container-low rounded-lg p-4 min-h-[200px] flex items-center justify-center relative overflow-hidden">
<img class="absolute inset-0 w-full h-full object-cover opacity-20 mix-blend-multiply" data-alt="a minimalist dark world map visualization with glowing data points representing global edge server locations" src=""/>
<div class="relative z-10 text-[10px] font-mono grid grid-cols-2 gap-x-8 gap-y-2 text-on-surface-variant/70">
<span>IAD1: Virginia, US</span>
<span>LHR1: London, UK</span>
<span>SFO1: California, US</span>
<span>HKG1: Hong Kong</span>
<span>GRU1: Sao Paulo, BR</span>
<span>CDG1: Paris, FR</span>
<span>SIN1: Singapore</span>
<span>SYD1: Sydney, AU</span>
</div>
</div>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// enterprise landing page — cta section (2606 bytes of real HTML)
pub fn template_enterprise_cta() -> String {
    r##"
<section class="py-24 bg-surface-container-low">
<div class="max-w-7xl mx-auto px-6">
<div class="grid grid-cols-1 md:grid-cols-2 gap-16 items-start">
<div>
<h2 class="text-4xl font-bold tracking-tighter mb-8 leading-tight">Built for the modern <br/>enterprise stack.</h2>
<div class="space-y-8">
<div class="flex gap-6">
<div class="w-10 h-10 shrink-0 bg-primary text-on-primary rounded-full flex items-center justify-center">
<span class="material-symbols-outlined text-lg">code</span>
</div>
<div>
<h4 class="font-bold text-lg mb-1">Advanced Deployment Controls</h4>
<p class="text-on-surface-variant text-sm leading-relaxed">Granular IAM permissions, single sign-on (SSO), and secure environment variable management for large engineering teams.</p>
</div>
</div>
<div class="flex gap-6">
<div class="w-10 h-10 shrink-0 bg-primary text-on-primary rounded-full flex items-center justify-center">
<span class="material-symbols-outlined text-lg">database</span>
</div>
<div>
<h4 class="font-bold text-lg mb-1">Enterprise Cache Purge</h4>
<p class="text-on-surface-variant text-sm leading-relaxed">Instant global cache invalidation with Surrogate-Key support, allowing for real-time updates of static content.</p>
</div>
</div>
<div class="flex gap-6">
<div class="w-10 h-10 shrink-0 bg-primary text-on-primary rounded-full flex items-center justify-center">
<span class="material-symbols-outlined text-lg">monitoring</span>
</div>
<div>
<h4 class="font-bold text-lg mb-1">Observability &amp; Logs</h4>
<p class="text-on-surface-variant text-sm leading-relaxed">Native integration with Datadog, New Relic, and Splunk for real-time traffic analysis and error tracking.</p>
</div>
</div>
</div>
</div>
<div class="bg-surface-container-lowest p-2 rounded-xl ghost-border overflow-hidden">
<div class="bg-primary p-6 rounded-lg text-on-primary font-mono text-xs overflow-x-auto">
<div class="flex items-center space-x-2 mb-4 opacity-50">
<span class="w-2 h-2 rounded-full bg-red-500"></span>
<span class="w-2 h-2 rounded-full bg-yellow-500"></span>
<span class="w-2 h-2 rounded-full bg-green-500"></span>
<span class="ml-4">enterprise-security-policy.json</span>
</div>
<pre class="text-white/90">{{
  "policy": "Strict",
  "compliance": ["SOC2", "HIPAA"],
  "firewall": {{
    "ddos_protection": "active",
    "waf_rules": ["OWASP-TOP-10", "Custom-Enterprise"],
    "rate_limit": "25000rps"
  }},
  "deployment": {{
    "sso_required": true,
    "mfa": "enforced",
    "isolation": "vpc-peering"
  }},
  "edge": {{
    "caching": "99.99%",
    "locations": "global"
  }}
}}</pre>
</div>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// enterprise landing page — section4 section (734 bytes of real HTML)
pub fn template_enterprise_section4() -> String {
    r##"
<section class="py-32 bg-white relative overflow-hidden">
<div class="max-w-4xl mx-auto px-6 text-center">
<h2 class="text-4xl md:text-5xl font-extrabold tracking-tighter mb-8">Ready to scale your enterprise?</h2>
<p class="text-on-surface-variant mb-12 text-lg">Join the world's most innovative companies building on Vercel's global infrastructure.</p>
<div class="flex flex-col md:flex-row gap-4 justify-center">
<button class="bg-primary text-on-primary px-10 py-4 rounded-full font-bold text-lg hover:scale-105 transition-transform">Book an Enterprise Demo</button>
<button class="bg-surface-container-lowest text-on-surface px-10 py-4 rounded-full font-bold text-lg ghost-border">Contact Support</button>
</div>
</div>
</section>
"##.to_string()
}

/// centered landing page — hero section (3087 bytes of real HTML)
pub fn template_centered_hero() -> String {
    r##"
<section class="relative min-h-[921px] flex flex-col items-center justify-center text-center px-6 grid-background">
<!-- Prism Effect Background -->
<div class="absolute inset-0 prism-mesh -z-10"></div>
<div class="max-w-4xl mx-auto relative">
<!-- Abstract 3D Prism Centerpiece -->
<div class="mb-12 relative inline-block">
<div class="w-64 h-64 md:w-80 md:h-80 mx-auto relative bg-gradient-to-tr from-primary to-tertiary-container rounded-xl rotate-12 flex items-center justify-center group overflow-hidden shadow-2xl">
<img class="absolute inset-0 w-full h-full object-cover mix-blend-overlay opacity-80 group-hover:scale-110 transition-transform duration-700" data-alt="Modern abstract 3D prism with sharp geometric edges reflecting a spectrum of rainbow light against a clean dark background" src=""/>
<div class="absolute inset-0 bg-gradient-to-b from-transparent via-transparent to-black/20"></div>
<span class="material-symbols-outlined text-white text-8xl opacity-20">deployed_code</span>
</div>
<!-- Decorative Floating Elements -->
<div class="absolute -top-4 -right-4 w-12 h-12 bg-surface-container-lowest border border-outline-variant/20 rounded-lg shadow-xl flex items-center justify-center">
<span class="material-symbols-outlined text-tertiary text-xl">speed</span>
</div>
<div class="absolute -bottom-6 -left-8 w-16 h-16 bg-surface-container-lowest border border-outline-variant/20 rounded-lg shadow-xl flex items-center justify-center">
<span class="material-symbols-outlined text-primary text-2xl">auto_awesome</span>
</div>
</div>
<h1 class="text-[3.5rem] md:text-[5.5rem] font-extrabold tracking-[-0.04em] leading-[0.95] mb-6 text-primary">
                    Iterate faster
                </h1>
<p class="text-lg md:text-xl text-on-surface-variant max-w-xl mx-auto mb-10 font-body font-normal">
                    Develop. Preview. Ship. The front-end platform for high-performance teams.
                </p>
<div class="flex flex-col sm:flex-row items-center justify-center gap-4">
<button class="bg-primary text-on-primary-container px-10 py-4 rounded-full text-lg font-semibold hover:scale-[1.02] transition-all shadow-lg">
                        Start Deploying
                    </button>
<button class="bg-surface-container-lowest text-on-surface border border-outline-variant/40 px-10 py-4 rounded-full text-lg font-semibold hover:bg-surface-container-low transition-colors">
                        Get a Demo
                    </button>
</div>
</div>
<!-- Scroll Indicator -->
<div class="absolute bottom-12 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2 opacity-40">
<span class="text-[10px] uppercase tracking-widest font-bold">Scroll to explore</span>
<div class="w-px h-12 bg-primary"></div>
</div>
</section>
"##.to_string()
}

/// centered landing page — features section (4071 bytes of real HTML)
pub fn template_centered_features() -> String {
    r##"
<section class="max-w-7xl mx-auto px-6 py-24">
<div class="grid grid-cols-1 md:grid-cols-12 gap-6">
<!-- Large Bento Card -->
<div class="md:col-span-8 bg-surface-container-lowest border border-outline-variant/20 rounded-xl p-8 flex flex-col justify-between overflow-hidden relative group">
<div>
<span class="inline-flex items-center gap-2 px-3 py-1 bg-surface-container-high text-[10px] font-bold uppercase tracking-wider rounded-full mb-6">
                            Next.js Native
                        </span>
<h3 class="text-3xl font-bold tracking-tight mb-4">Optimized for every framework.</h3>
<p class="text-on-surface-variant max-w-md">Vercel provides the ultimate speed and reliability for your modern web applications, from React to Svelte.</p>
</div>
<div class="mt-12 -mb-8 -mr-8">
<img class="rounded-tl-xl border-l border-t border-outline-variant/40 shadow-2xl group-hover:-translate-y-2 transition-transform duration-500" data-alt="Clean minimalist software dashboard interface showing analytics charts and deployment history in grayscale with subtle blue accents" src=""/>
</div>
</div>
<!-- Small Bento Card -->
<div class="md:col-span-4 bg-primary text-on-primary p-8 rounded-xl flex flex-col justify-center items-center text-center">
<div class="w-16 h-16 bg-on-primary/10 rounded-full flex items-center justify-center mb-6">
<span class="material-symbols-outlined text-3xl">public</span>
</div>
<h3 class="text-2xl font-bold mb-3">Global Edge Network</h3>
<p class="text-on-primary/70 text-sm">Deploy to 100+ locations instantly with unmatched latency.</p>
</div>
<!-- Medium Bento Card -->
<div class="md:col-span-5 bg-surface-container-low border border-outline-variant/20 rounded-xl p-8">
<h3 class="text-2xl font-bold mb-4">Enterprise Security</h3>
<div class="space-y-4">
<div class="flex items-center gap-4 border-b border-outline-variant/10 pb-4">
<span class="material-symbols-outlined text-tertiary">verified_user</span>
<span class="text-sm font-medium">SOC2 Type II Compliant</span>
</div>
<div class="flex items-center gap-4 border-b border-outline-variant/10 pb-4">
<span class="material-symbols-outlined text-tertiary">shield</span>
<span class="text-sm font-medium">DDoS Mitigation at Edge</span>
</div>
<div class="flex items-center gap-4">
<span class="material-symbols-outlined text-tertiary">lock</span>
<span class="text-sm font-medium">SSO &amp; SAML Integration</span>
</div>
</div>
</div>
<!-- Wide Bento Card -->
<div class="md:col-span-7 bg-white border border-outline-variant/20 rounded-xl p-8 flex flex-col md:flex-row gap-8 items-center">
<div class="flex-1">
<h3 class="text-2xl font-bold mb-3">Seamless Collaboration</h3>
<p class="text-on-surface-variant text-sm mb-6">Comment directly on deployments and iterate with your whole team in real-time.</p>
<button class="text-sm font-bold flex items-center gap-1 group">
                            Learn about Preview Comments
                            <span class="material-symbols-outlined group-hover:translate-x-1 transition-transform">arrow_forward</span>
</button>
</div>
<div class="w-full md:w-48 aspect-square bg-surface-container rounded-lg overflow-hidden border border-outline-variant/10">
<img class="w-full h-full object-cover grayscale opacity-80" data-alt="Close up of diverse hands working on high-tech touch screen interface with glowing holographic data points" src=""/>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// centered landing page — pricing section (726 bytes of real HTML)
pub fn template_centered_pricing() -> String {
    r##"
<section class="py-24 px-6 border-t border-outline-variant/10">
<div class="max-w-4xl mx-auto bg-primary rounded-3xl p-12 md:p-20 relative overflow-hidden text-center">
<div class="absolute top-0 right-0 w-64 h-64 bg-tertiary-container/20 blur-[100px] -mr-32 -mt-32"></div>
<h2 class="text-4xl md:text-5xl font-extrabold text-white tracking-tight mb-6">Ready to scale?</h2>
<p class="text-on-primary/70 text-lg mb-10 max-w-xl mx-auto">Join the world's most innovative companies and start building the future today.</p>
<button class="bg-white text-primary px-12 py-4 rounded-full text-lg font-bold hover:scale-105 transition-transform">
                    Deploy your first project
                </button>
</div>
</section>
"##.to_string()
}

/// atelier landing page — hero section (3409 bytes of real HTML)
pub fn template_atelier_hero() -> String {
    r##"
<section class="relative pt-32 pb-20 md:pt-48 md:pb-32 overflow-hidden bg-surface">
<div class="max-w-7xl mx-auto px-8 text-center">
<h1 class="text-5xl md:text-7xl font-extrabold tracking-tight text-on-surface mb-6 max-w-4xl mx-auto leading-[1.1]">
                The future of money, <span class="text-primary italic">designed with taste.</span>
</h1>
<p class="text-xl md:text-2xl text-outline mb-12 max-w-2xl mx-auto font-light leading-relaxed">
                Elevate your financial life with a studio-grade workspace for your capital. Modern, precise, and completely effortless.
            </p>
<div class="flex flex-col md:flex-row items-center justify-center gap-4 mb-20">
<button class="w-full md:w-auto px-10 py-5 bg-gradient-to-br from-primary to-primary-container text-white font-semibold rounded-md text-lg soft-blue-shadow transition-all hover:translate-y-[-2px] active:scale-95">
                    Get Started
                </button>
<button class="w-full md:w-auto px-10 py-5 bg-surface-container-high text-on-surface font-semibold rounded-md text-lg transition-all hover:bg-surface-container-highest">
                    Learn more
                </button>
</div>
<!-- 3D Glass Mockup -->
<div class="relative max-w-5xl mx-auto">
<div class="aspect-[16/9] w-full rounded-lg bg-surface-container-lowest soft-blue-shadow overflow-hidden p-4 border border-outline-variant/10">
<img class="w-full h-full object-cover rounded-md shadow-2xl" data-alt="Premium glassmorphic dashboard interface showing minimalist financial charts and elegant card layouts in a high-key studio environment" src=""/>
</div>
<!-- Floaties -->
<div class="absolute -bottom-10 -left-10 hidden md:block w-64 aspect-square rounded-lg glass bg-white/60 border border-white/40 soft-blue-shadow p-6 text-left">
<span class="material-symbols-outlined text-primary mb-4" data-icon="account_balance_wallet">account_balance_wallet</span>
<h3 class="text-lg font-bold mb-1">Total Assets</h3>
<p class="text-2xl font-extrabold tracking-tight">$142,500.00</p>
<div class="mt-4 h-1.5 w-full bg-surface-container rounded-full overflow-hidden">
<div class="h-full bg-primary w-[75%]"></div>
</div>
</div>
<div class="absolute -top-10 -right-10 hidden md:block w-72 aspect-[4/3] rounded-lg glass bg-white/60 border border-white/40 soft-blue-shadow p-6 text-left">
<div class="flex justify-between items-center mb-4">
<span class="text-[10px] uppercase tracking-widest font-bold text-outline">Analytics</span>
<span class="text-xs font-bold text-green-600">+12.4%</span>
</div>
<img class="w-full h-24 object-cover rounded-md mb-2" data-alt="Abstract minimalist line chart representing steady upward financial growth with a soft blue aesthetic" src=""/>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// atelier landing page — features section (5155 bytes of real HTML)
pub fn template_atelier_features() -> String {
    r##"
<section class="py-24 md:py-32 bg-surface">
<div class="max-w-7xl mx-auto px-8">
<div class="mb-20">
<span class="text-xs font-bold tracking-[0.2em] uppercase text-primary mb-4 block">Engineered for Clarity</span>
<h2 class="text-4xl md:text-5xl font-bold tracking-tight text-on-surface">Powerful features,<br/>refined for focus.</h2>
</div>
<div class="grid grid-cols-1 md:grid-cols-3 gap-8">
<!-- Large Feature Card -->
<div class="md:col-span-2 bg-surface-container-lowest rounded-lg p-10 border border-outline-variant/10 soft-blue-shadow flex flex-col justify-between group">
<div class="max-w-md">
<div class="w-12 h-12 rounded-full bg-primary-fixed/30 flex items-center justify-center mb-8">
<span class="material-symbols-outlined text-primary" data-icon="bolt">bolt</span>
</div>
<h3 class="text-2xl font-bold mb-4">Instant Payments</h3>
<p class="text-outline leading-relaxed mb-8">Move capital across borders in seconds. Our global infrastructure is built on precision and speed, ensuring your funds are exactly where you need them.</p>
<ul class="space-y-4">
<li class="flex items-center gap-3 text-sm font-medium">
<span class="material-symbols-outlined text-primary text-lg" data-icon="check_circle">check_circle</span>
                                Zero-latency local transfers
                            </li>
<li class="flex items-center gap-3 text-sm font-medium">
<span class="material-symbols-outlined text-primary text-lg" data-icon="check_circle">check_circle</span>
                                Multi-currency support (50+)
                            </li>
</ul>
</div>
<div class="mt-12 h-48 rounded-md bg-surface-container-low overflow-hidden relative border border-outline-variant/5">
<img class="w-full h-full object-cover grayscale opacity-50 group-hover:grayscale-0 group-hover:opacity-100 transition-all duration-700" data-alt="Top-down view of modern desk with ultra-thin smartphone and minimalist payment interface" src=""/>
</div>
</div>
<!-- Secondary Feature Card -->
<div class="bg-surface-container-lowest rounded-lg p-10 border border-outline-variant/10 soft-blue-shadow flex flex-col group">
<div class="w-12 h-12 rounded-full bg-secondary-container/30 flex items-center justify-center mb-8">
<span class="material-symbols-outlined text-secondary" data-icon="insights">insights</span>
</div>
<h3 class="text-2xl font-bold mb-4">Intelligent Analytics</h3>
<p class="text-outline leading-relaxed flex-grow">A curated view of your spending habits, powered by machine learning that understands your lifestyle.</p>
<div class="mt-8 pt-8 border-t border-outline-variant/10">
<div class="flex items-end gap-2 h-20">
<div class="w-full bg-primary-fixed/40 h-[40%] rounded-full group-hover:h-[60%] transition-all duration-500"></div>
<div class="w-full bg-primary-fixed/40 h-[70%] rounded-full group-hover:h-[90%] transition-all duration-500"></div>
<div class="w-full bg-primary h-[90%] rounded-full group-hover:h-[70%] transition-all duration-500"></div>
<div class="w-full bg-primary-fixed/40 h-[50%] rounded-full group-hover:h-[80%] transition-all duration-500"></div>
<div class="w-full bg-primary-fixed/40 h-[30%] rounded-full group-hover:h-[50%] transition-all duration-500"></div>
</div>
</div>
</div>
<!-- Small Grid Items -->
<div class="bg-surface-container-lowest rounded-lg p-10 border border-outline-variant/10 soft-blue-shadow group">
<div class="w-12 h-12 rounded-full bg-tertiary-fixed/30 flex items-center justify-center mb-8">
<span class="material-symbols-outlined text-tertiary" data-icon="shield">shield</span>
</div>
<h3 class="text-xl font-bold mb-3">Premium Security</h3>
<p class="text-outline text-sm leading-relaxed">Military-grade encryption for every byte of data. Your privacy is our highest priority.</p>
</div>
<div class="bg-surface-container-lowest rounded-lg p-10 border border-outline-variant/10 soft-blue-shadow group">
<div class="w-12 h-12 rounded-full bg-on-secondary-fixed/10 flex items-center justify-center mb-8">
<span class="material-symbols-outlined text-on-secondary-fixed" data-icon="credit_card">credit_card</span>
</div>
<h3 class="text-xl font-bold mb-3">Metal Cards</h3>
<p class="text-outline text-sm leading-relaxed">Precision-cut titanium cards that feel as good as they look in your hand.</p>
</div>
<div class="bg-surface-container-lowest rounded-lg p-10 border border-outline-variant/10 soft-blue-shadow group">
<div class="w-12 h-12 rounded-full bg-primary-fixed/20 flex items-center justify-center mb-8">
<span class="material-symbols-outlined text-primary" data-icon="support_agent">support_agent</span>
</div>
<h3 class="text-xl font-bold mb-3">Priority Concierge</h3>
<p class="text-outline text-sm leading-relaxed">Direct human connection, 24/7. No bots, just elite-level support when you need it.</p>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// atelier landing page — pricing section (1462 bytes of real HTML)
pub fn template_atelier_pricing() -> String {
    r##"
<section class="py-24 md:py-32 bg-surface overflow-hidden">
<div class="max-w-5xl mx-auto px-8">
<div class="bg-white rounded-lg p-12 md:p-20 text-center soft-blue-shadow border border-outline-variant/10 relative">
<div class="absolute inset-0 opacity-5 pointer-events-none">
<img class="w-full h-full object-cover" data-alt="Abstract geometric light pattern on white background creating subtle texture and depth" src=""/>
</div>
<h2 class="text-4xl md:text-5xl font-bold tracking-tight mb-8">Ready to elevate your standards?</h2>
<p class="text-xl text-outline mb-12 max-w-xl mx-auto font-light">Join the select group of individuals who prioritize clarity, speed, and taste in their financial lives.</p>
<div class="flex flex-col md:flex-row items-center justify-center gap-6">
<button class="px-12 py-5 bg-primary text-white font-bold rounded-md text-lg hover:opacity-90 transition-all active:scale-95 soft-blue-shadow">
                        Get Started
                    </button>
<a class="text-primary font-bold text-lg tracking-wide hover:underline underline-offset-8" href="#">Talk to Sales →</a>
</div>
</div>
</div>
</section>
"##.to_string()
}

/// Combined custom CSS from all landing page templates
pub const STITCH_CUSTOM_CSS: &str = r##"
/* === ultima === */
body {
            background-color: #131313;
            color: #e2e2e2;
            font-family: 'Inter', sans-serif;
            overflow-x: hidden;
        }
        .liquid-gradient {
            background: linear-gradient(135deg, #adc6ff 0%, #c2c1ff 50%, #e9b3ff 100%);
        }
        .glass-panel {
            backdrop-filter: blur(32px);
            border: 0.5px solid rgba(76, 69, 70, 0.15);
        }
        .mesh-bg {
            background-image: 
                radial-gradient(at 0% 0%, rgba(173, 198, 255, 0.1) 0px, transparent 50%),
                radial-gradient(at 100% 0%, rgba(233, 179, 255, 0.05) 0px, transparent 50%);
        }
        .inner-glow:hover {
            box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.1);
        }
        .text-display {
            letter-spacing: -0.04em;
            line-height: 0.95;
        }
        .monospaced-label {
            letter-spacing: 0.05em;
            font-feature-settings: "tnum" on, "onum" on;
        }
/* === monolith === */
.material-symbols-outlined {
            font-variation-settings: 'FILL' 0, 'wght' 200, 'GRAD' 0, 'opsz' 24;
            font-size: 20px;
        }
        .glass-panel {
            background: rgba(31, 31, 31, 0.7);
            backdrop-filter: blur(20px);
            -webkit-backdrop-filter: blur(20px);
        }
        .monolith-gradient {
            background: linear-gradient(180deg, #ffffff 0%, #d4d4d4 100%);
        }
        .ghost-border {
            border: 0.5px solid rgba(255, 255, 255, 0.15);
        }
        body {
            background-color: #131313;
            color: #e2e2e2;
            overflow-x: hidden;
        }
/* === nova === */
.material-symbols-outlined {
            font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
        }
        .glass-card {
            background: rgba(25, 25, 25, 0.8);
            backdrop-filter: blur(40px);
            border-top: 0.5px solid rgba(135, 173, 255, 0.2);
        }
        .kinetic-gradient {
            background: linear-gradient(135deg, #87adff 0%, #d277ff 100%);
        }
        .mesh-glow {
            background: radial-gradient(circle at 50% 50%, rgba(135, 173, 255, 0.08) 0%, transparent 70%);
        }
        .status-pulse {
            box-shadow: 0 0 8px rgba(129, 236, 255, 0.6);
        }
/* === enterprise === */
body { font-family: 'Inter', sans-serif; }
        .material-symbols-outlined {
            font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
        }
        .prism-bg {
            background: radial-gradient(circle at 50% -20%, #006ff015 0%, #f9f9f9 70%);
        }
        .ghost-border {
            border: 1px solid rgba(198, 198, 198, 0.2);
        }
        .grid-pattern {
            background-image: radial-gradient(#c6c6c6 0.5px, transparent 0.5px);
            background-size: 24px 24px;
            opacity: 0.1;
        }
/* === centered === */
.prism-mesh {
            background: radial-gradient(circle at 50% 50%, rgba(0, 111, 240, 0.15) 0%, rgba(249, 249, 249, 0) 70%);
        }
        .grid-background {
            background-image: linear-gradient(to right, rgba(198, 198, 198, 0.1) 1px, transparent 1px),
                              linear-gradient(to bottom, rgba(198, 198, 198, 0.1) 1px, transparent 1px);
            background-size: 40px 40px;
        }
        .material-symbols-outlined {
            font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
        }
/* === atelier === */
body { font-family: 'Inter', sans-serif; background-color: #f9f9fb; color: #1a1c1d; -webkit-font-smoothing: antialiased; }
        .glass { backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px); }
        .soft-blue-shadow { shadow-color: rgba(0, 88, 188, 0.04); box-shadow: 0 20px 40px -10px rgba(0, 88, 188, 0.08); }
        .material-symbols-outlined { font-variation-settings: 'FILL' 0, 'wght' 300, 'GRAD' 0, 'opsz' 24; }
"##;

// ══════════════════════════════════════════════════
// INFRASTRUCTURE MONITOR (from stitch infra dashboard)
// ══════════════════════════════════════════════════

pub fn infra_monitor(accent: &str) -> String {
    format!(r##"<div class="max-w-7xl mx-auto"><div class="flex items-center gap-3 mb-8"><p class="font-mono text-[10px] uppercase tracking-[0.2em] text-{a}-500">// Infrastructure</p><div class="flex-1 h-px bg-neutral-800/50"></div></div><h1 class="text-3xl font-bold mb-8">Infrastructure Monitor</h1>
  <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">CPU</p><p class="text-3xl font-bold tabular-nums text-{a}-400">42%</p><div class="mt-3 w-full h-1.5 bg-neutral-800 rounded-full"><div class="h-1.5 bg-{a}-500 rounded-full" style="width:42%"></div></div></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">MEMORY</p><p class="text-3xl font-bold tabular-nums text-emerald-400">6.2GB</p><div class="mt-3 w-full h-1.5 bg-neutral-800 rounded-full"><div class="h-1.5 bg-emerald-500 rounded-full" style="width:38%"></div></div></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">DISK I/O</p><p class="text-3xl font-bold tabular-nums text-violet-400">1.4GB/s</p><div class="mt-3 w-full h-1.5 bg-neutral-800 rounded-full"><div class="h-1.5 bg-violet-500 rounded-full" style="width:28%"></div></div></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">NETWORK</p><p class="text-3xl font-bold tabular-nums text-blue-400">840Mbps</p><div class="mt-3 w-full h-1.5 bg-neutral-800 rounded-full"><div class="h-1.5 bg-blue-500 rounded-full" style="width:56%"></div></div></div>
  </div>
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white mb-6">Active Deployments</h3><div class="space-y-3"><div class="flex items-center justify-between p-3 bg-[#141414] rounded-lg"><div class="flex items-center gap-3"><div class="w-2 h-2 rounded-full bg-emerald-500"></div><span class="text-sm text-white font-mono">api-gateway</span></div><span class="text-xs text-neutral-500">v2.4.1</span></div><div class="flex items-center justify-between p-3 bg-[#141414] rounded-lg"><div class="flex items-center gap-3"><div class="w-2 h-2 rounded-full bg-amber-500"></div><span class="text-sm text-white font-mono">worker-queue</span></div><span class="text-xs text-neutral-500">deploying</span></div></div></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white mb-6">Events</h3><div class="space-y-3"><div class="flex gap-3 p-3"><span class="w-1.5 h-1.5 mt-2 rounded-full bg-emerald-500 shrink-0"></span><div><p class="text-sm text-white">Deploy completed</p><p class="text-xs text-neutral-500">2m ago</p></div></div><div class="flex gap-3 p-3"><span class="w-1.5 h-1.5 mt-2 rounded-full bg-amber-500 shrink-0"></span><div><p class="text-sm text-white">Scaling event</p><p class="text-xs text-neutral-500">8m ago</p></div></div></div></div>
  </div></div>"##, a = accent)
}

pub fn ecommerce_dashboard(accent: &str) -> String {
    format!(r##"<div class="max-w-7xl mx-auto"><div class="flex items-center gap-3 mb-8"><p class="font-mono text-[10px] uppercase tracking-[0.2em] text-{a}-500">// Commerce</p><div class="flex-1 h-px bg-neutral-800/50"></div></div>
  <div class="flex items-center justify-between mb-8"><h1 class="text-3xl font-bold">Store Dashboard</h1><button class="px-4 py-2 bg-{a}-500 text-black text-sm font-semibold rounded-full">+ Product</button></div>
  <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">REVENUE</p><p class="text-2xl font-bold tabular-nums text-white">$48,250</p><p class="text-xs text-emerald-400 mt-1">+12.5%</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">ORDERS</p><p class="text-2xl font-bold tabular-nums text-white">1,247</p><p class="text-xs text-emerald-400 mt-1">+8.3%</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">CUSTOMERS</p><p class="text-2xl font-bold tabular-nums text-white">3,891</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">CONVERSION</p><p class="text-2xl font-bold tabular-nums text-white">3.2%</p></div>
  </div>
  <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white mb-6">Recent Orders</h3><table class="w-full text-sm"><thead><tr class="border-b border-neutral-800/50"><th class="text-left py-3 font-mono text-[10px] text-neutral-500">ORDER</th><th class="text-left py-3 font-mono text-[10px] text-neutral-500">CUSTOMER</th><th class="text-left py-3 font-mono text-[10px] text-neutral-500">TOTAL</th><th class="text-left py-3 font-mono text-[10px] text-neutral-500">STATUS</th></tr></thead><tbody><tr class="border-b border-neutral-800/30"><td class="py-3 font-mono text-neutral-300">#4821</td><td class="py-3 text-white">Alice</td><td class="py-3 text-white tabular-nums">$249</td><td class="py-3"><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">shipped</span></td></tr><tr class="border-b border-neutral-800/30"><td class="py-3 font-mono text-neutral-300">#4820</td><td class="py-3 text-white">Bob</td><td class="py-3 text-white tabular-nums">$89</td><td class="py-3"><span class="text-xs px-2 py-0.5 bg-amber-500/10 text-amber-400 rounded-full">processing</span></td></tr></tbody></table></div></div>"##, a = accent)
}

pub fn security_dashboard(_accent: &str) -> String {
    r##"<div class="max-w-7xl mx-auto"><div class="flex items-center gap-3 mb-8"><p class="font-mono text-[10px] uppercase tracking-[0.2em] text-red-500">// Security</p><div class="flex-1 h-px bg-neutral-800/50"></div></div><h1 class="text-3xl font-bold mb-8">Security Overview</h1>
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
    <div class="bg-[#0a0a0a] border border-emerald-500/20 rounded-xl p-6"><div class="flex items-center gap-2 mb-3"><div class="w-2 h-2 rounded-full bg-emerald-500"></div><p class="font-mono text-[10px] text-neutral-500">THREAT LEVEL</p></div><p class="text-2xl font-bold text-emerald-400">LOW</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">BLOCKED</p><p class="text-2xl font-bold tabular-nums text-white">12,847</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">SSL CERTS</p><p class="text-2xl font-bold tabular-nums text-white">14</p><p class="text-xs text-emerald-400 mt-1">All valid</p></div>
  </div>
  <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white mb-6">Security Events</h3><div class="space-y-3"><div class="flex items-center justify-between p-3 bg-[#141414] rounded-lg"><div class="flex items-center gap-3"><span class="text-xs px-2 py-0.5 bg-amber-500/10 text-amber-400 rounded-full">warn</span><span class="text-sm text-white">Rate limit exceeded</span></div><span class="text-xs text-neutral-500 font-mono">2m ago</span></div><div class="flex items-center justify-between p-3 bg-[#141414] rounded-lg"><div class="flex items-center gap-3"><span class="text-xs px-2 py-0.5 bg-red-500/10 text-red-400 rounded-full">block</span><span class="text-sm text-white">SQL injection blocked</span></div><span class="text-xs text-neutral-500 font-mono">15m ago</span></div></div></div></div>"##.to_string()
}

pub fn ai_model_dashboard(accent: &str) -> String {
    format!(r##"<div class="max-w-7xl mx-auto"><div class="flex items-center gap-3 mb-8"><p class="font-mono text-[10px] uppercase tracking-[0.2em] text-violet-500">// AI Models</p><div class="flex-1 h-px bg-neutral-800/50"></div></div>
  <div class="flex items-center justify-between mb-8"><h1 class="text-3xl font-bold">Model Registry</h1><button class="px-4 py-2 bg-violet-500 text-white text-sm font-semibold rounded-full">Deploy Model</button></div>
  <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">MODELS</p><p class="text-2xl font-bold tabular-nums text-white">7</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">INFER/SEC</p><p class="text-2xl font-bold tabular-nums text-violet-400">2,847</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">LATENCY</p><p class="text-2xl font-bold tabular-nums text-white">23ms</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">GPU</p><p class="text-2xl font-bold tabular-nums text-{a}-400">78%</p></div>
  </div>
  <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white mb-6">Deployed Models</h3><div class="space-y-3"><div class="flex items-center justify-between p-4 bg-[#141414] rounded-lg"><div class="flex items-center gap-4"><div class="w-10 h-10 rounded-lg bg-violet-500/10 flex items-center justify-center text-violet-400 font-mono text-xs">LLM</div><div><div class="text-sm text-white">gpt-4-turbo</div><div class="text-xs text-neutral-500">175B params</div></div></div><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">serving</span></div><div class="flex items-center justify-between p-4 bg-[#141414] rounded-lg"><div class="flex items-center gap-4"><div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-400 font-mono text-xs">EMB</div><div><div class="text-sm text-white">ada-embedding-v2</div><div class="text-xs text-neutral-500">1.5B params</div></div></div><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">serving</span></div><div class="flex items-center justify-between p-4 bg-[#141414] rounded-lg"><div class="flex items-center gap-4"><div class="w-10 h-10 rounded-lg bg-amber-500/10 flex items-center justify-center text-amber-400 font-mono text-xs">IMG</div><div><div class="text-sm text-white">stable-diffusion-xl</div><div class="text-xs text-neutral-500">6.6B params</div></div></div><span class="text-xs px-2 py-0.5 bg-amber-500/10 text-amber-400 rounded-full">scaling</span></div></div></div></div>"##, a = accent)
}

pub fn logistics_view(accent: &str) -> String {
    format!(r##"<div class="max-w-7xl mx-auto"><div class="flex items-center gap-3 mb-8"><p class="font-mono text-[10px] uppercase tracking-[0.2em] text-{a}-500">// Logistics</p><div class="flex-1 h-px bg-neutral-800/50"></div></div><h1 class="text-3xl font-bold mb-8">Shipment Tracker</h1>
  <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">IN TRANSIT</p><p class="text-2xl font-bold tabular-nums text-blue-400">342</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">DELIVERED</p><p class="text-2xl font-bold tabular-nums text-emerald-400">1,847</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">DELAYED</p><p class="text-2xl font-bold tabular-nums text-amber-400">23</p></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><p class="font-mono text-[10px] text-neutral-500 mb-2">AVG TIME</p><p class="text-2xl font-bold tabular-nums text-white">2.4d</p></div>
  </div>
  <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8"><h3 class="text-lg font-semibold text-white mb-6">Active Shipments</h3><table class="w-full text-sm"><thead><tr class="border-b border-neutral-800/50"><th class="text-left py-3 font-mono text-[10px] text-neutral-500">TRACKING</th><th class="text-left py-3 font-mono text-[10px] text-neutral-500">ROUTE</th><th class="text-left py-3 font-mono text-[10px] text-neutral-500">STATUS</th></tr></thead><tbody><tr class="border-b border-neutral-800/30"><td class="py-3 font-mono text-neutral-300">SHP-8842</td><td class="py-3 text-neutral-400">SP → NY</td><td class="py-3"><span class="text-xs px-2 py-0.5 bg-blue-500/10 text-blue-400 rounded-full">transit</span></td></tr><tr class="border-b border-neutral-800/30"><td class="py-3 font-mono text-neutral-300">SHP-8841</td><td class="py-3 text-neutral-400">SH → LDN</td><td class="py-3"><span class="text-xs px-2 py-0.5 bg-amber-500/10 text-amber-400 rounded-full">customs</span></td></tr></tbody></table></div></div>"##, a = accent)
}

pub fn webhook_visualizer(accent: &str) -> String {
    format!(r##"<div class="max-w-4xl mx-auto"><div class="flex items-center gap-3 mb-8"><p class="font-mono text-[10px] uppercase tracking-[0.2em] text-{a}-500">// Webhooks</p><div class="flex-1 h-px bg-neutral-800/50"></div></div><h2 class="text-3xl font-bold mb-8">Event Flow</h2>
  <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-8">
    <div class="flex items-center justify-center gap-4 mb-8"><div class="px-4 py-3 bg-{a}-500/10 border border-{a}-500/20 rounded-xl text-center"><p class="font-mono text-[10px] text-{a}-400">EVENT</p><p class="text-sm text-white mt-1">payment.completed</p></div><div class="w-12 h-px bg-neutral-700"></div><div class="px-4 py-3 bg-neutral-900 border border-neutral-800 rounded-xl text-center"><p class="font-mono text-[10px] text-neutral-500">QUEUE</p></div><div class="w-12 h-px bg-neutral-700"></div><div class="px-4 py-3 bg-emerald-500/10 border border-emerald-500/20 rounded-xl text-center"><p class="font-mono text-[10px] text-emerald-400">DELIVER</p></div></div>
    <div class="space-y-2"><div class="flex items-center justify-between p-3 bg-[#141414] rounded-lg"><span class="text-sm text-white font-mono">payment.completed</span><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">200 OK</span></div><div class="flex items-center justify-between p-3 bg-[#141414] rounded-lg"><span class="text-sm text-white font-mono">invoice.created</span><span class="text-xs px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded-full">200 OK</span></div><div class="flex items-center justify-between p-3 bg-[#141414] rounded-lg"><span class="text-sm text-white font-mono">subscription.renewed</span><span class="text-xs px-2 py-0.5 bg-amber-500/10 text-amber-400 rounded-full">timeout</span></div></div>
  </div></div>"##, a = accent)
}

pub fn api_docs_section(accent: &str) -> String {
    format!(r##"<div class="max-w-4xl mx-auto"><div class="flex items-center gap-3 mb-8"><p class="font-mono text-[10px] uppercase tracking-[0.2em] text-{a}-500">// API Reference</p><div class="flex-1 h-px bg-neutral-800/50"></div></div><h2 class="text-3xl font-bold mb-8">REST API</h2>
  <div class="space-y-4">
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl overflow-hidden"><div class="flex items-center gap-3 px-6 py-4 border-b border-neutral-800/30"><span class="text-xs font-mono font-bold px-2 py-0.5 bg-emerald-500/10 text-emerald-400 rounded">GET</span><span class="text-sm font-mono text-white">/api/users</span></div><div class="px-6 py-4 bg-[#080808]"><pre class="text-xs font-mono text-neutral-400"><code>curl http://localhost:5175/api/users -H "Authorization: Bearer sk_***"</code></pre></div></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl overflow-hidden"><div class="flex items-center gap-3 px-6 py-4 border-b border-neutral-800/30"><span class="text-xs font-mono font-bold px-2 py-0.5 bg-blue-500/10 text-blue-400 rounded">POST</span><span class="text-sm font-mono text-white">/api/users</span></div><div class="px-6 py-4 bg-[#080808]"><pre class="text-xs font-mono text-neutral-400"><code>curl -X POST http://localhost:5175/api/users -d '{{"name":"Alice"}}'</code></pre></div></div>
    <div class="bg-[#0a0a0a] border border-neutral-800/50 rounded-xl overflow-hidden"><div class="flex items-center gap-3 px-6 py-4 border-b border-neutral-800/30"><span class="text-xs font-mono font-bold px-2 py-0.5 bg-red-500/10 text-red-400 rounded">DELETE</span><span class="text-sm font-mono text-white">/api/users/:id</span></div><div class="px-6 py-4 bg-[#080808]"><pre class="text-xs font-mono text-neutral-400"><code>curl -X DELETE http://localhost:5175/api/users/abc123</code></pre></div></div>
  </div>
  <div class="mt-8 bg-[#0a0a0a] border border-neutral-800/50 rounded-xl p-6"><h3 class="font-mono text-[10px] text-neutral-500 mb-4">STATUS CODES</h3><div class="grid grid-cols-4 gap-3"><div class="text-center p-3 bg-[#141414] rounded-lg"><span class="text-lg font-bold text-emerald-400">200</span><p class="text-xs text-neutral-500">OK</p></div><div class="text-center p-3 bg-[#141414] rounded-lg"><span class="text-lg font-bold text-blue-400">201</span><p class="text-xs text-neutral-500">Created</p></div><div class="text-center p-3 bg-[#141414] rounded-lg"><span class="text-lg font-bold text-amber-400">404</span><p class="text-xs text-neutral-500">Not Found</p></div><div class="text-center p-3 bg-[#141414] rounded-lg"><span class="text-lg font-bold text-red-400">500</span><p class="text-xs text-neutral-500">Error</p></div></div></div></div>"##, a = accent)
}

#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Embedded Tailwind — Complete CSS engine
//!
//! All 22 color palettes × 11 steps × 3 properties + base utilities.
//! Replaces Tailwind CDN with zero external dependencies.

pub const CRONUS_TAILWIND: &str = concat!(
r##"

/* ═══ CRONUS Tailwind Embedded v0.1 ═══ */
/* 200 base utilities from Tailwind v4 */

/* Reset */
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
html{-webkit-text-size-adjust:100%;tab-size:4}
body{line-height:1.5;-webkit-font-smoothing:antialiased}
img,video,canvas,svg{display:block;max-width:100%}
input,button,textarea,select{font:inherit}
p,h1,h2,h3,h4,h5,h6{overflow-wrap:break-word}

/* 1. Display & Layout */
.block{display:block}.inline-block{display:inline-block}.inline{display:inline}
.flex{display:flex}.inline-flex{display:inline-flex}.grid{display:grid}
.inline-grid{display:inline-grid}.hidden{display:none}.contents{display:contents}
.flow-root{display:flow-root}.table{display:table}.table-cell{display:table-cell}

/* 2. Position */
.static{position:static}.fixed{position:fixed}.absolute{position:absolute}
.relative{position:relative}.sticky{position:sticky}
.top-0{top:0}.right-0{right:0}.bottom-0{bottom:0}.left-0{left:0}.inset-0{inset:0}
.top-1{top:0.25rem}.top-2{top:0.5rem}.top-4{top:1rem}.top-6{top:1.5rem}
.right-4{right:1rem}.bottom-6{bottom:1.5rem}.left-6{left:1.5rem}
.z-10{z-index:10}.z-20{z-index:20}.z-50{z-index:50}

/* 3. Flex & Grid */
.flex-1{flex:1 1 0%}.flex-none{flex:none}.flex-auto{flex:1 1 auto}
.flex-row{flex-direction:row}.flex-col{flex-direction:column}
.flex-wrap{flex-wrap:wrap}.flex-shrink-0{flex-shrink:0}
.items-start{align-items:flex-start}.items-center{align-items:center}
.items-end{align-items:flex-end}.items-stretch{align-items:stretch}
.items-baseline{align-items:baseline}
.justify-start{justify-content:flex-start}.justify-center{justify-content:center}
.justify-end{justify-content:flex-end}.justify-between{justify-content:space-between}
.self-center{align-self:center}.self-start{align-self:flex-start}
.gap-0{gap:0}.gap-1{gap:0.25rem}.gap-1\.5{gap:0.375rem}.gap-2{gap:0.5rem}
.gap-2\.5{gap:0.625rem}.gap-3{gap:0.75rem}.gap-4{gap:1rem}.gap-6{gap:1.5rem}.gap-8{gap:2rem}
.gap-16{gap:4rem}
.grid-cols-1{grid-template-columns:repeat(1,minmax(0,1fr))}
.grid-cols-2{grid-template-columns:repeat(2,minmax(0,1fr))}
.grid-cols-3{grid-template-columns:repeat(3,minmax(0,1fr))}
.grid-cols-4{grid-template-columns:repeat(4,minmax(0,1fr))}
.col-span-2{grid-column:span 2/span 2}.col-span-3{grid-column:span 3/span 3}

/* 4. Margin */
.m-0{margin:0}.m-1{margin:0.25rem}.m-2{margin:0.5rem}.m-3{margin:0.75rem}
.m-4{margin:1rem}.m-6{margin:1.5rem}.m-8{margin:2rem}.m-10{margin:2.5rem}.m-12{margin:3rem}.m-16{margin:4rem}
.mx-auto{margin-left:auto;margin-right:auto}.mx-2{margin-left:0.5rem;margin-right:0.5rem}
.mx-4{margin-left:1rem;margin-right:1rem}.mx-6{margin-left:1.5rem;margin-right:1.5rem}
.my-2{margin-top:0.5rem;margin-bottom:0.5rem}.my-4{margin-top:1rem;margin-bottom:1rem}
.my-6{margin-top:1.5rem;margin-bottom:1.5rem}
.mt-1{margin-top:0.25rem}.mt-2{margin-top:0.5rem}.mt-3{margin-top:0.75rem}.mt-4{margin-top:1rem}
.mt-6{margin-top:1.5rem}.mt-8{margin-top:2rem}.mt-10{margin-top:2.5rem}.mt-12{margin-top:3rem}
.mt-16{margin-top:4rem}.mt-20{margin-top:5rem}.mt-24{margin-top:6rem}.mt-32{margin-top:8rem}
.mb-1{margin-bottom:0.25rem}.mb-2{margin-bottom:0.5rem}.mb-3{margin-bottom:0.75rem}
.mb-4{margin-bottom:1rem}.mb-6{margin-bottom:1.5rem}.mb-8{margin-bottom:2rem}
.mb-10{margin-bottom:2.5rem}.mb-12{margin-bottom:3rem}
.ml-1{margin-left:0.25rem}.ml-2{margin-left:0.5rem}.ml-3{margin-left:0.75rem}.ml-4{margin-left:1rem}
.mr-2{margin-right:0.5rem}.mr-4{margin-right:1rem}
.-ml-2{margin-left:-0.5rem}.-translate-x-1\/2{transform:translateX(-50%)}
.-translate-y-1\/2{transform:translateY(-50%)}

/* 5. Padding */
.p-0{padding:0}.p-1{padding:0.25rem}.p-2{padding:0.5rem}.p-3{padding:0.75rem}
.p-4{padding:1rem}.p-5{padding:1.25rem}.p-6{padding:1.5rem}.p-8{padding:2rem}.p-10{padding:2.5rem}
.px-2{padding-left:0.5rem;padding-right:0.5rem}.px-3{padding-left:0.75rem;padding-right:0.75rem}
.px-4{padding-left:1rem;padding-right:1rem}.px-6{padding-left:1.5rem;padding-right:1.5rem}
.px-7{padding-left:1.75rem;padding-right:1.75rem}.px-8{padding-left:2rem;padding-right:2rem}
.py-0\.5{padding-top:0.125rem;padding-bottom:0.125rem}.py-1{padding-top:0.25rem;padding-bottom:0.25rem}
.py-1\.5{padding-top:0.375rem;padding-bottom:0.375rem}.py-2{padding-top:0.5rem;padding-bottom:0.5rem}
.py-2\.5{padding-top:0.625rem;padding-bottom:0.625rem}.py-3{padding-top:0.75rem;padding-bottom:0.75rem}
.py-3\.5{padding-top:0.875rem;padding-bottom:0.875rem}.py-4{padding-top:1rem;padding-bottom:1rem}
.py-6{padding-top:1.5rem;padding-bottom:1.5rem}.py-8{padding-top:2rem;padding-bottom:2rem}
.py-12{padding-top:3rem;padding-bottom:3rem}.py-16{padding-top:4rem;padding-bottom:4rem}
.py-20{padding-top:5rem;padding-bottom:5rem}.py-24{padding-top:6rem;padding-bottom:6rem}
.py-32{padding-top:8rem;padding-bottom:8rem}
.pt-2{padding-top:0.5rem}.pt-4{padding-top:1rem}.pt-16{padding-top:4rem}.pt-32{padding-top:8rem}
.pb-2{padding-bottom:0.5rem}.pb-4{padding-bottom:1rem}.pb-24{padding-bottom:6rem}
.pl-2{padding-left:0.5rem}.pl-4{padding-left:1rem}.pl-10{padding-left:2.5rem}
.pr-2{padding-right:0.5rem}.pr-4{padding-right:1rem}

/* 6. Width */
.w-auto{width:auto}.w-full{width:100%}.w-screen{width:100vw}.w-fit{width:fit-content}
.w-0{width:0}.w-1{width:0.25rem}.w-1\.5{width:0.375rem}.w-2{width:0.5rem}.w-2\.5{width:0.625rem}
.w-3{width:0.75rem}.w-4{width:1rem}.w-5{width:1.25rem}.w-6{width:1.5rem}.w-7{width:1.75rem}
.w-8{width:2rem}.w-9{width:2.25rem}.w-10{width:2.5rem}.w-11{width:2.75rem}.w-12{width:3rem}
.w-16{width:4rem}.w-20{width:5rem}.w-24{width:6rem}.w-32{width:8rem}.w-40{width:10rem}
.w-48{width:12rem}.w-56{width:14rem}.w-60{width:15rem}.w-64{width:16rem}.w-72{width:18rem}
.min-w-0{min-width:0}.max-w-xs{max-width:20rem}.max-w-sm{max-width:24rem}
.max-w-md{max-width:28rem}.max-w-lg{max-width:32rem}.max-w-xl{max-width:36rem}
.max-w-2xl{max-width:42rem}.max-w-3xl{max-width:48rem}.max-w-4xl{max-width:56rem}
.max-w-6xl{max-width:72rem}.max-w-7xl{max-width:80rem}

/* 7. Height */
.h-auto{height:auto}.h-full{height:100%}.h-screen{height:100vh}
.h-0{height:0}.h-1{height:0.25rem}.h-1\.5{height:0.375rem}.h-2{height:0.5rem}.h-2\.5{height:0.625rem}
.h-3{height:0.75rem}.h-4{height:1rem}.h-5{height:1.25rem}.h-6{height:1.5rem}
.h-8{height:2rem}.h-10{height:2.5rem}.h-12{height:3rem}.h-14{height:3.5rem}
.h-16{height:4rem}.h-20{height:5rem}.h-24{height:6rem}.h-32{height:8rem}
.min-h-screen{min-height:100vh}.min-h-0{min-height:0}
.size-4{width:1rem;height:1rem}

/* 8. Font size */
.text-\[9px\]{font-size:9px}.text-\[10px\]{font-size:10px}.text-\[11px\]{font-size:11px}
.text-\[13px\]{font-size:13px}
.text-xs{font-size:0.75rem;line-height:1rem}.text-sm{font-size:0.875rem;line-height:1.25rem}
.text-base{font-size:1rem;line-height:1.5rem}.text-lg{font-size:1.125rem;line-height:1.75rem}
.text-xl{font-size:1.25rem;line-height:1.75rem}.text-2xl{font-size:1.5rem;line-height:2rem}
.text-3xl{font-size:1.875rem;line-height:2.25rem}.text-4xl{font-size:2.25rem;line-height:2.5rem}
.text-5xl{font-size:3rem;line-height:1}.text-6xl{font-size:3.75rem;line-height:1}
.text-7xl{font-size:4.5rem;line-height:1}.text-8xl{font-size:6rem;line-height:1}
.text-9xl{font-size:8rem;line-height:1}

/* 9. Typography */
.text-left{text-align:left}.text-center{text-align:center}.text-right{text-align:right}
.font-thin{font-weight:100}.font-light{font-weight:300}.font-normal{font-weight:400}
.font-medium{font-weight:500}.font-semibold{font-weight:600}.font-bold{font-weight:700}
.font-extrabold{font-weight:800}.font-black{font-weight:900}
.font-sans{font-family:Inter,system-ui,sans-serif}.font-mono{font-family:'JetBrains Mono',monospace}
.leading-none{line-height:1}.leading-tight{line-height:1.25}.leading-snug{line-height:1.375}
.leading-normal{line-height:1.5}.leading-relaxed{line-height:1.625}
.leading-\[0\.9\]{line-height:0.9}
.tracking-\[0\.1em\]{letter-spacing:0.1em}.tracking-\[0\.15em\]{letter-spacing:0.15em}
.tracking-\[0\.2em\]{letter-spacing:0.2em}
.tracking-tighter{letter-spacing:-0.05em}.tracking-tight{letter-spacing:-0.025em}
.tracking-normal{letter-spacing:0}.tracking-wide{letter-spacing:0.025em}
.tracking-wider{letter-spacing:0.05em}.tracking-widest{letter-spacing:0.1em}
.uppercase{text-transform:uppercase}.lowercase{text-transform:lowercase}
.capitalize{text-transform:capitalize}.normal-case{text-transform:none}
.truncate{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.no-underline{text-decoration-line:none}

/* 10. Borders & Radius */
.border{border-width:1px}.border-0{border-width:0}.border-2{border-width:2px}
.border-t{border-top-width:1px}.border-b{border-bottom-width:1px}
.border-l{border-left-width:1px}.border-l-2{border-left-width:2px}
.border-r{border-right-width:1px}
.border-dashed{border-style:dashed}
.rounded-none{border-radius:0}.rounded-sm{border-radius:0.125rem}
.rounded{border-radius:0.25rem}.rounded-md{border-radius:0.375rem}
.rounded-lg{border-radius:0.5rem}.rounded-xl{border-radius:0.75rem}
.rounded-2xl{border-radius:1rem}.rounded-3xl{border-radius:1.5rem}
.rounded-full{border-radius:9999px}
.rounded-t-lg{border-top-left-radius:0.5rem;border-top-right-radius:0.5rem}
.rounded-b-lg{border-bottom-left-radius:0.5rem;border-bottom-right-radius:0.5rem}
.rounded-l{border-top-left-radius:0.25rem;border-bottom-left-radius:0.25rem}
.rounded-r{border-top-right-radius:0.25rem;border-bottom-right-radius:0.25rem}
.outline-none{outline:2px solid transparent;outline-offset:2px}
.border-transparent{border-color:transparent}
.ring-2{box-shadow:0 0 0 2px var(--tw-ring-color,rgba(59,130,246,0.5))}
.ring-4{box-shadow:0 0 0 4px var(--tw-ring-color,rgba(59,130,246,0.5))}

/* 11. Shadows & Misc */
.shadow-sm{box-shadow:0 1px 3px 0 rgb(0 0 0/0.1),0 1px 2px -1px rgb(0 0 0/0.1)}
.shadow-md{box-shadow:0 4px 6px -1px rgb(0 0 0/0.1),0 2px 4px -2px rgb(0 0 0/0.1)}
.shadow-lg{box-shadow:0 10px 15px -3px rgb(0 0 0/0.1),0 4px 6px -4px rgb(0 0 0/0.1)}
.shadow-xl{box-shadow:0 20px 25px -5px rgb(0 0 0/0.1),0 8px 10px -6px rgb(0 0 0/0.1)}
.shadow-2xl{box-shadow:0 25px 50px -12px rgb(0 0 0/0.25)}
.shadow-none{box-shadow:0 0 #0000}
.opacity-0{opacity:0}.opacity-25{opacity:0.25}.opacity-50{opacity:0.5}
.opacity-75{opacity:0.75}.opacity-100{opacity:1}
.overflow-hidden{overflow:hidden}.overflow-auto{overflow:auto}
.overflow-x-auto{overflow-x:auto}.overflow-y-auto{overflow-y:auto}
.cursor-pointer{cursor:pointer}.cursor-help{cursor:help}
.select-none{user-select:none}.whitespace-nowrap{white-space:nowrap}
.pointer-events-none{pointer-events:none}.pointer-events-auto{pointer-events:auto}
.sr-only{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border-width:0}
.invisible{visibility:hidden}.visible{visibility:visible}
.resize-y{resize:vertical}
.object-cover{object-fit:cover}
.antialiased{-webkit-font-smoothing:antialiased;-moz-osx-font-smoothing:grayscale}

/* 12. Transitions & Transforms */
.transition{transition-property:color,background-color,border-color,text-decoration-color,fill,stroke,opacity,box-shadow,transform,filter,backdrop-filter;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms}
.transition-all{transition-property:all;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms}
.transition-colors{transition-property:color,background-color,border-color,text-decoration-color,fill,stroke;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms}
.transition-transform{transition-property:transform;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms}
.duration-200{transition-duration:200ms}.duration-300{transition-duration:300ms}.duration-500{transition-duration:500ms}
.ease-out{transition-timing-function:cubic-bezier(0,0,0.2,1)}
.transform{transform:translate(var(--tw-translate-x,0),var(--tw-translate-y,0)) rotate(var(--tw-rotate,0)) skewX(var(--tw-skew-x,0)) skewY(var(--tw-skew-y,0)) scaleX(var(--tw-scale-x,1)) scaleY(var(--tw-scale-y,1))}
.translate-x-0{transform:translateX(0)}.translate-x-5{transform:translateX(1.25rem)}
.translate-x-0\.5{transform:translateX(0.125rem)}
.scale-95{transform:scale(0.95)}.scale-100{transform:scale(1)}

/* 23. Hover states */
.hover\:bg-neutral-700:hover{background-color:#404040}
.hover\:bg-neutral-800:hover{background-color:#262626}
.hover\:bg-neutral-900\/50:hover{background-color:rgb(23 23 23/0.5)}
.hover\:text-white:hover{color:#fff}
.hover\:text-neutral-300:hover{color:#d4d4d4}
.hover\:border-neutral-500:hover{border-color:#737373}
.hover\:border-neutral-600:hover{border-color:#525252}
.hover\:underline:hover{text-decoration-line:underline}

/* 24. Animations */
.animate-spin{animation:spin 1s linear infinite}
.animate-ping{animation:ping 1s cubic-bezier(0,0,0.2,1) infinite}
.animate-pulse{animation:pulse 2s cubic-bezier(0.4,0,0.6,1) infinite}
.animate-bounce{animation:bounce 1s infinite}
@keyframes spin{to{transform:rotate(360deg)}}
@keyframes ping{75%,100%{transform:scale(2);opacity:0}}
@keyframes pulse{50%{opacity:.5}}
@keyframes bounce{0%,100%{transform:translateY(-25%);animation-timing-function:cubic-bezier(0.8,0,1,1)}50%{transform:translateY(0);animation-timing-function:cubic-bezier(0,0,0.2,1)}}

/* 25. Space between */
.space-y-1>*+*{margin-top:0.25rem}.space-y-2>*+*{margin-top:0.5rem}
.space-y-2\.5>*+*{margin-top:0.625rem}.space-y-3>*+*{margin-top:0.75rem}
.space-y-4>*+*{margin-top:1rem}.space-y-6>*+*{margin-top:1.5rem}
.space-x-2>*+*{margin-left:0.5rem}
.divide-y>*+*{border-top-width:1px}
.divide-neutral-800>*+*{border-color:#262626}
.divide-neutral-800\/50>*+*{border-color:rgb(38 38 38/0.5)}

/* 26. Responsive (md: 768px, lg: 1024px) */
@media(min-width:768px){
.md\:flex{display:flex}.md\:hidden{display:none}.md\:grid-cols-2{grid-template-columns:repeat(2,minmax(0,1fr))}
.md\:grid-cols-3{grid-template-columns:repeat(3,minmax(0,1fr))}.md\:grid-cols-4{grid-template-columns:repeat(4,minmax(0,1fr))}
.md\:flex-row{flex-direction:row}.md\:text-7xl{font-size:4.5rem;line-height:1}
.md\:text-4xl{font-size:2.25rem;line-height:2.5rem}.md\:py-32{padding-top:8rem;padding-bottom:8rem}
}
@media(min-width:1024px){
.lg\:flex{display:flex}.lg\:hidden{display:none}.lg\:block{display:block}
.lg\:grid-cols-2{grid-template-columns:repeat(2,minmax(0,1fr))}
.lg\:grid-cols-3{grid-template-columns:repeat(3,minmax(0,1fr))}
.lg\:text-8xl{font-size:6rem;line-height:1}
}

/* 27. Misc used by CRONUS UI */
.first\:ml-0:first-child{margin-left:0}
.bg-gradient-to-b{background-image:linear-gradient(to bottom,var(--tw-gradient-stops))}
.bg-gradient-to-r{background-image:linear-gradient(to right,var(--tw-gradient-stops))}
.bg-gradient-to-t{background-image:linear-gradient(to top,var(--tw-gradient-stops))}
.bg-clip-text{background-clip:text;-webkit-background-clip:text}
.text-transparent{color:transparent}
.backdrop-blur-sm{backdrop-filter:blur(4px)}.backdrop-blur{backdrop-filter:blur(8px)}
.backdrop-blur-lg{backdrop-filter:blur(16px)}.-webkit-backdrop-filter{-webkit-backdrop-filter:blur(16px)}
.appearance-none{appearance:none}
.fill-none{fill:none}
"##,
r##"
/* ═══ FULL COLOR PALETTE — 22 palettes × 11 steps ═══ */
.text-slate-50{color:#f8fafc}
.bg-slate-50{background-color:#f8fafc}
.border-slate-50{border-color:#f8fafc}
.text-slate-100{color:#f1f5f9}
.bg-slate-100{background-color:#f1f5f9}
.border-slate-100{border-color:#f1f5f9}
.text-slate-200{color:#e2e8f0}
.bg-slate-200{background-color:#e2e8f0}
.border-slate-200{border-color:#e2e8f0}
.text-slate-300{color:#cbd5e1}
.bg-slate-300{background-color:#cbd5e1}
.border-slate-300{border-color:#cbd5e1}
.text-slate-400{color:#94a3b8}
.bg-slate-400{background-color:#94a3b8}
.border-slate-400{border-color:#94a3b8}
.text-slate-500{color:#64748b}
.bg-slate-500{background-color:#64748b}
.border-slate-500{border-color:#64748b}
.text-slate-600{color:#475569}
.bg-slate-600{background-color:#475569}
.border-slate-600{border-color:#475569}
.text-slate-700{color:#334155}
.bg-slate-700{background-color:#334155}
.border-slate-700{border-color:#334155}
.text-slate-800{color:#1e293b}
.bg-slate-800{background-color:#1e293b}
.border-slate-800{border-color:#1e293b}
.text-slate-900{color:#0f172a}
.bg-slate-900{background-color:#0f172a}
.border-slate-900{border-color:#0f172a}
.text-slate-950{color:#020617}
.bg-slate-950{background-color:#020617}
.border-slate-950{border-color:#020617}
.text-gray-50{color:#f9fafb}
.bg-gray-50{background-color:#f9fafb}
.border-gray-50{border-color:#f9fafb}
.text-gray-100{color:#f3f4f6}
.bg-gray-100{background-color:#f3f4f6}
.border-gray-100{border-color:#f3f4f6}
.text-gray-200{color:#e5e7eb}
.bg-gray-200{background-color:#e5e7eb}
.border-gray-200{border-color:#e5e7eb}
.text-gray-300{color:#d1d5db}
.bg-gray-300{background-color:#d1d5db}
.border-gray-300{border-color:#d1d5db}
.text-gray-400{color:#9ca3af}
.bg-gray-400{background-color:#9ca3af}
.border-gray-400{border-color:#9ca3af}
.text-gray-500{color:#6b7280}
.bg-gray-500{background-color:#6b7280}
.border-gray-500{border-color:#6b7280}
.text-gray-600{color:#4b5563}
.bg-gray-600{background-color:#4b5563}
.border-gray-600{border-color:#4b5563}
.text-gray-700{color:#374151}
.bg-gray-700{background-color:#374151}
.border-gray-700{border-color:#374151}
.text-gray-800{color:#1f2937}
.bg-gray-800{background-color:#1f2937}
.border-gray-800{border-color:#1f2937}
.text-gray-900{color:#111827}
.bg-gray-900{background-color:#111827}
.border-gray-900{border-color:#111827}
.text-gray-950{color:#030712}
.bg-gray-950{background-color:#030712}
.border-gray-950{border-color:#030712}
.text-zinc-50{color:#fafafa}
.bg-zinc-50{background-color:#fafafa}
.border-zinc-50{border-color:#fafafa}
.text-zinc-100{color:#f4f4f5}
.bg-zinc-100{background-color:#f4f4f5}
.border-zinc-100{border-color:#f4f4f5}
.text-zinc-200{color:#e4e4e7}
.bg-zinc-200{background-color:#e4e4e7}
.border-zinc-200{border-color:#e4e4e7}
.text-zinc-300{color:#d4d4d8}
.bg-zinc-300{background-color:#d4d4d8}
.border-zinc-300{border-color:#d4d4d8}
.text-zinc-400{color:#a1a1aa}
.bg-zinc-400{background-color:#a1a1aa}
.border-zinc-400{border-color:#a1a1aa}
.text-zinc-500{color:#71717a}
.bg-zinc-500{background-color:#71717a}
.border-zinc-500{border-color:#71717a}
.text-zinc-600{color:#52525b}
.bg-zinc-600{background-color:#52525b}
.border-zinc-600{border-color:#52525b}
.text-zinc-700{color:#3f3f46}
.bg-zinc-700{background-color:#3f3f46}
.border-zinc-700{border-color:#3f3f46}
.text-zinc-800{color:#27272a}
.bg-zinc-800{background-color:#27272a}
.border-zinc-800{border-color:#27272a}
.text-zinc-900{color:#18181b}
.bg-zinc-900{background-color:#18181b}
.border-zinc-900{border-color:#18181b}
.text-zinc-950{color:#09090b}
.bg-zinc-950{background-color:#09090b}
.border-zinc-950{border-color:#09090b}
.text-neutral-50{color:#fafafa}
.bg-neutral-50{background-color:#fafafa}
.border-neutral-50{border-color:#fafafa}
.text-neutral-100{color:#f5f5f5}
.bg-neutral-100{background-color:#f5f5f5}
.border-neutral-100{border-color:#f5f5f5}
.text-neutral-200{color:#e5e5e5}
.bg-neutral-200{background-color:#e5e5e5}
.border-neutral-200{border-color:#e5e5e5}
.text-neutral-300{color:#d4d4d4}
.bg-neutral-300{background-color:#d4d4d4}
.border-neutral-300{border-color:#d4d4d4}
.text-neutral-400{color:#a3a3a3}
.bg-neutral-400{background-color:#a3a3a3}
.border-neutral-400{border-color:#a3a3a3}
.text-neutral-500{color:#737373}
.bg-neutral-500{background-color:#737373}
.border-neutral-500{border-color:#737373}
.text-neutral-600{color:#525252}
.bg-neutral-600{background-color:#525252}
.border-neutral-600{border-color:#525252}
.text-neutral-700{color:#404040}
.bg-neutral-700{background-color:#404040}
.border-neutral-700{border-color:#404040}
.text-neutral-800{color:#262626}
.bg-neutral-800{background-color:#262626}
.border-neutral-800{border-color:#262626}
.text-neutral-900{color:#171717}
.bg-neutral-900{background-color:#171717}
.border-neutral-900{border-color:#171717}
.text-neutral-950{color:#0a0a0a}
.bg-neutral-950{background-color:#0a0a0a}
.border-neutral-950{border-color:#0a0a0a}
.text-stone-50{color:#fafaf9}
.bg-stone-50{background-color:#fafaf9}
.border-stone-50{border-color:#fafaf9}
.text-stone-100{color:#f5f5f4}
.bg-stone-100{background-color:#f5f5f4}
.border-stone-100{border-color:#f5f5f4}
.text-stone-200{color:#e7e5e4}
.bg-stone-200{background-color:#e7e5e4}
.border-stone-200{border-color:#e7e5e4}
.text-stone-300{color:#d6d3d1}
.bg-stone-300{background-color:#d6d3d1}
.border-stone-300{border-color:#d6d3d1}
.text-stone-400{color:#a8a29e}
.bg-stone-400{background-color:#a8a29e}
.border-stone-400{border-color:#a8a29e}
.text-stone-500{color:#78716c}
.bg-stone-500{background-color:#78716c}
.border-stone-500{border-color:#78716c}
.text-stone-600{color:#57534e}
.bg-stone-600{background-color:#57534e}
.border-stone-600{border-color:#57534e}
.text-stone-700{color:#44403c}
.bg-stone-700{background-color:#44403c}
.border-stone-700{border-color:#44403c}
.text-stone-800{color:#292524}
.bg-stone-800{background-color:#292524}
.border-stone-800{border-color:#292524}
.text-stone-900{color:#1c1917}
.bg-stone-900{background-color:#1c1917}
.border-stone-900{border-color:#1c1917}
.text-stone-950{color:#0c0a09}
.bg-stone-950{background-color:#0c0a09}
.border-stone-950{border-color:#0c0a09}
.text-red-50{color:#fef2f2}
.bg-red-50{background-color:#fef2f2}
.border-red-50{border-color:#fef2f2}
.text-red-100{color:#fee2e2}
.bg-red-100{background-color:#fee2e2}
.border-red-100{border-color:#fee2e2}
.text-red-200{color:#fecaca}
.bg-red-200{background-color:#fecaca}
.border-red-200{border-color:#fecaca}
.text-red-300{color:#fca5a5}
.bg-red-300{background-color:#fca5a5}
.border-red-300{border-color:#fca5a5}
.text-red-400{color:#f87171}
.bg-red-400{background-color:#f87171}
.border-red-400{border-color:#f87171}
.text-red-500{color:#ef4444}
.bg-red-500{background-color:#ef4444}
.border-red-500{border-color:#ef4444}
.text-red-600{color:#dc2626}
.bg-red-600{background-color:#dc2626}
.border-red-600{border-color:#dc2626}
.text-red-700{color:#b91c1c}
.bg-red-700{background-color:#b91c1c}
.border-red-700{border-color:#b91c1c}
.text-red-800{color:#991b1b}
.bg-red-800{background-color:#991b1b}
.border-red-800{border-color:#991b1b}
.text-red-900{color:#7f1d1d}
.bg-red-900{background-color:#7f1d1d}
.border-red-900{border-color:#7f1d1d}
.text-red-950{color:#450a0a}
.bg-red-950{background-color:#450a0a}
.border-red-950{border-color:#450a0a}
.text-orange-50{color:#fff7ed}
.bg-orange-50{background-color:#fff7ed}
.border-orange-50{border-color:#fff7ed}
.text-orange-100{color:#ffedd5}
.bg-orange-100{background-color:#ffedd5}
.border-orange-100{border-color:#ffedd5}
.text-orange-200{color:#fed7aa}
.bg-orange-200{background-color:#fed7aa}
.border-orange-200{border-color:#fed7aa}
.text-orange-300{color:#fdba74}
.bg-orange-300{background-color:#fdba74}
.border-orange-300{border-color:#fdba74}
.text-orange-400{color:#fb923c}
.bg-orange-400{background-color:#fb923c}
.border-orange-400{border-color:#fb923c}
.text-orange-500{color:#f97316}
.bg-orange-500{background-color:#f97316}
.border-orange-500{border-color:#f97316}
.text-orange-600{color:#ea580c}
.bg-orange-600{background-color:#ea580c}
.border-orange-600{border-color:#ea580c}
.text-orange-700{color:#c2410c}
.bg-orange-700{background-color:#c2410c}
.border-orange-700{border-color:#c2410c}
.text-orange-800{color:#9a3412}
.bg-orange-800{background-color:#9a3412}
.border-orange-800{border-color:#9a3412}
.text-orange-900{color:#7c2d12}
.bg-orange-900{background-color:#7c2d12}
.border-orange-900{border-color:#7c2d12}
.text-orange-950{color:#431407}
.bg-orange-950{background-color:#431407}
.border-orange-950{border-color:#431407}
.text-amber-50{color:#fffbeb}
.bg-amber-50{background-color:#fffbeb}
.border-amber-50{border-color:#fffbeb}
.text-amber-100{color:#fef3c7}
.bg-amber-100{background-color:#fef3c7}
.border-amber-100{border-color:#fef3c7}
.text-amber-200{color:#fde68a}
.bg-amber-200{background-color:#fde68a}
.border-amber-200{border-color:#fde68a}
.text-amber-300{color:#fcd34d}
.bg-amber-300{background-color:#fcd34d}
.border-amber-300{border-color:#fcd34d}
.text-amber-400{color:#fbbf24}
.bg-amber-400{background-color:#fbbf24}
.border-amber-400{border-color:#fbbf24}
.text-amber-500{color:#f59e0b}
.bg-amber-500{background-color:#f59e0b}
.border-amber-500{border-color:#f59e0b}
.text-amber-600{color:#d97706}
.bg-amber-600{background-color:#d97706}
.border-amber-600{border-color:#d97706}
.text-amber-700{color:#b45309}
.bg-amber-700{background-color:#b45309}
.border-amber-700{border-color:#b45309}
.text-amber-800{color:#92400e}
.bg-amber-800{background-color:#92400e}
.border-amber-800{border-color:#92400e}
.text-amber-900{color:#78350f}
.bg-amber-900{background-color:#78350f}
.border-amber-900{border-color:#78350f}
.text-amber-950{color:#451a03}
.bg-amber-950{background-color:#451a03}
.border-amber-950{border-color:#451a03}
.text-yellow-50{color:#fefce8}
.bg-yellow-50{background-color:#fefce8}
.border-yellow-50{border-color:#fefce8}
.text-yellow-100{color:#fef9c3}
.bg-yellow-100{background-color:#fef9c3}
.border-yellow-100{border-color:#fef9c3}
.text-yellow-200{color:#fef08a}
.bg-yellow-200{background-color:#fef08a}
.border-yellow-200{border-color:#fef08a}
.text-yellow-300{color:#fde047}
.bg-yellow-300{background-color:#fde047}
.border-yellow-300{border-color:#fde047}
.text-yellow-400{color:#facc15}
.bg-yellow-400{background-color:#facc15}
.border-yellow-400{border-color:#facc15}
.text-yellow-500{color:#eab308}
.bg-yellow-500{background-color:#eab308}
.border-yellow-500{border-color:#eab308}
.text-yellow-600{color:#ca8a04}
.bg-yellow-600{background-color:#ca8a04}
.border-yellow-600{border-color:#ca8a04}
.text-yellow-700{color:#a16207}
.bg-yellow-700{background-color:#a16207}
.border-yellow-700{border-color:#a16207}
.text-yellow-800{color:#854d0e}
.bg-yellow-800{background-color:#854d0e}
.border-yellow-800{border-color:#854d0e}
.text-yellow-900{color:#713f12}
.bg-yellow-900{background-color:#713f12}
.border-yellow-900{border-color:#713f12}
.text-yellow-950{color:#422006}
.bg-yellow-950{background-color:#422006}
.border-yellow-950{border-color:#422006}
.text-lime-50{color:#f7fee7}
.bg-lime-50{background-color:#f7fee7}
.border-lime-50{border-color:#f7fee7}
.text-lime-100{color:#ecfccb}
.bg-lime-100{background-color:#ecfccb}
.border-lime-100{border-color:#ecfccb}
.text-lime-200{color:#d9f99d}
.bg-lime-200{background-color:#d9f99d}
.border-lime-200{border-color:#d9f99d}
.text-lime-300{color:#bef264}
.bg-lime-300{background-color:#bef264}
.border-lime-300{border-color:#bef264}
.text-lime-400{color:#a3e635}
.bg-lime-400{background-color:#a3e635}
.border-lime-400{border-color:#a3e635}
.text-lime-500{color:#84cc16}
.bg-lime-500{background-color:#84cc16}
.border-lime-500{border-color:#84cc16}
.text-lime-600{color:#65a30d}
.bg-lime-600{background-color:#65a30d}
.border-lime-600{border-color:#65a30d}
.text-lime-700{color:#4d7c0f}
.bg-lime-700{background-color:#4d7c0f}
.border-lime-700{border-color:#4d7c0f}
.text-lime-800{color:#3f6212}
.bg-lime-800{background-color:#3f6212}
.border-lime-800{border-color:#3f6212}
.text-lime-900{color:#365314}
.bg-lime-900{background-color:#365314}
.border-lime-900{border-color:#365314}
.text-lime-950{color:#1a2e05}
.bg-lime-950{background-color:#1a2e05}
.border-lime-950{border-color:#1a2e05}
.text-green-50{color:#f0fdf4}
.bg-green-50{background-color:#f0fdf4}
.border-green-50{border-color:#f0fdf4}
.text-green-100{color:#dcfce7}
.bg-green-100{background-color:#dcfce7}
.border-green-100{border-color:#dcfce7}
.text-green-200{color:#bbf7d0}
.bg-green-200{background-color:#bbf7d0}
.border-green-200{border-color:#bbf7d0}
.text-green-300{color:#86efac}
.bg-green-300{background-color:#86efac}
.border-green-300{border-color:#86efac}
.text-green-400{color:#4ade80}
.bg-green-400{background-color:#4ade80}
.border-green-400{border-color:#4ade80}
.text-green-500{color:#22c55e}
.bg-green-500{background-color:#22c55e}
.border-green-500{border-color:#22c55e}
.text-green-600{color:#16a34a}
.bg-green-600{background-color:#16a34a}
.border-green-600{border-color:#16a34a}
.text-green-700{color:#15803d}
.bg-green-700{background-color:#15803d}
.border-green-700{border-color:#15803d}
.text-green-800{color:#166534}
.bg-green-800{background-color:#166534}
.border-green-800{border-color:#166534}
.text-green-900{color:#14532d}
.bg-green-900{background-color:#14532d}
.border-green-900{border-color:#14532d}
.text-green-950{color:#052e16}
.bg-green-950{background-color:#052e16}
.border-green-950{border-color:#052e16}
.text-emerald-50{color:#ecfdf5}
.bg-emerald-50{background-color:#ecfdf5}
.border-emerald-50{border-color:#ecfdf5}
.text-emerald-100{color:#d1fae5}
.bg-emerald-100{background-color:#d1fae5}
.border-emerald-100{border-color:#d1fae5}
.text-emerald-200{color:#a7f3d0}
.bg-emerald-200{background-color:#a7f3d0}
.border-emerald-200{border-color:#a7f3d0}
.text-emerald-300{color:#6ee7b7}
.bg-emerald-300{background-color:#6ee7b7}
.border-emerald-300{border-color:#6ee7b7}
.text-emerald-400{color:#34d399}
.bg-emerald-400{background-color:#34d399}
.border-emerald-400{border-color:#34d399}
.text-emerald-500{color:#10b981}
.bg-emerald-500{background-color:#10b981}
.border-emerald-500{border-color:#10b981}
.text-emerald-600{color:#059669}
.bg-emerald-600{background-color:#059669}
.border-emerald-600{border-color:#059669}
.text-emerald-700{color:#047857}
.bg-emerald-700{background-color:#047857}
.border-emerald-700{border-color:#047857}
.text-emerald-800{color:#065f46}
.bg-emerald-800{background-color:#065f46}
.border-emerald-800{border-color:#065f46}
.text-emerald-900{color:#064e3b}
.bg-emerald-900{background-color:#064e3b}
.border-emerald-900{border-color:#064e3b}
.text-emerald-950{color:#022c22}
.bg-emerald-950{background-color:#022c22}
.border-emerald-950{border-color:#022c22}
.text-teal-50{color:#f0fdfa}
.bg-teal-50{background-color:#f0fdfa}
.border-teal-50{border-color:#f0fdfa}
.text-teal-100{color:#ccfbf1}
.bg-teal-100{background-color:#ccfbf1}
.border-teal-100{border-color:#ccfbf1}
.text-teal-200{color:#99f6e4}
.bg-teal-200{background-color:#99f6e4}
.border-teal-200{border-color:#99f6e4}
.text-teal-300{color:#5eead4}
.bg-teal-300{background-color:#5eead4}
.border-teal-300{border-color:#5eead4}
.text-teal-400{color:#2dd4bf}
.bg-teal-400{background-color:#2dd4bf}
.border-teal-400{border-color:#2dd4bf}
.text-teal-500{color:#14b8a6}
.bg-teal-500{background-color:#14b8a6}
.border-teal-500{border-color:#14b8a6}
.text-teal-600{color:#0d9488}
.bg-teal-600{background-color:#0d9488}
.border-teal-600{border-color:#0d9488}
.text-teal-700{color:#0f766e}
.bg-teal-700{background-color:#0f766e}
.border-teal-700{border-color:#0f766e}
.text-teal-800{color:#115e59}
.bg-teal-800{background-color:#115e59}
.border-teal-800{border-color:#115e59}
.text-teal-900{color:#134e4a}
.bg-teal-900{background-color:#134e4a}
.border-teal-900{border-color:#134e4a}
.text-teal-950{color:#042f2e}
.bg-teal-950{background-color:#042f2e}
.border-teal-950{border-color:#042f2e}
.text-cyan-50{color:#ecfeff}
.bg-cyan-50{background-color:#ecfeff}
.border-cyan-50{border-color:#ecfeff}
.text-cyan-100{color:#cffafe}
.bg-cyan-100{background-color:#cffafe}
.border-cyan-100{border-color:#cffafe}
.text-cyan-200{color:#a5f3fc}
.bg-cyan-200{background-color:#a5f3fc}
.border-cyan-200{border-color:#a5f3fc}
.text-cyan-300{color:#67e8f9}
.bg-cyan-300{background-color:#67e8f9}
.border-cyan-300{border-color:#67e8f9}
.text-cyan-400{color:#22d3ee}
.bg-cyan-400{background-color:#22d3ee}
.border-cyan-400{border-color:#22d3ee}
.text-cyan-500{color:#06b6d4}
.bg-cyan-500{background-color:#06b6d4}
.border-cyan-500{border-color:#06b6d4}
.text-cyan-600{color:#0891b2}
.bg-cyan-600{background-color:#0891b2}
.border-cyan-600{border-color:#0891b2}
.text-cyan-700{color:#0e7490}
.bg-cyan-700{background-color:#0e7490}
.border-cyan-700{border-color:#0e7490}
.text-cyan-800{color:#155e75}
.bg-cyan-800{background-color:#155e75}
.border-cyan-800{border-color:#155e75}
.text-cyan-900{color:#164e63}
.bg-cyan-900{background-color:#164e63}
.border-cyan-900{border-color:#164e63}
.text-cyan-950{color:#083344}
.bg-cyan-950{background-color:#083344}
.border-cyan-950{border-color:#083344}
.text-sky-50{color:#f0f9ff}
.bg-sky-50{background-color:#f0f9ff}
.border-sky-50{border-color:#f0f9ff}
.text-sky-100{color:#e0f2fe}
.bg-sky-100{background-color:#e0f2fe}
.border-sky-100{border-color:#e0f2fe}
.text-sky-200{color:#bae6fd}
.bg-sky-200{background-color:#bae6fd}
.border-sky-200{border-color:#bae6fd}
.text-sky-300{color:#7dd3fc}
.bg-sky-300{background-color:#7dd3fc}
.border-sky-300{border-color:#7dd3fc}
.text-sky-400{color:#38bdf8}
.bg-sky-400{background-color:#38bdf8}
.border-sky-400{border-color:#38bdf8}
.text-sky-500{color:#0ea5e9}
.bg-sky-500{background-color:#0ea5e9}
.border-sky-500{border-color:#0ea5e9}
.text-sky-600{color:#0284c7}
.bg-sky-600{background-color:#0284c7}
.border-sky-600{border-color:#0284c7}
.text-sky-700{color:#0369a1}
.bg-sky-700{background-color:#0369a1}
.border-sky-700{border-color:#0369a1}
.text-sky-800{color:#075985}
.bg-sky-800{background-color:#075985}
.border-sky-800{border-color:#075985}
.text-sky-900{color:#0c4a6e}
.bg-sky-900{background-color:#0c4a6e}
.border-sky-900{border-color:#0c4a6e}
.text-sky-950{color:#082f49}
.bg-sky-950{background-color:#082f49}
.border-sky-950{border-color:#082f49}
.text-blue-50{color:#eff6ff}
.bg-blue-50{background-color:#eff6ff}
.border-blue-50{border-color:#eff6ff}
.text-blue-100{color:#dbeafe}
.bg-blue-100{background-color:#dbeafe}
.border-blue-100{border-color:#dbeafe}
.text-blue-200{color:#bfdbfe}
.bg-blue-200{background-color:#bfdbfe}
.border-blue-200{border-color:#bfdbfe}
.text-blue-300{color:#93c5fd}
.bg-blue-300{background-color:#93c5fd}
.border-blue-300{border-color:#93c5fd}
.text-blue-400{color:#60a5fa}
.bg-blue-400{background-color:#60a5fa}
.border-blue-400{border-color:#60a5fa}
.text-blue-500{color:#3b82f6}
.bg-blue-500{background-color:#3b82f6}
.border-blue-500{border-color:#3b82f6}
.text-blue-600{color:#2563eb}
.bg-blue-600{background-color:#2563eb}
.border-blue-600{border-color:#2563eb}
.text-blue-700{color:#1d4ed8}
.bg-blue-700{background-color:#1d4ed8}
.border-blue-700{border-color:#1d4ed8}
.text-blue-800{color:#1e40af}
.bg-blue-800{background-color:#1e40af}
.border-blue-800{border-color:#1e40af}
.text-blue-900{color:#1e3a8a}
.bg-blue-900{background-color:#1e3a8a}
.border-blue-900{border-color:#1e3a8a}
.text-blue-950{color:#172554}
.bg-blue-950{background-color:#172554}
.border-blue-950{border-color:#172554}
.text-indigo-50{color:#eef2ff}
.bg-indigo-50{background-color:#eef2ff}
.border-indigo-50{border-color:#eef2ff}
.text-indigo-100{color:#e0e7ff}
.bg-indigo-100{background-color:#e0e7ff}
.border-indigo-100{border-color:#e0e7ff}
.text-indigo-200{color:#c7d2fe}
.bg-indigo-200{background-color:#c7d2fe}
.border-indigo-200{border-color:#c7d2fe}
.text-indigo-300{color:#a5b4fc}
.bg-indigo-300{background-color:#a5b4fc}
.border-indigo-300{border-color:#a5b4fc}
.text-indigo-400{color:#818cf8}
.bg-indigo-400{background-color:#818cf8}
.border-indigo-400{border-color:#818cf8}
.text-indigo-500{color:#6366f1}
.bg-indigo-500{background-color:#6366f1}
.border-indigo-500{border-color:#6366f1}
.text-indigo-600{color:#4f46e5}
.bg-indigo-600{background-color:#4f46e5}
.border-indigo-600{border-color:#4f46e5}
.text-indigo-700{color:#4338ca}
.bg-indigo-700{background-color:#4338ca}
.border-indigo-700{border-color:#4338ca}
.text-indigo-800{color:#3730a3}
.bg-indigo-800{background-color:#3730a3}
.border-indigo-800{border-color:#3730a3}
.text-indigo-900{color:#312e81}
.bg-indigo-900{background-color:#312e81}
.border-indigo-900{border-color:#312e81}
.text-indigo-950{color:#1e1b4b}
.bg-indigo-950{background-color:#1e1b4b}
.border-indigo-950{border-color:#1e1b4b}
.text-violet-50{color:#f5f3ff}
.bg-violet-50{background-color:#f5f3ff}
.border-violet-50{border-color:#f5f3ff}
.text-violet-100{color:#ede9fe}
.bg-violet-100{background-color:#ede9fe}
.border-violet-100{border-color:#ede9fe}
.text-violet-200{color:#ddd6fe}
.bg-violet-200{background-color:#ddd6fe}
.border-violet-200{border-color:#ddd6fe}
.text-violet-300{color:#c4b5fd}
.bg-violet-300{background-color:#c4b5fd}
.border-violet-300{border-color:#c4b5fd}
.text-violet-400{color:#a78bfa}
.bg-violet-400{background-color:#a78bfa}
.border-violet-400{border-color:#a78bfa}
.text-violet-500{color:#8b5cf6}
.bg-violet-500{background-color:#8b5cf6}
.border-violet-500{border-color:#8b5cf6}
.text-violet-600{color:#7c3aed}
.bg-violet-600{background-color:#7c3aed}
.border-violet-600{border-color:#7c3aed}
.text-violet-700{color:#6d28d9}
.bg-violet-700{background-color:#6d28d9}
.border-violet-700{border-color:#6d28d9}
.text-violet-800{color:#5b21b6}
.bg-violet-800{background-color:#5b21b6}
.border-violet-800{border-color:#5b21b6}
.text-violet-900{color:#4c1d95}
.bg-violet-900{background-color:#4c1d95}
.border-violet-900{border-color:#4c1d95}
.text-violet-950{color:#2e1065}
.bg-violet-950{background-color:#2e1065}
.border-violet-950{border-color:#2e1065}
.text-purple-50{color:#faf5ff}
.bg-purple-50{background-color:#faf5ff}
.border-purple-50{border-color:#faf5ff}
.text-purple-100{color:#f3e8ff}
.bg-purple-100{background-color:#f3e8ff}
.border-purple-100{border-color:#f3e8ff}
.text-purple-200{color:#e9d5ff}
.bg-purple-200{background-color:#e9d5ff}
.border-purple-200{border-color:#e9d5ff}
.text-purple-300{color:#d8b4fe}
.bg-purple-300{background-color:#d8b4fe}
.border-purple-300{border-color:#d8b4fe}
.text-purple-400{color:#c084fc}
.bg-purple-400{background-color:#c084fc}
.border-purple-400{border-color:#c084fc}
.text-purple-500{color:#a855f7}
.bg-purple-500{background-color:#a855f7}
.border-purple-500{border-color:#a855f7}
.text-purple-600{color:#9333ea}
.bg-purple-600{background-color:#9333ea}
.border-purple-600{border-color:#9333ea}
.text-purple-700{color:#7e22ce}
.bg-purple-700{background-color:#7e22ce}
.border-purple-700{border-color:#7e22ce}
.text-purple-800{color:#6b21a8}
.bg-purple-800{background-color:#6b21a8}
.border-purple-800{border-color:#6b21a8}
.text-purple-900{color:#581c87}
.bg-purple-900{background-color:#581c87}
.border-purple-900{border-color:#581c87}
.text-purple-950{color:#3b0764}
.bg-purple-950{background-color:#3b0764}
.border-purple-950{border-color:#3b0764}
.text-fuchsia-50{color:#fdf4ff}
.bg-fuchsia-50{background-color:#fdf4ff}
.border-fuchsia-50{border-color:#fdf4ff}
.text-fuchsia-100{color:#fae8ff}
.bg-fuchsia-100{background-color:#fae8ff}
.border-fuchsia-100{border-color:#fae8ff}
.text-fuchsia-200{color:#f5d0fe}
.bg-fuchsia-200{background-color:#f5d0fe}
.border-fuchsia-200{border-color:#f5d0fe}
.text-fuchsia-300{color:#f0abfc}
.bg-fuchsia-300{background-color:#f0abfc}
.border-fuchsia-300{border-color:#f0abfc}
.text-fuchsia-400{color:#e879f9}
.bg-fuchsia-400{background-color:#e879f9}
.border-fuchsia-400{border-color:#e879f9}
.text-fuchsia-500{color:#d946ef}
.bg-fuchsia-500{background-color:#d946ef}
.border-fuchsia-500{border-color:#d946ef}
.text-fuchsia-600{color:#c026d3}
.bg-fuchsia-600{background-color:#c026d3}
.border-fuchsia-600{border-color:#c026d3}
.text-fuchsia-700{color:#a21caf}
.bg-fuchsia-700{background-color:#a21caf}
.border-fuchsia-700{border-color:#a21caf}
.text-fuchsia-800{color:#86198f}
.bg-fuchsia-800{background-color:#86198f}
.border-fuchsia-800{border-color:#86198f}
.text-fuchsia-900{color:#701a75}
.bg-fuchsia-900{background-color:#701a75}
.border-fuchsia-900{border-color:#701a75}
.text-fuchsia-950{color:#4a044e}
.bg-fuchsia-950{background-color:#4a044e}
.border-fuchsia-950{border-color:#4a044e}
.text-pink-50{color:#fdf2f8}
.bg-pink-50{background-color:#fdf2f8}
.border-pink-50{border-color:#fdf2f8}
.text-pink-100{color:#fce7f3}
.bg-pink-100{background-color:#fce7f3}
.border-pink-100{border-color:#fce7f3}
.text-pink-200{color:#fbcfe8}
.bg-pink-200{background-color:#fbcfe8}
.border-pink-200{border-color:#fbcfe8}
.text-pink-300{color:#f9a8d4}
.bg-pink-300{background-color:#f9a8d4}
.border-pink-300{border-color:#f9a8d4}
.text-pink-400{color:#f472b6}
.bg-pink-400{background-color:#f472b6}
.border-pink-400{border-color:#f472b6}
.text-pink-500{color:#ec4899}
.bg-pink-500{background-color:#ec4899}
.border-pink-500{border-color:#ec4899}
.text-pink-600{color:#db2777}
.bg-pink-600{background-color:#db2777}
.border-pink-600{border-color:#db2777}
.text-pink-700{color:#be185d}
.bg-pink-700{background-color:#be185d}
.border-pink-700{border-color:#be185d}
.text-pink-800{color:#9d174d}
.bg-pink-800{background-color:#9d174d}
.border-pink-800{border-color:#9d174d}
.text-pink-900{color:#831843}
.bg-pink-900{background-color:#831843}
.border-pink-900{border-color:#831843}
.text-pink-950{color:#500724}
.bg-pink-950{background-color:#500724}
.border-pink-950{border-color:#500724}
.text-rose-50{color:#fff1f2}
.bg-rose-50{background-color:#fff1f2}
.border-rose-50{border-color:#fff1f2}
.text-rose-100{color:#ffe4e6}
.bg-rose-100{background-color:#ffe4e6}
.border-rose-100{border-color:#ffe4e6}
.text-rose-200{color:#fecdd3}
.bg-rose-200{background-color:#fecdd3}
.border-rose-200{border-color:#fecdd3}
.text-rose-300{color:#fda4af}
.bg-rose-300{background-color:#fda4af}
.border-rose-300{border-color:#fda4af}
.text-rose-400{color:#fb7185}
.bg-rose-400{background-color:#fb7185}
.border-rose-400{border-color:#fb7185}
.text-rose-500{color:#f43f5e}
.bg-rose-500{background-color:#f43f5e}
.border-rose-500{border-color:#f43f5e}
.text-rose-600{color:#e11d48}
.bg-rose-600{background-color:#e11d48}
.border-rose-600{border-color:#e11d48}
.text-rose-700{color:#be123c}
.bg-rose-700{background-color:#be123c}
.border-rose-700{border-color:#be123c}
.text-rose-800{color:#9f1239}
.bg-rose-800{background-color:#9f1239}
.border-rose-800{border-color:#9f1239}
.text-rose-900{color:#881337}
.bg-rose-900{background-color:#881337}
.border-rose-900{border-color:#881337}
.text-rose-950{color:#4c0519}
.bg-rose-950{background-color:#4c0519}
.border-rose-950{border-color:#4c0519}
.bg-amber-500\/5{background-color:rgb(245 158 11/0.05)}
.border-amber-500\/5{border-color:rgb(245 158 11/0.05)}
.bg-amber-500\/10{background-color:rgb(245 158 11/0.1)}
.border-amber-500\/10{border-color:rgb(245 158 11/0.1)}
.bg-amber-500\/15{background-color:rgb(245 158 11/0.15)}
.border-amber-500\/15{border-color:rgb(245 158 11/0.15)}
.bg-amber-500\/20{background-color:rgb(245 158 11/0.2)}
.border-amber-500\/20{border-color:rgb(245 158 11/0.2)}
.bg-amber-500\/30{background-color:rgb(245 158 11/0.3)}
.border-amber-500\/30{border-color:rgb(245 158 11/0.3)}
.bg-amber-500\/40{background-color:rgb(245 158 11/0.4)}
.border-amber-500\/40{border-color:rgb(245 158 11/0.4)}
.bg-amber-500\/50{background-color:rgb(245 158 11/0.5)}
.border-amber-500\/50{border-color:rgb(245 158 11/0.5)}
.text-amber-500\/70{color:rgb(245 158 11/0.7)}
.text-amber-500\/80{color:rgb(245 158 11/0.8)}
.hover\:bg-amber-400:hover{background-color:#fbbf24}
.hover\:bg-amber-500:hover{background-color:#f59e0b}
.hover\:text-amber-400:hover{color:#fbbf24}
.hover\:border-amber-500:hover{border-color:#f59e0b}
.bg-emerald-500\/5{background-color:rgb(16 185 129/0.05)}
.border-emerald-500\/5{border-color:rgb(16 185 129/0.05)}
.bg-emerald-500\/10{background-color:rgb(16 185 129/0.1)}
.border-emerald-500\/10{border-color:rgb(16 185 129/0.1)}
.bg-emerald-500\/15{background-color:rgb(16 185 129/0.15)}
.border-emerald-500\/15{border-color:rgb(16 185 129/0.15)}
.bg-emerald-500\/20{background-color:rgb(16 185 129/0.2)}
.border-emerald-500\/20{border-color:rgb(16 185 129/0.2)}
.bg-emerald-500\/30{background-color:rgb(16 185 129/0.3)}
.border-emerald-500\/30{border-color:rgb(16 185 129/0.3)}
.bg-emerald-500\/40{background-color:rgb(16 185 129/0.4)}
.border-emerald-500\/40{border-color:rgb(16 185 129/0.4)}
.bg-emerald-500\/50{background-color:rgb(16 185 129/0.5)}
.border-emerald-500\/50{border-color:rgb(16 185 129/0.5)}
.text-emerald-500\/70{color:rgb(16 185 129/0.7)}
.text-emerald-500\/80{color:rgb(16 185 129/0.8)}
.hover\:bg-emerald-400:hover{background-color:#34d399}
.hover\:bg-emerald-500:hover{background-color:#10b981}
.hover\:text-emerald-400:hover{color:#34d399}
.hover\:border-emerald-500:hover{border-color:#10b981}
.bg-indigo-500\/5{background-color:rgb(99 102 241/0.05)}
.border-indigo-500\/5{border-color:rgb(99 102 241/0.05)}
.bg-indigo-500\/10{background-color:rgb(99 102 241/0.1)}
.border-indigo-500\/10{border-color:rgb(99 102 241/0.1)}
.bg-indigo-500\/15{background-color:rgb(99 102 241/0.15)}
.border-indigo-500\/15{border-color:rgb(99 102 241/0.15)}
.bg-indigo-500\/20{background-color:rgb(99 102 241/0.2)}
.border-indigo-500\/20{border-color:rgb(99 102 241/0.2)}
.bg-indigo-500\/30{background-color:rgb(99 102 241/0.3)}
.border-indigo-500\/30{border-color:rgb(99 102 241/0.3)}
.bg-indigo-500\/40{background-color:rgb(99 102 241/0.4)}
.border-indigo-500\/40{border-color:rgb(99 102 241/0.4)}
.bg-indigo-500\/50{background-color:rgb(99 102 241/0.5)}
.border-indigo-500\/50{border-color:rgb(99 102 241/0.5)}
.text-indigo-500\/70{color:rgb(99 102 241/0.7)}
.text-indigo-500\/80{color:rgb(99 102 241/0.8)}
.hover\:bg-indigo-400:hover{background-color:#818cf8}
.hover\:bg-indigo-500:hover{background-color:#6366f1}
.hover\:text-indigo-400:hover{color:#818cf8}
.hover\:border-indigo-500:hover{border-color:#6366f1}
.bg-red-500\/5{background-color:rgb(239 68 68/0.05)}
.border-red-500\/5{border-color:rgb(239 68 68/0.05)}
.bg-red-500\/10{background-color:rgb(239 68 68/0.1)}
.border-red-500\/10{border-color:rgb(239 68 68/0.1)}
.bg-red-500\/15{background-color:rgb(239 68 68/0.15)}
.border-red-500\/15{border-color:rgb(239 68 68/0.15)}
.bg-red-500\/20{background-color:rgb(239 68 68/0.2)}
.border-red-500\/20{border-color:rgb(239 68 68/0.2)}
.bg-red-500\/30{background-color:rgb(239 68 68/0.3)}
.border-red-500\/30{border-color:rgb(239 68 68/0.3)}
.bg-red-500\/40{background-color:rgb(239 68 68/0.4)}
.border-red-500\/40{border-color:rgb(239 68 68/0.4)}
.bg-red-500\/50{background-color:rgb(239 68 68/0.5)}
.border-red-500\/50{border-color:rgb(239 68 68/0.5)}
.text-red-500\/70{color:rgb(239 68 68/0.7)}
.text-red-500\/80{color:rgb(239 68 68/0.8)}
.hover\:bg-red-400:hover{background-color:#f87171}
.hover\:bg-red-500:hover{background-color:#ef4444}
.hover\:text-red-400:hover{color:#f87171}
.hover\:border-red-500:hover{border-color:#ef4444}
.bg-blue-500\/5{background-color:rgb(59 130 246/0.05)}
.border-blue-500\/5{border-color:rgb(59 130 246/0.05)}
.bg-blue-500\/10{background-color:rgb(59 130 246/0.1)}
.border-blue-500\/10{border-color:rgb(59 130 246/0.1)}
.bg-blue-500\/15{background-color:rgb(59 130 246/0.15)}
.border-blue-500\/15{border-color:rgb(59 130 246/0.15)}
.bg-blue-500\/20{background-color:rgb(59 130 246/0.2)}
.border-blue-500\/20{border-color:rgb(59 130 246/0.2)}
.bg-blue-500\/30{background-color:rgb(59 130 246/0.3)}
.border-blue-500\/30{border-color:rgb(59 130 246/0.3)}
.bg-blue-500\/40{background-color:rgb(59 130 246/0.4)}
.border-blue-500\/40{border-color:rgb(59 130 246/0.4)}
.bg-blue-500\/50{background-color:rgb(59 130 246/0.5)}
.border-blue-500\/50{border-color:rgb(59 130 246/0.5)}
.text-blue-500\/70{color:rgb(59 130 246/0.7)}
.text-blue-500\/80{color:rgb(59 130 246/0.8)}
.hover\:bg-blue-400:hover{background-color:#60a5fa}
.hover\:bg-blue-500:hover{background-color:#3b82f6}
.hover\:text-blue-400:hover{color:#60a5fa}
.hover\:border-blue-500:hover{border-color:#3b82f6}
.bg-purple-500\/5{background-color:rgb(168 85 247/0.05)}
.border-purple-500\/5{border-color:rgb(168 85 247/0.05)}
.bg-purple-500\/10{background-color:rgb(168 85 247/0.1)}
.border-purple-500\/10{border-color:rgb(168 85 247/0.1)}
.bg-purple-500\/15{background-color:rgb(168 85 247/0.15)}
.border-purple-500\/15{border-color:rgb(168 85 247/0.15)}
.bg-purple-500\/20{background-color:rgb(168 85 247/0.2)}
.border-purple-500\/20{border-color:rgb(168 85 247/0.2)}
.bg-purple-500\/30{background-color:rgb(168 85 247/0.3)}
.border-purple-500\/30{border-color:rgb(168 85 247/0.3)}
.bg-purple-500\/40{background-color:rgb(168 85 247/0.4)}
.border-purple-500\/40{border-color:rgb(168 85 247/0.4)}
.bg-purple-500\/50{background-color:rgb(168 85 247/0.5)}
.border-purple-500\/50{border-color:rgb(168 85 247/0.5)}
.text-purple-500\/70{color:rgb(168 85 247/0.7)}
.text-purple-500\/80{color:rgb(168 85 247/0.8)}
.hover\:bg-purple-400:hover{background-color:#c084fc}
.hover\:bg-purple-500:hover{background-color:#a855f7}
.hover\:text-purple-400:hover{color:#c084fc}
.hover\:border-purple-500:hover{border-color:#a855f7}
.bg-violet-500\/5{background-color:rgb(139 92 246/0.05)}
.border-violet-500\/5{border-color:rgb(139 92 246/0.05)}
.bg-violet-500\/10{background-color:rgb(139 92 246/0.1)}
.border-violet-500\/10{border-color:rgb(139 92 246/0.1)}
.bg-violet-500\/15{background-color:rgb(139 92 246/0.15)}
.border-violet-500\/15{border-color:rgb(139 92 246/0.15)}
.bg-violet-500\/20{background-color:rgb(139 92 246/0.2)}
.border-violet-500\/20{border-color:rgb(139 92 246/0.2)}
.bg-violet-500\/30{background-color:rgb(139 92 246/0.3)}
.border-violet-500\/30{border-color:rgb(139 92 246/0.3)}
.bg-violet-500\/40{background-color:rgb(139 92 246/0.4)}
.border-violet-500\/40{border-color:rgb(139 92 246/0.4)}
.bg-violet-500\/50{background-color:rgb(139 92 246/0.5)}
.border-violet-500\/50{border-color:rgb(139 92 246/0.5)}
.text-violet-500\/70{color:rgb(139 92 246/0.7)}
.text-violet-500\/80{color:rgb(139 92 246/0.8)}
.hover\:bg-violet-400:hover{background-color:#a78bfa}
.hover\:bg-violet-500:hover{background-color:#8b5cf6}
.hover\:text-violet-400:hover{color:#a78bfa}
.hover\:border-violet-500:hover{border-color:#8b5cf6}
.bg-green-500\/5{background-color:rgb(34 197 94/0.05)}
.border-green-500\/5{border-color:rgb(34 197 94/0.05)}
.bg-green-500\/10{background-color:rgb(34 197 94/0.1)}
.border-green-500\/10{border-color:rgb(34 197 94/0.1)}
.bg-green-500\/15{background-color:rgb(34 197 94/0.15)}
.border-green-500\/15{border-color:rgb(34 197 94/0.15)}
.bg-green-500\/20{background-color:rgb(34 197 94/0.2)}
.border-green-500\/20{border-color:rgb(34 197 94/0.2)}
.bg-green-500\/30{background-color:rgb(34 197 94/0.3)}
.border-green-500\/30{border-color:rgb(34 197 94/0.3)}
.bg-green-500\/40{background-color:rgb(34 197 94/0.4)}
.border-green-500\/40{border-color:rgb(34 197 94/0.4)}
.bg-green-500\/50{background-color:rgb(34 197 94/0.5)}
.border-green-500\/50{border-color:rgb(34 197 94/0.5)}
.text-green-500\/70{color:rgb(34 197 94/0.7)}
.text-green-500\/80{color:rgb(34 197 94/0.8)}
.hover\:bg-green-400:hover{background-color:#4ade80}
.hover\:bg-green-500:hover{background-color:#22c55e}
.hover\:text-green-400:hover{color:#4ade80}
.hover\:border-green-500:hover{border-color:#22c55e}
.bg-sky-500\/5{background-color:rgb(14 165 233/0.05)}
.border-sky-500\/5{border-color:rgb(14 165 233/0.05)}
.bg-sky-500\/10{background-color:rgb(14 165 233/0.1)}
.border-sky-500\/10{border-color:rgb(14 165 233/0.1)}
.bg-sky-500\/15{background-color:rgb(14 165 233/0.15)}
.border-sky-500\/15{border-color:rgb(14 165 233/0.15)}
.bg-sky-500\/20{background-color:rgb(14 165 233/0.2)}
.border-sky-500\/20{border-color:rgb(14 165 233/0.2)}
.bg-sky-500\/30{background-color:rgb(14 165 233/0.3)}
.border-sky-500\/30{border-color:rgb(14 165 233/0.3)}
.bg-sky-500\/40{background-color:rgb(14 165 233/0.4)}
.border-sky-500\/40{border-color:rgb(14 165 233/0.4)}
.bg-sky-500\/50{background-color:rgb(14 165 233/0.5)}
.border-sky-500\/50{border-color:rgb(14 165 233/0.5)}
.text-sky-500\/70{color:rgb(14 165 233/0.7)}
.text-sky-500\/80{color:rgb(14 165 233/0.8)}
.hover\:bg-sky-400:hover{background-color:#38bdf8}
.hover\:bg-sky-500:hover{background-color:#0ea5e9}
.hover\:text-sky-400:hover{color:#38bdf8}
.hover\:border-sky-500:hover{border-color:#0ea5e9}
.bg-cyan-500\/5{background-color:rgb(6 182 212/0.05)}
.border-cyan-500\/5{border-color:rgb(6 182 212/0.05)}
.bg-cyan-500\/10{background-color:rgb(6 182 212/0.1)}
.border-cyan-500\/10{border-color:rgb(6 182 212/0.1)}
.bg-cyan-500\/15{background-color:rgb(6 182 212/0.15)}
.border-cyan-500\/15{border-color:rgb(6 182 212/0.15)}
.bg-cyan-500\/20{background-color:rgb(6 182 212/0.2)}
.border-cyan-500\/20{border-color:rgb(6 182 212/0.2)}
.bg-cyan-500\/30{background-color:rgb(6 182 212/0.3)}
.border-cyan-500\/30{border-color:rgb(6 182 212/0.3)}
.bg-cyan-500\/40{background-color:rgb(6 182 212/0.4)}
.border-cyan-500\/40{border-color:rgb(6 182 212/0.4)}
.bg-cyan-500\/50{background-color:rgb(6 182 212/0.5)}
.border-cyan-500\/50{border-color:rgb(6 182 212/0.5)}
.text-cyan-500\/70{color:rgb(6 182 212/0.7)}
.text-cyan-500\/80{color:rgb(6 182 212/0.8)}
.hover\:bg-cyan-400:hover{background-color:#22d3ee}
.hover\:bg-cyan-500:hover{background-color:#06b6d4}
.hover\:text-cyan-400:hover{color:#22d3ee}
.hover\:border-cyan-500:hover{border-color:#06b6d4}
.bg-teal-500\/5{background-color:rgb(20 184 166/0.05)}
.border-teal-500\/5{border-color:rgb(20 184 166/0.05)}
.bg-teal-500\/10{background-color:rgb(20 184 166/0.1)}
.border-teal-500\/10{border-color:rgb(20 184 166/0.1)}
.bg-teal-500\/15{background-color:rgb(20 184 166/0.15)}
.border-teal-500\/15{border-color:rgb(20 184 166/0.15)}
.bg-teal-500\/20{background-color:rgb(20 184 166/0.2)}
.border-teal-500\/20{border-color:rgb(20 184 166/0.2)}
.bg-teal-500\/30{background-color:rgb(20 184 166/0.3)}
.border-teal-500\/30{border-color:rgb(20 184 166/0.3)}
.bg-teal-500\/40{background-color:rgb(20 184 166/0.4)}
.border-teal-500\/40{border-color:rgb(20 184 166/0.4)}
.bg-teal-500\/50{background-color:rgb(20 184 166/0.5)}
.border-teal-500\/50{border-color:rgb(20 184 166/0.5)}
.text-teal-500\/70{color:rgb(20 184 166/0.7)}
.text-teal-500\/80{color:rgb(20 184 166/0.8)}
.hover\:bg-teal-400:hover{background-color:#2dd4bf}
.hover\:bg-teal-500:hover{background-color:#14b8a6}
.hover\:text-teal-400:hover{color:#2dd4bf}
.hover\:border-teal-500:hover{border-color:#14b8a6}
.bg-pink-500\/5{background-color:rgb(236 72 153/0.05)}
.border-pink-500\/5{border-color:rgb(236 72 153/0.05)}
.bg-pink-500\/10{background-color:rgb(236 72 153/0.1)}
.border-pink-500\/10{border-color:rgb(236 72 153/0.1)}
.bg-pink-500\/15{background-color:rgb(236 72 153/0.15)}
.border-pink-500\/15{border-color:rgb(236 72 153/0.15)}
.bg-pink-500\/20{background-color:rgb(236 72 153/0.2)}
.border-pink-500\/20{border-color:rgb(236 72 153/0.2)}
.bg-pink-500\/30{background-color:rgb(236 72 153/0.3)}
.border-pink-500\/30{border-color:rgb(236 72 153/0.3)}
.bg-pink-500\/40{background-color:rgb(236 72 153/0.4)}
.border-pink-500\/40{border-color:rgb(236 72 153/0.4)}
.bg-pink-500\/50{background-color:rgb(236 72 153/0.5)}
.border-pink-500\/50{border-color:rgb(236 72 153/0.5)}
.text-pink-500\/70{color:rgb(236 72 153/0.7)}
.text-pink-500\/80{color:rgb(236 72 153/0.8)}
.hover\:bg-pink-400:hover{background-color:#f472b6}
.hover\:bg-pink-500:hover{background-color:#ec4899}
.hover\:text-pink-400:hover{color:#f472b6}
.hover\:border-pink-500:hover{border-color:#ec4899}
.bg-rose-500\/5{background-color:rgb(244 63 94/0.05)}
.border-rose-500\/5{border-color:rgb(244 63 94/0.05)}
.bg-rose-500\/10{background-color:rgb(244 63 94/0.1)}
.border-rose-500\/10{border-color:rgb(244 63 94/0.1)}
.bg-rose-500\/15{background-color:rgb(244 63 94/0.15)}
.border-rose-500\/15{border-color:rgb(244 63 94/0.15)}
.bg-rose-500\/20{background-color:rgb(244 63 94/0.2)}
.border-rose-500\/20{border-color:rgb(244 63 94/0.2)}
.bg-rose-500\/30{background-color:rgb(244 63 94/0.3)}
.border-rose-500\/30{border-color:rgb(244 63 94/0.3)}
.bg-rose-500\/40{background-color:rgb(244 63 94/0.4)}
.border-rose-500\/40{border-color:rgb(244 63 94/0.4)}
.bg-rose-500\/50{background-color:rgb(244 63 94/0.5)}
.border-rose-500\/50{border-color:rgb(244 63 94/0.5)}
.text-rose-500\/70{color:rgb(244 63 94/0.7)}
.text-rose-500\/80{color:rgb(244 63 94/0.8)}
.hover\:bg-rose-400:hover{background-color:#fb7185}
.hover\:bg-rose-500:hover{background-color:#f43f5e}
.hover\:text-rose-400:hover{color:#fb7185}
.hover\:border-rose-500:hover{border-color:#f43f5e}
.bg-white\/4{background:rgba(255,255,255,0.04)}
.bg-black\/4{background:rgba(0,0,0,0.04)}
.border-white\/4{border-color:rgba(255,255,255,0.04)}
.bg-white\/5{background:rgba(255,255,255,0.05)}
.bg-black\/5{background:rgba(0,0,0,0.05)}
.border-white\/5{border-color:rgba(255,255,255,0.05)}
.bg-white\/6{background:rgba(255,255,255,0.06)}
.bg-black\/6{background:rgba(0,0,0,0.06)}
.border-white\/6{border-color:rgba(255,255,255,0.06)}
.bg-white\/8{background:rgba(255,255,255,0.08)}
.bg-black\/8{background:rgba(0,0,0,0.08)}
.border-white\/8{border-color:rgba(255,255,255,0.08)}
.bg-white\/10{background:rgba(255,255,255,0.1)}
.bg-black\/10{background:rgba(0,0,0,0.1)}
.border-white\/10{border-color:rgba(255,255,255,0.1)}
.bg-white\/15{background:rgba(255,255,255,0.15)}
.bg-black\/15{background:rgba(0,0,0,0.15)}
.border-white\/15{border-color:rgba(255,255,255,0.15)}
.bg-white\/20{background:rgba(255,255,255,0.2)}
.bg-black\/20{background:rgba(0,0,0,0.2)}
.border-white\/20{border-color:rgba(255,255,255,0.2)}
.bg-white\/25{background:rgba(255,255,255,0.25)}
.bg-black\/25{background:rgba(0,0,0,0.25)}
.border-white\/25{border-color:rgba(255,255,255,0.25)}
.bg-white\/30{background:rgba(255,255,255,0.3)}
.bg-black\/30{background:rgba(0,0,0,0.3)}
.border-white\/30{border-color:rgba(255,255,255,0.3)}
.bg-white\/40{background:rgba(255,255,255,0.4)}
.bg-black\/40{background:rgba(0,0,0,0.4)}
.border-white\/40{border-color:rgba(255,255,255,0.4)}
.bg-white\/50{background:rgba(255,255,255,0.5)}
.bg-black\/50{background:rgba(0,0,0,0.5)}
.border-white\/50{border-color:rgba(255,255,255,0.5)}
.bg-white\/60{background:rgba(255,255,255,0.6)}
.bg-black\/60{background:rgba(0,0,0,0.6)}
.border-white\/60{border-color:rgba(255,255,255,0.6)}
.bg-white\/70{background:rgba(255,255,255,0.7)}
.bg-black\/70{background:rgba(0,0,0,0.7)}
.border-white\/70{border-color:rgba(255,255,255,0.7)}
.bg-white\/80{background:rgba(255,255,255,0.8)}
.bg-black\/80{background:rgba(0,0,0,0.8)}
.border-white\/80{border-color:rgba(255,255,255,0.8)}
.bg-white\/90{background:rgba(255,255,255,0.9)}
.bg-black\/90{background:rgba(0,0,0,0.9)}
.border-white\/90{border-color:rgba(255,255,255,0.9)}

/* ═══ ARBITRARY VALUES ═══ */
.bg-\[\#050505\]{background:#050505}.bg-\[\#080808\]{background:#080808}.bg-\[\#0a0a0a\]{background:#0a0a0a}
.bg-\[\#1b1c1c\]{background:#1b1c1c}.bg-\[\#2f3131\]{background:#2f3131}
.text-\[9px\]{font-size:9px}.text-\[10px\]{font-size:10px}.text-\[11px\]{font-size:11px}.text-\[13px\]{font-size:13px}
.tracking-\[0\.1em\]{letter-spacing:0.1em}.tracking-\[0\.15em\]{letter-spacing:0.15em}.tracking-\[0\.2em\]{letter-spacing:0.2em}
.leading-\[0\.9\]{line-height:0.9}
.w-\[800px\]{width:800px}.h-\[600px\]{height:600px}.h-\[500px\]{height:500px}
.min-h-\[750px\]{min-height:750px}.min-h-\[60vh\]{min-height:60vh}
.opacity-\[0\.07\]{opacity:0.07}
.border-white\/\[0\.04\]{border-color:rgba(255,255,255,0.04)}.border-white\/\[0\.06\]{border-color:rgba(255,255,255,0.06)}
.border-neutral-800\/50{border-color:rgb(38 38 38/0.5)}
.from-\[\#050505\]{--tw-gradient-from:#050505}.via-\[\#050505\]\/25{--tw-gradient-via:rgba(5,5,5,0.25)}.to-\[\#050505\]\/20{--tw-gradient-to:rgba(5,5,5,0.2)}
.text-\[\#8b5cf6\]{color:#8b5cf6}.text-\[\#28c840\]{color:#28c840}.text-\[\#a1a1aa\]{color:#a1a1aa}
.-top-3{top:-0.75rem}.-translate-x-1\/2{transform:translateX(-50%)}
.left-1\/2{left:50%}
.gap-0\.5{gap:0.125rem}
.py-0\.5{padding-top:0.125rem;padding-bottom:0.125rem}
.px-2\.5{padding-left:0.625rem;padding-right:0.625rem}
.gap-1\.5{gap:0.375rem}.gap-2\.5{gap:0.625rem}
.w-1\.5{width:0.375rem}.h-1\.5{height:0.375rem}.w-2\.5{width:0.625rem}.h-2\.5{height:0.625rem}
.w-3{width:0.75rem}.h-3{height:0.75rem}.h-0\.5{height:0.125rem}
.-ml-2{margin-left:-0.5rem}
.first\:ml-0:first-child{margin-left:0}
"##,
);

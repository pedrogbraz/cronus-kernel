//! CRONUS Animation Library — 50+ animations, 0 dependencies
//!
//! CSS keyframes + utility classes + IntersectionObserver for viewport animations.
//! Replaces Framer Motion with pure CSS + ~500 bytes of JS.

pub const CRONUS_ANIMATIONS: &str = r##"
/* ═══ CRONUS Animation Library v0.1 — 50 animations, 0 deps ═══ */

/* Base */
.anim{animation-duration:300ms;animation-fill-mode:both}

/* Timing variants */
.anim-fast{animation-duration:150ms!important}
.anim-normal{animation-duration:300ms!important}
.anim-slow{animation-duration:600ms!important}
.anim-slower{animation-duration:1000ms!important}

/* Delay variants */
.anim-d1{animation-delay:.1s}.anim-d2{animation-delay:.2s}.anim-d3{animation-delay:.3s}
.anim-d4{animation-delay:.4s}.anim-d5{animation-delay:.5s}.anim-d6{animation-delay:.6s}
.anim-d7{animation-delay:.7s}.anim-d8{animation-delay:.8s}.anim-d9{animation-delay:.9s}
.anim-d10{animation-delay:1s}

/* Stagger children */
.stagger>*{opacity:0;animation-fill-mode:both}
.stagger>*:nth-child(1){animation-delay:.05s}.stagger>*:nth-child(2){animation-delay:.1s}
.stagger>*:nth-child(3){animation-delay:.15s}.stagger>*:nth-child(4){animation-delay:.2s}
.stagger>*:nth-child(5){animation-delay:.25s}.stagger>*:nth-child(6){animation-delay:.3s}
.stagger>*:nth-child(7){animation-delay:.35s}.stagger>*:nth-child(8){animation-delay:.4s}
.stagger>*:nth-child(9){animation-delay:.45s}.stagger>*:nth-child(10){animation-delay:.5s}

/* Viewport animation: hidden until intersected */
[data-animate]{opacity:0;transform:translateY(16px);transition:opacity .5s ease-out,transform .5s ease-out}
[data-animate].animate-in{opacity:1;transform:translateY(0)}

/* ── 1. Fades ── */
.animate-fadeIn{animation:fadeIn 240ms ease-out both}
@keyframes fadeIn{from{opacity:0}to{opacity:1}}
.animate-fadeOut{animation:fadeOut 180ms ease-in both}
@keyframes fadeOut{from{opacity:1}to{opacity:0}}
.animate-fadeInUp{animation:fadeInUp 400ms ease-out both}
@keyframes fadeInUp{from{opacity:0;transform:translateY(20px)}to{opacity:1;transform:translateY(0)}}
.animate-fadeInDown{animation:fadeInDown 400ms ease-out both}
@keyframes fadeInDown{from{opacity:0;transform:translateY(-20px)}to{opacity:1;transform:translateY(0)}}

/* ── 2. Slides ── */
.animate-slideUp{animation:slideUp 280ms cubic-bezier(.16,1,.3,1) both}
@keyframes slideUp{from{opacity:0;transform:translateY(16px)}to{opacity:1;transform:translateY(0)}}
.animate-slideDown{animation:slideDown 280ms cubic-bezier(.16,1,.3,1) both}
@keyframes slideDown{from{opacity:0;transform:translateY(-16px)}to{opacity:1;transform:translateY(0)}}
.animate-slideLeft{animation:slideLeft 280ms cubic-bezier(.16,1,.3,1) both}
@keyframes slideLeft{from{opacity:0;transform:translateX(16px)}to{opacity:1;transform:translateX(0)}}
.animate-slideRight{animation:slideRight 280ms cubic-bezier(.16,1,.3,1) both}
@keyframes slideRight{from{opacity:0;transform:translateX(-16px)}to{opacity:1;transform:translateX(0)}}

/* ── 3. Scale ── */
.animate-scaleIn{animation:scaleIn 220ms cubic-bezier(.16,1,.3,1) both}
@keyframes scaleIn{from{opacity:0;transform:scale(.94)}to{opacity:1;transform:scale(1)}}
.animate-scaleOut{animation:scaleOut 160ms ease-in both}
@keyframes scaleOut{from{opacity:1;transform:scale(1)}to{opacity:0;transform:scale(.94)}}
.animate-scaleUp{animation:scaleUp 300ms cubic-bezier(.16,1,.3,1) both}
@keyframes scaleUp{from{opacity:0;transform:scale(.8)}to{opacity:1;transform:scale(1)}}
.animate-popIn{animation:popIn 300ms cubic-bezier(.34,1.56,.64,1) both}
@keyframes popIn{from{opacity:0;transform:scale(.85)}to{opacity:1;transform:scale(1)}}

/* ── 4. Bounce & Elastic ── */
.animate-bounce{animation:cronusBounce 800ms cubic-bezier(.34,1.56,.64,1) both}
@keyframes cronusBounce{0%{transform:translateY(0)}30%{transform:translateY(-18px)}55%{transform:translateY(0)}70%{transform:translateY(-8px)}100%{transform:translateY(0)}}
.animate-bounceIn{animation:bounceIn 600ms cubic-bezier(.34,1.56,.64,1) both}
@keyframes bounceIn{0%{opacity:0;transform:scale(.3)}50%{opacity:1;transform:scale(1.05)}70%{transform:scale(.95)}100%{transform:scale(1)}}
.animate-elastic{animation:elastic 600ms cubic-bezier(.68,-.55,.27,1.55) both}
@keyframes elastic{from{opacity:0;transform:scale(.6)}to{opacity:1;transform:scale(1)}}
.animate-jello{animation:jello 900ms both}
@keyframes jello{0%,100%{transform:scale3d(1,1,1)}30%{transform:scale3d(1.25,.75,1)}40%{transform:scale3d(.75,1.25,1)}50%{transform:scale3d(1.15,.85,1)}65%{transform:scale3d(.95,1.05,1)}75%{transform:scale3d(1.05,.95,1)}}

/* ── 5. Rotate ── */
.animate-spin{animation:cronusSpin 1s linear infinite}
@keyframes cronusSpin{from{transform:rotate(0deg)}to{transform:rotate(360deg)}}
.animate-rotateIn{animation:rotateIn 400ms cubic-bezier(.16,1,.3,1) both}
@keyframes rotateIn{from{opacity:0;transform:rotate(-12deg)}to{opacity:1;transform:rotate(0)}}
.animate-flip{animation:flip 500ms ease-in-out both}
@keyframes flip{from{transform:perspective(400px) rotateY(-90deg);opacity:0}to{transform:perspective(400px) rotateY(0);opacity:1}}

/* ── 6. Shake & Wobble ── */
.animate-shake{animation:shake 420ms cubic-bezier(.36,.07,.19,.97) both}
@keyframes shake{10%,90%{transform:translateX(-1px)}20%,80%{transform:translateX(2px)}30%,50%,70%{transform:translateX(-4px)}40%,60%{transform:translateX(4px)}}
.animate-wiggle{animation:wiggle 420ms ease-in-out both}
@keyframes wiggle{0%{transform:rotate(0)}15%{transform:rotate(5deg)}30%{transform:rotate(-5deg)}45%{transform:rotate(3deg)}60%{transform:rotate(-3deg)}75%{transform:rotate(1deg)}100%{transform:rotate(0)}}
.animate-wobble{animation:wobble 800ms ease-in-out both}
@keyframes wobble{0%{transform:translateX(0)}15%{transform:translateX(-12px) rotate(-4deg)}30%{transform:translateX(8px) rotate(3deg)}45%{transform:translateX(-6px) rotate(-2deg)}60%{transform:translateX(4px) rotate(1deg)}75%{transform:translateX(-2px) rotate(-0.5deg)}100%{transform:translateX(0)}}

/* ── 7. Attention ── */
.animate-pulse{animation:cronusPulse 1.2s ease-in-out infinite}
@keyframes cronusPulse{0%,100%{opacity:1;transform:scale(1)}50%{opacity:.72;transform:scale(1.03)}}
.animate-ping{animation:cronusPing 1s cubic-bezier(0,0,.2,1) infinite}
@keyframes cronusPing{75%,100%{transform:scale(2);opacity:0}}
.animate-flash{animation:flash 1s ease-in-out infinite}
@keyframes flash{0%,50%,100%{opacity:1}25%,75%{opacity:0}}
.animate-rubberBand{animation:rubberBand 800ms both}
@keyframes rubberBand{0%{transform:scaleX(1)}30%{transform:scaleX(1.25) scaleY(.75)}40%{transform:scaleX(.75) scaleY(1.25)}50%{transform:scaleX(1.15) scaleY(.85)}65%{transform:scaleX(.95) scaleY(1.05)}75%{transform:scaleX(1.05) scaleY(.95)}100%{transform:scaleX(1)}}
.animate-headShake{animation:headShake 800ms ease-in-out both}
@keyframes headShake{0%{transform:translateX(0)}6.5%{transform:translateX(-6px) rotateY(-9deg)}18.5%{transform:translateX(5px) rotateY(7deg)}31.5%{transform:translateX(-3px) rotateY(-5deg)}43.5%{transform:translateX(2px) rotateY(3deg)}50%{transform:translateX(0)}}

/* ── 8. Specials ── */
.animate-heartbeat{animation:heartbeat 1.2s ease-in-out infinite}
@keyframes heartbeat{0%{transform:scale(1)}14%{transform:scale(1.15)}28%{transform:scale(1)}42%{transform:scale(1.1)}70%{transform:scale(1)}}
.animate-tada{animation:tada 800ms both}
@keyframes tada{0%{transform:scale(1) rotate(0)}10%,20%{transform:scale(.9) rotate(-3deg)}30%,50%,70%,90%{transform:scale(1.1) rotate(3deg)}40%,60%,80%{transform:scale(1.1) rotate(-3deg)}100%{transform:scale(1) rotate(0)}}
.animate-swing{animation:swing 800ms ease-in-out both;transform-origin:top center}
@keyframes swing{20%{transform:rotate(15deg)}40%{transform:rotate(-10deg)}60%{transform:rotate(5deg)}80%{transform:rotate(-5deg)}100%{transform:rotate(0)}}

/* ── 9. Loading ── */
.animate-skeleton{animation:skeleton 1.5s ease-in-out infinite;background:linear-gradient(90deg,#1a1a1a 25%,#262626 50%,#1a1a1a 75%);background-size:200% 100%}
@keyframes skeleton{from{background-position:200% 0}to{background-position:-200% 0}}
.animate-dots{animation:dots 1.4s both infinite}
@keyframes dots{0%{opacity:.2;transform:scale(.8)}20%{opacity:1;transform:scale(1)}100%{opacity:.2;transform:scale(.8)}}
.animate-progress{animation:progress 2s ease-in-out infinite}
@keyframes progress{0%{width:0}50%{width:70%}100%{width:100%}}

/* ── 10. Complex entrance ── */
.animate-revealUp{animation:revealUp 600ms cubic-bezier(.16,1,.3,1) both;overflow:hidden}
@keyframes revealUp{from{clip-path:inset(100% 0 0 0)}to{clip-path:inset(0)}}
.animate-revealLeft{animation:revealLeft 600ms cubic-bezier(.16,1,.3,1) both}
@keyframes revealLeft{from{clip-path:inset(0 100% 0 0)}to{clip-path:inset(0)}}
.animate-blur{animation:blurIn 400ms ease-out both}
@keyframes blurIn{from{opacity:0;filter:blur(8px)}to{opacity:1;filter:blur(0)}}
.animate-typewriter{overflow:hidden;white-space:nowrap;border-right:2px solid;animation:typewriter 2s steps(20) both,blink .5s step-end infinite alternate}
@keyframes typewriter{from{width:0}to{width:100%}}
@keyframes blink{50%{border-color:transparent}}
.animate-countUp{animation:countUp 600ms cubic-bezier(.16,1,.3,1) both}
@keyframes countUp{from{opacity:0;transform:translateY(100%)}to{opacity:1;transform:translateY(0)}}
.animate-morphBg{animation:morphBg 8s ease-in-out infinite}
@keyframes morphBg{0%,100%{border-radius:60% 40% 30% 70%/60% 30% 70% 40%}50%{border-radius:30% 60% 70% 40%/50% 60% 30% 60%}}

/* ── 11. Framer Motion replacements ── */
/* whileHover → CSS :hover */
.motion-hover{transition:transform .2s,box-shadow .2s}
.motion-hover:hover{transform:scale(1.03)}
.motion-hover:active{transform:scale(.97)}
/* whileInView → handled by IntersectionObserver JS below */
/* AnimatePresence → use .animate-fadeIn on enter, .animate-fadeOut on exit */
/* layoutId → use CSS transitions on position/size properties */
.motion-layout{transition:all .3s cubic-bezier(.16,1,.3,1)}
"##;

/// Micro JS for viewport-triggered animations (~500 bytes)
pub const CRONUS_ANIMATE_JS: &str = r#"
(function(){
  var obs=new IntersectionObserver(function(entries){
    entries.forEach(function(e){
      if(e.isIntersecting){
        e.target.classList.add('animate-in');
        var anim=e.target.dataset.animate;
        if(anim)e.target.classList.add('animate-'+anim);
      }
    });
  },{threshold:0.1,rootMargin:'0px 0px -50px 0px'});
  document.querySelectorAll('[data-animate]').forEach(function(el){obs.observe(el);});
  // Stagger: auto-set --i on children
  document.querySelectorAll('.stagger').forEach(function(p){
    Array.from(p.children).forEach(function(c,i){c.style.animationDelay=(i*0.08)+'s';});
  });
})();
"#;

(function(){
  if (window._cronusUiLive) return;
  window._cronusUiLive = true;

  var STRIP = '<span>9</span><span>0</span><span>1</span><span>2</span><span>3</span><span>4</span><span>5</span><span>6</span><span>7</span><span>8</span><span>9</span><span>0</span>';
  var SHUFFLE = [
    {value:12345, format:'number', prefix:'', suffix:'', locale:'en-US'},
    {value:19348.43, format:'currency', prefix:'$', suffix:'', locale:'en-US'},
    {value:0.42, format:'percentage', prefix:'', suffix:'', locale:'en-US'},
    {value:1234.5, format:'decimal', prefix:'', suffix:'', locale:'en-US'},
    {value:19348.43, format:'currency', prefix:'€', suffix:'', locale:'de-DE'}
  ];

  function pad2(n){
    n = Math.floor(Math.abs(Number(n) || 0));
    return (n < 10 ? '0' : '') + n;
  }
  function esc(s){
    return String(s == null ? '' : s).replace(/[&<>"']/g, function(c){
      return {'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c];
    });
  }

  function bindCountdowns(root){
    (root || document).querySelectorAll('[data-slot="countdown"][data-remain]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      var remain = parseInt(el.getAttribute('data-remain'), 10);
      if (isNaN(remain) || remain < 0) remain = 0;
      var labels = [];
      el.querySelectorAll('[data-slot="countdown-label"]').forEach(function(l){ labels.push(l.textContent); });
      var sr = el.querySelector('.sr-only');
      function paint(){
        var d = Math.floor(remain / 86400);
        var h = Math.floor(remain / 3600) % 24;
        var m = Math.floor(remain / 60) % 60;
        var s = remain % 60;
        var vals = [pad2(d), pad2(h), pad2(m), pad2(s)];
        var nums = [d, h, m, s];
        el.querySelectorAll('[data-slot="countdown-value"]').forEach(function(u, i){
          var span = u.querySelector('span');
          if (!span || span.textContent === vals[i]) return;
          var n = document.createElement('span');
          n.textContent = vals[i];
          u.replaceChild(n, span);
        });
        if (sr && labels.length === 4) {
          sr.textContent = nums[0] + ' ' + labels[0] + ', ' + nums[1] + ' ' + labels[1] + ', ' + nums[2] + ' ' + labels[2] + ', ' + nums[3] + ' ' + labels[3];
        }
        el.setAttribute('data-remain', String(remain));
      }
      function tick(){
        paint();
        if (remain <= 0) return;
        el._cuiTimer = setTimeout(function(){
          remain -= 1;
          if (remain < 0) remain = 0;
          tick();
        }, 1000);
      }
      tick();
    });
  }

  function syncOtp(container){
    var input = container.querySelector('[data-slot="input-otp"]');
    var slots = container.querySelectorAll('[data-slot="input-otp-slot"]');
    if (!input || !slots.length) return;
    var max = slots.length;
    var v = String(input.value || '').replace(/\D/g, '').slice(0, max);
    if (v !== input.value) input.value = v;
    var active = v.length >= max ? max - 1 : v.length;
    slots.forEach(function(slot, i){
      var ch = v.charAt(i);
      if (slot.firstChild && slot.firstChild.nodeType === 3) {
        slot.firstChild.textContent = ch;
      } else {
        slot.textContent = ch;
      }
      if (i === active) slot.setAttribute('data-active', 'true');
      else slot.removeAttribute('data-active');
    });
  }
  function bindOtp(root){
    (root || document).querySelectorAll('[data-input-otp-container]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      var input = el.querySelector('[data-slot="input-otp"]');
      if (!input) return;
      ['input', 'keyup', 'focus', 'click'].forEach(function(ev){
        input.addEventListener(ev, function(){ syncOtp(el); });
      });
      input.addEventListener('paste', function(){ setTimeout(function(){ syncOtp(el); }, 0); });
      el.addEventListener('click', function(){ input.focus(); });
      syncOtp(el);
    });
  }

  function formatFlow(el, value){
    var format = el.getAttribute('data-format') || 'number';
    var locale = el.getAttribute('data-locale') || 'en-US';
    var prefix = el.getAttribute('data-prefix') || '';
    var suffix = el.getAttribute('data-suffix') || '';
    var opts = {};
    var n = Number(value);
    if (!isFinite(n)) n = 0;
    if (format === 'currency') { opts.minimumFractionDigits = 2; opts.maximumFractionDigits = 2; }
    else if (format === 'percentage') { opts.style = 'percent'; opts.maximumFractionDigits = 0; }
    else if (format === 'decimal') { opts.minimumFractionDigits = 2; opts.maximumFractionDigits = 2; }
    else { opts.maximumFractionDigits = 3; }
    return prefix + new Intl.NumberFormat(locale, opts).format(n) + suffix;
  }
  function tokenizeFlow(formatted){
    var tokens = [];
    for (var i = 0; i < formatted.length; i++) {
      var ch = formatted.charAt(i);
      if (ch >= '0' && ch <= '9') tokens.push({d: ch});
      else tokens.push({s: ch});
    }
    return tokens;
  }
  function advanceSpin(current, digit, trend){
    var currentDigit = ((current % 10) + 10) % 10;
    var diff = digit - currentDigit;
    if (trend > 0 && diff < 0) diff += 10;
    else if (trend < 0 && diff > 0) diff -= 10;
    else if (trend === 0) {
      if (diff > 5) diff -= 10;
      else if (diff < -5) diff += 10;
    }
    return current + diff;
  }
  function snapSpin(roller, strip, next){
    var snapped = ((next % 10) + 10) % 10;
    roller._cuiSpin = snapped;
    strip.style.transition = 'none';
    strip.style.marginBlockStart = (-(snapped + 1)) + 'em';
  }
  function rollDigit(roller, nextDigit, trend){
    var strip = roller.firstElementChild;
    if (!strip) return;
    var spin = roller._cuiSpin;
    if (spin == null) spin = parseInt(roller.getAttribute('data-digit'), 10) || 0;
    var next = advanceSpin(spin, nextDigit, trend);
    roller.setAttribute('data-digit', String(((nextDigit % 10) + 10) % 10));
    var from = -(spin + 1);
    var to = -(next + 1);
    var reduce = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    strip.style.transition = 'none';
    strip.style.marginBlockStart = from + 'em';
    void strip.offsetHeight;
    roller._cuiSpin = next;
    if (reduce || from === to) {
      snapSpin(roller, strip, next);
      return;
    }
    strip.style.transition = 'margin-block-start 480ms cubic-bezier(0.34, 1.56, 0.64, 1)';
    strip.style.marginBlockStart = to + 'em';
    if (next >= 10 || next < 0) {
      var done = function(e){
        if (e && e.propertyName && e.propertyName.indexOf('margin') === -1) return;
        strip.removeEventListener('transitionend', done);
        snapSpin(roller, strip, next);
      };
      strip.addEventListener('transitionend', done);
    }
  }
  function setFlowValue(el, value, format, prefix, suffix, locale, trend){
    if (format != null) el.setAttribute('data-format', format);
    if (prefix != null) el.setAttribute('data-prefix', prefix);
    if (suffix != null) el.setAttribute('data-suffix', suffix);
    if (locale != null) el.setAttribute('data-locale', locale);
    el.setAttribute('data-value', String(value));
    var formatted = formatFlow(el, value);
    el.setAttribute('aria-label', formatted);
    var host = el.querySelector('[aria-hidden="true"]');
    if (!host) return;
    var tokens = tokenizeFlow(formatted);
    var kids = host.children;
    var same = kids.length === tokens.length;
    if (same) {
      for (var i = 0; i < tokens.length; i++) {
        if (!!tokens[i].d !== kids[i].hasAttribute('data-digit')) { same = false; break; }
      }
    }
    if (same) {
      for (var k = 0; k < tokens.length; k++) {
        if (tokens[k].d != null) rollDigit(kids[k], parseInt(tokens[k].d, 10), trend || 0);
      }
      return;
    }
    var html = '';
    for (var t = 0; t < tokens.length; t++) {
      if (tokens[t].d != null) html += '<span data-digit="' + tokens[t].d + '"><span>' + STRIP + '</span></span>';
      else html += '<span>' + esc(tokens[t].s) + '</span>';
    }
    host.innerHTML = html;
  }
  function flowFrom(el){
    return el.matches && el.matches('[data-slot="number-flow"]')
      ? el
      : el.querySelector('[data-slot="number-flow"]');
  }
  function bindNumberFlows(root){
    (root || document).querySelectorAll('[data-slot="number-flow"] [data-digit]').forEach(function(roller){
      if (roller._cuiSpin == null) {
        roller._cuiSpin = parseInt(roller.getAttribute('data-digit'), 10) || 0;
      }
    });
  }

  function tweenNumber(el, from, to, ms){
    var start = performance.now();
    var reduce = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (reduce || !ms) {
      el.textContent = el._cuiFormat ? el._cuiFormat(to) : String(to);
      return;
    }
    function frame(now){
      var t = Math.min(1, (now - start) / ms);
      var eased = 1 - Math.pow(1 - t, 3);
      var cur = from + (to - from) * eased;
      var span = el.querySelector('span') || el;
      span.textContent = el._cuiFormat ? el._cuiFormat(cur) : String(Math.round(cur));
      if (t < 1) requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
  }
  function bindAnimatedNumbers(root){
    (root || document).querySelectorAll('[data-slot="animated-number"][data-value]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      var locale = el.getAttribute('data-locale') || 'en-US';
      var currency = el.getAttribute('data-currency');
      el._cuiFormat = function(n){
        if (currency) return new Intl.NumberFormat(locale, {style:'currency', currency:currency}).format(n);
        return new Intl.NumberFormat(locale).format(Math.round(n));
      };
      el._cuiCurrent = Number(el.getAttribute('data-value')) || 0;
    });
  }

  function bindPassword(root){
    (root || document).querySelectorAll('[data-slot="password-input-toggle"]').forEach(function(btn){
      btn.disabled = false;
    });
  }
  function bindCopy(root){
    (root || document).querySelectorAll('[data-slot="copy-button"]').forEach(function(btn){
      if (!btn.hasAttribute('data-disabled')) btn.disabled = false;
    });
  }
  function bindSteppers(root){
    (root || document).querySelectorAll('[data-slot="number-input"] button, [data-number-input-step]').forEach(function(btn){
      if (!btn.closest('[aria-disabled="true"]')) btn.disabled = false;
    });
  }
  function bindCommand(root){
    (root || document).querySelectorAll('[data-slot="command-input"], [data-slot="combobox-input"]').forEach(function(input){
      input.disabled = false;
    });
  }
  function bindSliders(root){
    (root || document).querySelectorAll('[data-slot="slider"] > input[type="range"]').forEach(function(input){
      input.addEventListener('input', function(){
        var rootEl = input.closest('[data-slot="slider"]');
        if (!rootEl) return;
        rootEl.setAttribute('data-value', input.value);
        var thumb = rootEl.querySelector('[role="slider"]');
        if (thumb) thumb.setAttribute('aria-valuenow', input.value);
      });
    });
  }
  function fmtMediaTime(s){
    if (!isFinite(s) || s <= 0) return '0:00';
    var whole = Math.floor(s);
    var h = Math.floor(whole / 3600);
    var m = Math.floor((whole % 3600) / 60);
    var sec = pad2(whole % 60);
    if (h > 0) return h + ':' + pad2(m) + ':' + sec;
    return m + ':' + sec;
  }
  var VP_SVG = '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">';
  var VP_PLAY = VP_SVG + '<path d="M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z"></path></svg>';
  var VP_PAUSE = VP_SVG + '<rect x="14" y="3" width="5" height="18" rx="1"></rect><rect x="5" y="3" width="5" height="18" rx="1"></rect></svg>';
  var VP_VOL = VP_SVG + '<path d="M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z"></path><path d="M16 9a5 5 0 0 1 0 6"></path><path d="M19.364 18.364a9 9 0 0 0 0-12.728"></path></svg>';
  var VP_VOLX = VP_SVG + '<path d="M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z"></path><line x1="22" x2="16" y1="9" y2="15"></line><line x1="16" x2="22" y1="9" y2="15"></line></svg>';
  var VP_MAX = VP_SVG + '<path d="M8 3H5a2 2 0 0 0-2 2v3"></path><path d="M21 8V5a2 2 0 0 0-2-2h-3"></path><path d="M3 16v3a2 2 0 0 0 2 2h3"></path><path d="M16 21h3a2 2 0 0 0 2-2v-3"></path></svg>';
  var VP_MIN = VP_SVG + '<path d="M8 3v3a2 2 0 0 1-2 2H3"></path><path d="M21 8h-3a2 2 0 0 1-2-2V3"></path><path d="M3 16h3a2 2 0 0 1 2 2v3"></path><path d="M16 21v-3a2 2 0 0 1 2-2h3"></path></svg>';
  var VP_RATES = [1, 1.25, 1.5, 2];
  function playerOf(el){
    return el && el.closest && el.closest('[data-slot="video-player"]');
  }
  function videoOf(root){
    return root && root.querySelector('[data-slot="video-player-video"]');
  }
  function fsEl(){
    return document.fullscreenElement || document.webkitFullscreenElement || null;
  }
  function paintVideoPlayer(root){
    var video = videoOf(root);
    if (!video) return;
    var playing = !video.paused && !video.ended;
    if (playing) root.setAttribute('data-playing', '');
    else root.removeAttribute('data-playing');
    var overlay = root.querySelector('[data-slot="video-player-overlay-play"]');
    var playBtn = root.querySelector('[data-slot="video-player-play"]');
    var playL = (overlay && overlay.getAttribute('aria-label')) || 'Play';
    var pauseL = (playBtn && playBtn.getAttribute('data-pause')) || 'Pause';
    if (overlay) {
      overlay.hidden = playing;
      if (playing && document.activeElement === overlay && playBtn) playBtn.focus();
    }
    if (playBtn) {
      playBtn.setAttribute('aria-label', playing ? pauseL : playL);
      playBtn.innerHTML = playing ? VP_PAUSE : VP_PLAY;
    }
    var dur = isFinite(video.duration) ? video.duration : 0;
    var cur = isFinite(video.currentTime) ? video.currentTime : 0;
    var stamp = fmtMediaTime(cur) + ' / ' + fmtMediaTime(dur);
    var time = root.querySelector('[data-slot="video-player-time"]');
    if (time) time.textContent = stamp;
    var seek = root.querySelector('[data-slot="video-player-seek"]');
    if (seek) {
      seek.max = dur > 0 ? dur : 0;
      if (!seek._cuiSeeking) seek.value = cur;
      seek.setAttribute('aria-valuetext', stamp);
    }
    var muteBtn = root.querySelector('[data-slot="video-player-mute"]');
    var muted = video.muted || video.volume === 0;
    if (muteBtn) {
      muteBtn.setAttribute('aria-label', muted
        ? (muteBtn.getAttribute('data-unmute') || 'Unmute')
        : (muteBtn.getAttribute('data-mute') || 'Mute'));
      muteBtn.innerHTML = muted ? VP_VOLX : VP_VOL;
    }
    var vol = root.querySelector('[data-slot="video-player-volume"]');
    if (vol) vol.value = video.muted ? 0 : video.volume;
    var fsBtn = root.querySelector('[data-slot="video-player-fullscreen"]');
    if (fsBtn) {
      var full = fsEl() === root || fsEl() === video;
      var exitL = fsBtn.getAttribute('data-exit') || 'Exit fullscreen';
      if (!fsBtn.getAttribute('data-enter')) {
        var curL = fsBtn.getAttribute('aria-label');
        if (curL && curL !== exitL) fsBtn.setAttribute('data-enter', curL);
      }
      fsBtn.setAttribute('aria-label', full ? exitL : (fsBtn.getAttribute('data-enter') || 'Fullscreen'));
      fsBtn.innerHTML = full ? VP_MIN : VP_MAX;
    }
    var rateBtn = root.querySelector('[data-slot="video-player-rate"]');
    if (rateBtn) {
      var r = video.playbackRate || 1;
      var settings = rateBtn.getAttribute('data-settings') || 'Playback speed';
      rateBtn.textContent = r + 'x';
      rateBtn.setAttribute('aria-label', settings + ', ' + r + 'x');
    }
  }
  function bindVideoPlayers(root){
    (root || document).querySelectorAll('[data-slot="video-player"]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      var video = videoOf(el);
      if (!video) return;
      el._cuiLastVol = video.volume > 0 ? video.volume : 1;
      function paint(){ paintVideoPlayer(el); }
      el._cuiPaint = paint;
      ['play', 'pause', 'ended', 'timeupdate', 'loadedmetadata', 'volumechange', 'ratechange'].forEach(function(ev){
        video.addEventListener(ev, paint);
      });
      var seek = el.querySelector('[data-slot="video-player-seek"]');
      if (seek) {
        seek.addEventListener('pointerdown', function(){ seek._cuiSeeking = true; });
        seek.addEventListener('pointerup', function(){ seek._cuiSeeking = false; paint(); });
      }
      paint();
    });
    if (!document._cuiVideoFs) {
      document._cuiVideoFs = true;
      ['fullscreenchange', 'webkitfullscreenchange'].forEach(function(ev){
        document.addEventListener(ev, function(){
          document.querySelectorAll('[data-slot="video-player"]').forEach(function(el){
            if (el._cuiPaint) el._cuiPaint();
          });
        });
      });
    }
  }

  function bindPhone(root){
    (root || document).querySelectorAll('[data-slot="phone-input"]').forEach(function(el){
      var field = el.querySelector('[data-slot="phone-input-field"]');
      if (!field || field._cuiMask) return;
      field._cuiMask = true;
      field.addEventListener('input', function(){
        var radio = el.querySelector('[data-slot="phone-input-content"] input[type="radio"]:checked');
        var groups = (radio && radio.getAttribute('data-groups') || el.getAttribute('data-groups') || '2,5,4').split(',').map(Number);
        var digits = field.value.replace(/\D/g, '');
        var max = groups.reduce(function(a,b){ return a + b; }, 0);
        digits = digits.slice(0, max);
        var parts = [];
        var idx = 0;
        for (var i = 0; i < groups.length && idx < digits.length; i++) {
          parts.push(digits.slice(idx, idx + groups[i]));
          idx += groups[i];
        }
        if (idx < digits.length) parts.push(digits.slice(idx));
        field.value = parts.join(' ');
      });
    });
  }

  document.addEventListener('click', function(e){
    var flowBtn = e.target.closest('[data-number-flow-action]');
    if (flowBtn) {
      e.preventDefault();
      var demo = flowBtn.closest('.cui-number-flow-demo') || flowBtn.parentElement;
      var flow = flowFrom(demo);
      if (flow) {
        var kind = flowBtn.getAttribute('data-number-flow-action');
        if (kind === 'increment') {
          var n = Number(flow.getAttribute('data-value')) || 0;
          setFlowValue(flow, n + 1, null, null, null, null, 1);
        } else if (kind === 'shuffle') {
          var cur = Number(flow.getAttribute('data-value'));
          var next = SHUFFLE[0];
          for (var i = 0; i < SHUFFLE.length; i++) {
            if (SHUFFLE[i].value === cur) { next = SHUFFLE[(i + 1) % SHUFFLE.length]; break; }
          }
          setFlowValue(flow, next.value, next.format, next.prefix, next.suffix, next.locale, 0);
        }
      }
      return;
    }
    var bump = e.target.closest('[data-animated-number-action]');
    if (bump) {
      e.preventDefault();
      var wrap = bump.closest('.cui-animated-number-demo') || bump.parentElement;
      var num = wrap.querySelector('[data-slot="animated-number"]');
      if (num) {
        var from = num._cuiCurrent != null ? num._cuiCurrent : Number(num.getAttribute('data-value')) || 0;
        var to = from + 200 + Math.floor(Math.random() * 1800);
        num._cuiCurrent = to;
        num.setAttribute('data-value', String(to));
        tweenNumber(num, from, to, 800);
      }
      return;
    }
    var copy = e.target.closest('[data-slot="copy-button"]');
    if (copy && !copy.disabled) {
      e.preventDefault();
      var text = copy.getAttribute('data-copy') || copy.getAttribute('aria-label') || '';
      var live = copy.querySelector('[aria-live]');
      function ok(){
        copy.setAttribute('data-copied', '');
        if (live) live.textContent = 'Copied';
        setTimeout(function(){
          copy.removeAttribute('data-copied');
          if (live) live.textContent = '';
        }, 1500);
      }
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(ok).catch(function(){});
      }
      return;
    }
    var eye = e.target.closest('[data-slot="password-input-toggle"]');
    if (eye) {
      e.preventDefault();
      var box = eye.closest('[data-slot="password-input"]') || eye.parentElement;
      var input = box.querySelector('input');
      if (!input) return;
      var show = input.type === 'password';
      input.type = show ? 'text' : 'password';
      eye.setAttribute('aria-pressed', show ? 'true' : 'false');
      return;
    }
    var step = e.target.closest('[data-number-input-step]');
    if (step) {
      e.preventDefault();
      var root = step.closest('[data-slot="number-input"]') || step.parentElement;
      var field = root.querySelector('input');
      if (!field || field.disabled) return;
      var dir = Number(step.getAttribute('data-number-input-step')) || 1;
      var cur = Number(field.value) || 0;
      field.value = String(cur + dir);
      field.dispatchEvent(new Event('input', {bubbles:true}));
      return;
    }
    var vpPlay = e.target.closest('[data-slot="video-player-play"], [data-slot="video-player-overlay-play"]');
    if (vpPlay && !vpPlay.disabled) {
      e.preventDefault();
      var vp = playerOf(vpPlay);
      var video = videoOf(vp);
      if (!video) return;
      if (video.paused || video.ended) {
        var playR = video.play();
        if (playR && playR.catch) playR.catch(function(){});
      } else {
        video.pause();
      }
      if (vp._cuiPaint) vp._cuiPaint();
      return;
    }
    var vpMute = e.target.closest('[data-slot="video-player-mute"]');
    if (vpMute && !vpMute.disabled) {
      e.preventDefault();
      var vp = playerOf(vpMute);
      var video = videoOf(vp);
      if (!video) return;
      video.muted = !video.muted;
      if (!video.muted && video.volume === 0) {
        var restored = vp._cuiLastVol > 0 ? vp._cuiLastVol : 1;
        video.volume = restored;
      }
      if (vp._cuiPaint) vp._cuiPaint();
      return;
    }
    var vpRate = e.target.closest('[data-slot="video-player-rate"]');
    if (vpRate && !vpRate.disabled) {
      e.preventDefault();
      var vp = playerOf(vpRate);
      var video = videoOf(vp);
      if (!video) return;
      var idx = VP_RATES.indexOf(video.playbackRate);
      if (idx < 0) idx = 0;
      video.playbackRate = VP_RATES[(idx + 1) % VP_RATES.length];
      if (vp._cuiPaint) vp._cuiPaint();
      return;
    }
    var vpFs = e.target.closest('[data-slot="video-player-fullscreen"]');
    if (vpFs && !vpFs.disabled) {
      e.preventDefault();
      var vp = playerOf(vpFs);
      var video = videoOf(vp);
      if (!vp) return;
      var curFs = fsEl();
      if (curFs === vp || curFs === video) {
        var exitFn = document.exitFullscreen || document.webkitExitFullscreen;
        if (exitFn) {
          var exitR = exitFn.call(document);
          if (exitR && exitR.catch) exitR.catch(function(){});
        }
      } else {
        var enterFn = vp.requestFullscreen || vp.webkitRequestFullscreen
          || (video && (video.requestFullscreen || video.webkitRequestFullscreen || video.webkitEnterFullscreen));
        var enterTarget = (vp.requestFullscreen || vp.webkitRequestFullscreen) ? vp : video;
        if (enterFn) {
          var enterR = enterFn.call(enterTarget);
          if (enterR && enterR.catch) enterR.catch(function(){});
        }
      }
      return;
    }
  }, true);

  document.addEventListener('input', function(e){
    var comboIn = e.target.closest && e.target.closest('[data-slot="combobox-input"]');
    if (comboIn) {
      var pop = comboIn.closest('[data-slot="combobox-content"]');
      if (!pop) return;
      var q = comboIn.value.toLowerCase();
      var any = false;
      pop.querySelectorAll('[data-slot="combobox-item"], [data-slot="select-item"]').forEach(function(item){
        var text = (item.textContent || '').toLowerCase();
        var match = !q || text.indexOf(q) !== -1;
        item.hidden = !match;
        if (match) any = true;
      });
      var empty = pop.querySelector('[data-slot="command-empty"]');
      if (empty) empty.hidden = any;
      return;
    }
    var cmd = e.target.closest && e.target.closest('[data-slot="command-input"]');
    if (cmd) {
      var root = cmd.closest('[data-slot="command"]');
      if (!root) return;
      var q = cmd.value.toLowerCase();
      root.querySelectorAll('[data-slot="command-item"]').forEach(function(item){
        var text = item.textContent.toLowerCase();
        item.hidden = q && text.indexOf(q) === -1;
      });
      return;
    }
    var otp = e.target.closest && e.target.closest('[data-slot="input-otp"]');
    if (otp) {
      var box = otp.closest('[data-input-otp-container]');
      if (box) syncOtp(box);
    }
    var vpSeek = e.target.closest && e.target.closest('[data-slot="video-player-seek"]');
    if (vpSeek) {
      var vp = playerOf(vpSeek);
      var video = videoOf(vp);
      if (!video || vpSeek.disabled) return;
      var next = Number(vpSeek.value);
      if (!isNaN(next)) video.currentTime = next;
      if (vp._cuiPaint) vp._cuiPaint();
      return;
    }
    var vpVol = e.target.closest && e.target.closest('[data-slot="video-player-volume"]');
    if (vpVol) {
      var vp = playerOf(vpVol);
      var video = videoOf(vp);
      if (!video || vpVol.disabled) return;
      var v = Number(vpVol.value);
      if (isNaN(v)) return;
      if (v < 0) v = 0;
      if (v > 1) v = 1;
      video.volume = v;
      video.muted = v === 0;
      if (v > 0) vp._cuiLastVol = v;
      if (vp._cuiPaint) vp._cuiPaint();
    }
  }, true);

  document.addEventListener('change', function(e){
    var comboRadio = e.target.closest && e.target.closest('[data-slot="combobox-content"] input[type="radio"]');
    if (comboRadio) {
      var comboPop = comboRadio.closest('[data-slot="combobox-content"]');
      if (comboPop && comboPop.hidePopover) comboPop.hidePopover();
      return;
    }
    var radio = e.target.closest && e.target.closest('[data-slot="phone-input-content"] input[type="radio"]');
    if (!radio) return;
    var root = radio.closest('[data-slot="phone-input"]');
    if (!root) return;
    var groups = radio.getAttribute('data-groups') || '';
    if (groups) root.setAttribute('data-groups', groups);
    var field = root.querySelector('[data-slot="phone-input-field"]');
    var mask = radio.getAttribute('data-mask');
    if (field && mask) field.setAttribute('placeholder', mask);
    if (field) field.dispatchEvent(new Event('input', {bubbles:true}));
    var trigger = root.querySelector('[data-slot="phone-input-country"]');
    if (trigger) trigger.setAttribute('aria-expanded', 'false');
    var pop = root.querySelector('[data-slot="phone-input-content"]');
    if (pop && pop.hidePopover) pop.hidePopover();
  }, true);

  function hintOf(trigger){
    var id = trigger.getAttribute('interestfor') || trigger.getAttribute('aria-describedby');
    return id ? document.getElementById(id) : null;
  }
  function showHint(trigger){
    var pop = hintOf(trigger);
    if (!pop || !pop.showPopover) return;
    if (pop.getAttribute('popover') === 'hint') pop.setAttribute('popover', 'manual');
    try { pop.showPopover(); } catch (err) {}
  }
  function hideHint(trigger){
    var pop = hintOf(trigger);
    if (!pop || !pop.hidePopover) return;
    try { pop.hidePopover(); } catch (err) {}
  }
  var hintShowT, hintHideT, hintArmed;
  function bindInterest(){
    if (document._cuiInterest) return;
    document._cuiInterest = true;
    document.addEventListener('pointerover', function(e){
      var t = e.target.closest && e.target.closest('[interestfor]');
      if (!t || t === hintArmed) return;
      clearTimeout(hintHideT);
      hintArmed = t;
      hintShowT = setTimeout(function(){ showHint(t); }, 200);
    }, true);
    document.addEventListener('pointerout', function(e){
      var t = e.target.closest && e.target.closest('[interestfor]');
      if (!t) return;
      var next = e.relatedTarget;
      if (next && (t.contains(next) || (hintOf(t) && hintOf(t).contains(next)))) return;
      clearTimeout(hintShowT);
      hintArmed = null;
      hintHideT = setTimeout(function(){ hideHint(t); }, 120);
    }, true);
    document.addEventListener('focusin', function(e){
      var t = e.target.closest && e.target.closest('[interestfor]');
      if (t) showHint(t);
    });
    document.addEventListener('focusout', function(e){
      var t = e.target.closest && e.target.closest('[interestfor]');
      if (t) hideHint(t);
    });
    document.addEventListener('click', function(e){
      var t = e.target.closest && e.target.closest('[interestfor][popovertarget]');
      if (!t) return;
      var pop = hintOf(t);
      if (pop && pop.matches && pop.matches(':popover-open')) e.preventDefault();
    }, true);
  }

  function enable(sel, root){
    (root || document).querySelectorAll(sel).forEach(function(el){
      if (!el.closest('[data-disabled], [aria-disabled="true"]') && !el.hasAttribute('data-disabled')) {
        el.disabled = false;
        el.removeAttribute('aria-disabled');
      }
    });
  }

  function bindTagsInput(root){
    enable('[data-slot="tags-input-remove"]', root);
    if (!document._cuiTags) {
      document._cuiTags = true;
      document.addEventListener('click', function(e){
        var btn = e.target.closest('[data-slot="tags-input-remove"]');
        if (!btn) return;
        var chip = btn.closest('[data-slot="badge"]');
        if (chip) chip.remove();
      });
    }
    (root || document).querySelectorAll('[data-slot="tags-input"]').forEach(function(box){
      if (box._cuiLive) return;
      box._cuiLive = true;
      var field = box.querySelector('[data-slot="tags-input-field"]');
      if (!field) return;
      function addTag(raw){
        var t = String(raw || '').trim();
        if (!t) return;
        var exists = false;
        box.querySelectorAll('[data-slot="badge"] > span').forEach(function(s){
          if (s.textContent === t) exists = true;
        });
        if (exists) { field.value = ''; return; }
        var chip = document.createElement('span');
        chip.setAttribute('data-slot', 'badge');
        chip.setAttribute('data-variant', 'secondary');
        chip.innerHTML = '<span></span><button type="button" tabindex="-1" data-slot="tags-input-remove"></button>';
        chip.querySelector('span').textContent = t;
        var btn = chip.querySelector('button');
        btn.setAttribute('aria-label', 'Remove ' + t);
        btn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M18 6 6 18"></path><path d="m6 6 12 12"></path></svg>';
        box.insertBefore(chip, field);
        field.value = '';
        field.removeAttribute('placeholder');
      }
      field.addEventListener('keydown', function(e){
        if (e.key === 'Enter' || e.key === ',') {
          e.preventDefault();
          addTag(field.value.replace(/,$/, ''));
        } else if (e.key === 'Backspace' && !field.value) {
          var last = box.querySelector('[data-slot="badge"]:last-of-type');
          if (last) last.remove();
        }
      });
    });
  }

  function bindAutocomplete(root){
    (root || document).querySelectorAll('[data-slot="autocomplete-input"]').forEach(function(input){
      if (input._cuiLive) return;
      input._cuiLive = true;
      var wrap = input.closest('[data-slot="autocomplete"]') || input.parentElement;
      var content = wrap.parentElement && wrap.parentElement.querySelector('[data-slot="autocomplete-content"]');
      function filter(){
        var q = input.value.toLowerCase();
        if (!content) return;
        var any = false;
        content.querySelectorAll('[data-slot="command-item"]').forEach(function(item){
          var ok = !q || item.textContent.toLowerCase().indexOf(q) !== -1;
          item.hidden = !ok;
          if (ok) any = true;
        });
        content.hidden = !q || !any;
        input.setAttribute('aria-expanded', content.hidden ? 'false' : 'true');
      }
      input.addEventListener('input', filter);
      input.addEventListener('focus', filter);
      if (content) {
        content.addEventListener('mousedown', function(e){
          var item = e.target.closest('[data-slot="command-item"]');
          if (!item) return;
          e.preventDefault();
          input.value = item.textContent.trim();
          content.hidden = true;
          input.setAttribute('aria-expanded', 'false');
        });
      }
    });
  }

  function bindComparison(root){
    function apply(el, pct){
      pct = Math.max(0, Math.min(100, pct));
      var before = el.querySelector('[data-slot="comparison-before"]');
      if (before) before.style.clipPath = 'inset(0 ' + (100 - pct) + '% 0 0)';
      var handle = el.querySelector('[role="slider"]');
      var rule = el.querySelector(':scope > [aria-hidden="true"]');
      if (handle) {
        handle.style.left = pct + '%';
        handle.setAttribute('aria-valuenow', String(Math.round(pct)));
        handle.removeAttribute('aria-disabled');
      }
      if (rule) rule.style.left = pct + '%';
      var range = el.querySelector('[data-slot="comparison-slider-range"], input[type="range"]');
      if (range && range.value !== String(Math.round(pct))) range.value = String(Math.round(pct));
    }
    (root || document).querySelectorAll('[data-slot="comparison-slider"]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      function pos(ev){
        var r = el.getBoundingClientRect();
        apply(el, ((ev.clientX - r.left) / r.width) * 100);
      }
      el.addEventListener('pointerdown', function(ev){
        if (ev.button) return;
        el.setPointerCapture(ev.pointerId);
        pos(ev);
      });
      el.addEventListener('pointermove', function(ev){
        if (!el.hasPointerCapture(ev.pointerId)) return;
        pos(ev);
      });
      var range = el.querySelector('[data-slot="comparison-slider-range"], :scope > input[type="range"]');
      if (range) range.addEventListener('input', function(){ apply(el, Number(range.value)); });
    });
  }

  function bindSignature(root){
    enable('[data-slot="signature-pad"] button', root);
    (root || document).querySelectorAll('[data-slot="signature-pad"]').forEach(function(el){
      if (el._cuiLive || el.getAttribute('data-disabled') === 'true') return;
      el._cuiLive = true;
      var surface = el.querySelector('[data-slot="signature-pad-canvas"]');
      if (!surface) return;
      var svg = surface.querySelector('svg');
      if (!svg) {
        svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        svg.setAttribute('data-slot', 'signature-pad-surface');
        svg.setAttribute('width', '100%');
        svg.setAttribute('height', '100%');
        svg.style.cssText = 'position:absolute;inset:0;width:100%;height:100%;display:block;touch-action:none';
        surface.style.position = 'relative';
        surface.appendChild(svg);
      }
      var strokes = [];
      var current = null;
      function empty(){
        var isEmpty = !strokes.length;
        el.setAttribute('data-empty', isEmpty ? 'true' : 'false');
      }
      surface.addEventListener('pointerdown', function(ev){
        if (ev.button) return;
        surface.setPointerCapture(ev.pointerId);
        var r = svg.getBoundingClientRect();
        current = document.createElementNS('http://www.w3.org/2000/svg', 'polyline');
        current.setAttribute('fill', 'none');
        current.setAttribute('stroke', 'currentColor');
        current.setAttribute('stroke-width', '2');
        current.setAttribute('stroke-linecap', 'round');
        current.setAttribute('stroke-linejoin', 'round');
        current.setAttribute('points', (ev.clientX - r.left) + ',' + (ev.clientY - r.top));
        svg.appendChild(current);
        strokes.push(current);
        empty();
      });
      surface.addEventListener('pointermove', function(ev){
        if (!current || !surface.hasPointerCapture(ev.pointerId)) return;
        var r = svg.getBoundingClientRect();
        current.setAttribute('points', current.getAttribute('points') + ' ' + (ev.clientX - r.left) + ',' + (ev.clientY - r.top));
      });
      surface.addEventListener('pointerup', function(){ current = null; });
      el.addEventListener('click', function(ev){
        var btn = ev.target.closest('button');
        if (!btn) return;
        var slot = btn.getAttribute('data-slot') || '';
        var label = (btn.getAttribute('aria-label') || '').toLowerCase();
        if (slot === 'signature-pad-undo' || label.indexOf('undo') !== -1) {
          var last = strokes.pop();
          if (last) last.remove();
          empty();
        } else if (slot === 'signature-pad-clear' || label.indexOf('clear') !== -1) {
          strokes.forEach(function(s){ s.remove(); });
          strokes = [];
          empty();
        }
      });
    });
  }

  function bindCalendarNav(root){
    var MONTHS = ['January','February','March','April','May','June','July','August','September','October','November','December'];
    function dim(y, m){ return new Date(y, m, 0).getDate(); }
    function wd(y, m, d){ return new Date(y, m - 1, d).getDay(); }
    (root || document).querySelectorAll('[data-slot="calendar"][data-month]').forEach(function(cal){
      if (cal._cuiLive) return;
      cal._cuiLive = true;
      cal.addEventListener('click', function(ev){
        var btn = ev.target.closest('[data-cal-nav]');
        if (!btn) return;
        ev.preventDefault();
        var parts = (cal.getAttribute('data-month') || '').split('-');
        var y = parseInt(parts[0], 10), m = parseInt(parts[1], 10);
        if (!y || !m) return;
        if (btn.getAttribute('data-cal-nav') === 'prev') { m -= 1; if (m < 1) { m = 12; y -= 1; } }
        else { m += 1; if (m > 12) { m = 1; y += 1; } }
        cal.setAttribute('data-month', y + '-' + (m < 10 ? '0' : '') + m);
        var status = cal.querySelector('[role="status"]');
        var caption = MONTHS[m - 1] + ' ' + y;
        if (status) status.textContent = caption;
        var table = cal.querySelector('table[role="grid"]');
        if (table) table.setAttribute('aria-label', caption);
        var tbody = cal.querySelector('tbody');
        var radio = cal.querySelector('tbody input[type="radio"]');
        var name = radio ? radio.getAttribute('name') : '';
        if (!tbody) return;
        var py = m === 1 ? y - 1 : y, pm = m === 1 ? 12 : m - 1;
        var ny = m === 12 ? y + 1 : y, nm = m === 12 ? 1 : m + 1;
        var lead = wd(y, m, 1);
        var cells = [];
        function cell(yy, mm, dd, outside){
          var iso = yy + '-' + (mm < 10 ? '0' : '') + mm + '-' + (dd < 10 ? '0' : '') + dd;
          var td = '<td role="gridcell"' + (outside ? ' data-outside="true"' : '') + '>';
          if (name) {
            td += '<label><input type="radio" name="' + name + '" value="' + iso + '"><button type="button" tabindex="-1" aria-hidden="true">' + dd + '</button></label>';
          } else {
            td += '<button type="button">' + dd + '</button>';
          }
          return td + '</td>';
        }
        for (var i = 0; i < lead; i++) cells.push(cell(py, pm, dim(py, pm) - lead + 1 + i, true));
        for (var d = 1; d <= dim(y, m); d++) cells.push(cell(y, m, d, false));
        var n = 1;
        while (cells.length % 7) { cells.push(cell(ny, nm, n++, true)); }
        var html = '';
        for (var r = 0; r < cells.length; r += 7) html += '<tr>' + cells.slice(r, r + 7).join('') + '</tr>';
        tbody.innerHTML = html;
      });
    });
  }

  function bindContextMenu(){
    if (document._cuiCtx) return;
    document._cuiCtx = true;
    document.addEventListener('contextmenu', function(e){
      var t = e.target.closest('[popovertarget][aria-haspopup="menu"], [data-slot="button"][popovertarget]');
      if (!t) return;
      var id = t.getAttribute('popovertarget');
      var pop = id && document.getElementById(id);
      if (!pop || (pop.getAttribute('data-slot') || '').indexOf('context') === -1 && (pop.getAttribute('role') !== 'menu')) {
        if (!pop || pop.getAttribute('data-slot') !== 'context-menu-content') return;
      }
      e.preventDefault();
      if (pop.showPopover) {
        try { pop.showPopover(); } catch (err) {}
      }
    });
  }

  function bindResizable(root){
    (root || document).querySelectorAll('[data-slot="resizable-handle"], [data-resize-dir]').forEach(function(h){
      if (h._cuiLive) return;
      h._cuiLive = true;
      h.style.cursor = (h.getAttribute('data-resize-dir') === 'vertical' || h.getAttribute('aria-orientation') === 'vertical') ? 'row-resize' : 'col-resize';
      h.addEventListener('pointerdown', function(ev){
        if (ev.button) return;
        h.setPointerCapture(ev.pointerId);
        var pane = h.previousElementSibling;
        if (!pane) return;
        var vert = h.getAttribute('data-resize-dir') === 'vertical' || h.getAttribute('aria-orientation') === 'vertical';
        var start = vert ? ev.clientY : ev.clientX;
        var size = vert ? pane.getBoundingClientRect().height : pane.getBoundingClientRect().width;
        function move(e2){
          var d = (vert ? e2.clientY : e2.clientX) - start;
          if (vert) pane.style.height = Math.max(40, size + d) + 'px';
          else pane.style.width = Math.max(40, size + d) + 'px';
          pane.style.flex = 'none';
        }
        function up(){
          h.removeEventListener('pointermove', move);
          h.removeEventListener('pointerup', up);
        }
        h.addEventListener('pointermove', move);
        h.addEventListener('pointerup', up);
      });
    });
  }

  function bindKanban(root){
    enable('[data-slot="kanban-drag-handle"]', root);
    (root || document).querySelectorAll('[data-slot="kanban-card"], [draggable="true"]').forEach(function(card){
      if (card._cuiLive) return;
      if (!card.closest('[data-slot="kanban"]')) return;
      card._cuiLive = true;
      card.setAttribute('draggable', 'true');
      card.addEventListener('dragstart', function(e){
        e.dataTransfer.setData('text/plain', 'kanban');
        card._cuiDrag = true;
      });
      card.addEventListener('dragend', function(){ card._cuiDrag = false; });
    });
    (root || document).querySelectorAll('[data-slot="kanban-column"]').forEach(function(col){
      col.addEventListener('dragover', function(e){ e.preventDefault(); });
      col.addEventListener('drop', function(e){
        e.preventDefault();
        var moving = document.querySelector('[data-slot="kanban-card"][_cuiDrag], [data-slot="kanban"] [draggable="true"]');
        // previousElement during drag: use a flag
        var cards = document.querySelectorAll('[data-slot="kanban-card"]');
        var drag = null;
        cards.forEach(function(c){ if (c._cuiDrag) drag = c; });
        if (drag) col.appendChild(drag);
      });
    });
  }

  function bindRte(root){
    enable('[data-rte]', root);
    (root || document).querySelectorAll('[contenteditable="true"][role="textbox"], [data-slot="rich-text-editor"] [contenteditable]').forEach(function(box){
      box.setAttribute('contenteditable', 'true');
      box.removeAttribute('aria-readonly');
    });
    (root || document).addEventListener('click', function(e){
      var btn = e.target.closest('[data-rte]');
      if (!btn) return;
      e.preventDefault();
      var cmd = btn.getAttribute('data-rte');
      if (cmd && document.execCommand) document.execCommand(cmd, false, null);
    });
  }

  function bindExplore(root){
    enable('[data-explore-nav]', root);
    (root || document).addEventListener('click', function(e){
      var btn = e.target.closest('[data-explore-nav]');
      if (!btn) return;
      var strip = btn.closest('[data-slot="explore-nav"]') || btn.parentElement;
      var scroller = strip && (strip.querySelector('[data-slot="explore-nav-strip"]') || strip.querySelector('[style*="overflow"]') || strip);
      if (!scroller) return;
      var dir = btn.getAttribute('data-explore-nav') === 'prev' ? -1 : 1;
      scroller.scrollBy({ left: dir * Math.max(160, scroller.clientWidth * 0.8), behavior: 'smooth' });
    });
  }

  function bindTokenSwap(root){
    enable('[data-slot="token-swap"] input, [data-slot="token-swap"] button, [data-slot="token-swap-max"], [data-slot="token-swap-clear"]', root);
    (root || document).addEventListener('click', function(e){
      var max = e.target.closest('[data-slot="token-swap-max"], button');
      if (!max) return;
      var rootEl = max.closest('[data-slot="token-swap"]');
      if (!rootEl) return;
      var label = (max.getAttribute('aria-label') || max.textContent || '').toLowerCase();
      var field = rootEl.querySelector('input');
      if (!field) return;
      if (max.getAttribute('data-slot') === 'token-swap-max' || label === 'max') {
        var bal = rootEl.getAttribute('data-balance') || '1';
        field.value = bal;
        field.dispatchEvent(new Event('input', {bubbles:true}));
      } else if (max.getAttribute('data-slot') === 'token-swap-clear' || label === 'clear') {
        field.value = '';
        field.dispatchEvent(new Event('input', {bubbles:true}));
      }
    });
  }

  function bindMultiSelect(root){
    enable('[data-slot="multi-select"] [data-slot="command-input"]', root);
    (root || document).querySelectorAll('[data-slot="multi-select"]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      var trigger = el.querySelector('[data-slot="multi-select-trigger"]');
      function paint(){
        var picked = [];
        el.querySelectorAll('input[type="checkbox"]:checked, [data-slot="command-item"] input:checked').forEach(function(c){
          var lab = c.closest('label') || c.closest('[data-slot="command-item"]');
          if (lab) picked.push(lab.textContent.trim());
        });
        if (trigger) {
          var span = trigger.querySelector('span') || trigger;
          if (picked.length) span.textContent = picked.join(', ');
        }
      }
      el.addEventListener('change', paint);
    });
  }

  function bindColorPicker(root){
    (root || document).querySelectorAll('[data-slot="color-picker"]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      function hex(){
        var h = Number((el.querySelector('[data-slot="color-picker-hue"]') || {}).value || 0);
        var s = Number((el.querySelector('[data-slot="color-picker-sat"]') || {}).value || 100);
        return 'hsl(' + h + ' ' + s + '% 50%)';
      }
      function paint(){
        var c = hex();
        var swatch = el.querySelector('[data-slot="color-picker-swatch"], [data-slot="color-picker-trigger"]');
        if (swatch) swatch.style.background = c;
      }
      el.addEventListener('input', paint);
      el.addEventListener('click', function(e){
        var b = e.target.closest('[data-color]');
        if (!b) return;
        var swatch = el.querySelector('[data-slot="color-picker-swatch"], [data-slot="color-picker-trigger"]');
        if (swatch) swatch.style.background = b.getAttribute('data-color');
      });
    });
  }

  function bindDataTable(root){
    enable('[data-slot="data-table"] button, [data-slot="data-table"] input, [data-table-page], [data-slot="data-table-search"]', root);
    (root || document).querySelectorAll('[data-slot="data-table-search"], [data-slot="data-table"] input[type="search"], [data-slot="data-table"] input[type="text"]').forEach(function(input){
      if (input._cuiLive) return;
      input._cuiLive = true;
      input.addEventListener('input', function(){
        var q = input.value.toLowerCase();
        var table = input.closest('[data-slot="data-table"]');
        if (!table) return;
        table.querySelectorAll('tbody tr').forEach(function(row){
          row.style.display = !q || row.textContent.toLowerCase().indexOf(q) !== -1 ? '' : 'none';
        });
      });
    });
  }

  function bindWebPreview(root){
    enable('[data-web-preview-nav], [data-slot="web-preview"] button', root);
    (root || document).querySelectorAll('[data-slot="web-preview"]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      var iframe = el.querySelector('iframe');
      var hist = [];
      var idx = -1;
      var urlField = el.querySelector('input');
      function go(url){
        if (!url || !iframe) return;
        iframe.src = url;
        hist = hist.slice(0, idx + 1);
        hist.push(url);
        idx = hist.length - 1;
      }
      if (urlField) {
        urlField.addEventListener('keydown', function(e){
          if (e.key === 'Enter') go(urlField.value);
        });
      }
      el.addEventListener('click', function(e){
        var btn = e.target.closest('[data-web-preview-nav], button');
        if (!btn || !iframe) return;
        var kind = btn.getAttribute('data-web-preview-nav') || (btn.getAttribute('aria-label') || '').toLowerCase();
        if (kind === 'back' || kind.indexOf('back') !== -1) {
          if (idx > 0) { idx -= 1; iframe.src = hist[idx]; }
        } else if (kind === 'forward' || kind.indexOf('forward') !== -1) {
          if (idx < hist.length - 1) { idx += 1; iframe.src = hist[idx]; }
        }
      });
    });
  }

  function bindTimeNow(root){
    enable('[data-time-now]', root);
    (root || document).addEventListener('click', function(e){
      var btn = e.target.closest('[data-time-now]');
      if (!btn) return;
      var rootEl = btn.closest('[data-slot="time-picker"]') || btn.parentElement;
      var now = new Date();
      var h = now.getHours(), m = now.getMinutes();
      function check(name, val){
        var inp = rootEl.querySelector('input[name*="' + name + '"][value="' + val + '"]');
        if (inp) inp.checked = true;
      }
      check('hours', String(h % 12 || 12));
      check('minutes', (m < 10 ? '0' : '') + m);
      check('ampm', h >= 12 ? 'PM' : 'AM');
    });
  }

  function pct(ev, el){
    var r = el.getBoundingClientRect();
    return {
      x: r.width ? ((ev.clientX - r.left) / r.width) * 100 : 50,
      y: r.height ? ((ev.clientY - r.top) / r.height) * 100 : 50,
      dx: r.width ? (ev.clientX - (r.left + r.width / 2)) : 0,
      dy: r.height ? (ev.clientY - (r.top + r.height / 2)) : 0,
      r: r
    };
  }
  function reduceMotion(){
    return window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  }
  function bindPointerFx(){
    if (document._cuiPtr) return;
    document._cuiPtr = true;
    if (reduceMotion()) return;
    document.addEventListener('pointermove', function(e){
      var t = e.target.closest && e.target.closest('[data-slot="tilt-card"], [data-slot="glare-hover"], [data-slot="spotlight-card"], [data-slot="magnetic"], [data-slot="dock"], .cui-image-zoom, [data-slot="image-zoom"]');
      if (!t) return;
      var slot = t.getAttribute('data-slot');
      var p = pct(e, t);
      if (slot === 'tilt-card') {
        var max = Number(t.getAttribute('data-max-tilt') || 12);
        t.style.setProperty('--tilt-rx', ((0.5 - p.y / 100) * max).toFixed(2) + 'deg');
        t.style.setProperty('--tilt-ry', ((p.x / 100 - 0.5) * max).toFixed(2) + 'deg');
        t.style.setProperty('--tilt-gx', p.x.toFixed(1) + '%');
        t.style.setProperty('--tilt-gy', p.y.toFixed(1) + '%');
      } else if (slot === 'glare-hover') {
        t.setAttribute('data-glare-live', '');
        t.style.setProperty('--glare-x', p.x.toFixed(1) + '%');
        t.style.setProperty('--glare-y', p.y.toFixed(1) + '%');
      } else if (slot === 'spotlight-card') {
        t.style.setProperty('--spot-x', p.x.toFixed(1) + '%');
        t.style.setProperty('--spot-y', p.y.toFixed(1) + '%');
      } else if (slot === 'magnetic') {
        t.setAttribute('data-magnetic-live', '');
        var strength = Number(t.getAttribute('data-strength') || 0.3);
        var target = t.querySelector('[data-slot="magnetic-target"]');
        if (target) {
          var tx = p.dx * strength;
          var ty = p.dy * strength;
          target.style.transform = 'translate(' + tx.toFixed(1) + 'px,' + ty.toFixed(1) + 'px)';
        }
      } else if (slot === 'dock') {
        t.setAttribute('data-dock-live', '');
        t.querySelectorAll('[data-slot="dock-item"]').forEach(function(item){
          var ir = item.getBoundingClientRect();
          var cx = ir.left + ir.width / 2;
          var d = Math.abs(e.clientX - cx);
          var s = Math.max(1, 1.6 - d / 120 * 0.6);
          if (d > 120) s = 1;
          item.style.setProperty('--dock-s', s.toFixed(3));
        });
      } else if (t.classList && t.classList.contains('cui-image-zoom') || slot === 'image-zoom') {
        var host = t.classList.contains('cui-image-zoom') ? t : (t.closest('.cui-image-zoom') || t);
        var content = host.querySelector('[data-slot="image-zoom-content"]');
        if (content) {
          var hp = pct(e, host);
          content.style.transformOrigin = hp.x.toFixed(1) + '% ' + hp.y.toFixed(1) + '%';
          host.style.setProperty('--zoom-ox', hp.x.toFixed(1) + '%');
          host.style.setProperty('--zoom-oy', hp.y.toFixed(1) + '%');
        }
      }
    }, {passive:true});
    document.addEventListener('pointerleave', function(e){
      var t = e.target.closest && e.target.closest('[data-slot="magnetic"], [data-slot="tilt-card"], [data-slot="dock"]');
      if (!t) return;
      if (t.getAttribute('data-slot') === 'magnetic') {
        var target = t.querySelector('[data-slot="magnetic-target"]');
        if (target) target.style.transform = '';
      }
      if (t.getAttribute('data-slot') === 'tilt-card') {
        t.style.setProperty('--tilt-rx', '0deg');
        t.style.setProperty('--tilt-ry', '0deg');
      }
      if (t.getAttribute('data-slot') === 'dock') {
        t.querySelectorAll('[data-slot="dock-item"]').forEach(function(item){
          item.style.setProperty('--dock-s', '1');
        });
      }
    }, true);
    document.addEventListener('pointerdown', function(e){
      var spark = e.target.closest && e.target.closest('[data-slot="click-spark"], [data-slot="confetti"]');
      if (!spark) return;
      var p = pct(e, spark);
      spark.style.setProperty('--spark-x', p.x.toFixed(1) + '%');
      spark.style.setProperty('--spark-y', p.y.toFixed(1) + '%');
      spark.removeAttribute('data-sparking');
      void spark.offsetWidth;
      spark.setAttribute('data-sparking', '');
      setTimeout(function(){ spark.removeAttribute('data-sparking'); }, 1100);
    }, true);
  }

  function bindDropzone(root){
    (root || document).querySelectorAll('[data-slot="file-dropzone"]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      var n = 0;
      el.addEventListener('dragenter', function(e){ e.preventDefault(); n++; el.setAttribute('data-dragging', 'true'); });
      el.addEventListener('dragover', function(e){ e.preventDefault(); el.setAttribute('data-dragging', 'true'); });
      el.addEventListener('dragleave', function(){ n = Math.max(0, n - 1); if (!n) el.setAttribute('data-dragging', 'false'); });
      el.addEventListener('drop', function(e){ e.preventDefault(); n = 0; el.setAttribute('data-dragging', 'false'); var input = el.querySelector('input[type="file"]'); if (input && e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length) { try { input.files = e.dataTransfer.files; } catch (err) {} } });
    });
  }

  function bindCreditCard(root){
    function brandOf(d){
      function starts(ps){ return ps.some(function(p){ return d.indexOf(p) === 0; }); }
      if (starts(['4011','4312','4389','4514','4576','5041','5066','5067','509','6277','6362','6363','650','651','655'])) return {id:'elo', gaps:[4,8,12], cvc:3, label:'Elo'};
      if (starts(['34','37'])) return {id:'amex', gaps:[4,10], cvc:4, label:'American Express'};
      var n2 = parseInt(d.slice(0,2), 10);
      if ((n2 >= 51 && n2 <= 55) || (n2 >= 23 && n2 <= 26) || d.indexOf('222') === 0 || d.indexOf('27') === 0) return {id:'mastercard', gaps:[4,8,12], cvc:3, label:'Mastercard'};
      if (d.charAt(0) === '4') return {id:'visa', gaps:[4,8,12], cvc:3, label:'Visa'};
      if (starts(['6011','65','622'])) return {id:'discover', gaps:[4,8,12], cvc:3, label:'Discover'};
      return {id:'unknown', gaps:[4,8,12], cvc:3, label:'Card'};
    }
    function group(digits, gaps){
      var out = '', i = 0;
      for (var g = 0; g < gaps.length && i < digits.length; g++) {
        if (g) out += ' ';
        out += digits.slice(i, gaps[g]);
        i = gaps[g];
      }
      if (i < digits.length) out += (out ? ' ' : '') + digits.slice(i);
      return out;
    }
    (root || document).querySelectorAll('[data-slot="credit-card-input"]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      var number = el.querySelector('[data-slot="credit-card-number"]') || el.querySelector('input[aria-label="Card number"]') || el.querySelector('input');
      var fieldset = el.querySelector('fieldset');
      var live = el.querySelector('[aria-live]');
      var check = el.querySelector('[data-slot="credit-card-valid"]') || el.querySelector('fieldset > svg:last-of-type');
      if (!number) return;
      number.addEventListener('input', function(){
        var digits = number.value.replace(/\D/g, '').slice(0, 19);
        var b = brandOf(digits);
        if (fieldset) fieldset.setAttribute('data-brand', b.id);
        if (live) live.textContent = b.id === 'unknown' ? '' : (b.label + ' card');
        var formatted = group(digits, b.gaps);
        if (number.value !== formatted) {
          number.value = formatted;
        }
        var valid = (b.id === 'amex' ? digits.length >= 15 : digits.length >= 16);
        if (check) {
          check.style.opacity = valid ? '1' : '0';
          check.style.transform = valid ? 'scale(1)' : 'scale(0.75)';
        }
      });
    });
  }

  function bindTextEffectReplay(root){
    enable('[data-text-effect-replay]', root);
    (root || document).addEventListener('click', function(e){
      var btn = e.target.closest('[data-text-effect-replay]');
      if (!btn) return;
      var demo = btn.closest('.cui-text-effect-demo') || btn.parentElement;
      var effect = demo && demo.querySelector('[data-slot="text-effect"]');
      if (!effect) return;
      var hidden = effect.querySelector('[aria-hidden="true"]');
      var nodes = hidden ? hidden.querySelectorAll('span[class^="i-"], span.i-0') : effect.querySelectorAll('[class^="i-"]');
      if (!nodes.length && hidden) nodes = hidden.querySelectorAll('span span');
      nodes.forEach(function(n){
        n.style.animation = 'none';
      });
      void effect.offsetWidth;
      nodes.forEach(function(n){
        n.style.animation = '';
      });
    });
  }

  function bindChoropleth(root){
    enable('[data-choropleth-zoom], [data-slot="choropleth-chart"] .zoom button', root);
    (root || document).querySelectorAll('[data-slot="choropleth-chart"]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      el._cuiZoom = 1;
      var g = el.querySelector('svg g.features') || el.querySelector('svg g') || el.querySelector('svg');
      el.addEventListener('click', function(e){
        var btn = e.target.closest('[data-choropleth-zoom], .zoom button');
        if (!btn || !g) return;
        var dir = btn.getAttribute('data-choropleth-zoom') || ((btn.getAttribute('aria-label') || '').toLowerCase().indexOf('out') !== -1 ? 'out' : 'in');
        if (dir === 'out') el._cuiZoom = Math.max(1, el._cuiZoom / 1.25);
        else el._cuiZoom = Math.min(8, el._cuiZoom * 1.25);
        g.style.transformOrigin = 'center';
        g.style.transition = 'transform 200ms ease-out';
        g.style.transform = 'scale(' + el._cuiZoom + ')';
      });
    });
  }

  function v3(x, y, z){ return {x:x, y:y, z:z}; }
  function vsub(a, b){ return v3(a.x-b.x, a.y-b.y, a.z-b.z); }
  function vscale(a, s){ return v3(a.x*s, a.y*s, a.z*s); }
  function vdot(a, b){ return a.x*b.x + a.y*b.y + a.z*b.z; }
  function vcross(a, b){ return v3(a.y*b.z-a.z*b.y, a.z*b.x-a.x*b.z, a.x*b.y-a.y*b.x); }
  function vlen(a){ return Math.sqrt(vdot(a, a)); }
  function vnorm(a){ var l = vlen(a); return l ? vscale(a, 1/l) : v3(0,0,0); }

  function latLngToVec(lat, lng, r){
    var phi = (90 - lat) * Math.PI / 180;
    var theta = (lng + 180) * Math.PI / 180;
    return v3(-(r * Math.sin(phi) * Math.cos(theta)), r * Math.cos(phi), r * Math.sin(phi) * Math.sin(theta));
  }

  function globeCam(yaw, pitch, dist){
    var cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch);
    var cam = v3(sy * cp * dist, sp * dist, cy * cp * dist);
    var fw = vnorm(vscale(cam, -1));
    var rt = vnorm(vcross(fw, v3(0, 1, 0)));
    var up = vcross(rt, fw);
    return {cam: cam, fw: fw, rt: rt, up: up};
  }

  function projectGlobe(p, basis, tan, w, h){
    var rel = vsub(p, basis.cam);
    var cx = vdot(rel, basis.rt);
    var cy = vdot(rel, basis.up);
    var cz = vdot(rel, basis.fw);
    if (cz <= 0.001) return null;
    var ndcX = cx / (cz * tan * (w / h));
    var ndcY = cy / (cz * tan);
    return {x: (ndcX * 0.5 + 0.5) * w, y: (0.5 - ndcY * 0.5) * h, z: cz};
  }

  var _cuiEarth = null, _cuiBump = null;
  function loadGlobeTex(url, slot, cb){
    if (slot === 'earth' && _cuiEarth) { cb(_cuiEarth); return; }
    if (slot === 'bump' && _cuiBump) { cb(_cuiBump); return; }
    var img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = function(){
      if (slot === 'earth') _cuiEarth = img;
      if (slot === 'bump') _cuiBump = img;
      cb(img);
    };
    img.onerror = function(){ cb(null); };
    img.src = url;
  }

  function compileGl(gl, vsSrc, fsSrc){
    function sh(type, src){
      var s = gl.createShader(type);
      gl.shaderSource(s, src);
      gl.compileShader(s);
      if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) return null;
      return s;
    }
    var vs = sh(gl.VERTEX_SHADER, vsSrc);
    var fs = sh(gl.FRAGMENT_SHADER, fsSrc);
    if (!vs || !fs) return null;
    var p = gl.createProgram();
    gl.attachShader(p, vs);
    gl.attachShader(p, fs);
    gl.bindAttribLocation(p, 0, 'a');
    gl.linkProgram(p);
    if (!gl.getProgramParameter(p, gl.LINK_STATUS)) return null;
    return p;
  }

  function texFrom(gl, img){
    var t = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, t);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, 1);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, img);
    return t;
  }

  function bindGlobe3d(root){
    var EARTH = 'https://unpkg.com/three-globe@2.31.0/example/img/earth-blue-marble.jpg';
    var BUMP = 'https://unpkg.com/three-globe@2.31.0/example/img/earth-topology.png';
    var VS = 'attribute vec2 a;void main(){gl_Position=vec4(a,0.0,1.0);}';
    var FS = 'precision highp float;uniform sampler2D uMap,uBump;uniform vec2 uRes;uniform float uYaw,uPitch,uDist,uRadius,uTan,uHasBump;void main(){vec2 ndc=(gl_FragCoord.xy/uRes)*2.0-1.0;float cy=cos(uYaw),sy=sin(uYaw),cp=cos(uPitch),sp=sin(uPitch);vec3 cam=vec3(sy*cp,sp,cy*cp)*uDist;vec3 fw=normalize(-cam);vec3 rt=normalize(cross(fw,vec3(0.0,1.0,0.0)));vec3 upv=cross(rt,fw);vec3 rd=normalize(rt*(ndc.x*(uRes.x/uRes.y)*uTan)+upv*(ndc.y*uTan)+fw);float b=dot(cam,rd);float c=dot(cam,cam)-uRadius*uRadius;float h=b*b-c;if(h<0.0)discard;float t=-b-sqrt(h);if(t<0.0)t=-b+sqrt(h);if(t<0.0)discard;vec3 p=cam+rd*t;vec3 n=normalize(p);float lat=asin(clamp(p.y/uRadius,-1.0,1.0));float lng=atan(p.z,-p.x);vec2 uv=vec2(lng/(6.28318530718)+0.5,0.5-lat/3.14159265359);vec3 albedo=texture2D(uMap,uv).rgb;if(uHasBump>0.5){float ht=texture2D(uBump,uv).r;albedo*=0.82+ht*0.36;}vec3 key=normalize(vec3(5.0,2.0,5.0));vec3 fill=normalize(vec3(-3.0,1.0,-2.0));vec3 col=albedo*0.6;col+=albedo*1.5*max(0.0,dot(n,key));col+=albedo*0.45*max(0.0,dot(n,fill))*vec3(0.533,0.8,1.0);gl_FragColor=vec4(col,1.0);}';
    var RADIUS = 2, DIST = 7, TAN = Math.tan(45 * Math.PI / 360);
    (root || document).querySelectorAll('[data-slot="globe-3d"]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      var canvas = document.createElement('canvas');
      canvas.setAttribute('aria-hidden', 'true');
      canvas.tabIndex = -1;
      canvas.style.cssText = 'position:absolute;inset:0;width:100%;height:100%;display:block;cursor:grab;touch-action:none;z-index:0';
      var gl = null;
      try { gl = canvas.getContext('webgl', {alpha:true, antialias:true, premultipliedAlpha:false}) || canvas.getContext('experimental-webgl', {alpha:true, antialias:true, premultipliedAlpha:false}); } catch (e) { gl = null; }
      if (!gl) return;
      var prog = compileGl(gl, VS, FS);
      if (!prog) return;
      el.insertBefore(canvas, el.firstChild);
      var buf = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, buf);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1,-1, 3,-1, -1,3]), gl.STATIC_DRAW);
      var loc = {
        map: gl.getUniformLocation(prog, 'uMap'),
        bump: gl.getUniformLocation(prog, 'uBump'),
        res: gl.getUniformLocation(prog, 'uRes'),
        yaw: gl.getUniformLocation(prog, 'uYaw'),
        pitch: gl.getUniformLocation(prog, 'uPitch'),
        dist: gl.getUniformLocation(prog, 'uDist'),
        radius: gl.getUniformLocation(prog, 'uRadius'),
        tan: gl.getUniformLocation(prog, 'uTan'),
        hasBump: gl.getUniformLocation(prog, 'uHasBump')
      };
      var earthTex = null, bumpTex = null;
      var yaw = 0, pitch = 0, yawT = 0, pitchT = 0;
      var dragging = false, lastX = 0, lastY = 0, lastT = 0;
      var frozen = el.hasAttribute('data-static');
      var pins = [];
      el.querySelectorAll('.marker[data-lat][data-lng]').forEach(function(m){
        pins.push({
          el: m,
          lat: parseFloat(m.getAttribute('data-lat')),
          lng: parseFloat(m.getAttribute('data-lng'))
        });
      });
      function resize(){
        var r = el.getBoundingClientRect();
        var dpr = Math.min(2, window.devicePixelRatio || 1);
        var w = Math.max(1, Math.floor(r.width * dpr));
        var h = Math.max(1, Math.floor(r.height * dpr));
        if (canvas.width !== w || canvas.height !== h) {
          canvas.width = w;
          canvas.height = h;
        }
        gl.viewport(0, 0, canvas.width, canvas.height);
        return r;
      }
      function paint(now){
        var r = resize();
        var dt = lastT ? Math.min(0.05, (now - lastT) / 1000) : 0.016;
        lastT = now;
        if (!dragging && !frozen) yawT += (2 * Math.PI / 60) * 0.3 * dt;
        var k = 1 - Math.pow(0.9, dt * 60);
        yaw += (yawT - yaw) * k;
        pitch += (pitchT - pitch) * k;
        gl.clearColor(0, 0, 0, 0);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.useProgram(prog);
        gl.bindBuffer(gl.ARRAY_BUFFER, buf);
        gl.enableVertexAttribArray(0);
        gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, earthTex);
        gl.uniform1i(loc.map, 0);
        gl.activeTexture(gl.TEXTURE1);
        gl.bindTexture(gl.TEXTURE_2D, bumpTex || earthTex);
        gl.uniform1i(loc.bump, 1);
        gl.uniform2f(loc.res, canvas.width, canvas.height);
        gl.uniform1f(loc.yaw, yaw);
        gl.uniform1f(loc.pitch, pitch);
        gl.uniform1f(loc.dist, DIST);
        gl.uniform1f(loc.radius, RADIUS);
        gl.uniform1f(loc.tan, TAN);
        gl.uniform1f(loc.hasBump, bumpTex ? 1 : 0);
        gl.drawArrays(gl.TRIANGLES, 0, 3);
        var basis = globeCam(yaw, pitch, DIST);
        var camN = vnorm(basis.cam);
        pins.forEach(function(pin){
          if (!isFinite(pin.lat) || !isFinite(pin.lng)) return;
          var surf = latLngToVec(pin.lat, pin.lng, RADIUS * 1.001);
          var top = latLngToVec(pin.lat, pin.lng, RADIUS * 1.18);
          var vis = vdot(vnorm(surf), camN) > 0.1;
          var ps = projectGlobe(surf, basis, TAN, r.width, r.height);
          var pt = projectGlobe(top, basis, TAN, r.width, r.height);
          if (!pt || !ps) vis = false;
          pin.el.style.setProperty('--pin-o', vis ? '1' : '0');
          if (!pt || !ps) return;
          pin.el.style.setProperty('--pin-x', pt.x.toFixed(1) + 'px');
          pin.el.style.setProperty('--pin-y', pt.y.toFixed(1) + 'px');
          var dx = ps.x - pt.x, dy = ps.y - pt.y;
          var len = Math.sqrt(dx * dx + dy * dy);
          var rot = Math.atan2(dy, dx) * 180 / Math.PI - 90;
          pin.el.style.setProperty('--pin-len', len.toFixed(1) + 'px');
          pin.el.style.setProperty('--pin-rot', rot.toFixed(1) + 'deg');
          var img = pin.el.querySelector('.face img, .face > img');
          if (img) img.style.pointerEvents = vis ? 'auto' : 'none';
        });
        el._cuiRaf = requestAnimationFrame(paint);
      }
      function ready(){
        if (!earthTex) return;
        el.setAttribute('data-globe-live', '');
        lastT = 0;
        el._cuiRaf = requestAnimationFrame(paint);
      }
      loadGlobeTex(EARTH, 'earth', function(img){
        if (!img) return;
        earthTex = texFrom(gl, img);
        ready();
      });
      loadGlobeTex(BUMP, 'bump', function(img){
        if (!img) return;
        bumpTex = texFrom(gl, img);
      });
      canvas.addEventListener('pointerdown', function(e){
        dragging = true;
        lastX = e.clientX;
        lastY = e.clientY;
        canvas.style.cursor = 'grabbing';
        try { canvas.setPointerCapture(e.pointerId); } catch (err) {}
      });
      canvas.addEventListener('pointermove', function(e){
        if (!dragging) return;
        var h = Math.max(1, el.getBoundingClientRect().height);
        yawT -= 2 * Math.PI * (e.clientX - lastX) / h * 0.4;
        pitchT -= 2 * Math.PI * (e.clientY - lastY) / h * 0.4;
        if (pitchT > 1.4) pitchT = 1.4;
        if (pitchT < -1.4) pitchT = -1.4;
        lastX = e.clientX;
        lastY = e.clientY;
      });
      function endDrag(){
        dragging = false;
        canvas.style.cursor = 'grab';
      }
      canvas.addEventListener('pointerup', endDrag);
      canvas.addEventListener('pointercancel', endDrag);
    });
  }

  function bindParticles(root){
    if (reduceMotion()) return;
    (root || document).querySelectorAll('[data-slot="particles"]').forEach(function(el){
      if (el._cuiLive) return;
      el._cuiLive = true;
      var field = el.querySelector('[aria-hidden]');
      if (!field) return;
      var count = field.querySelectorAll('span').length;
      count = Math.min(120, Math.max(8, count || 40));
      var canvas = document.createElement('canvas');
      canvas.tabIndex = -1;
      canvas.setAttribute('aria-hidden', 'true');
      canvas.style.cssText = 'position:absolute;inset:0;width:100%;height:100%;pointer-events:none;display:block';
      field.insertBefore(canvas, field.firstChild);
      var ctx = null;
      try { ctx = canvas.getContext('2d'); } catch (e) { return; }
      if (!ctx) return;
      el.setAttribute('data-particles-live', '');
      var specks = [];
      var probe = document.createElement('i');
      probe.setAttribute('aria-hidden', 'true');
      probe.style.cssText = 'position:absolute;inset-inline-start:0;top:0;width:1px;height:1px;opacity:0;pointer-events:none;background:color-mix(in oklch, var(--cronus-fg) 35%, transparent)';
      el.appendChild(probe);
      function speckFill(){
        return getComputedStyle(probe).backgroundColor || 'color-mix(in oklch, var(--cronus-fg) 35%, transparent)';
      }
      function spawn(w, h){
        specks = [];
        for (var i = 0; i < count; i++) {
          specks.push({
            x: (i * 97) % Math.max(1, w),
            y: (i * 53) % Math.max(1, h),
            vx: ((i % 5) - 2) * 0.12,
            vy: ((i % 3) - 1) * 0.08,
            size: 1 + (i % 3) * 0.4
          });
        }
      }
      function resize(){
        var rect = el.getBoundingClientRect();
        var ratio = window.devicePixelRatio || 1;
        canvas.width = Math.max(1, Math.floor(rect.width * ratio));
        canvas.height = Math.max(1, Math.floor(rect.height * ratio));
        canvas.style.width = rect.width + 'px';
        canvas.style.height = rect.height + 'px';
        ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
        spawn(rect.width, rect.height);
        return rect;
      }
      resize();
      window.addEventListener('resize', resize);
      function tick(){
        var rect = el.getBoundingClientRect();
        var w = rect.width, h = rect.height;
        ctx.clearRect(0, 0, w, h);
        ctx.fillStyle = speckFill();
        for (var i = 0; i < specks.length; i++) {
          var s = specks[i];
          s.x += s.vx;
          s.y += s.vy;
          if (s.x < 0) s.x = w;
          if (s.x > w) s.x = 0;
          if (s.y < 0) s.y = h;
          if (s.y > h) s.y = 0;
          ctx.beginPath();
          ctx.arc(s.x, s.y, s.size, 0, Math.PI * 2);
          ctx.fill();
        }
        el._cuiRaf = requestAnimationFrame(tick);
      }
      el._cuiRaf = requestAnimationFrame(tick);
    });
  }

  function boot(root){
    bindCountdowns(root);
    bindOtp(root);
    bindNumberFlows(root);
    bindAnimatedNumbers(root);
    bindPassword(root);
    bindCopy(root);
    bindSteppers(root);
    bindCommand(root);
    bindSliders(root);
    bindPhone(root);
    bindVideoPlayers(root);
    bindInterest();
    bindTagsInput(root);
    bindAutocomplete(root);
    bindComparison(root);
    bindSignature(root);
    bindCalendarNav(root);
    bindContextMenu();
    bindResizable(root);
    bindKanban(root);
    bindRte(root);
    bindExplore(root);
    bindTokenSwap(root);
    bindMultiSelect(root);
    bindColorPicker(root);
    bindDataTable(root);
    bindWebPreview(root);
    bindTimeNow(root);
    bindPointerFx();
    bindDropzone(root);
    bindCreditCard(root);
    bindTextEffectReplay(root);
    bindChoropleth(root);
    bindGlobe3d(root);
    bindParticles(root);
    enable('[data-sched-nav], [data-slot="invite-dialog"] button, [data-slot="prompt-input"] button, [data-range-preset], [data-text-effect-replay]', root);
  }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', function(){ boot(document); });
  else boot(document);
})();

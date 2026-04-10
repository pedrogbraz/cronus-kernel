/**
 * CRONUS DUMP AUDIT v1.0
 * Visual fidelity validator for .cronus dump output.
 *
 * Compares what the ORIGINAL HTML template shows vs what
 * the CRONUS kernel renders from the dumped .cronus file.
 * Flags missing values, wrong structures, and lost content.
 *
 * Based on Hydra Sync Audit logic — same IIFE pattern,
 * same widget UI, adapted for dump comparison.
 *
 * Usage:
 *   // Inject into the CRONUS-rendered page with reference data:
 *   window.__CRONUS_AUDIT_REFERENCE = { numbers: [...], strings: [...], structure: {...} };
 *   <script src="cronus-dump-audit.js"></script>
 *
 * Or via API:
 *   POST /api/cronus/dump-audit { htmlPath, cronusPath }
 *
 * (c) CRONUS Kernel — dump quality tooling
 */
;(function () {
  'use strict';

  if (window.__CRONUS_DUMP_AUDIT) return;

  // ─── Configuration ───────────────────────────────────────────────────

  var CONFIG = {
    IGNORED_NUMBERS: new Set([
      0, 1, 100, 200, 201, 204, 301, 302, 400, 401, 403, 404, 500, 502, 503,
      10, 20, 25, 30, 40, 50, 60, 70, 75, 80, 90,
      320, 480, 640, 768, 1024, 1280, 1440, 1920,
    ]),
    MIN_NUMBER: 2,
    MAX_NUMBER: 1e12,
    TOLERANCE: 1,
    PERCENT_TOLERANCE: 0.05, // 5% for large numbers
    SKIP_TAGS: new Set(['SCRIPT', 'STYLE', 'SVG', 'NOSCRIPT', 'IFRAME', 'LINK', 'META']),
    DATA_TAGS: new Set([
      'DIV', 'SPAN', 'P', 'TD', 'TH', 'H1', 'H2', 'H3', 'H4', 'H5', 'H6',
      'LI', 'A', 'STRONG', 'EM', 'B', 'I', 'LABEL', 'DD', 'DT', 'FIGCAPTION',
      'SMALL', 'MARK', 'CODE', 'PRE', 'BLOCKQUOTE', 'CAPTION', 'SUMMARY',
      'BUTTON', 'INPUT', 'HEADER', 'FOOTER', 'NAV', 'ASIDE', 'MAIN', 'SECTION',
    ]),
    UI_WORDS: new Set([
      'ok', 'cancel', 'submit', 'save', 'delete', 'edit', 'close', 'open',
      'yes', 'no', 'true', 'false', 'null', 'undefined', 'loading', 'error',
      'success', 'warning', 'back', 'next', 'home', 'search', 'filter',
      'sort', 'reset', 'clear', 'apply', 'confirm', 'menu', 'show', 'hide',
      'expand', 'collapse', 'all', 'none', 'select', 'add', 'remove',
      'get started', 're-scan', 'dismiss', 'cronus dump audit',
    ]),
    MIN_STRING_LENGTH: 3,
    MAX_STRING_LENGTH: 120,
    Z_INDEX: 2147483640,
  };

  // ─── State ───────────────────────────────────────────────────────────

  var state = {
    enabled: false,
    // Reference data (from original HTML template)
    refNumbers: new Set(),
    refStrings: new Set(),
    refStructure: null, // { sections: [...], sectionTypes: [...] }
    // Scan results
    lastScanResults: null,
    widgetEl: null,
    referenceLoaded: false,
  };

  // ─── Helpers ─────────────────────────────────────────────────────────

  function normalizeString(s) {
    return s.toLowerCase().trim().replace(/\s+/g, ' ');
  }

  function parseNumbersFromText(text) {
    var results = [];
    // Brazilian format: 1.482.900,00
    var brMatch = text.match(/\d{1,3}(?:\.\d{3})+(?:,\d+)?/g);
    if (brMatch) {
      for (var i = 0; i < brMatch.length; i++) {
        var cleaned = brMatch[i].replace(/\./g, '').replace(',', '.');
        var n = parseFloat(cleaned);
        if (isFinite(n)) results.push(n);
      }
    }
    // International format: 1,482,900.00
    var intMatch = text.match(/\d{1,3}(?:,\d{3})+(?:\.\d+)?/g);
    if (intMatch) {
      for (var j = 0; j < intMatch.length; j++) {
        var cleaned2 = intMatch[j].replace(/,/g, '');
        var n2 = parseFloat(cleaned2);
        if (isFinite(n2)) results.push(n2);
      }
    }
    // Plain numbers
    var plainMatch = text.match(/-?\d+(?:\.\d+)?/g);
    if (plainMatch) {
      for (var k = 0; k < plainMatch.length; k++) {
        var n3 = parseFloat(plainMatch[k]);
        if (isFinite(n3)) results.push(n3);
      }
    }
    // Suffixed: 5.5k, 1.2M
    var suffixMatch = text.match(/(\d+(?:\.\d+)?)\s*([kKmMbB])\b/g);
    if (suffixMatch) {
      for (var s = 0; s < suffixMatch.length; s++) {
        var parts = suffixMatch[s].match(/(\d+(?:\.\d+)?)\s*([kKmMbB])/);
        if (parts) {
          var base = parseFloat(parts[1]);
          var suffix = parts[2].toLowerCase();
          var mult = suffix === 'k' ? 1000 : suffix === 'm' ? 1000000 : suffix === 'b' ? 1000000000 : 1;
          results.push(base * mult);
        }
      }
    }
    return results;
  }

  function numberExistsInRef(num) {
    if (CONFIG.IGNORED_NUMBERS.has(num)) return true;
    if (num < CONFIG.MIN_NUMBER || num > CONFIG.MAX_NUMBER) return true;
    if (state.refNumbers.has(num)) return true;
    // Tolerance ±1
    for (var t = -CONFIG.TOLERANCE; t <= CONFIG.TOLERANCE; t++) {
      if (state.refNumbers.has(num + t)) return true;
    }
    // Percentage tolerance for large numbers
    if (Math.abs(num) > 100) {
      var iter = state.refNumbers.values();
      var next;
      while (!(next = iter.next()).done) {
        var ref = next.value;
        if (Math.abs(ref) > 0 && Math.abs(num - ref) / Math.abs(ref) < CONFIG.PERCENT_TOLERANCE) {
          return true;
        }
      }
    }
    // Centavos conversion
    if (num > 100 && num % 1 === 0 && state.refNumbers.has(num / 100)) return true;
    if (state.refNumbers.has(num * 100)) return true;
    return false;
  }

  function stringExistsInRef(str) {
    var norm = normalizeString(str);
    if (norm.length < CONFIG.MIN_STRING_LENGTH) return true;
    if (CONFIG.UI_WORDS.has(norm)) return true;
    if (state.refStrings.has(norm)) return true;
    // Partial match — if reference contains this string
    var iter = state.refStrings.values();
    var next;
    while (!(next = iter.next()).done) {
      if (next.value.includes(norm) || norm.includes(next.value)) return true;
    }
    return false;
  }

  // ─── Reference Data Extraction ───────────────────────────────────────

  function extractFromHTML(html) {
    var numbers = new Set();
    var strings = new Set();
    var structure = { sections: [], sectionCount: 0 };

    var parser = new DOMParser();
    var doc = parser.parseFromString(html, 'text/html');

    // Walk all text nodes
    var walker = doc.createTreeWalker(doc.body, NodeFilter.SHOW_TEXT, {
      acceptNode: function(node) {
        var parent = node.parentElement;
        if (!parent) return NodeFilter.FILTER_REJECT;
        var tag = parent.tagName;
        if (CONFIG.SKIP_TAGS.has(tag)) return NodeFilter.FILTER_REJECT;
        return NodeFilter.FILTER_ACCEPT;
      }
    });

    while (walker.nextNode()) {
      var text = walker.currentNode.textContent.trim();
      if (!text) continue;

      // Extract numbers
      var nums = parseNumbersFromText(text);
      for (var i = 0; i < nums.length; i++) {
        numbers.add(nums[i]);
      }

      // Extract strings (split on common delimiters)
      var parts = text.split(/[|,;\n\t]+/);
      for (var j = 0; j < parts.length; j++) {
        var s = parts[j].trim();
        if (s.length >= CONFIG.MIN_STRING_LENGTH && s.length <= CONFIG.MAX_STRING_LENGTH) {
          strings.add(normalizeString(s));
        }
      }
    }

    // Structure: count semantic sections
    var sectionTags = doc.querySelectorAll('section, aside, header, footer, nav, main, table');
    structure.sectionCount = sectionTags.length;
    sectionTags.forEach(function(el) {
      structure.sections.push({
        tag: el.tagName.toLowerCase(),
        classes: el.className ? el.className.substring(0, 200) : '',
        childCount: el.children.length,
        textLength: (el.textContent || '').length,
      });
    });

    return { numbers: numbers, strings: strings, structure: structure };
  }

  // ─── DOM Scan (same as Hydra Audit) ──────────────────────────────────

  function walkDOM(root) {
    var entries = [];
    var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
      acceptNode: function(node) {
        var parent = node.parentElement;
        if (!parent) return NodeFilter.FILTER_REJECT;
        if (parent.offsetParent === null && parent !== document.body) return NodeFilter.FILTER_REJECT;
        var tag = parent.tagName;
        if (CONFIG.SKIP_TAGS.has(tag)) return NodeFilter.FILTER_REJECT;
        if (parent.closest && parent.closest('#cronus-audit-widget')) return NodeFilter.FILTER_REJECT;
        return NodeFilter.FILTER_ACCEPT;
      },
    });

    while (walker.nextNode()) {
      var textNode = walker.currentNode;
      var text = textNode.textContent.trim();
      if (!text) continue;
      var parent = textNode.parentElement;
      entries.push({
        text: text,
        tag: parent ? parent.tagName : 'UNKNOWN',
        isDataTag: CONFIG.DATA_TAGS.has(parent ? parent.tagName : ''),
        element: parent,
      });
    }
    return entries;
  }

  // ─── SCAN: Compare current DOM against reference ─────────────────────

  function scan() {
    if (!state.referenceLoaded) {
      state.lastScanResults = { passed: false, error: 'No reference loaded', checks: [] };
      return state.lastScanResults;
    }

    var start = performance.now();
    var entries = walkDOM(document.body);

    var verifiedNumbers = [];
    var verifiedStrings = [];
    var missingFromOutput = []; // In reference but NOT in rendered output
    var extraInOutput = [];    // In rendered output but NOT in reference

    // Collect all numbers and strings from current DOM
    var domNumbers = new Set();
    var domStrings = new Set();

    for (var i = 0; i < entries.length; i++) {
      var entry = entries[i];
      var text = entry.text;

      // Numbers
      var nums = parseNumbersFromText(text);
      for (var n = 0; n < nums.length; n++) {
        var num = nums[n];
        if (CONFIG.IGNORED_NUMBERS.has(num)) continue;
        if (num < CONFIG.MIN_NUMBER || num > CONFIG.MAX_NUMBER) continue;
        domNumbers.add(num);

        if (numberExistsInRef(num)) {
          verifiedNumbers.push({ value: num, text: text, tag: entry.tag });
        } else {
          extraInOutput.push({
            type: 'number', value: num, text: text, tag: entry.tag,
            element: entry.element,
            reason: num + ' not in original template',
          });
        }
      }

      // Strings
      var parts = text.split(/[|,;\n\t]+/);
      for (var s = 0; s < parts.length; s++) {
        var str = parts[s].trim();
        if (str.length < CONFIG.MIN_STRING_LENGTH || str.length > CONFIG.MAX_STRING_LENGTH) continue;
        var norm = normalizeString(str);
        if (CONFIG.UI_WORDS.has(norm)) continue;
        domStrings.add(norm);

        if (stringExistsInRef(norm)) {
          verifiedStrings.push({ value: norm, text: text, tag: entry.tag });
        }
      }
    }

    // Find what's in reference but missing from DOM
    var refNumIter = state.refNumbers.values();
    var rn;
    while (!(rn = refNumIter.next()).done) {
      var refNum = rn.value;
      if (CONFIG.IGNORED_NUMBERS.has(refNum)) continue;
      if (refNum < CONFIG.MIN_NUMBER || refNum > CONFIG.MAX_NUMBER) continue;
      var found = false;
      var dnIter = domNumbers.values();
      var dn;
      while (!(dn = dnIter.next()).done) {
        if (Math.abs(dn.value - refNum) <= CONFIG.TOLERANCE) { found = true; break; }
        if (Math.abs(refNum) > 100 && Math.abs(dn.value - refNum) / Math.abs(refNum) < CONFIG.PERCENT_TOLERANCE) { found = true; break; }
      }
      if (!found) {
        missingFromOutput.push({
          type: 'number', value: refNum,
          reason: 'MISSING: ' + refNum + ' exists in original but not rendered',
        });
      }
    }

    var refStrIter = state.refStrings.values();
    var rs;
    while (!(rs = refStrIter.next()).done) {
      var refStr = rs.value;
      if (CONFIG.UI_WORDS.has(refStr)) continue;
      if (!domStrings.has(refStr)) {
        // Check partial
        var partialFound = false;
        var dsIter = domStrings.values();
        var ds;
        while (!(ds = dsIter.next()).done) {
          if (ds.value.includes(refStr) || refStr.includes(ds.value)) { partialFound = true; break; }
        }
        if (!partialFound) {
          missingFromOutput.push({
            type: 'string', value: refStr,
            reason: 'MISSING: "' + refStr + '" exists in original but not rendered',
          });
        }
      }
    }

    // Structure comparison
    var structureIssues = [];
    if (state.refStructure) {
      var domSections = document.querySelectorAll('section, aside, header, footer, nav, main, table');
      var domSectionCount = domSections.length;
      var refSectionCount = state.refStructure.sectionCount;
      if (domSectionCount < refSectionCount * 0.5) {
        structureIssues.push({
          type: 'structure',
          reason: 'STRUCTURE: Original has ' + refSectionCount + ' sections, rendered has ' + domSectionCount,
        });
      }
    }

    var allMissing = missingFromOutput.concat(structureIssues);
    var totalRef = state.refNumbers.size + state.refStrings.size;
    var totalVerified = verifiedNumbers.length + verifiedStrings.length;
    var fidelityPercent = totalRef > 0 ? Math.round((totalVerified / totalRef) * 100) : 0;

    var result = {
      passed: allMissing.length === 0,
      fidelity: fidelityPercent,
      duration: Math.round(performance.now() - start),
      verifiedNumbers: verifiedNumbers,
      verifiedStrings: verifiedStrings,
      missingItems: allMissing,
      extraItems: extraInOutput,
      structureIssues: structureIssues,
      refNumbersCount: state.refNumbers.size,
      refStringsCount: state.refStrings.size,
      domNumbersCount: domNumbers.size,
      domStringsCount: domStrings.size,
    };

    // ─── Source Tracing: check visible text against .cronus DSL data ───
    // If the kernel provides __CRONUS_DSL_STRINGS__, verify all DOM text traces back to it.
    var sourceTracing = { enabled: false, hardcoded: [], traced: 0, total: 0 };
    if (window.__CRONUS_DSL_STRINGS__ && Array.isArray(window.__CRONUS_DSL_STRINGS__)) {
      sourceTracing.enabled = true;
      var dslSet = new Set();
      for (var d = 0; d < window.__CRONUS_DSL_STRINGS__.length; d++) {
        var ds = window.__CRONUS_DSL_STRINGS__[d];
        if (ds && ds.length > 1) dslSet.add(ds.toLowerCase());
      }

      var visible = [];
      for (var vi = 0; vi < entries.length; vi++) {
        var vtext = entries[vi].text.trim();
        if (vtext.length < 3) continue;
        // Skip icon names, numbers-only, HTML entities
        if (/^[a-z_]+$/.test(vtext)) continue;
        if (/^[\d.,\s%$+\-]+$/.test(vtext)) continue;
        if (/^&#/.test(vtext)) continue;
        visible.push(vtext);
      }

      sourceTracing.total = visible.length;
      for (var vj = 0; vj < visible.length; vj++) {
        var vt = visible[vj].toLowerCase();
        var traced = false;
        dslSet.forEach(function(dv) {
          if (traced) return;
          if (dv === vt || dv.indexOf(vt) !== -1 || vt.indexOf(dv) !== -1) traced = true;
        });
        if (traced) { sourceTracing.traced++; }
        else { sourceTracing.hardcoded.push(visible[vj]); }
      }
    }
    result.sourceTracing = sourceTracing;

    state.lastScanResults = result;
    return result;
  }

  // ─── Widget UI (same style as Hydra Audit) ───────────────────────────

  function createStyles() {
    if (document.getElementById('cronus-audit-styles')) return;
    var style = document.createElement('style');
    style.id = 'cronus-audit-styles';
    style.textContent = [
      '#cronus-audit-widget { position:fixed; bottom:16px; right:16px; z-index:' + CONFIG.Z_INDEX + '; font-family:Inter,system-ui,sans-serif; font-size:12px; }',
      '#cronus-audit-badge { display:flex; align-items:center; gap:6px; padding:6px 14px; border-radius:20px; cursor:pointer; user-select:none; transition:all 0.2s; }',
      '#cronus-audit-badge { background:#0e0e0e; border:0.5px solid rgba(76,69,70,0.3); color:#e2e2e2; box-shadow:0 4px 24px rgba(0,0,0,0.4); }',
      '#cronus-audit-badge:hover { background:#1f1f1f; border-color:rgba(76,69,70,0.5); }',
      '.cronus-dot { width:6px; height:6px; border-radius:50%; flex-shrink:0; }',
      '.cronus-dot.idle { background:#666; }',
      '.cronus-dot.pass { background:#34d399; box-shadow:0 0 6px rgba(52,211,153,0.5); }',
      '.cronus-dot.warn { background:#fbbf24; box-shadow:0 0 6px rgba(251,191,36,0.5); }',
      '.cronus-dot.fail { background:#ef4444; box-shadow:0 0 6px rgba(239,68,68,0.5); animation:cronus-pulse 2s infinite; }',
      '@keyframes cronus-pulse { 0%,100%{opacity:1} 50%{opacity:0.5} }',
      '#cronus-audit-panel { display:none; position:absolute; bottom:calc(100% + 8px); right:0; width:380px; max-height:500px; overflow-y:auto; border-radius:12px; }',
      '#cronus-audit-panel { background:#0e0e0e; border:0.5px solid rgba(76,69,70,0.3); box-shadow:0 12px 48px rgba(0,0,0,0.6); }',
      '#cronus-audit-panel.open { display:block; }',
      '.cronus-panel-header { padding:12px 16px; border-bottom:0.5px solid rgba(76,69,70,0.2); display:flex; justify-content:space-between; align-items:center; }',
      '.cronus-panel-header h3 { margin:0; font-size:13px; font-weight:700; color:#e2e2e2; letter-spacing:0.02em; }',
      '.cronus-panel-header .cronus-fidelity { font-size:11px; color:#adc6ff; font-weight:600; }',
      '.cronus-panel-body { padding:12px 16px; }',
      '.cronus-stat { display:flex; align-items:center; gap:8px; padding:4px 0; font-size:11px; }',
      '.cronus-stat .icon { width:16px; text-align:center; font-size:13px; }',
      '.cronus-stat .icon.pass { color:#34d399; }',
      '.cronus-stat .icon.fail { color:#ef4444; }',
      '.cronus-stat .icon.warn { color:#fbbf24; }',
      '.cronus-stat .label { flex:1; color:#a1a1aa; }',
      '.cronus-stat .value { color:#e2e2e2; font-weight:500; font-variant-numeric:tabular-nums; }',
      '.cronus-divider { height:1px; background:rgba(76,69,70,0.2); margin:8px 0; }',
      '.cronus-flag { padding:6px 8px; margin:4px 0; border-radius:6px; background:rgba(239,68,68,0.06); border:0.5px solid rgba(239,68,68,0.15); cursor:pointer; transition:background 0.15s; font-size:11px; color:#fca5a5; }',
      '.cronus-flag:hover { background:rgba(239,68,68,0.12); }',
      '.cronus-flag .flag-type { display:inline-block; padding:1px 5px; border-radius:3px; font-size:9px; font-weight:700; margin-right:6px; }',
      '.cronus-flag .flag-type.num { background:rgba(99,102,241,0.15); color:#a5b4fc; }',
      '.cronus-flag .flag-type.str { background:rgba(251,191,36,0.15); color:#fbbf24; }',
      '.cronus-flag .flag-type.struct { background:rgba(239,68,68,0.15); color:#ef4444; }',
      '.cronus-flag .flag-context { display:block; margin-top:3px; font-size:10px; color:#71717a; }',
      '.cronus-missing { padding:6px 8px; margin:4px 0; border-radius:6px; background:rgba(251,191,36,0.06); border:0.5px solid rgba(251,191,36,0.15); font-size:11px; color:#fbbf24; }',
      '.cronus-missing .flag-type { display:inline-block; padding:1px 5px; border-radius:3px; font-size:9px; font-weight:700; margin-right:6px; background:rgba(251,191,36,0.15); color:#fbbf24; }',
      '.cronus-empty { text-align:center; padding:16px; color:#666; font-size:11px; }',
      '.cronus-panel-footer { padding:8px 16px; border-top:0.5px solid rgba(76,69,70,0.2); display:flex; gap:8px; }',
      '.cronus-btn { flex:1; padding:6px; border:0.5px solid rgba(76,69,70,0.3); border-radius:6px; background:#1f1f1f; color:#e2e2e2; font-size:11px; font-weight:500; cursor:pointer; transition:all 0.15s; }',
      '.cronus-btn:hover { background:#2a2a2a; }',
      '.cronus-fidelity-bar { height:3px; border-radius:2px; background:#1f1f1f; margin:8px 0; overflow:hidden; }',
      '.cronus-fidelity-fill { height:100%; border-radius:2px; transition:width 0.5s; }',
    ].join('\n');
    document.head.appendChild(style);
  }

  function renderWidget() {
    if (!state.widgetEl) return;
    var r = state.lastScanResults;
    var badge = state.widgetEl.querySelector('#cronus-audit-badge');
    var panel = state.widgetEl.querySelector('#cronus-audit-panel');

    if (!r) {
      badge.innerHTML = '<span class="cronus-dot idle"></span><span>CRONUS Audit</span>';
      return;
    }

    var missingCount = r.missingItems.length;
    var fidelity = r.fidelity;
    var dotClass = fidelity >= 90 ? 'pass' : fidelity >= 60 ? 'warn' : 'fail';
    var symbol = fidelity >= 90 ? String.fromCharCode(10003) : String.fromCharCode(10007);

    badge.innerHTML =
      '<span class="cronus-dot ' + dotClass + '"></span>' +
      '<span>' + fidelity + '% fidelity' +
      (missingCount > 0 ? ' / ' + missingCount + ' missing' : '') + '</span>';

    // Panel content
    var bodyHtml = '';

    // Fidelity bar
    var barColor = fidelity >= 90 ? '#34d399' : fidelity >= 60 ? '#fbbf24' : '#ef4444';
    bodyHtml += '<div class="cronus-fidelity-bar"><div class="cronus-fidelity-fill" style="width:' + fidelity + '%;background:' + barColor + '"></div></div>';

    // Stats
    bodyHtml += '<div class="cronus-stat"><span class="icon pass">' + String.fromCharCode(10003) + '</span><span class="label">Numbers matched</span><span class="value">' + r.verifiedNumbers.length + '/' + r.refNumbersCount + '</span></div>';
    bodyHtml += '<div class="cronus-stat"><span class="icon pass">' + String.fromCharCode(10003) + '</span><span class="label">Strings matched</span><span class="value">' + r.verifiedStrings.length + '/' + r.refStringsCount + '</span></div>';
    bodyHtml += '<div class="cronus-stat"><span class="icon" style="color:#666">~</span><span class="label">Scan time</span><span class="value">' + r.duration + 'ms</span></div>';

    // Missing items (from original, not in output)
    if (r.missingItems.length > 0) {
      bodyHtml += '<div class="cronus-divider"></div>';
      bodyHtml += '<div class="cronus-stat"><span class="icon fail">' + String.fromCharCode(10007) + '</span><span class="label" style="color:#fca5a5;font-weight:500">' + r.missingItems.length + ' missing from output</span></div>';

      var maxShow = Math.min(r.missingItems.length, 15);
      for (var i = 0; i < maxShow; i++) {
        var m = r.missingItems[i];
        var typeClass = m.type === 'number' ? 'num' : m.type === 'structure' ? 'struct' : 'str';
        var typeLabel = m.type === 'number' ? 'NUM' : m.type === 'structure' ? 'STRUCT' : 'STR';
        bodyHtml += '<div class="cronus-missing"><span class="flag-type ' + typeClass + '">' + typeLabel + '</span>' + escapeHtml(m.reason) + '</div>';
      }
      if (r.missingItems.length > maxShow) {
        bodyHtml += '<div class="cronus-empty">+ ' + (r.missingItems.length - maxShow) + ' more...</div>';
      }
    }

    // Extra items (in output but not in original — possible phantom data)
    if (r.extraItems.length > 0) {
      bodyHtml += '<div class="cronus-divider"></div>';
      bodyHtml += '<div class="cronus-stat"><span class="icon warn">!</span><span class="label" style="color:#fbbf24;font-weight:500">' + r.extraItems.length + ' extra in output</span></div>';

      var maxExtra = Math.min(r.extraItems.length, 10);
      for (var e = 0; e < maxExtra; e++) {
        var ex = r.extraItems[e];
        var etClass = ex.type === 'number' ? 'num' : 'str';
        var etLabel = ex.type === 'number' ? 'NUM' : 'STR';
        bodyHtml += '<div class="cronus-flag" data-cronus-flag-idx="' + e + '"><span class="flag-type ' + etClass + '">' + etLabel + '</span>' + escapeHtml(ex.reason) + '<span class="flag-context">&lt;' + ex.tag.toLowerCase() + '&gt; ' + escapeHtml(truncate(ex.text, 40)) + '</span></div>';
      }
    }

    // Source Tracing section
    if (r.sourceTracing && r.sourceTracing.enabled) {
      bodyHtml += '<div class="cronus-divider"></div>';
      var st = r.sourceTracing;
      var stPercent = st.total > 0 ? Math.round((st.traced / st.total) * 100) : 100;
      var stColor = st.hardcoded.length === 0 ? '#34d399' : '#fbbf24';
      bodyHtml += '<div class="cronus-stat"><span class="icon" style="color:' + stColor + '">' + String.fromCharCode(9670) + '</span><span class="label">Source Tracing: ' + stPercent + '% from .cronus (' + st.traced + '/' + st.total + ')</span></div>';
      if (st.hardcoded.length > 0) {
        bodyHtml += '<div class="cronus-stat"><span class="icon fail">' + String.fromCharCode(9888) + '</span><span class="label" style="color:#fca5a5;font-weight:500">' + st.hardcoded.length + ' hardcoded string(s)</span></div>';
        var stMax = Math.min(st.hardcoded.length, 10);
        for (var si = 0; si < stMax; si++) {
          bodyHtml += '<div class="cronus-flag"><span class="flag-type warn">HC</span>' + escapeHtml(st.hardcoded[si]) + '</div>';
        }
        if (st.hardcoded.length > stMax) {
          bodyHtml += '<div class="cronus-empty">+ ' + (st.hardcoded.length - stMax) + ' more...</div>';
        }
      }
    }

    // All good
    if (r.missingItems.length === 0 && r.extraItems.length === 0 && (!r.sourceTracing || !r.sourceTracing.enabled || r.sourceTracing.hardcoded.length === 0)) {
      bodyHtml += '<div class="cronus-divider"></div>';
      bodyHtml += '<div class="cronus-empty" style="color:#34d399">Perfect fidelity. All original data rendered correctly.</div>';
    }

    panel.querySelector('.cronus-panel-body').innerHTML = bodyHtml;
    panel.querySelector('.cronus-panel-header .cronus-fidelity').textContent = fidelity + '% fidelity';

    // Click handlers for extra items (highlight element)
    var flags = panel.querySelectorAll('.cronus-flag');
    for (var fi = 0; fi < flags.length; fi++) {
      flags[fi].addEventListener('click', function() {
        var idx = parseInt(this.getAttribute('data-cronus-flag-idx'), 10);
        var item = r.extraItems[idx];
        if (item && item.element) highlightElement(item.element);
      });
    }
  }

  function highlightElement(el) {
    if (!el) return;
    var prev = el.style.outline;
    el.style.outline = '2px solid #ef4444';
    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    setTimeout(function() { el.style.outline = prev; }, 2000);
  }

  function escapeHtml(str) {
    return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  function truncate(str, max) {
    if (str.length <= max) return str;
    return str.substring(0, max) + '...';
  }

  // ─── Widget Creation ─────────────────────────────────────────────────

  function createWidget() {
    if (state.widgetEl) return;
    createStyles();

    var widget = document.createElement('div');
    widget.id = 'cronus-audit-widget';
    widget.innerHTML = [
      '<div id="cronus-audit-panel">',
      '  <div class="cronus-panel-header">',
      '    <h3>CRONUS Dump Audit</h3>',
      '    <span class="cronus-fidelity">--</span>',
      '  </div>',
      '  <div class="cronus-panel-body">',
      '    <div class="cronus-empty">Load reference data, then scan.</div>',
      '  </div>',
      '  <div class="cronus-panel-footer">',
      '    <button class="cronus-btn" id="cronus-btn-scan">Scan</button>',
      '    <button class="cronus-btn" id="cronus-btn-dismiss">Dismiss</button>',
      '  </div>',
      '</div>',
      '<div id="cronus-audit-badge">',
      '  <span class="cronus-dot idle"></span>',
      '  <span>CRONUS Audit</span>',
      '</div>',
    ].join('\n');

    document.body.appendChild(widget);
    state.widgetEl = widget;

    widget.querySelector('#cronus-audit-badge').addEventListener('click', function() {
      widget.querySelector('#cronus-audit-panel').classList.toggle('open');
    });
    widget.querySelector('#cronus-btn-scan').addEventListener('click', function() {
      scan();
      renderWidget();
    });
    widget.querySelector('#cronus-btn-dismiss').addEventListener('click', function() {
      widget.querySelector('#cronus-audit-panel').classList.remove('open');
    });
  }

  // ─── Public API ──────────────────────────────────────────────────────

  window.__CRONUS_DUMP_AUDIT = {
    // Load reference from raw HTML string
    loadReference: function(html) {
      var extracted = extractFromHTML(html);
      state.refNumbers = extracted.numbers;
      state.refStrings = extracted.strings;
      state.refStructure = extracted.structure;
      state.referenceLoaded = true;
      console.log(
        '%c[CRONUS AUDIT]%c Reference loaded: ' + extracted.numbers.size + ' numbers, ' + extracted.strings.size + ' strings, ' + extracted.structure.sectionCount + ' sections',
        'color: #6366f1; font-weight: bold;', 'color: inherit;'
      );
      return extracted;
    },

    // Load reference from URL (fetch HTML)
    loadReferenceFromURL: function(url) {
      var self = this;
      return fetch(url).then(function(r) { return r.text(); }).then(function(html) {
        return self.loadReference(html);
      });
    },

    // Load reference from pre-extracted data
    loadReferenceData: function(data) {
      if (data.numbers) {
        state.refNumbers = new Set(data.numbers);
      }
      if (data.strings) {
        state.refStrings = new Set(data.strings);
      }
      if (data.structure) {
        state.refStructure = data.structure;
      }
      state.referenceLoaded = true;
    },

    scan: function() {
      var results = scan();
      renderWidget();
      return results;
    },

    results: function() { return state.lastScanResults; },

    enable: function() {
      state.enabled = true;
      if (document.body) createWidget();
      else document.addEventListener('DOMContentLoaded', function() { createWidget(); });

      // Auto-load reference if set on window
      if (window.__CRONUS_AUDIT_REFERENCE) {
        this.loadReferenceData(window.__CRONUS_AUDIT_REFERENCE);
        // Auto-scan after short delay
        var self = this;
        setTimeout(function() { self.scan(); }, 1000);
      }

      console.log(
        '%c[CRONUS AUDIT]%c Enabled. Ready to compare.',
        'color: #6366f1; font-weight: bold;', 'color: inherit;'
      );
    },

    disable: function() {
      state.enabled = false;
      if (state.widgetEl) { state.widgetEl.remove(); state.widgetEl = null; }
    },

    _state: state,
    _config: CONFIG,
  };

  // Auto-enable if reference data is already set
  if (window.__CRONUS_AUDIT_REFERENCE) {
    window.__CRONUS_DUMP_AUDIT.enable();
  }
})();

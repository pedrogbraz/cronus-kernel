#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Reactive Engine — Client-side micro-framework
//!
//! ~5KB JS that replaces React with HTML data attributes.
//! State management, DOM binding, template loops, events, auto-fetch.

/// The complete CRONUS reactive JS runtime, inlined into HTML pages.
pub const CRONUS_REACTIVE_JS: &str = r##"
// ═══════════════════════════════════════════════════
// CRONUS Reactive Engine v0.1
// State + DOM binding + loops + events + auto-fetch
// Replaces React in ~5KB
// ═══════════════════════════════════════════════════

const CRONUS = {
  _state: {},
  _watchers: new Map(),
  _templates: new Map(),

  // ── State Management ──

  state(name, initial) {
    this._state[name] = initial;
    this._watchers.set(name, []);
    return initial;
  },

  set(name, value) {
    const old = this._state[name];
    this._state[name] = value;
    if (old !== value) this._notify(name);
  },

  get(name) {
    // Support dot notation: "users.length", "user.name"
    const parts = name.split('.');
    let val = this._state[parts[0]];
    for (let i = 1; i < parts.length; i++) {
      if (val == null) return undefined;
      val = val[parts[i]];
    }
    return val;
  },

  update(name, fn) {
    const val = this._state[name];
    this.set(name, fn(val));
  },

  watch(name, callback) {
    if (!this._watchers.has(name)) this._watchers.set(name, []);
    this._watchers.get(name).push(callback);
  },

  _notify(name) {
    // Update all bound DOM elements
    this._updateBindings(name);
    this._updateVisibility(name);
    this._updateLoops(name);

    // Call watchers
    const watchers = this._watchers.get(name) || [];
    const val = this._state[name];
    watchers.forEach(function(fn) { fn(val); });
  },

  // ── DOM Binding: data-bind ──

  _updateBindings(name) {
    var els = document.querySelectorAll('[data-bind]');
    for (var i = 0; i < els.length; i++) {
      var el = els[i];
      var bind = el.getAttribute('data-bind');
      if (bind === name || bind.startsWith(name + '.')) {
        var val = this.get(bind);
        if (val === undefined || val === null) val = '';
        if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT') {
          if (el.type === 'checkbox') {
            el.checked = !!val;
          } else {
            el.value = val;
          }
        } else {
          el.textContent = val;
        }
      }
    }
  },

  // ── Visibility: data-show / data-hide ──

  _updateVisibility(name) {
    var shows = document.querySelectorAll('[data-show]');
    for (var i = 0; i < shows.length; i++) {
      var el = shows[i];
      var key = el.getAttribute('data-show');
      if (key === name || key.startsWith(name + '.')) {
        el.style.display = this.get(key) ? '' : 'none';
      }
    }
    var hides = document.querySelectorAll('[data-hide]');
    for (var i = 0; i < hides.length; i++) {
      var el = hides[i];
      var key = el.getAttribute('data-hide');
      if (key === name || key.startsWith(name + '.')) {
        el.style.display = this.get(key) ? 'none' : '';
      }
    }
  },

  // ── Template Loops: data-for ──

  _updateLoops(name) {
    var loops = document.querySelectorAll('[data-for]');
    for (var i = 0; i < loops.length; i++) {
      var container = loops[i];
      var expr = container.getAttribute('data-for');
      // Parse "item in items"
      var parts = expr.split(' in ');
      if (parts.length !== 2) continue;
      var itemName = parts[0].trim();
      var listName = parts[1].trim();

      if (listName !== name) continue;

      var items = this._state[listName];
      if (!Array.isArray(items)) continue;

      // Cache template on first use
      if (!this._templates.has(container)) {
        this._templates.set(container, container.innerHTML);
      }
      var template = this._templates.get(container);

      // Render
      var html = '';
      for (var j = 0; j < items.length; j++) {
        var item = items[j];
        var row = template;
        // Replace data-bind="itemName.field" with actual values
        // Also replace {{field}} mustache syntax
        if (typeof item === 'object' && item !== null) {
          var keys = Object.keys(item);
          for (var k = 0; k < keys.length; k++) {
            var field = keys[k];
            var val = item[field] != null ? item[field] : '';
            // Replace mustache: {{field}}
            row = row.split('{{' + field + '}}').join(val);
            // Replace data-bind references
            var bindAttr = 'data-bind="' + itemName + '.' + field + '"';
            var bindReplace = 'data-bind="' + itemName + '.' + field + '"';
            // We'll handle data-bind in rendered items by injecting text content
            row = row.replace(
              new RegExp('data-bind="' + itemName + '\\.' + field + '"', 'g'),
              'data-static="' + field + '"'
            );
            row = row.replace(
              new RegExp('(<[^>]*data-static="' + field + '"[^>]*>)[^<]*(<)', 'g'),
              '$1' + val + '$2'
            );
          }
        } else {
          row = row.split('{{value}}').join(item);
          row = row.split('{{' + itemName + '}}').join(item);
        }
        html += row;
      }
      container.innerHTML = html;
    }
  },

  // ── Events: data-click, data-submit ──

  _bindEvents() {
    var self = this;

    // data-click
    document.addEventListener('click', function(e) {
      var el = e.target.closest('[data-click]');
      if (!el) return;
      e.preventDefault();
      var action = el.getAttribute('data-click');
      self._execAction(action, el);
    });

    // data-submit
    document.addEventListener('submit', function(e) {
      var form = e.target.closest('[data-submit]');
      if (!form) return;
      e.preventDefault();
      var action = form.getAttribute('data-submit');
      var data = {};
      var inputs = form.querySelectorAll('input, textarea, select');
      for (var i = 0; i < inputs.length; i++) {
        var input = inputs[i];
        var name = input.name || input.id;
        if (!name) continue;
        if (input.type === 'checkbox') {
          data[name] = input.checked;
        } else {
          data[name] = input.value;
        }
      }
      self._execAction(action, form, data);
    });

    // data-model (two-way binding)
    document.addEventListener('input', function(e) {
      var el = e.target;
      var model = el.getAttribute('data-model');
      if (!model) return;
      if (el.type === 'checkbox') {
        self.set(model, el.checked);
      } else {
        self.set(model, el.value);
      }
    });
  },

  _execAction(action, el, formData) {
    // Check if it's a registered function
    if (typeof window[action] === 'function') {
      window[action](formData || {}, el);
      return;
    }
    // Check if it's a function call with args: "deleteUser(123)"
    var match = action.match(/^(\w+)\(([^)]*)\)$/);
    if (match && typeof window[match[1]] === 'function') {
      var args = match[2].split(',').map(function(a) { return a.trim().replace(/^['"]|['"]$/g, ''); });
      window[match[1]].apply(null, args);
      return;
    }
    console.warn('[CRONUS] Unknown action:', action);
  },

  // ── Auto-Fetch: data-fetch ──

  _autoFetch() {
    var self = this;
    var fetchers = document.querySelectorAll('[data-fetch]');
    for (var i = 0; i < fetchers.length; i++) {
      (function(el) {
        var url = el.getAttribute('data-fetch');
        var into = el.getAttribute('data-into');
        var loadingKey = el.getAttribute('data-loading');
        var interval = el.getAttribute('data-poll'); // optional polling interval in ms

        if (!into) return;

        // Initialize state if not exists
        if (!(into in self._state)) self.state(into, []);
        if (loadingKey && !(loadingKey in self._state)) self.state(loadingKey, true);

        function doFetch() {
          if (loadingKey) self.set(loadingKey, true);
          fetch(url)
            .then(function(r) { return r.json(); })
            .then(function(data) {
              self.set(into, data);
              if (loadingKey) self.set(loadingKey, false);
            })
            .catch(function() {
              if (loadingKey) self.set(loadingKey, false);
            });
        }

        doFetch();

        // Optional polling
        if (interval) {
          var ms = parseInt(interval, 10);
          if (ms > 0) setInterval(doFetch, ms);
        }
      })(fetchers[i]);
    }
  },

  // ── Initialization ──

  init() {
    this._bindEvents();
    this._autoFetch();

    // Initial render of all bindings
    var self = this;
    Object.keys(this._state).forEach(function(name) {
      self._updateBindings(name);
      self._updateVisibility(name);
      self._updateLoops(name);
    });
  },

  // ── Utility: fetch + update state ──

  async fetch(url, options) {
    var resp = await window.fetch(url, options);
    return resp.json();
  },

  async post(url, data) {
    var resp = await window.fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    return resp.json();
  },

  async del(url) {
    var resp = await window.fetch(url, { method: 'DELETE' });
    return resp.json();
  },
};

// Auto-init on DOM ready
document.addEventListener('DOMContentLoaded', function() { CRONUS.init(); });
"##;

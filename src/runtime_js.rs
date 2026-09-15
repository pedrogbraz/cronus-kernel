/// Minimal client-side runtime for CRONUS actions (~120 lines of JS)
/// Handles: button actions, form submissions, toast notifications, navigation
pub const CRONUS_ACTION_JS: &str = r#"
<script>
(function() {
  // Toast system
  function showToast(msg, style) {
    var colors = { success: '#10b981', error: '#ef4444', warning: '#f59e0b', info: '#3b82f6' };
    var t = document.createElement('div');
    t.textContent = msg;
    t.style.cssText = 'position:fixed;bottom:24px;right:24px;padding:12px 24px;border-radius:8px;color:#fff;font-size:14px;z-index:9999;opacity:0;transition:opacity 0.3s;background:' + (colors[style] || colors.info);
    document.body.appendChild(t);
    requestAnimationFrame(function() { t.style.opacity = '1'; });
    setTimeout(function() { t.style.opacity = '0'; setTimeout(function() { t.remove(); }, 300); }, 3000);
  }

  // Effect applier
  function applyEffects(effects) {
    for (var i = 0; i < (effects || []).length; i++) {
      var fx = effects[i];
      switch(fx.type) {
        case 'toast': showToast(fx.target, fx.style || 'info'); break;
        case 'navigate':
          if (fx.target === 'back') history.back();
          else window.location.href = fx.target;
          break;
        case 'refresh': if(window.CRONUS&&window.CRONUS.reload)window.CRONUS.reload();else window.location.reload(); break;
        case 'open':
          var dlg = document.querySelector(fx.target);
          if (dlg && dlg.showModal) dlg.showModal();
          break;
        case 'close':
          var d = document.querySelector(fx.target);
          if (d && d.close) d.close();
          break;
      }
    }
  }

  // Action click handler (delegated)
  document.addEventListener('click', function(e) {
    var btn = e.target.closest('[data-cronus-action]');
    if (!btn) return;
    e.preventDefault();

    var confirmMsg = btn.getAttribute('data-cronus-confirm');
    if (confirmMsg && !confirm(confirmMsg)) return;

    var xhr = new XMLHttpRequest();
    xhr.open('POST', '/_action/' + (btn.getAttribute('data-cronus-section') || ''));
    xhr.setRequestHeader('Content-Type', 'application/json');
    xhr.onload = function() {
      try {
        var data = JSON.parse(xhr.responseText);
        applyEffects(data.effects);
      } catch(err) {
        showToast('Action failed', 'error');
      }
    };
    xhr.onerror = function() { showToast('Action failed', 'error'); };
    // Reference the declared action by id only; the server runs its own copy.
    xhr.send(JSON.stringify({
      action_id: btn.getAttribute('data-action-id') || '',
      entity: btn.getAttribute('data-cronus-entity') || '',
      id: btn.getAttribute('data-cronus-id') || ''
    }));
  });

  // Form submit handler (delegated)
  document.addEventListener('submit', function(e) {
    var form = e.target.closest('[data-cronus-form]');
    if (!form) return;
    e.preventDefault();

    var formData = {};
    var inputs = form.querySelectorAll('input, textarea, select');
    var pending = 0;
    function send() {
      if (pending > 0) return;
      var entity = form.getAttribute('data-cronus-entity') || '';
      var cronusMethod = form.getAttribute('data-cronus-method') || 'POST';
      var cronusId = form.getAttribute('data-cronus-id') || '';
      var xhr = new XMLHttpRequest();
      var formUrl = cronusMethod === 'PATCH' && cronusId
        ? '/_form/' + (form.getAttribute('data-cronus-section') || '') + '/' + cronusId
        : '/_form/' + (form.getAttribute('data-cronus-section') || '');
      xhr.open(cronusMethod, formUrl);
      xhr.setRequestHeader('Content-Type', 'application/json');
      xhr.onload = function() {
        try {
          var data = JSON.parse(xhr.responseText);
          var errs = form.querySelectorAll('.cronus-error');
          for (var j = 0; j < errs.length; j++) errs[j].remove();
          if (data.errors) {
            for (var field in data.errors) {
              var input = form.querySelector('[name="' + field + '"]');
              if (input) {
                var err = document.createElement('span');
                err.className = 'cronus-error';
                err.style.cssText = 'color:#ef4444;font-size:12px;display:block;margin-top:4px';
                err.textContent = data.errors[field];
                input.parentNode.appendChild(err);
              }
            }
          }
          applyEffects(data.effects);
        } catch(err) {
          showToast('Submit failed', 'error');
        }
      };
      xhr.onerror = function() { showToast('Submit failed', 'error'); };
      xhr.send(JSON.stringify({ entity: entity, data: formData }));
    }
    for (var i = 0; i < inputs.length; i++) {
      var inp = inputs[i];
      var name = inp.name || inp.id;
      if (!name || name === '_id') continue;
      if (inp.type === 'file') {
        if (inp.files && inp.files[0]) {
          pending++;
          (function(field, file) {
            var reader = new FileReader();
            reader.onload = function() {
              formData[field] = reader.result;
              pending--;
              send();
            };
            reader.onerror = function() {
              pending--;
              send();
            };
            reader.readAsDataURL(file);
          })(name, inp.files[0]);
        }
        continue;
      }
      formData[name] = inp.type === 'checkbox' ? inp.checked : inp.value;
    }
    send();
  });

  window._cronusActions = true;
})();
</script>
"#;

#[cfg(test)]
mod tests {
    use super::CRONUS_ACTION_JS;

    /// The action payload carries the clicked button's `data-cronus-id` (the
    /// bound row's id), next to the declared action id and entity.
    #[test]
    fn action_payload_reads_record_id_from_the_button() {
        let start = CRONUS_ACTION_JS
            .find("xhr.send(JSON.stringify({\n      action_id:")
            .expect("action payload");
        let payload = &CRONUS_ACTION_JS[start..start + 260];
        assert!(payload.contains("action_id: btn.getAttribute('data-action-id')"));
        assert!(payload.contains("entity: btn.getAttribute('data-cronus-entity')"));
        assert!(payload.contains("id: btn.getAttribute('data-cronus-id')"));
        assert!(
            !payload.contains("data-cronus-action"),
            "no client instructions"
        );
    }

    #[test]
    fn form_submit_reads_file_inputs_as_data_urls() {
        assert!(CRONUS_ACTION_JS.contains("inp.type === 'file'"));
        assert!(CRONUS_ACTION_JS.contains("readAsDataURL"));
    }
}

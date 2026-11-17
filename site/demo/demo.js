/* Aegis Review Center SIMULATION. Toy regexes on fake formats. No network, no storage, no real engine. */
(function () {
  'use strict';

  // Fake sample secrets are assembled from parts so the source holds no key-shaped literals.
  var FAKE_AWS = 'AKIA' + 'IOSFODNN7' + 'EXAMPLE';
  var FAKE_SK = 'sk-' + 'DEMO' + '0000000000000000000000' + 'FAKE';

  var SAMPLES = {
    email: 'Please draft a polite reply to Priya Raman (priya.raman@example.com) and copy ops@example.org about the delayed invoice.',
    key: 'Why does this config fail?\naws_access_key_id = ' + FAKE_AWS + '\nregion = eu-west-1',
    mixed: 'Email jo.tester@example.com the deploy notes. Use token ' + FAKE_SK + ' for the staging call.',
    clean: 'Summarise the main trade-offs of local-first software in three short bullet points.'
  };

  // Toy patterns. Severity: critical blocks (mandatory, no override); moderate goes to review.
  var PATTERNS = [
    { id: 'cloud-key', cat: 'Credentials / secrets', sev: 'critical', re: /\bAKIA[0-9A-Z]{16}\b/g, reason: 'Matches a cloud access key ID shape (AKIA plus 16 characters).' },
    { id: 'sk-token', cat: 'Credentials / secrets', sev: 'critical', re: /\bsk-[A-Za-z0-9_-]{20,}\b/g, reason: 'Looks like a provider secret key (sk- prefix, long token).' },
    { id: 'vcs-token', cat: 'Credentials / secrets', sev: 'critical', re: /\bgh[pousr]_[A-Za-z0-9]{30,}\b/g, reason: 'Looks like a source-host access token (gh*_ prefix).' },
    { id: 'pem', cat: 'Credentials / secrets', sev: 'critical', re: /-----BEGIN [A-Z ]*PRIVATE KEY-----/g, reason: 'Private key header found.' },
    { id: 'jwt', cat: 'Credentials / secrets', sev: 'critical', re: /\beyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}/g, reason: 'JWT-shaped string (three dot-separated segments).' },
    { id: 'email', cat: 'Personal identity (email)', sev: 'moderate', re: /[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g, reason: 'Looks like an email address, which can identify a person.' }
  ];

  var $ = function (id) { return document.getElementById(id); };
  var S = null; // null = nothing inspected yet

  function esc(s) { return String(s).replace(/[&<>"']/g, function (c) { return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]; }); }

  function detect(text) {
    var out = [];
    PATTERNS.forEach(function (p) {
      var re = new RegExp(p.re.source, 'g'), m;
      while ((m = re.exec(text))) {
        if (m[0].length === 0) { re.lastIndex++; continue; }
        out.push({ pid: p.id, cat: p.cat, sev: p.sev, reason: p.reason, start: m.index, end: m.index + m[0].length, value: m[0] });
      }
    });
    out.sort(function (a, b) { return a.start - b.start; });
    // drop overlaps (keep earliest)
    var res = [], last = -1;
    out.forEach(function (f) { if (f.start >= last) { res.push(f); last = f.end; } });
    res.forEach(function (f) {
      var pre = text.slice(0, f.start).split('\n');
      f.line = pre.length; f.col = pre[pre.length - 1].length + 1;
    });
    return res;
  }

  function mask(v) { return v.length <= 8 ? '****' : v.slice(0, 4) + '****' + v.slice(-2); }

  function propose(text, findings) {
    var map = {}, n = 0, out = '', pos = 0, parts = [];
    findings.forEach(function (f) {
      if (f.sev !== 'moderate') return;
      if (!map[f.value]) map[f.value] = '[EMAIL_' + (++n) + ']';
      out += text.slice(pos, f.start) + map[f.value];
      parts.push({ type: 'eq', t: text.slice(pos, f.start) }, { type: 'chg', del: f.value, ins: map[f.value] });
      pos = f.end;
    });
    out += text.slice(pos); parts.push({ type: 'eq', t: text.slice(pos) });
    return { text: out, parts: parts };
  }

  function hash(s) { // FNV-1a, display only, NOT cryptographic
    var h = 0x811c9dc5;
    for (var i = 0; i < s.length; i++) { h ^= s.charCodeAt(i); h = Math.imul(h, 0x01000193) >>> 0; }
    return 'demo-' + h.toString(16).padStart(8, '0') + '-' + s.length;
  }

  function highlight(text, findings) {
    var h = '', pos = 0;
    findings.forEach(function (f) {
      h += esc(text.slice(pos, f.start)) + '<mark class="hl ' + (f.sev === 'critical' ? 'c' : '') + '">' + esc(f.sev === 'critical' ? mask(f.value) : f.value) + '</mark>';
      pos = f.end;
    });
    return h + esc(text.slice(pos));
  }

  function sevBadge(sev) {
    return sev === 'critical'
      ? '<span class="badge b-crit"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 2 2 7v6c0 5 4 8.5 10 9 6-.5 10-4 10-9V7z"/></svg>Critical</span>'
      : '<span class="badge b-warn"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3 2 21h20z"/></svg>Moderate</span>';
  }

  // ---- state ----
  function inspect(text) {
    var findings = detect(text);
    var crit = findings.some(function (f) { return f.sev === 'critical'; });
    S = { orig: text, findings: findings, blocked: crit, editing: false, proposed: null, xform: false, sent: false, rejected: false, editBlocked: false, note: '' };
    if (!crit && findings.length) { var p = propose(text, findings); S.proposed = p.text; S.parts = p.parts; S.base = p.text; }
    render('res-h');
  }

  function reinspectEdit() {
    var f = detect(S.proposed);
    S.editFindings = f;
    S.editBlocked = f.some(function (x) { return x.sev === 'critical'; });
    S.xform = false; // any edit clears approval
    S.sent = false;
  }

  // ---- views ----
  function banner() {
    var ic = function (p) { return '<svg class="i lg" viewBox="0 0 24 24" aria-hidden="true">' + p + '</svg>'; };
    if (S.sent) return '<div class="ws done" role="status"><span class="ws-k">' + ic('<path d="m22 2-7 20-4-9-9-4z"/><path d="M22 2 11 13"/>') + (S.findings.length ? 'SIMULATED: TRANSFORMED TEXT WAS "SENT"' : 'SIMULATED: ORIGINAL TEXT WAS "SENT"') + '</span><span class="ws-d">No network request was made. This is what the real flow would have transmitted, as approved:</span><div class="sent-box">' + esc(S.findings.length ? S.proposed : S.orig) + '</div></div>';
    if (S.blocked || S.editBlocked) return '<div class="ws none" role="status"><span class="ws-k">' + ic('<circle cx="12" cy="12" r="10"/><path d="m4.9 4.9 14.2 14.2"/>') + 'NOTHING WILL BE SENT</span><span class="ws-d">' + (S.blocked ? 'Blocked by mandatory policy. No override exists in the design.' : 'The edited text now contains a Critical pattern. It is blocked and nothing will be sent.') + '</span></div>';
    if (S.rejected) return '<div class="ws none" role="status"><span class="ws-k">' + ic('<circle cx="12" cy="12" r="10"/><path d="m4.9 4.9 14.2 14.2"/>') + 'NOTHING WILL BE SENT</span><span class="ws-d">You rejected this request. It is not submitted.</span></div>';
    if (!S.findings.length) return '<div class="ws orig" role="status"><span class="ws-k">' + ic('<path d="M12 2 2 7v6c0 5 4 8.5 10 9 6-.5 10-4 10-9V7z"/><path d="M12 8v4M12 16h.01"/>') + 'SIMULATED: ORIGINAL TEXT WOULD BE SENT</span><span class="ws-d">No toy pattern matched. This does not mean the text is safe; this simulation checks very little.</span></div>';
    return '<div class="ws xform" role="status"><span class="ws-k">' + ic('<path d="M12 2 2 7v6c0 5 4 8.5 10 9 6-.5 10-4 10-9V7z"/><path d="m9 12 2 2 4-4"/>') + 'SIMULATED: TRANSFORMED TEXT WILL BE SENT, NOT THE ORIGINAL</span><span class="ws-d">' + (S.xform ? 'Transformation approved for' : 'If approved, hash') + ' <code class="hash">' + hash(S.proposed) + '</code> (non-cryptographic demo checksum). Any edit changes the hash and clears approval.</span></div>';
  }

  function findingsHtml() {
    if (!S.findings.length) return '<p class="muted">No findings from the toy patterns.</p>';
    return S.findings.map(function (f, i) {
      return '<div class="finding"><div>' + sevBadge(f.sev) + '</div><div><strong>' + esc(f.cat) + '</strong><div class="meta">' + esc(f.reason) + '</div><div class="meta">Location: line ' + f.line + ', column ' + f.col + '. Match: <code>' + esc(f.sev === 'critical' ? mask(f.value) : f.value) + '</code>. Confidence: toy pattern match, not measured.</div></div></div>';
    }).join('');
  }

  function renderBlocked() {
    return '<div class="alert crit" role="alert" style="margin-top:16px"><svg class="i lg" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="10"/><path d="m4.9 4.9 14.2 14.2"/></svg><div><div class="ttl">Blocked: Critical finding</div><p>In the design, blocking mandatory-critical content is not a modification and needs no approval. There is no approve, edit or override control for this state, only dismissal and remediation.</p></div></div>' +
      '<h3 style="margin-top:16px">What to do instead</h3><ol class="plain-list"><li>Remove the secret from the text.</li><li>If a real secret was ever pasted into any tool, treat it as exposed and rotate it.</li><li>Use an environment variable or a placeholder in prompts.</li></ol>' +
      '<div class="row wrap" style="margin-top:16px"><button class="btn" type="button" data-act="dismiss">Dismiss and start over</button></div>';
  }

  function renderReview() {
    var left = '<div class="pane"><h3>Original <span class="badge b-neu">stays in this page</span></h3><div class="body" tabindex="0" role="region" aria-label="Original text with findings highlighted">' + highlight(S.orig, S.findings) + '</div></div>';
    var right;
    if (S.editing) {
      right = '<div class="pane"><h3>Proposed (editing) <span class="badge b-acc">would be sent</span></h3><label class="sr" for="edit">Edit proposed text</label><textarea id="edit" class="input mono" rows="7" spellcheck="false">' + esc(S.proposed) + '</textarea><p class="help" style="margin-top:6px">Edits are re-inspected on every change and clear any approval.</p>' + (S.editBlocked ? '<div class="alert crit" role="alert" style="margin-top:8px"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="10"/><path d="m4.9 4.9 14.2 14.2"/></svg><div><div class="ttl">Blocked after edit</div><p>A Critical pattern is present in the edited text.</p></div></div>' : '') + '<div class="row wrap" style="margin-top:8px"><button class="btn sm" type="button" data-act="editdone">Done editing</button><button class="btn sm ghost" type="button" data-act="editreset">Reset to proposal</button></div></div>';
    } else {
      var d = S.parts.map(function (p) { return p.type === 'eq' ? esc(p.t) : '<del>' + esc(p.del) + '</del><ins>' + esc(p.ins) + '</ins>'; }).join('');
      var shown = S.proposed === S.base ? d : esc(S.proposed);
      right = '<div class="pane" style="border-color:var(--accent)"><h3>Proposed <span class="badge b-acc">would be sent</span></h3><div class="body" tabindex="0" role="region" aria-label="Proposed text with changes marked">' + shown + '</div><p class="help" style="margin-top:8px">' + (S.proposed === S.base ? 'Struck text is removed, underlined text is added. Why: each email address is replaced by a stable placeholder.' : 'Edited by you. Changes are not marked.') + '</p><div class="row wrap" style="margin-top:8px"><button class="btn sm" type="button" data-act="edit" id="btn-edit">Edit proposed text</button></div></div>';
    }
    var blockedNow = S.editBlocked;
    var s1done = S.xform;
    var step1 = '<div class="step-card' + (s1done ? ' done' : '') + '"><h3>Step 1: Approve transformation</h3><p>Agree that the proposed text replaces the original.</p>' + (s1done ? '<span class="badge b-ok">Approved</span>' : '<button class="btn primary" type="button" data-act="approve-x" id="btn-x"' + (blockedNow ? ' aria-disabled="true" aria-describedby="why-x"' : '') + '>Approve transformation</button>') + (blockedNow ? '<p id="why-x">Unavailable: the edited text is blocked.</p>' : '') + '</div>';
    var s2ok = s1done && !blockedNow;
    var step2 = '<div class="step-card"><h3>Step 2: Approve and send</h3><p>A separate approval, bound to this exact text.</p><button class="btn primary" type="button" data-act="approve-t" id="btn-t"' + (s2ok ? '' : ' aria-disabled="true" aria-describedby="why-t"') + '>Approve and send (simulated)</button><p id="why-t">' + (s2ok ? 'Bound to ' + hash(S.proposed) + '.' : 'Locked until step 1 is approved and re-inspection passes.') + '</p></div>';
    return '<div class="cmp" style="margin-top:16px">' + left + right + '</div><div class="steps">' + step1 + step2 + '</div><div class="row wrap" style="margin-top:16px"><button class="btn danger" type="button" data-act="reject">Reject (send nothing)</button><button class="btn ghost" type="button" data-act="dismiss">Start over</button></div>';
  }

  function renderClean() {
    return '<div class="row wrap" style="margin-top:16px"><button class="btn primary" type="button" data-act="approve-t" id="btn-t">Continue (simulated send)</button><button class="btn ghost" type="button" data-act="dismiss">Start over</button></div>';
  }

  function render(focusId) {
    var host = $('result');
    if (!S) { host.innerHTML = ''; return; }
    var ae = document.activeElement, keep = ae && ae.id && host.contains(ae) ? { id: ae.id, s: ae.selectionStart, e: ae.selectionEnd } : null;
    var body;
    if (S.sent) body = '';
    else if (S.blocked) body = renderBlocked();
    else if (S.rejected) body = '<div class="row wrap" style="margin-top:16px"><button class="btn" type="button" data-act="dismiss">Start over</button></div>';
    else if (!S.findings.length) body = renderClean();
    else body = renderReview();
    var tail = S.sent ? '<div class="row wrap" style="margin-top:16px"><button class="btn" type="button" data-act="dismiss" id="btn-again">Start over</button></div>' : '';
    var sev = S.blocked || S.editBlocked ? '<span class="badge b-crit">Blocked</span>' : S.sent ? '<span class="badge b-ok">Simulated send done</span>' : S.rejected ? '<span class="badge b-neu">Rejected</span>' : S.findings.length ? '<span class="badge b-warn">Review required</span>' : '<span class="badge b-info">Notify only</span>';
    host.innerHTML = '<div class="card"><div class="row between wrap"><h2 class="t-page" id="res-h" tabindex="-1" style="font-size:20px">2. Review result</h2><div class="row wrap" style="gap:8px">' + sev + '<span class="badge b-warn">Simulation</span></div></div>' +
      '<div class="stack-gap"><div id="ban">' + banner() + '</div>' +
      '<div><h3 style="margin-top:16px">Findings (' + S.findings.length + ')</h3>' + findingsHtml() + '</div></div>' + body + tail + '</div>';
    var target = keep && $(keep.id) ? $(keep.id) : (focusId ? $(focusId) : null);
    if (target) { target.focus({ preventScroll: !!keep }); if (keep && target.setSelectionRange && keep.s != null) { try { target.setSelectionRange(keep.s, keep.e); } catch (e) {} } }
    if (focusId && !keep) { var t = $(focusId); if (t && t.scrollIntoView) t.scrollIntoView({ block: 'start' }); }
    announce();
  }

  function announce() {
    var m = S.sent ? 'Simulated send complete. Nothing left your browser.' : S.blocked || S.editBlocked ? 'Blocked. Nothing will be sent.' : S.rejected ? 'Rejected. Nothing will be sent.' : S.findings.length ? 'Review required. ' + S.findings.length + ' findings. Transformed text would be sent after two approvals.' : 'No toy pattern matched.';
    $('live').textContent = ''; setTimeout(function () { $('live').textContent = m; }, 30);
  }

  // ---- events ----
  document.addEventListener('click', function (ev) {
    var b = ev.target.closest('[data-act]'); if (!b) return;
    var a = b.getAttribute('data-act');
    if (b.getAttribute('aria-disabled') === 'true') { ev.preventDefault(); return; }
    if (a.indexOf('sample:') === 0) { $('inp').value = SAMPLES[a.slice(7)]; S = null; render(); $('inp').focus(); return; }
    if (a === 'inspect') { var t = $('inp').value; if (!t.trim()) { $('live').textContent = 'Enter some text first.'; $('inp').focus(); return; } inspect(t); return; }
    if (a === 'clear') { $('inp').value = ''; S = null; render(); $('inp').focus(); return; }
    if (a === 'dismiss') { S = null; render(); $('inp').focus(); return; }
    if (!S) return;
    if (a === 'approve-x') { if (S.editBlocked) return; S.xform = true; render('btn-t'); return; }
    if (a === 'approve-t') { if (S.findings.length && !S.xform) return; S.sent = true; render('btn-again'); return; }
    if (a === 'reject') { S.rejected = true; render('res-h'); return; }
    if (a === 'edit') { S.editing = true; reinspectEdit(); S.xform = false; render('edit'); return; }
    if (a === 'editdone') { S.editing = false; render('btn-edit'); return; }
    if (a === 'editreset') { S.proposed = S.base; reinspectEdit(); render('edit'); return; }
  });
  document.addEventListener('input', function (ev) {
    if (ev.target.id === 'edit' && S) { S.proposed = ev.target.value; reinspectEdit(); render(); }
  });
})();

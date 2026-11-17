/* Extension popup and side panel, rendered from contract v1.1 Coverage objects (03-api-contracts.md 10.3).
   The UI renders `coverage.integration.label` verbatim (FR-BRW-3) and never decides coverage itself.
   UI-local state the daemon cannot send: companion/daemon disconnected (fail closed, AT-14), loading, unreadable coverage.
   Sample data only. Design copy that is NOT in the contract is marked ASSUMPTION in MASTER.md section 11. */
(function(){
const esc=s=>String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;');
const LBL={protected:['prot','Protected'],partially_protected:['part','Partially protected'],not_protected:['none','Not protected']};
const iso=d=>new Date(Date.now()-d*86400000).toISOString();
const C=(tier,label,outbound,inbound,adapter,limitations)=>({integration:{tier,label,outbound,inbound,adapter,limitations},parts:[],stage1:'ran',stage2:'ran',contextual_check_skipped_notice:false,uninspected_override:null});
const SITES=[
 {id:'a',host:'chatgpt.com',cov:C('A','protected','enforced','detect_only',{name:'ChatGPT adapter',version:'sample-0.1',tested_on:iso(2),canary:'pass'},['File uploads are not inspected.'])},
 {id:'b',host:'claude.ai',cov:C('B','partially_protected','advisory','none',{name:'Claude adapter',version:'sample-0.1',tested_on:iso(9),canary:'unknown'},['Only typed text in the message box is checked.','Responses are not inspected.'])},
 {id:'c',host:'gemini.example',cov:C('C','not_protected','none','none',{name:'Gemini adapter',version:'sample-0.1',tested_on:iso(30),canary:'fail'},['The page layout changed and Aegis cannot reliably read the message box.'])},
 {id:'d',host:'unsupported-ai.example',cov:C('D','not_protected','none','none',undefined,['Aegis has no adapter for this site, so it cannot inspect anything here.'])}
];
const when=t=>new Date(t).toLocaleDateString(undefined,{day:'numeric',month:'short'});
const canaryB=c=>c=='pass'?badge('ok','check','Self-test passed'):c=='fail'?badge('crit','x','Self-test failed'):badge('neu','info','Self-test not run');
/* ctx: where the popup state comes from */
function head(statusHtml){return `<div class="ph">${ic('shieldcheck','lg')}<b class="grow">Aegis</b>${statusHtml}</div>`}
function frame(label,body,wide){return `<div><div class="frame-lbl">${esc(label)}</div><div class="popup" style="${wide?'width:min(440px,100%)':''}" role="group" aria-label="${esc(label)}">${body}</div></div>`}
function ctxRows(pending){return `<div class="row between"><span class="t-sec">Mode</span><span class="badge b-acc">${ic('lock')}${S.mode.replace('_',' ')}</span></div><div class="row between"><span class="t-sec">Profile</span><b>${esc(S.profile)}</b></div><div class="row between"><span class="t-sec">Pending reviews</span>${pending?badge('warn','alert',pending):badge('ok','check','None')}</div>`}
function covBlock(s,detail){const i=s.cov.integration,L=LBL[i.label];
 return `${head(st(L[0]))}<div class="pb"><div><div class="t-meta">Current site</div><div style="font-weight:600" class="mono">${esc(s.host)}</div>
 <div class="row wrap" style="gap:8px;margin-top:6px"><span class="badge b-neu">Tier ${esc(i.tier)}</span><span class="t-sec">Sending: ${esc(i.outbound)} · Responses: ${esc(i.inbound.replace('_',' '))}</span></div></div>
 ${i.label=='not_protected'?`<div class="alert crit">${ic('shieldoff')}<div><div class="ttl">Not protected</div><p>Aegis is not inspecting this site. Do not paste sensitive information here. For apps you control, use the API gateway.</p></div></div>`:''}
 ${i.limitations.length?`<ul class="t-sec" style="margin:0;padding-left:18px">${i.limitations.map(x=>`<li>${esc(x)}</li>`).join('')}</ul>`:''}
 ${detail&&i.adapter?`<div class="pane"><div class="t-meta">Adapter</div><div class="mono">${esc(i.adapter.name)} v${esc(i.adapter.version)}</div><div class="row wrap" style="gap:8px;margin-top:6px">${canaryB(i.adapter.canary)}<span class="t-meta">tested ${when(i.adapter.tested_on)}</span></div></div>`:''}
 ${i.label=='not_protected'?'':ctxRows(pendingCount())}
 ${i.label!='not_protected'&&pendingCount()?`<div class="pane"><div class="t-meta">Waiting for you</div><div style="font-weight:550">${esc(D_title())}</div></div>`:''}
 <a class="btn ${i.label=='not_protected'?'':'primary'}" href="#review">${ic('ext')}Open full dashboard</a></div>`}
function D_title(){const r=(S.reviews||[]).find(x=>!['released','rejected','blocked','response_blocked','expired','cancelled','failed'].includes(x.state));return r?(r.risk.reasons[0]?.title||'Review waiting'):'Review waiting'}
const disconnected=()=>`${head(badge('neu','wifioff','Disconnected'))}<div class="pb"><div class="alert crit" role="alert">${ic('octagon','lg')}<div><div class="ttl">Aegis is not running</div><p>Requests from supported sites are blocked until Aegis is running again. Nothing is being inspected, and nothing is being sent.</p></div></div><ol class="t-sec" style="margin:0;padding-left:18px"><li>Open the Aegis app on this computer.</li><li>Select Retry below.</li></ol><div class="row wrap"><button class="btn primary" data-act="ext:retry">${ic('refresh')}Retry connection</button></div><p class="t-meta">Site status is unknown while Aegis is not running.</p></div>`;
const loading=()=>`${head(badge('neu','refresh','Checking'))}<div class="pb"><div class="skel" style="width:50%"></div><div class="skel" style="width:90%"></div><div class="skel" style="width:70%"></div><p class="t-meta">Asking Aegis about this site…</p></div>`;
const unreadable=()=>`${head(st('none'))}<div class="pb"><div class="alert warn">${ic('alert','lg')}<div><div class="ttl">Could not read Aegis's answer</div><p>Treat this site as not protected. Aegis ignored an unreadable message and nothing was sent.</p></div></div><button class="btn" data-act="ext:retry">${ic('refresh')}Check again</button></div>`;
const sidepanel=s=>`<div class="ph">${ic('shieldcheck','lg')}<b class="grow">Aegis</b>${st(LBL[s.cov.integration.label][0])}</div><div class="pb"><h3 class="t-section">This site</h3>${covBlock(s,true).replace(/^.*?<div class="pb">/s,'').replace(/<\/div>$/,'')}<div class="divider"></div><h3 class="t-section">Recent activity (metadata only)</h3><div class="list">${[['Email address transformed','11 min ago'],['Inspection passed, no findings','26 min ago']].map(([a,b])=>`<div class="li"><div class="grow">${a}</div><span class="t-meta">${b}</span></div>`).join('')}</div><p class="t-meta">No prompt text is stored or shown here.</p></div>`;
window.EXTRA.extension=function(){return `<div class="sample">${ic('info')}<span>Rendered from contract v1.1 Coverage objects. Labels are the daemon's; the UI does not decide coverage. Sample data. Disconnected, checking and unreadable states are UI-local because Aegis cannot send them.</span></div>
<h2 class="t-section">Popup (360 px)</h2>
<div class="row wrap" style="align-items:flex-start;gap:24px">${SITES.map(s=>frame(s.host+' · Tier '+s.cov.integration.tier,covBlock(s,false))).join('')}</div>
<h2 class="t-section" style="margin-top:8px">Fail-closed and transient states</h2>
<div class="row wrap" style="align-items:flex-start;gap:24px">${frame('Companion disconnected (UI-local, AT-14)',disconnected())}${frame('Waiting for Aegis',loading())}${frame('Unreadable answer',unreadable())}</div>
<h2 class="t-section" style="margin-top:8px">Side panel (440 px, more detail)</h2>
<div class="row wrap" style="align-items:flex-start;gap:24px">${frame('chatgpt.com · side panel',sidepanel(SITES[0]),true)}${frame('gemini.example · side panel',sidepanel(SITES[2]),true)}</div>
<p class="t-meta">At 375 px the popup fills the width with a 16 px gutter. Keyboard: Tab order is status, limitations, dashboard link; there are no popup-only shortcuts.</p>`};
const prev=window.EXTRA_ACT;
window.EXTRA_ACT=function(k,v,el){if(k=='ext'){toast('Still not running (sample): Aegis did not answer. Nothing was sent.');return true}return prev?prev(k,v,el):false};
})();

/* Review Center v2. PROVISIONAL data contract (PRD-based interim; awaiting docs/architecture/03-api-contracts.md):
   finding{category,severity,location,confidence,reason}; original; proposed; diff; approvalKind{transform|transmit};
   contentHash; coverageTier; uninspected; expiresAt; state{pending,edited-reinspecting,approved-transform,approved-transmit,rejected,expired,blocked}
   Display-only placeholders NOT in contract: id, app. Timeout length (5 min) is a design assumption. Hash here is a non-cryptographic sample. */
(function(){
const H=s=>{let h1=0xdeadbeef,h2=0x41c6ce57;for(let i=0;i<s.length;i++){const c=s.charCodeAt(i);h1=Math.imul(h1^c,2654435761);h2=Math.imul(h2^c,1597334677)}h1=Math.imul(h1^h1>>>16,2246822507)^Math.imul(h2^h2>>>13,3266489909);h2=Math.imul(h2^h2>>>16,2246822507)^Math.imul(h1^h1>>>13,3266489909);return (h2>>>0).toString(16).padStart(8,'0')+(h1>>>0).toString(16).padStart(8,'0')};
const esc=s=>String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;');
const F=(o,sub,category,severity,confidence,reason)=>{const start=o.indexOf(sub);return{category,severity,location:[start,start+sub.length],confidence,reason}};
const T0=Date.now(),MIN=60000;
const mk=(id,app,original,findings,proposed,x={})=>({id,app,original,findings,proposed,state:'pending',coverageTier:'protected',uninspected:false,expiresAt:T0+5*MIN,approvalKind:null,...x});
let o1="Hi, I'm Priya Raman (priya.raman@example.com, +44 7700 900123). Please rewrite my cover letter for the Product Manager role at Northwind Health. My home address is 14 Alder Court, Leeds LS6 1AB.";
let o2="Why does this fail? AWS_SECRET_ACCESS_KEY=wJal••••••••••••••••EXAMPLE with boto3 put_object returns 403";
let o3="Summarise this public procurement notice for naval patrol vessels and list the delivery milestones.";
let o4="…then connect to build-srv-02.corp.example.internal on port 8443.";
let o5="Please summarise the attached file Q3-board-notes.pdf and list action items.";
let o6="Draft a reply to the supplier about invoice 4471 for Contoso Ltd.";
S.reviews=[
 mk('R-2041','ChatGPT · Chrome extension',o1,[F(o1,'Priya Raman','Personal identity','high',.97,'Full name of a private individual.'),F(o1,'priya.raman@example.com','Contact','high',.99,'Email address.'),F(o1,'+44 7700 900123','Contact','high',.96,'Phone number.'),F(o1,'Northwind Health','Contextual identity','med',.58,'Employer may identify you in combination with role and city.'),F(o1,'14 Alder Court, Leeds LS6 1AB','Personal identity','high',.95,'Home address.')],
  "Hi, I'm [PERSON_1] ([EMAIL_1], [PHONE_1]). Please rewrite my cover letter for the Product Manager role at a healthcare company. My home address is [a city in the UK]."),
 mk('R-2040','VS Code assistant · API gateway',o2,[F(o2,'AWS_SECRET_ACCESS_KEY=wJal••••••••••••••••EXAMPLE','Credential / secret','crit',.99,'Matches a cloud secret-key format.')],null,{state:'blocked'}),
 mk('R-2037','Claude · Chrome extension',o5,[],null,{uninspected:true,coverageTier:'partial'}),
 mk('R-2039','Claude · Chrome extension',o3,[F(o3,'public procurement notice for naval patrol vessels','Defence-related (contextual)','med',.41,'Topic is defence-related but no marking, unit or restricted reference was found. Not treated as classified.')],o3),
 mk('R-2038','Inbound response · API gateway',o4,[F(o4,'build-srv-02.corp.example.internal','Technical (inbound)','low',.74,'Internal hostname repeated by the model.')],"…then connect to [INTERNAL_HOST] on port 8443."),
 mk('R-2036','ChatGPT · Chrome extension',o6,[F(o6,'Contoso Ltd','Contextual identity','med',.62,'Named organisation.')],"Draft a reply to the supplier about invoice 4471 for [ORG_1].",{state:'expired',expiresAt:T0-2*MIN})
];
S.sel=0;S.ack=false;S.editing=false;
const OPEN=r=>['pending','edited-reinspecting','approved-transform'].includes(r.state);
const SEVR={crit:4,high:3,med:2,low:1};
const sevOf=r=>r.state=='blocked'?'crit':(r.findings.reduce((m,f)=>SEVR[f.severity]>SEVR[m]?f.severity:m,'low'));
const text=r=>r.edited??r.proposed;
const hashT=r=>H(text(r)||'');const hashO=r=>H(r.original);
window.pendingCount=()=>S.reviews.filter(r=>OPEN(r)||r.state=='blocked').length;
window.blockedCount=()=>S.reviews.filter(r=>r.state=='blocked').length;
const stBadge={pending:['warn','alert','Awaiting review'],'edited-reinspecting':['info','refresh','Re-inspecting'],'approved-transform':['acc','check','Transformation approved'],'approved-transmit':['ok','send','Sent'],rejected:['neu','x','Rejected'],expired:['neu','alert','Expired'],blocked:['crit','ban','Blocked']};
const sb=s=>badge(...stBadge[s]);
/* word diff (LCS) for the proposed pane */
function diffIns(a,b){const A=a.split(/(\s+)/),B=b.split(/(\s+)/),n=A.length,m=B.length,L=Array.from({length:n+1},()=>new Array(m+1).fill(0));for(let i=n-1;i>=0;i--)for(let j=m-1;j>=0;j--)L[i][j]=A[i]==B[j]?L[i+1][j+1]+1:Math.max(L[i+1][j],L[i][j+1]);let i=0,j=0,out='';while(j<m){if(i<n&&A[i]==B[j]){out+=esc(B[j]);i++;j++}else if(i<n&&L[i+1][j]>=L[i][j+1])i++;else{out+=`<ins>${esc(B[j])}</ins>`;j++}}return out}
function origHtml(r){const fs=[...r.findings].sort((a,b)=>a.location[0]-b.location[0]);let p=0,h='';fs.forEach((f,i)=>{h+=esc(r.original.slice(p,f.location[0]));h+=`<mark class="hl ${f.severity=='crit'?'c':f.severity=='med'||f.severity=='low'?'i':''}" style="color:inherit" title="${esc(f.category)}">${esc(r.original.slice(...f.location))}</mark><sup class="t-meta">${i+1}</sup>`;p=f.location[1]});return h+esc(r.original.slice(p))}
const left=r=>Math.max(0,Math.round((r.expiresAt-Date.now())/1000));
const mmss=s=>Math.floor(s/60)+':'+String(s%60).padStart(2,'0');
/* The unmistakable indicator */
function willSend(r){
 if(['blocked','rejected','expired'].includes(r.state))return `<div class="ws none" role="status"><span class="ws-k">${ic('ban','lg')} NOTHING WILL BE SENT</span><span class="ws-d">${r.state=='blocked'?'Blocked by mandatory policy. No override exists.':r.state=='expired'?'The request timed out and was cancelled. Send it again from your app.':'You rejected this request.'}</span></div>`;
 if(r.state=='approved-transmit')return `<div class="ws done" role="status"><span class="ws-k">${ic('send','lg')} ${r.sentKind=='original'?'ORIGINAL':'TRANSFORMED'} TEXT WAS SENT</span><span class="ws-d">Hash <code>${r.sentHash}</code> as approved.</span></div>`;
 if(r.uninspected)return `<div class="ws orig" role="status"><span class="ws-k">${ic('octagon','lg')} ORIGINAL TEXT WOULD BE SENT: NOT INSPECTED</span><span class="ws-d">Only if you use the per-request override. Otherwise nothing is sent. Original hash <code>${hashO(r)}</code>.</span></div>`;
 const same=text(r)==r.original;
 return `<div class="ws xform" role="status"><span class="ws-k">${ic('shieldcheck','lg')} TRANSFORMED TEXT WILL BE SENT${same?' (identical to original)':''}, NOT THE ORIGINAL</span><span class="ws-d">${r.state=='approved-transform'?'Approval is bound to hash':'Hash if approved'} <code>${hashT(r)}</code>. Any edit changes the hash and clears approval.</span></div>`}
window.review=function(){
 const q=S.reviews.map((x,i)=>`<button class="queue-item" data-act="rv:sel:${i}" aria-current="${i==S.sel}"><div class="grow"><div class="row between">${sev(sevOf(x))}<span class="t-meta mono">${x.id}</span></div><div style="font-weight:550;margin-top:6px">${x.uninspected?'Uninspected content':esc(x.findings[0]?.category||'No findings')}</div><div class="t-meta">${x.app}</div><div class="row wrap" style="margin-top:6px;gap:6px">${sb(x.state)}${OPEN(x)?`<span class="t-meta mono" data-exp="${i}">${mmss(left(x))}</span>`:''}</div></div></button>`).join('');
 const allDone=S.reviews.every(x=>!OPEN(x)&&x.state!='blocked');
 return `<div class="alert info">${ic('info')}<div><div class="ttl">Provisional data contract</div><p>Fields follow the PRD-based interim set and may change when <code>docs/architecture/03-api-contracts.md</code> is published: finding{category, severity, location, confidence, reason}, original, proposed, diff, approvalKind, contentHash, coverageTier, uninspected, expiresAt, state. Request id and app name are display-only placeholders. Timeout of 5 minutes is a design assumption. Sample data.</p></div></div>
 <div class="split"><section class="card flat" aria-label="Review queue"><div class="card-h"><h2 class="t-section">Queue</h2><span class="t-meta">${pendingCount()} open</span></div><div class="col" style="gap:4px">${q}</div></section>
 <section class="stack" aria-live="polite">${allDone?`<div class="card empty"><div class="ico">${ic('check','lg')}</div><h2 class="t-section">Review queue is empty</h2><p class="t-sec">Nothing is waiting.</p></div>`:detail(S.reviews[S.sel])}</section></div>`};
function detail(r){
 const open=OPEN(r),busy=r.state=='edited-reinspecting',b=r.state=='blocked';
 const hdr=`<div class="card"><div class="card-h"><div><h2 class="t-section">${r.id} · ${r.app}</h2><div class="t-meta">${r.app.startsWith('Inbound')?'Inbound response':'Outbound prompt'} · coverage ${st(r.coverageTier=='protected'?'prot':r.coverageTier=='partial'?'part':'none')}</div></div><div class="row">${sev(sevOf(r))}${sb(r.state)}</div></div>
 ${open?`<div class="row wrap" style="gap:8px">${ic('alert')}<span>Expires in <b class="mono" data-exp="${S.sel}">${mmss(left(r))}</b>. If it expires nothing is sent.</span> <button class="btn ghost sm" data-act="rv:expire">Simulate timeout</button></div>`:''}</div>`;
 const ind=willSend(r);
 if(r.state=='expired')return hdr+ind+`<div class="alert warn">${ic('alert','lg')}<div><div class="ttl">Request expired</div><p>No decision was made before the time limit, so Aegis cancelled it. Nothing left your device. Send the prompt again from your app to start a new review.</p></div></div>`+orig(r,true);
 if(b)return hdr+ind+`<div class="alert crit">${ic('octagon','lg')}<div><div class="ttl">Blocked: credential detected. No override.</div><p>Mandatory policy: credentials and private keys are never transmitted, and Aegis will not reconstruct them. There is no approve, edit-and-send or override action for this request.</p></div></div>`+orig(r,true)+`<div class="alert warn">${ic('alert')}<div><div class="ttl">What to do next</div><p>1. Delete the key from your prompt and describe the error instead.<br>2. If the key was shared anywhere, rotate it in your cloud console.<br>3. Use an environment variable or secret reference in code samples.</p></div></div><div class="row"><button class="btn primary" data-act="rv:reject">${ic('x')}Dismiss request</button></div>`;
 let h=hdr+ind;
 if(r.uninspected){h+=orig(r,false)+`<div class="alert warn">${ic('alert','lg')}<div><div class="ttl">Aegis could not inspect part of this request</div><p>The attached file is not scanned (coverage: partially protected). Aegis cannot say what it contains.</p></div></div>
 <div class="card elev"><h2 class="t-section">Your options</h2><div class="grid g2" style="margin-top:12px"><div class="stack" style="gap:8px"><button class="btn primary" data-act="rv:reject">${ic('x')}Cancel request</button><span class="t-meta">Nothing is sent. Remove the attachment and retry.</span></div><div class="stack" style="gap:8px"><button class="btn danger" data-act="dlg:uninspected">${ic('octagon')}Send uninspected (this request only)</button><span class="t-meta">Sends the ORIGINAL text, not inspected. Applies to this one request. Is recorded as metadata only.</span></div></div></div>`;return h}
 // two-column compare
 h+=`<div class="cmp"><div class="pane" style="opacity:.95"><div class="row between" style="margin-bottom:8px"><span class="lbl">Original</span><span class="t-meta">stays on this device</span></div><div class="diff">${origHtml(r)}</div><div class="t-meta mono" style="margin-top:8px">hash ${hashO(r)}</div></div>
 <div class="pane" style="box-shadow:0 0 0 2px var(--accent)"><div class="row between" style="margin-bottom:8px"><span class="lbl">Proposed${r.edited!=null?' (edited)':''}</span><span class="badge b-acc">${ic('send')}This is what leaves</span></div>
 ${S.editing?`<label class="sr" for="ed">Edit proposed text</label><textarea id="ed" class="input" rows="6">${esc(text(r))}</textarea><div class="t-meta" style="margin-top:6px">Editing clears any approval and re-inspects the new text.</div>`:busy?`<div class="row" style="gap:10px;margin:12px 0"><span class="spin"></span> Re-inspecting edited text…</div><div class="skel" style="width:90%"></div><div class="skel" style="margin-top:8px;width:60%"></div>`:`<div class="diff">${r.edited!=null?esc(r.edited):diffIns(r.original,r.proposed)}</div>`}
 <div class="t-meta mono" style="margin-top:8px">hash ${busy?'…':hashT(r)}</div></div></div>`;
 h+=`<div class="card"><h2 class="t-section" style="margin-bottom:8px">Findings (${r.findings.length})</h2><div class="twrap"><table class="table"><thead><tr><th>#</th><th>Category</th><th>Severity</th><th>Location</th><th>Confidence</th><th>Why it matters</th></tr></thead><tbody>${r.findings.map((f,i)=>`<tr><td>${i+1}</td><td style="font-weight:550">${esc(f.category)}</td><td>${sev(f.severity)}</td><td class="mono">${f.location[0]}–${f.location[1]}</td><td><div class="row" style="gap:8px"><div class="bar" style="width:48px"><i style="width:${f.confidence*100}%"></i></div><span class="mono">${Math.round(f.confidence*100)}%</span></div></td><td class="t-sec">${esc(f.reason)}</td></tr>`).join('')}</tbody></table></div><p class="t-meta" style="margin-top:8px">Lower confidence means Aegis is less sure this is sensitive. Anonymisation reduces exposure; it does not guarantee re-identification is impossible.</p></div>`;
 if(r.note)h+=`<div class="alert ${r.noteKind||'ok'}">${ic(r.noteKind=='warn'?'alert':'shieldcheck','lg')}<div><div class="ttl">${r.noteTitle}</div><p>${r.note}</p></div></div>`;
 if(r.state=='approved-transform')h+=`<div class="alert ok">${ic('shieldcheck','lg')}<div><div class="ttl">Re-inspection passed on the exact text above</div><p>Nothing has been sent yet. Step 2 approves sending hash <code>${hashT(r)}</code> only.</p></div></div>`;
 if(r.state=='approved-transmit')h+=`<div class="alert ok">${ic('circlecheck','lg')}<div><div class="ttl">Sent after your two approvals</div><p>Transform approved, then transmit approved for hash <code>${r.sentHash}</code>. The response will be inspected on return.</p></div></div>`;
 if(r.state=='rejected')h+=`<div class="alert info">${ic('info','lg')}<div><div class="ttl">Request rejected</div><p>Nothing was sent. The AI app will show that the request was blocked by Aegis. Undo is available for 30 seconds (assumption).</p></div></div><button class="btn" data-act="rv:undo">${ic('undo')}Undo reject</button>`;
 if(open||r.state=='edited-reinspecting'){
  const can1=r.state=='pending'&&r.proposed!=null,can2=r.state=='approved-transform',canE=(r.state=='pending'||r.state=='approved-transform')&&!S.editing;
  h+=`<div class="card elev"><div class="lbl">Two separate decisions</div><div class="t-sec">Approving a change is not approval to send it.</div>
  <div class="grid g2" style="margin-top:14px"><div class="stack" style="gap:8px"><div class="t-meta">STEP 1 · APPROVE TRANSFORMATION (approvalKind: transform)</div>
   <button class="btn primary" data-act="rv:t" ${can1?'':'disabled'}>${ic('check')}${r.state=='approved-transform'?'Transformation approved':'Approve transformation'}</button>
   <button class="btn" data-act="${S.editing?'rv:save':'rv:edit'}" ${(canE||S.editing)?'':'disabled'}>${ic('pencil')}${S.editing?'Save and re-inspect':'Edit proposed text'}</button>
   <span class="t-meta">Consequence: Aegis prepares and re-inspects this text. Nothing is sent.</span></div>
  <div class="stack" style="gap:8px"><div class="t-meta">STEP 2 · APPROVE TRANSMISSION (approvalKind: transmit)</div>
   <button class="btn primary" data-act="rv:s" ${can2?'':'disabled aria-describedby="why-dis"'}>${ic('send')}Approve and send</button>
   <span class="t-meta" id="why-dis">${can2?`Consequence: sends the TRANSFORMED text with hash ${hashT(r)} to the AI app, once. The original never leaves.`:'Locked until step 1 is approved and re-inspection passes. Editing locks it again.'}</span></div></div>
  <div class="divider"></div><div class="row wrap"><button class="btn danger" data-act="dlg:reject">${ic('x')}Reject request</button><span class="t-meta grow">Consequence: nothing is sent; the app is told the request was blocked.</span></div></div>`}
 return h}
function orig(r,solo){return `<div class="pane"><div class="row between" style="margin-bottom:8px"><span class="lbl">What you wrote</span><span class="t-meta">${r.findings.length} finding${r.findings.length==1?'':'s'}</span></div><div class="diff">${origHtml(r)}</div></div>`}
/* dialogs */
const oldDialog=window.dialog;
window.dialog=function(){const r=S.reviews[S.sel];
 if(S.dlg=='reject')return `<div class="scrim" data-act="closeDlg"><div class="dialog" role="dialog" aria-modal="true" aria-labelledby="dt"><h2 id="dt">Reject this request?</h2><div class="t-sec">Nothing will be sent. The AI app will show that Aegis blocked the request. You can undo for 30 seconds.</div><div class="acts"><button class="btn" data-act="closeDlg">Keep reviewing</button><button class="btn danger" data-act="rv:reject">Reject request</button></div></div></div>`;
 if(S.dlg=='uninspected')return `<div class="scrim" data-act="closeDlg"><div class="dialog" role="dialog" aria-modal="true" aria-labelledby="dt"><h2 id="dt">Send without inspection?</h2><div class="t-sec">Aegis could not inspect the attached file. Sending the ORIGINAL means unknown content leaves your device and cannot be recalled. This applies to this one request only; the next one is inspected normally.</div><label class="check" style="margin-top:14px"><input type="checkbox" data-act="rv:ack" ${S.ack?'checked':''}><span>I understand this content has not been inspected.</span></label><div class="acts"><button class="btn" data-act="closeDlg">Cancel</button><button class="btn danger" data-act="rv:unins" ${S.ack?'':'disabled'}>Send uninspected, this request only</button></div></div></div>`;
 if(S.dlg=='why')return `<div class="scrim" data-act="closeDlg"><div class="dialog" role="dialog" aria-modal="true" aria-labelledby="dt"><h2 id="dt">Detailed explanation</h2><div class="list">${r.findings.map(f=>`<div class="li" style="align-items:flex-start"><b style="width:130px;flex:none">${esc(f.category)}</b><span class="t-sec">${esc(f.reason)}</span></div>`).join('')||'<p class="t-sec">No findings.</p>'}</div><div class="acts"><button class="btn primary" data-act="closeDlg">Close</button></div></div></div>`;
 return oldDialog()};
/* actions */
const prevAct=window.EXTRA_ACT;
window.EXTRA_ACT=function(k,v,el){
 if(k!='rv')return prevAct?prevAct(k,v,el):false;
 const [a,x]=v.split(':'),r=S.reviews[S.sel],go=()=>render();
 if(a=='sel'){S.sel=+x;S.editing=false;go()}
 else if(a=='t'){r.state='approved-transform';r.note=null;go()}
 else if(a=='s'){r.sentKind='transformed';r.sentHash=hashT(r);r.state='approved-transmit';go();toast('Sent. Approval was bound to hash '+r.sentHash)}
 else if(a=='edit'){S.editing=true;go()}
 else if(a=='save'){const t=document.getElementById('ed').value;r.edited=t;S.editing=false;r.state='edited-reinspecting';go();setTimeout(()=>{const em=/[\w.+-]+@[\w-]+\.[\w.]+/.test(t);r.state='pending';r.noteKind=em?'warn':'ok';r.noteTitle=em?'Re-inspection found an email address in your edit':'Edited text re-inspected: no new findings';r.note=em?'Remove it or accept the risk. Approval was cleared because the text changed.':'Your earlier approval was cleared because the content hash changed. Approve the transformation again.';go()},1200)}
 else if(a=='reject'){r.state=r.state=='blocked'?'rejected':'rejected';S.dlg=null;go();toast('Request rejected. Nothing was sent.')}
 else if(a=='undo'){r.state='pending';go()}
 else if(a=='expire'){r.expiresAt=Date.now();tick()}
 else if(a=='ack'){S.ack=!S.ack;go()}
 else if(a=='unins'){r.sentKind='original';r.sentHash=hashO(r);r.state='approved-transmit';S.dlg=null;S.ack=false;go();toast('Sent without inspection, this request only')}
 return true};
function tick(){let ch=false;S.reviews.forEach((r,i)=>{if(OPEN(r)){const s=left(r);document.querySelectorAll(`[data-exp="${i}"]`).forEach(e=>e.textContent=mmss(s));if(s<=0&&!S.editing){r.state='expired';ch=true}}});if(ch)render()}
setInterval(tick,1000);
R.review=window.review;
})();

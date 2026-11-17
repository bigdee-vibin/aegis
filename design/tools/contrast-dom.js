/* Rendered-pixel contrast check (WCAG 2.x). Paste into the browser console, or run through a browser tool, on the page
   you want to measure. Usage: aegisContrast(document.querySelector('#content'))  -> {checked, failures[], min}
   Method: for every element that owns visible text, take its computed colour, composite the translucent backgrounds
   of its ancestors over the body background, and compare. Text: 4.5:1 (3:1 for large text: >=24px or >=18.66px bold).
   Icons (svg with stroke currentColor): 3:1. Limits: ignores gradients, images and backdrop-filter blur (uses the
   background colour alpha only); does not measure :hover/:focus/disabled variants; disabled controls are skipped
   (WCAG exempts inactive components). */
(function(){
function parse(c){c=c.trim();let m=c.match(/^rgba?\(([^)]+)\)/);if(m){const p=m[1].split(/[ ,\/]+/).filter(Boolean).map(Number);return[p[0],p[1],p[2],p.length>3?p[3]:1]}
 m=c.match(/^color\(srgb ([^)]+)\)/);if(m){const p=m[1].split(/[ \/]+/).filter(Boolean).map(Number);return[p[0]*255,p[1]*255,p[2]*255,p.length>3?p[3]:1]}
 return[0,0,0,0]}
const over=(f,b)=>{const a=f[3];return[f[0]*a+b[0]*(1-a),f[1]*a+b[1]*(1-a),f[2]*a+b[2]*(1-a),1]};
const lum=c=>{const f=v=>{v/=255;return v<=.03928?v/12.92:Math.pow((v+.055)/1.055,2.4)};return .2126*f(c[0])+.7152*f(c[1])+.0722*f(c[2])};
const ratio=(a,b)=>{const l1=lum(a),l2=lum(b);const hi=Math.max(l1,l2),lo=Math.min(l1,l2);return (hi+.05)/(lo+.05)};
function bgOf(el){const stack=[];for(let e=el;e;e=e.parentElement){const c=parse(getComputedStyle(e).backgroundColor);if(c[3]>0)stack.push(c);if(c[3]>=1)break}
 let b=parse(getComputedStyle(document.body).backgroundColor);if(b[3]===0)b=[255,255,255,1];for(let i=stack.length-1;i>=0;i--)b=over(stack[i],b);return b}
window.aegisContrast=function(root){const out={checked:0,failures:[],min:99};
 const walker=document.createTreeWalker(root,NodeFilter.SHOW_TEXT);const seen=new Set();
 while(walker.nextNode()){const n=walker.currentNode;const t=n.nodeValue.trim();if(!t)continue;const el=n.parentElement;if(!el||seen.has(el))continue;
  const cs=getComputedStyle(el);if(cs.visibility==='hidden'||cs.display==='none'||el.closest('[disabled],[aria-disabled="true"],.sr'))continue;const r=el.getBoundingClientRect();if(r.width===0||r.height===0)continue;
  seen.add(el);const bg=bgOf(el);const fg=over(parse(cs.color),bg);const size=parseFloat(cs.fontSize),bold=parseInt(cs.fontWeight,10)>=700;const need=(size>=24||(size>=18.66&&bold))?3:4.5;
  const rt=ratio(fg,bg);out.checked++;out.min=Math.min(out.min,rt);if(rt<need)out.failures.push({text:t.slice(0,40),ratio:+rt.toFixed(2),need,color:cs.color})}
 root.querySelectorAll('svg.i').forEach(s=>{const cs=getComputedStyle(s);if(s.closest('[disabled],.sr')||s.getBoundingClientRect().width===0)return;const bg=bgOf(s);const fg=over(parse(cs.color),bg);const rt=ratio(fg,bg);out.checked++;out.min=Math.min(out.min,rt);if(rt<3)out.failures.push({text:'[icon]',ratio:+rt.toFixed(2),need:3,color:cs.color})});
 out.min=+out.min.toFixed(2);return out};
})();

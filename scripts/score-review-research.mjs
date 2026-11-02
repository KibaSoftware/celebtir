// Explicit evidence operation, never imported by offline tests. Four public GETs
// and at most two finalized RPC batches; no credentials, transaction submission or retries.
import fs from 'node:fs';
import crypto from 'node:crypto';
const solstice=process.argv.includes('--solstice');
const dir='output/reviews/score-review-2026-10-26/'+(solstice?'research-solstice':'research');
if(fs.existsSync(`${dir}/chain-observations.json`))throw new Error('Capture already exists; preserve it.');
fs.mkdirSync(dir,{recursive:true});
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const sources=[['marinade','https://docs.marinade.finance/developers/contract-addresses.md'],['phoenix','https://raw.githubusercontent.com/Ellipsis-Labs/phoenix-v1/master/README.md'],['solstice','https://docs.solstice.finance/'],['solstice-security','https://website.prod.solsticelabs.io/security']];
const records=[];
for(const [id,url] of solstice?[]:sources){
 try{const r=await fetch(url,{signal:AbortSignal.timeout(20000)});const b=Buffer.from(await r.arrayBuffer());const file=`${dir}/${id}.txt`;fs.writeFileSync(file,b);records.push({id,url,status:r.status,path:file,sha256:hash(b),captured_at_utc:new Date().toISOString()});}catch(e){records.push({id,url,status:null,error:e.message});}
}
fs.writeFileSync(`${dir}/sources.json`,JSON.stringify(records,null,2));
const ids=solstice?{solstice_usx:'USXyiSTsPEWz55pSK7sZoUL79ntoVGQbaTDT57tH6bx',solstice_yieldvault:'eUSXyKoZ6aGejYVbnp3wtWQ1E8zuokLAJPecPxxtgG3'}:{orca:'whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc',jito:'SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy',marinade:'MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD',phoenix:'PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY'};
const alphabet='123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
function base58(b){let n=BigInt('0x'+b.toString('hex')),s='';while(n){s=alphabet[Number(n%58n)]+s;n/=58n;}for(const x of b){if(x!==0)break;s='1'+s;}return s;}
const capture={rpc_host:'api.mainnet-beta.solana.com',commitment:'finalized',captured_at_utc:new Date().toISOString(),budget_calls:2,calls:0,programs:[],errors:[]};
async function batch(addresses,length,name){
 capture.calls++;
 const r=await fetch('https://api.mainnet-beta.solana.com',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({jsonrpc:'2.0',id:capture.calls,method:'getMultipleAccounts',params:[addresses,{commitment:'finalized',encoding:'base64',dataSlice:{offset:0,length}}]}),signal:AbortSignal.timeout(20000)});
 const b=Buffer.from(await r.arrayBuffer());fs.writeFileSync(`${dir}/${name}.json`,b);
 if(!r.ok)throw new Error(`RPC HTTP ${r.status}`);
 const j=JSON.parse(b);if(j.error)throw new Error(`RPC ${j.error.code}: ${j.error.message}`);return j.result;
}
try{
 const first=await batch(Object.values(ids),36,'program-accounts');
 for(const [i,[id,address]] of Object.entries(ids).entries()){
  const a=first.value[i],b=a?Buffer.from(a.data[0],'base64'):null;
  const programdata=a?.owner==='BPFLoaderUpgradeab1e11111111111111111111111'&&a.executable&&b?.length===36&&b.readUInt32LE(0)===2?base58(b.subarray(4)):null;
  capture.programs.push({id,address,slot:first.context.slot,executable:a?.executable??null,owner:a?.owner??null,programdata,upgrade_authority:null});
 }
 const targets=capture.programs.filter(p=>p.programdata);
 if(targets.length){const next=await batch(targets.map(p=>p.programdata),45,'programdata-accounts');for(const [i,p] of targets.entries()){
  const a=next.value[i],b=a?Buffer.from(a.data[0],'base64'):null;p.programdata_slot=next.context.slot;
  if(a?.owner==='BPFLoaderUpgradeab1e11111111111111111111111'&&b?.length>=13&&b.readUInt32LE(0)===3){p.last_deploy_slot=Number(b.readBigUInt64LE(4));p.upgrade_authority=b[12]===0?null:b.length===45?base58(b.subarray(13,45)):null;p.authority_read=b[12]===0||b[12]===1&&b.length===45;}else p.authority_read=false;
 }}
}catch(e){capture.errors.push(e.message);}
capture.raw_integrity=Object.fromEntries(['program-accounts','programdata-accounts'].filter(n=>fs.existsSync(`${dir}/${n}.json`)).map(n=>[n,hash(fs.readFileSync(`${dir}/${n}.json`))]));
fs.writeFileSync(`${dir}/chain-observations.json`,JSON.stringify(capture,null,2));
console.log(JSON.stringify({sources:records.map(r=>({id:r.id,status:r.status,error:r.error})),chain:capture}));

// Opt-in public HTTP/browser evidence operation. Normal offline gates skip it.
import {test,expect} from '@playwright/test';
import {mkdirSync,writeFileSync} from 'node:fs';
import path from 'node:path';
const enabled=process.env.CELEBTIR_HOSTED_REVIEW==='1';
const output=path.resolve(__dirname,'../../output/reviews/score-review-2026-10-26/hosted-browser');
test('srd_07 hosted route and methodology observations',async({page},info)=>{
 test.skip(!enabled||info.project.name!=='desktop','Explicit public-HTTP review only.');
 test.setTimeout(180000);mkdirSync(output,{recursive:true});
 const errors:string[]=[],failed:string[]=[],results:unknown[]=[];
 page.on('pageerror',e=>errors.push(e.message));page.on('requestfailed',r=>failed.push(r.url()));
 const summary=await (await page.request.get('https://celebtir.com/data/v2/summary.json')).json();
 const routes=['/','/risk/','/value/','/activity/','/tokens/','/compare/?p=orca,jito','/changelog/','/controllers/','/hyperevm/','/hip3/','/methodology/','/protocols/orca/','/api/','/tokens/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/'];
 for(const [width,height] of [[1440,900],[768,1024],[375,812]]){
  await page.setViewportSize({width,height});
  for(const [i,route] of routes.entries()){
   const r=await page.goto('https://celebtir.com'+route);await page.waitForLoadState('networkidle');
   const overflow=await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1);
   const text=await page.locator('body').innerText();
   results.push({width,route,status:r?.status(),overflow,methodology_label:text.includes(`methodology v${summary.methodology_version}`),snapshot_label:text.includes(summary.snapshot_date)});
   await page.screenshot({path:path.join(output,`${width}-${i}.png`),fullPage:true});
  }
 }
 const result={captured_at_utc:new Date().toISOString(),summary:{snapshot_date:summary.snapshot_date,methodology_version:summary.methodology_version},results,errors,failed};
 writeFileSync(path.join(output,'results.json'),JSON.stringify(result,null,2));
 expect(results.filter(r=>(r as {status:number}).status!==200)).toEqual([]);
 expect(errors).toEqual([]);expect(failed).toEqual([]);
 expect(results.filter(r=>(r as {overflow:boolean}).overflow)).toEqual([]);
 expect(results.filter(r=>!(r as {methodology_label:boolean}).methodology_label)).toEqual([]);
});

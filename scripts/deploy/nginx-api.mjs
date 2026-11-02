import fs from 'node:fs';
import { pathToFileURL } from 'node:url';

// Add v2 alongside v1: an unsuccessful later app build can still serve v1.
export function migrateApi(text) {
  if (!/^# Managed by Celebtir deployment\r?$/m.test(text)) throw new Error('Unmanaged Celebtir Nginx configuration');
  const block = version => new RegExp(`^([ \\t]*)location /data/${version}/ \\{\\r?\\n([\\s\\S]*?)^\\1\\}`, 'm').exec(text);
  const valid = b => b && !/[{}]/.test(b[2]) && /add_header\s+Access-Control-Allow-Origin\s+"\*"\s+always;/.test(b[2]) && /try_files\s+\$uri\s+=404;/.test(b[2]);
  if (/location[^\n]*\/data\/v2\//.test(text)) {
    if (!valid(block('v2'))) throw new Error('Existing v2 API location needs manual inspection');
    return text;
  }
  const old = block('v1');
  if (!valid(old)) throw new Error('Expected managed v1 API location with CORS and try_files');
  return text.replace(old[0], `${old[0]}\n${old[0].replace('/data/v1/', '/data/v2/')}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [input, output] = process.argv.slice(2);
  fs.writeFileSync(output, migrateApi(fs.readFileSync(input, 'utf8')));
}

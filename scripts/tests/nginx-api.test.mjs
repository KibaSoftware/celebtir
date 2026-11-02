import test from 'node:test';
import assert from 'node:assert/strict';
import { migrateApi } from '../deploy/nginx-api.mjs';
const config = `# Managed by Celebtir deployment
server {
    listen 443 ssl;
    server_name celebtir.com;
    ssl_certificate /etc/letsencrypt/live/celebtir.com/fullchain.pem;
    location /data/v1/ {
        add_header Access-Control-Allow-Origin "*" always;
        add_header X-Robots-Tag "noindex, nofollow" always;
        try_files $uri =404;
    }
    location / { try_files $uri $uri/ =404; }
}
`;
test('migration is additive, preserves TLS and all old routes, and is idempotent', () => {
  const next = migrateApi(config);
  const added = next.match(/    location \/data\/v2\/ \{[\s\S]*?    \}/)[0];
  assert.equal(next.replace('\n'+added,''),config);
  assert.equal(migrateApi(next),next);
});
for(const [name, input] of [
  ['unmanaged file',config.replace('# Managed by Celebtir deployment','')],
  ['missing route',config.replace('/data/v1/','/private/')],
  ['missing CORS',config.replace('Access-Control-Allow-Origin','Other-Header')],
  ['conflicting v2',config.replace('location /data/v1/', 'location ^~ /data/v2/')],
  ['nested directives',config.replace('try_files $uri =404;', 'if ($x) { return 404; }')],
]) test(`refuses ${name}`,()=>assert.throws(()=>migrateApi(input)));

import { pathToFileURL } from 'node:url';

export async function checkRpc(value, request = fetch) {
  const endpoint = value?.trim() || 'https://api.mainnet-beta.solana.com';
  let url;
  try { url = new URL(endpoint); } catch { throw new Error('Invalid Solana RPC URL: paste only the full URL, without quotes or VARIABLE=.'); }
  if (url.protocol !== 'https:' || url.username || url.password) throw new Error('Solana RPC must be an HTTPS URL without username/password credentials.');
  let response;
  try {
    response = await request(url, {
      method: 'POST', redirect: 'error', signal: AbortSignal.timeout(20000),
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({jsonrpc: '2.0', id: 1, method: 'getMultipleAccounts', params: [['11111111111111111111111111111111'], {commitment: 'finalized', encoding: 'base64', dataSlice: {offset: 0, length: 0}}]})
    });
  } catch { throw new Error('Solana RPC preflight could not connect or timed out. Endpoint details suppressed.'); }
  if (response.status === 401 || response.status === 403) throw new Error(`Solana RPC rejected authentication (HTTP ${response.status}). Check the complete provider URL/key and any provider access restrictions; run deploy.ps1 -Mode Install and choose to replace the saved Solana RPC URL.`);
  if (!response.ok) throw new Error(`Solana RPC preflight failed (HTTP ${response.status}).`);
  let body;
  try { body = await response.json(); } catch { throw new Error('Solana RPC returned invalid JSON.'); }
  if (body.error || !Array.isArray(body.result?.value) || body.result.value.length !== 1 || !body.result.value[0] || !Number.isInteger(body.result.context?.slot)) throw new Error('Solana RPC batch read was rejected or returned an unexpected response. Provider details suppressed.');
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  try { await checkRpc(process.env.CELEBTIR_RPC_URL); console.log('Solana RPC authentication and finalized batch read: OK'); }
  catch (error) { console.error(error.message); process.exitCode = 1; }
}

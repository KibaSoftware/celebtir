import test from 'node:test';
import assert from 'node:assert/strict';
import { checkRpc } from '../deploy/rpc-preflight.mjs';

test('preflight uses full keyed URL and finalized batch read without redirection', async () => {
  await checkRpc('https://example.invalid/?api-key=test-secret', async (url, init) => {
    assert.equal(url.searchParams.get('api-key'), 'test-secret');
    assert.equal(init.redirect, 'error');
    const body = JSON.parse(init.body);
    assert.equal(body.method, 'getMultipleAccounts');
    assert.equal(body.params[1].commitment, 'finalized');
    return {ok: true, json: async () => ({result: {context: {slot: 42}, value: [{}]}})};
  });
});
test('HTTP authentication, transport and provider errors do not disclose secrets', async () => {
  for (const request of [async () => ({status: 401}), async () => { throw new Error('test-secret'); }, async () => ({ok: true, json: async () => ({error: {message: 'test-secret'}})})]) {
    await assert.rejects(checkRpc('https://example.invalid/?api-key=test-secret', request), error => !error.message.includes('test-secret'));
  }
});
test('quoted or assignment input fails before a request', async () => {
  for (const value of ['"https://example.invalid"', 'CELEBTIR_RPC_URL=https://example.invalid', 'http://example.invalid']) {
    await assert.rejects(checkRpc(value, async () => { assert.fail('must not connect'); }));
  }
});

// Node.js example using fetch to drive magic link, TOTP, and refresh flows.
// Run with: node example.js

const fetch = (...args) => import('node-fetch').then(({default: f}) => f(...args));

async function requestMagic(email) {
  const res = await fetch('http://localhost:3000/request/magic', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email }),
  });
  console.log('requestMagic status', res.status);
}

async function verifyMagic(token) {
  const res = await fetch(`http://localhost:3000/verify/magic?token=${encodeURIComponent(token)}`);
  const json = await res.json();
  console.log('verifyMagic response', json);
  return json;
}

async function totpEnroll(email) {
  const res = await fetch('http://localhost:3000/totp/enroll', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email }),
  });
  const json = await res.json();
  console.log('TOTP enroll', json);
  return json;
}

async function totpVerify(email, code) {
  const res = await fetch('http://localhost:3000/totp/verify', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, code }),
  });
  const json = await res.json();
  console.log('TOTP verify', json);
  return json;
}

async function refreshToken(refresh_token) {
  const res = await fetch('http://localhost:3000/token/refresh', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ refresh_token }),
  });
  const json = await res.json();
  console.log('refreshToken', json);
  return json;
}

(async () => {
  const email = 'test@example.com';
  await requestMagic(email);
  console.log('Check DB for the magic token to simulate click (or implement an inbox).');
  // Further flows would require retrieving the token from DB or email.
})();

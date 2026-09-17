// Developer-only release guard. Reports filenames/categories, never secret values.
import { execFileSync } from 'node:child_process';
import { readFileSync, existsSync } from 'node:fs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '..');
const names = execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', '-z'], {cwd:root}).toString().split('\0').filter(Boolean);
const patterns = {
  privateKey: /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/,
  githubToken: /gh[pousr]_[A-Za-z0-9]{30,}/,
  providerKey: /sk-(?:proj-)?[A-Za-z0-9_-]{24,}/,
  awsKey: /AKIA[0-9A-Z]{16}/,
};
const findings=[];
for (const name of new Set(names)) {
  if (!existsSync(path.join(root,name))) continue;
  if (/\.(pfx|p12|pem|key|keystore|jks)$/i.test(name)) findings.push({file:name,type:'signing material'});
  const data=readFileSync(path.join(root,name));
  if (data.subarray(0,8000).includes(0)) continue;
  for (const [type,pattern] of Object.entries(patterns)) if (pattern.test(data.toString())) findings.push({file:name,type});
}
if(findings.length) { console.error(JSON.stringify(findings));process.exit(1); }
for(const extension of ['pfx','p12','pem','key','jks','keystore']) {
  execFileSync('git',['check-ignore','--no-index','--quiet',`release-credential.${extension}`],{cwd:root});
}
const config=JSON.parse(readFileSync(path.join(root,'src-tauri/tauri.conf.json')));
assert.equal(config.bundle.windows.webviewInstallMode.type,'skip');
assert.equal(config.bundle.windows.nsis.installMode,'currentUser');
assert(!JSON.stringify(config).includes(root));
const template=readFileSync(path.join(root,'src-tauri/installer/installer.nsi'),'utf8');
assert(!/NSISdl::download|ExecWait.*(?:powershell|cmd\.exe)|RmDir \/r\s/i.test(template));
assert(!/WriteReg.*CurrentVersion\\Run/i.test(template));
const manifestPath=path.join(root,'src-tauri/target/release/release-integrity/release-manifest.json');
if(existsSync(manifestPath)) {
  const manifest=JSON.parse(readFileSync(manifestPath,'utf8').replace(/^\uFEFF/,''));
  for(const entry of manifest.artifacts) {
    const file=path.join(root,'src-tauri/target/release',entry.file==='clearce.exe'?'':'bundle/nsis',entry.file);
    assert.equal(createHash('sha256').update(readFileSync(file)).digest('hex'),entry.sha256);
  }
  assert.equal(manifest.engineBundled,false);
  assert.equal(manifest.signed,manifest.artifacts.every(a=>a.signatureStatus==='Valid'));
}
console.log('Release guards passed: secret patterns, signing ignores, installer policy, config, available manifest hashes.');

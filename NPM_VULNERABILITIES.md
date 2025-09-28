PS D:\GitHub\BEAR-LLM> npm i --package-lock-only

up to date, audited 349 packages in 24s

124 packages are looking for funding
  run `npm fund` for details

5 moderate severity vulnerabilities

To address all issues (including breaking changes), run:
  npm audit fix --force

Run `npm audit` for details.
PS D:\GitHub\BEAR-LLM> npm audit fix

added 348 packages, and audited 349 packages in 4m

124 packages are looking for funding
  run `npm fund` for details

# npm audit report

esbuild  <=0.24.2
Severity: moderate
esbuild enables any website to send any requests to the development server and read the response - https://github.com/advisories/GHSA-67mh-4wv8-2f99
fix available via `npm audit fix --force`
Will install vite@7.1.7, which is a breaking change
node_modules/esbuild
  vite  0.11.0 - 6.1.6
  Depends on vulnerable versions of esbuild
  node_modules/vite

prismjs  <1.30.0
Severity: moderate
PrismJS DOM Clobbering vulnerability - https://github.com/advisories/GHSA-x7hr-w5r2-h6wg
fix available via `npm audit fix --force`
Will install react-syntax-highlighter@5.8.0, which is a breaking change
node_modules/refractor/node_modules/prismjs
  refractor  <=4.6.0
  Depends on vulnerable versions of prismjs
  node_modules/refractor
    react-syntax-highlighter  >=6.0.0
    Depends on vulnerable versions of refractor
    node_modules/react-syntax-highlighter

5 moderate severity vulnerabilities
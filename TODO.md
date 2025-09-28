[] THIRD_PARTY_LICENSES.txt does not exist
[] https://github.com/KingOfTheAce2/BEAR-LLM/ is the repo URL. Make sure README.md reflects this
[] CONTRIBUTE.md is absent; however this is not open source, only open code for transparancy reasons. Create CONTRIBUTE.md
[] Add PRIVACY.md and explain why PII is needed for this usecase and also for the reason it is required for lawyers in e.g. France, Germany, Beligum
[] Include US and Asian GDPR equivalents to PRIVACY.md for full world legal coverage
[] Check: 
running npm [ 'run', 'tauri', 'build' ]
> legal-ai-assistant@0.1.0 tauri
> tauri build
        Info Looking up installed tauri packages to check mismatched versions...
       Error Failed to parse version `2` for crate `tauri`
       Error Failed to parse version `2` for crate `tauri-plugin-dialog`
       Error Failed to parse version `2` for crate `tauri-plugin-os`
       Error Failed to parse version `2` for crate `tauri-plugin-shell`
       Error Failed to parse version `2` for crate `tauri-plugin-fs`
     Running beforeBuildCommand `npm run build`
> legal-ai-assistant@0.1.0 build
> vite build
vite v5.4.20 building for production...
transforming...
[vite:css] @import must precede all other statements (besides @charset or empty @layer)
3  |  @tailwind utilities;
4  |  
5  |  @import url('/fonts/inter.css');
   |  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
6  |  
7  |  :root {
✓ 2680 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                   0.50 kB │ gzip:   0.33 kB

(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rollupOptions.output.manualChunks to improve chunking: https://rollupjs.org/configuration-options/#output-manualchunks
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.
dist/assets/index-CRjY2IBW.css   19.70 kB │ gzip:   4.86 kB
dist/assets/index-YqA6Cuqi.js   931.77 kB │ gzip: 320.77 kB
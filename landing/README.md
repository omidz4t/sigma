# Sigma landing page

Svelte 5 + SvelteKit 3, built as a client-rendered SPA using adapter-static. The Rust CLI and its dependencies are independent of the website's Node toolchain. Use Node 24+ and npm.

```sh
cd landing
npm ci
npm run dev
```

Open the URL printed by Vite. Edits reload automatically. To retain the old preview address, run `python3 scripts/serve-landing.py --port 8080` from the repository root; this launches Vite with `/landing` as its base and redirects `/landing/index.html` to `/landing/`.

```sh
npm run check
npm run build
npm run preview
```

`src/routes/+page.svelte` owns the prompt modal, editable prompt, copy state and approximate token count. `src/lib/components/` contains the logo and use cases. `src/lib/logo/` contains horizontal-only mouse tracking and the slow scroll docking animation, with cleanup on unmount and reduced-motion support. Styles are in `src/app.css`. Heroicons remain inline SVG under their MIT license.

The five public generated Moai views live in `static/assets/logo/`. No external fonts, scripts, analytics, or account data are loaded at runtime. The emoji favicon is `static/logo.svg`.

## Reference and AI prompt

`static/reference.html` is the complete standalone reference, served at `/reference.html` (under the configured base). It embeds its styles and works offline. `docs/REFERENCE.md` contains the same material for tools and AI agents. Run `python3 scripts/build-reference.py` from the repository root after documentation changes.

After building the Rust release executable, run `python3 scripts/prepare-landing.py`. This generates `src/lib/prompt-data.js` as an ES module containing all command help and public versioned GitHub Release addresses. Automatic releases update these links too. Binaries and credentials never belong in the website source or build output.

## GitHub Pages

The workflow builds with `BASE_PATH=/sigma`, uploads only `landing/build/`, and deploys to https://omidz4t.github.io/sigma/. SvelteKit SSR is disabled; adapter-static generates `index.html` as the SPA entry. The workflow adds `404.html` for Pages fallback and `.nojekyll`. The standalone reference stays a static file. Pages source must be GitHub Actions.

Use `BASE_PATH=/another-path npm run build` for other subdirectory deployments. Leave it unset for a root deployment. Build output, dependencies and SvelteKit generated files are ignored by Git; the npm lockfile is committed source.

## Command documentation routes

`/docs/` is the SvelteKit command index; `/docs/COMMAND/` gives each command its own page. The sidebar searches names and descriptions. All 72 pages use `src/lib/commands.js`, which reads the same generated CLI help as the AI prompt, so `scripts/prepare-landing.py` updates both together. SvelteKit prerenders the documentation with trailing slashes for direct GitHub Pages links; subsequent navigation uses the SPA router. The complete standalone reference remains available at `/reference.html`.

## GitHub star count

The Star button uses a GitHub-style local UI and links to GitHub for starring. `python3 scripts/prepare-repo-stats.py` snapshots the official GitHub repository API's `stargazers_count` into `src/lib/repo-stats.json`. Pages refreshes it on every deployment and daily at 06:17 UTC. The displayed count is a build snapshot, not a live widget; its tooltip shows the fetch date. Visitors load no third-party scripts or API calls. API failures fail the deployment instead of publishing a fabricated zero. Only the public count and timestamp are written; the Actions token stays in the workflow environment.

## Terminal-to-phone walkthrough

`TerminalShowcase.svelte` uses six real Android screenshots in `static/assets/showcase/`. They were captured from Delta Chat 2.69.0 on Android using a fresh `Sigma Demo` phone profile and a separate Sigma bot account. The sequence shows bot initialization (not a connection), the phone QR screen, LINK → COPY TO CLIPBOARD, create-contact completing secure join, message receipt, and report.txt delivery. Commands were executed against the isolated bot account; the displayed invitation is a placeholder. The demo invitation visible in the QR/link captures was reset after recording; it is no longer usable. No account databases or credentials are published.

The walkthrough has three main steps: initialize, connect, and send. Initialize shows Delta Chat alone; Connect has four views (chat list, QR invitation, copy-link dialog, connected bot); Send has message and file views. The phone frame stays visible while scrolling. A scroll handler keeps the current result until the complete command box passes the sticky heading at the top; mobile keeps the current step until its inline phone capture passes that heading; numbered main-step and substep controls also let people jump directly. Overlay arrows point to the QR, LINK and COPY TO CLIPBOARD controls. Mobile shows a large screenshot beneath each step, and reduced-motion preference disables transitions and smooth scrolling. Screenshots are unedited 720×1656 captures of the app. The webpage clips the bottom 96 pixels containing the system navigation strip and displays the app in a 6:13 phone frame, without stretching it. The demo APK came from the official deltachat/deltachat-android v2.69.0 GitHub Release.

## Languages

The site supports English, German, Persian and Russian. On first visit it uses the first supported browser language, falling back to English. The language selector overrides this and remembers the choice locally. Persian uses right-to-left layout; executable commands, paths and code stay left-to-right. CLI syntax and the complete AI instruction prompt retain their original English form.

Dictionaries live in `src/lib/locales/`. The German, Persian and Russian translations were generated with `~/ai_local` using `openrouter/google/gemini-3.5-flash-lite`, then checked for exact key coverage. Native-language writing prompts in `scripts/translation-prompts/` define tone, terminology and preservation rules. From the repository root, run `python3 scripts/translate-site.py --check de fa ru` to validate coverage, or `python3 scripts/translate-site.py de fa ru` to regenerate using the model. Generation requires your configured ai_local account and incurs model usage. Add new UI source strings to the English dictionary before regenerating.

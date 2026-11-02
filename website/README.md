# vgui documentation site

Astro + Starlight site for vgui, with bilingual Start Here guides and API documentation.

## Commands

```sh
npm ci
npm run dev       # Starlight development server
npm run check     # Astro and TypeScript diagnostics
npm run build     # Check, build the site and Pagefind, then append cargo doc
npm run preview   # Preview the complete production build
npm run test:e2e  # Browser smoke tests; requires a build and Chromium
```

Install the test browser once with `npx playwright install chromium`.

Pagefind and the on-site API documentation are production-build features. `npm run dev` does not generate either one. Run `npm run build && npm run preview` to verify search and `/vgui/api/vgui/`.

The Rustdoc step runs `cargo doc --workspace --no-deps --locked` from the repository root, then copies `target/doc` to `dist/api`. Copying happens after Astro's Pagefind pass so generated Rustdoc pages do not replace or pollute the documentation search index.

## Content and locales

- English is the root locale; Simplified Chinese lives under `src/content/docs/zh-cn/`.
- Add matching relative paths for translated pages so Starlight's language selector preserves the current page.
- Shared navigation labels are in `src/content/i18n/en.json` and `zh-CN.json` and typed in `src/content.config.ts`.
- The GitHub Pages deployment uses base `/vgui`. Custom internal links must use `basePath` or `localizedPath` from `src/utils/paths.ts`, not hard-coded root URLs.
- Repository links and the version catalog are centralized in `src/config/site.ts`. Add a historical version only when its deployed snapshot exists.

The current `v0.1.0` entry mirrors the workspace version in the root `Cargo.toml`; update both during a release.

## Deployment

`.github/workflows/docs.yml` builds and tests the site, uploads `website/dist`, and deploys it to GitHub Pages on pushes to `master`/`main` or manual dispatch. Pull requests run the build without deploying.

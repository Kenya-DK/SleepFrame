# SleepFrame docs (GitHub Pages)

This folder is a self-contained static site (no build step) used as the SleepFrame
**how-to wiki**. Open `index.html` directly, or publish it with GitHub Pages.

## Publish with GitHub Pages

1. Push this repository to GitHub.
2. Go to **Settings → Pages**.
3. Under **Build and deployment**, set **Source** to **Deploy from a branch**.
4. Choose **Branch: `main`** and **Folder: `/docs`**, then **Save**.
5. After a minute the site is live at `https://<user>.github.io/<repo>/`.

## Editing

- The whole wiki is `docs/index.html` — a single file with inline CSS, so it works
  offline and needs no dependencies.
- `.nojekyll` tells GitHub Pages to serve the files as-is (skips Jekyll).
- Add sections by editing `index.html` and adding an entry to the `<nav class="toc">`.

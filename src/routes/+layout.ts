// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps

import { getAppState } from "$lib/stores/state.svelte";
import { redirect } from "@sveltejs/kit";

// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const ssr = false;

export const load = async () => {
  const appState = getAppState();

  await appState.loadState();

  if (appState.isFirstNavigation) {
    appState.isFirstNavigation = false;
    if (appState.selectedSpaceId) {
      redirect(302, `/${appState.selectedSpaceId}`);
    }
  }
};
